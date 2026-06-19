//! Backend-agnostic decision logic for inlining single-use functions.
//!
//! When a (non-generic, non-intrinsic) function is called from exactly one call
//! site in the whole reachable program and its body is a single `return <expr>`,
//! we can replace that call with the function's body, substituting the
//! parameters for the caller's argument expressions. This removes a function
//! boundary (and the defensive clones that come with it).
//!
//! This module only provides the *decision making* and the (backend-agnostic)
//! microstatement substitution. Each backend performs the actual splice in its
//! own codegen by rendering the substituted expression with its existing
//! machinery, so the inlined function is simply never emitted.
//!
//! "Single return expression" bodies inline as a pure expression substitution
//! and behave identically in Rust and JS. Multi-statement bodies (locals plus a
//! final return) additionally inline as a block expression in Rust (which also
//! lets values `drop` early); inner locals are renamed to a unique prefix so
//! they cannot shadow caller variables introduced by the substitution. The JS
//! backend currently only performs the single-expression form and falls back to
//! a normal call for multi-statement bodies.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{ArgKind, CType, FnKind, Function, Microstatement};

thread_local! {
    static INLINE_TARGETS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static INLINE_COUNTER: Cell<usize> = const { Cell::new(0) };
}

/// Returns a process-thread-unique id used to rename a multi-statement inline
/// expansion's inner locals so they cannot collide with the caller's variables
/// (or with other expansions).
fn next_inline_id() -> usize {
    INLINE_COUNTER.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    })
}

/// Canonical identity for a realized function, matching the `name_argtypes`
/// naming both backends use to deduplicate generated functions. Used to match
/// call sites against the set of single-use inline targets.
pub fn fn_identity(function: &Function) -> String {
    let arg_strs = function
        .args()
        .iter()
        .map(|(_, _, t)| t.clone().to_callable_string())
        .collect::<Vec<String>>();
    format!("{}_{}", function.name, arg_strs.join("_"))
}

/// If `function` is a `Normal` function whose body is a (possibly empty)
/// sequence of argument declarations followed by a single `return <expr>` whose
/// expression is itself simple enough to substitute, returns that expression.
pub fn single_return_expr(function: &Function) -> Option<&Microstatement> {
    if !matches!(function.kind, FnKind::Normal) {
        return None;
    }
    let (last, rest) = function.microstatements.split_last()?;
    // Every leading statement must be an argument declaration (a codegen no-op).
    if !rest
        .iter()
        .all(|ms| matches!(ms, Microstatement::Arg { .. }))
    {
        return None;
    }
    match last {
        Microstatement::Return { value: Some(v) }
            if expr_is_substitutable(v) && !textually_captures_params(v, function) =>
        {
            Some(v)
        }
        _ => None,
    }
}

/// Collects the `representation` string of every `Value` in `ms`.
fn collect_value_reprs<'a>(ms: &'a Microstatement, out: &mut Vec<&'a str>) {
    match ms {
        Microstatement::Value { representation, .. } => out.push(representation),
        Microstatement::FnCall { args, .. } => {
            args.iter().for_each(|a| collect_value_reprs(a, out))
        }
        Microstatement::Array { vals, .. } => vals.iter().for_each(|v| collect_value_reprs(v, out)),
        Microstatement::Assignment { value, .. } => collect_value_reprs(value, out),
        Microstatement::Return { value: Some(v) } => collect_value_reprs(v, out),
        _ => {}
    }
}

/// Some compiler-synthesized `Normal` functions (method/cast/operator wrappers)
/// store their body as a raw target-language code `Value`, e.g.
/// `Value("arg0.count_ones()")`, with the parameter name baked into the string.
/// Our structural substitution replaces whole-identifier `Value`s only, so it
/// cannot rewrite a parameter embedded as a substring of code text. Detect that
/// case so we leave such functions as real calls. This is sound: a captured
/// parameter always appears as a substring, so there are no false negatives
/// (only conservative skips when a name happens to be a substring of unrelated
/// text).
fn textually_captures_params(expr: &Microstatement, function: &Function) -> bool {
    let mut reprs = Vec::new();
    collect_value_reprs(expr, &mut reprs);
    let params = function.args();
    reprs.iter().any(|r| {
        params
            .iter()
            .any(|(p, _, _)| *r != p.as_str() && r.contains(p.as_str()))
    })
}

/// Conservatively restrict the expression shapes we will inline to the ones the
/// substitution below handles soundly: values, argument refs, function calls,
/// and array literals. Closures (capture semantics) and indirect `VarCall`s
/// (whose callee name could itself be a parameter) are excluded for now.
fn expr_is_substitutable(ms: &Microstatement) -> bool {
    match ms {
        Microstatement::Value { .. } | Microstatement::Arg { .. } => true,
        Microstatement::FnCall { args, .. } => args.iter().all(expr_is_substitutable),
        Microstatement::Array { vals, .. } => vals.iter().all(expr_is_substitutable),
        Microstatement::Assignment { value, .. } => expr_is_substitutable(value),
        Microstatement::Return { value: Some(v) } => expr_is_substitutable(v),
        _ => false,
    }
}

/// Returns true if every occurrence of the parameter `name` within the body
/// expression `ms` is a *non-consuming* use: passed as an `ArgKind::Ref`
/// (borrowed) or `ArgKind::Deref` (copied, the kind is only used for `Copy`
/// types) argument.
///
/// This is the key correctness gate for inlining: when we splice the body into
/// the caller and substitute a caller variable/expression for the parameter, any
/// position that *consumes* the value (move into an owned/mut argument, store
/// into an array, or appear as the returned value itself) would move or mutate
/// the caller's value. The original function boundary protected against that (it
/// received `&T` and worked on its own copy), so inlining such uses would change
/// ownership semantics (and typically fails to compile in Rust). Borrowing and
/// copying do not consume the caller's value, so they preserve the semantics.
fn param_only_borrowed(ms: &Microstatement, name: &str) -> bool {
    let is_named = |m: &Microstatement| {
        matches!(m, Microstatement::Value { representation, .. } if representation == name)
    };
    match ms {
        // A bare occurrence (e.g. the returned value itself) consumes the value.
        Microstatement::Value { representation, .. } => representation != name,
        Microstatement::FnCall { function, args } => {
            let params = function.args();
            for (i, arg) in args.iter().enumerate() {
                if is_named(arg) {
                    if !matches!(
                        params.get(i).map(|p| &p.1),
                        Some(ArgKind::Ref) | Some(ArgKind::Deref)
                    ) {
                        return false;
                    }
                } else if !param_only_borrowed(arg, name) {
                    return false;
                }
            }
            true
        }
        // Array elements are moved into the constructed vector.
        Microstatement::Array { vals, .. } => vals.iter().all(|v| !is_named(v) && param_only_borrowed(v, name)),
        // A direct `let y = <param>` moves the parameter into the local.
        Microstatement::Assignment { value, .. } => !is_named(value) && param_only_borrowed(value, name),
        Microstatement::Return { value: Some(v) } => !is_named(v) && param_only_borrowed(v, name),
        _ => true,
    }
}

/// Returns true if the type (after unwrapping `Type`/`Group`/`Shared`) supports
/// moving a field/element out of it (tuples, structs, fixed buffers, arrays, sum
/// types). Inlining a parameter of such a type is unsafe because the body may
/// project-and-move out of it (e.g. `b.0`), which — once the parameter is the
/// caller's own value rather than the function's private copy — moves out of the
/// caller's value. Scalars, strings, and opaque bound types have no such movable
/// projections.
fn type_has_movable_projection(t: &CType) -> bool {
    match t {
        CType::Type(_, inner) | CType::Group(inner) | CType::Shared(inner) => {
            type_has_movable_projection(inner)
        }
        CType::Tuple(..)
        | CType::Buffer(..)
        | CType::Array(_)
        | CType::Either(..)
        | CType::Field(..) => true,
        _ => false,
    }
}

/// Counts the number of times the variable `name` is referenced in `ms`.
fn count_var_uses(ms: &Microstatement, name: &str) -> usize {
    match ms {
        Microstatement::Value { representation, .. } => usize::from(representation == name),
        Microstatement::FnCall { args, .. } => {
            args.iter().map(|a| count_var_uses(a, name)).sum()
        }
        Microstatement::Array { vals, .. } => {
            vals.iter().map(|v| count_var_uses(v, name)).sum()
        }
        Microstatement::Assignment { value, .. } => count_var_uses(value, name),
        Microstatement::Return { value: Some(v) } => count_var_uses(v, name),
        _ => 0,
    }
}

/// Returns true if `ms` contains a call to the function identified by `id`
/// (used to reject self-recursive inline candidates).
fn expr_calls_identity(ms: &Microstatement, id: &str) -> bool {
    match ms {
        Microstatement::FnCall { function, args } => {
            fn_identity(function) == id || args.iter().any(|a| expr_calls_identity(a, id))
        }
        Microstatement::Array { vals, .. } => vals.iter().any(|v| expr_calls_identity(v, id)),
        Microstatement::Assignment { value, .. } => expr_calls_identity(value, id),
        Microstatement::Return { value: Some(v) } => expr_calls_identity(v, id),
        Microstatement::Closure { function } => function
            .microstatements
            .iter()
            .any(|m| expr_calls_identity(m, id)),
        Microstatement::VarCall { args, .. } => args.iter().any(|a| expr_calls_identity(a, id)),
        _ => false,
    }
}

fn collect(
    function: &Arc<Function>,
    counts: &mut HashMap<String, usize>,
    bodies: &mut HashMap<String, Arc<Function>>,
    visited: &mut HashSet<String>,
) {
    let id = fn_identity(function);
    if !visited.insert(id.clone()) {
        return;
    }
    if matches!(function.kind, FnKind::Normal) {
        bodies.insert(id, function.clone());
    }
    for ms in &function.microstatements {
        walk(ms, counts, bodies, visited);
    }
}

fn walk(
    ms: &Microstatement,
    counts: &mut HashMap<String, usize>,
    bodies: &mut HashMap<String, Arc<Function>>,
    visited: &mut HashSet<String>,
) {
    match ms {
        Microstatement::FnCall { function, args } => {
            if matches!(function.kind, FnKind::Normal) {
                *counts.entry(fn_identity(function)).or_insert(0) += 1;
            }
            collect(function, counts, bodies, visited);
            for a in args {
                walk(a, counts, bodies, visited);
            }
        }
        Microstatement::VarCall { args, .. } => {
            for a in args {
                walk(a, counts, bodies, visited);
            }
        }
        Microstatement::Array { vals, .. } => {
            for v in vals {
                walk(v, counts, bodies, visited);
            }
        }
        Microstatement::Assignment { value, .. } => walk(value, counts, bodies, visited),
        Microstatement::Return { value: Some(v) } => walk(v, counts, bodies, visited),
        Microstatement::Closure { function } => {
            for m in &function.microstatements {
                walk(m, counts, bodies, visited);
            }
        }
        Microstatement::Value { .. } | Microstatement::Arg { .. } | Microstatement::Return { .. } => {
        }
    }
}

/// Walks the reachable call graph from `entry` and returns the set of function
/// identities that are: referenced by exactly one `FnCall` site, inlinable as a
/// single return expression, and non-recursive.
pub fn compute_inline_targets(entry: &Arc<Function>) -> HashSet<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut bodies: HashMap<String, Arc<Function>> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    collect(entry, &mut counts, &mut bodies, &mut visited);
    counts
        .into_iter()
        .filter(|(_, n)| *n == 1)
        .filter_map(|(id, _)| {
            let f = bodies.get(&id)?;
            // Inlinable as either a single return expression or a multi-statement
            // (block) body.
            if single_return_expr(f).is_none() && multi_statement_body(f).is_none() {
                return None;
            }
            // Reject self-recursive bodies (would inline forever).
            if f.microstatements
                .iter()
                .any(|ms| expr_calls_identity(ms, &id))
            {
                return None;
            }
            Some(id)
        })
        .collect()
}

/// Installs the set of inline targets for the current codegen run.
pub fn set_inline_targets(targets: HashSet<String>) {
    INLINE_TARGETS.with(|t| *t.borrow_mut() = targets);
}

/// Returns true if the function identity is a single-use inline target.
pub fn is_inline_target(id: &str) -> bool {
    INLINE_TARGETS.with(|t| t.borrow().contains(id))
}

/// Builds the parameter->argument substitution map for inlining a call to
/// `function` with `args`, or `None` if doing so would be unsafe (mismatched
/// arity, or a non-trivial argument expression that would be duplicated or
/// dropped by the substitution).
pub fn build_inline_substitution(
    function: &Function,
    args: &[Microstatement],
) -> Option<HashMap<String, Microstatement>> {
    let params = function.args();
    if params.len() != args.len() {
        return None;
    }
    let expr = single_return_expr(function)?;
    let mut subs = HashMap::new();
    for ((pname, _kind, ptypen), arg) in params.iter().zip(args.iter()) {
        // A parameter whose type can have a field/element moved out of it is
        // unsafe to inline (a `b.0`-style projection would move the caller's val).
        if type_has_movable_projection(ptypen) {
            return None;
        }
        // The parameter must only be borrowed in the body, otherwise inlining
        // would move/mutate the caller's value (see `param_only_borrowed`).
        if !param_only_borrowed(expr, pname) {
            return None;
        }
        let uses = count_var_uses(expr, pname);
        let trivial = matches!(
            arg,
            Microstatement::Value { .. } | Microstatement::Arg { .. }
        );
        // A non-trivial argument is only safe to substitute when it is used
        // exactly once: more than once would duplicate (re-evaluate) it, zero
        // times would drop a potentially side-effecting expression.
        if !trivial && uses != 1 {
            return None;
        }
        subs.insert(pname.clone(), arg.clone());
    }
    Some(subs)
}

/// Produces a copy of `ms` with parameter references replaced per `subs`.
pub fn substitute(ms: &Microstatement, subs: &HashMap<String, Microstatement>) -> Microstatement {
    rewrite(ms, subs, &HashMap::new())
}

/// Rewrites a microstatement for inlining: parameter references (`Value`s whose
/// representation is in `param_subs`) are replaced by the caller's argument
/// microstatement, and inner local variables (`Value`/`Assignment` names in
/// `local_renames`) are renamed to their unique inlined name.
pub fn rewrite(
    ms: &Microstatement,
    param_subs: &HashMap<String, Microstatement>,
    local_renames: &HashMap<String, String>,
) -> Microstatement {
    match ms {
        Microstatement::Value {
            representation,
            typen,
        } => {
            if let Some(replacement) = param_subs.get(representation) {
                replacement.clone()
            } else if let Some(newname) = local_renames.get(representation) {
                Microstatement::Value {
                    representation: newname.clone(),
                    typen: typen.clone(),
                }
            } else {
                ms.clone()
            }
        }
        Microstatement::FnCall { function, args } => Microstatement::FnCall {
            function: function.clone(),
            args: args
                .iter()
                .map(|a| rewrite(a, param_subs, local_renames))
                .collect(),
        },
        Microstatement::Array { typen, vals } => Microstatement::Array {
            typen: typen.clone(),
            vals: vals
                .iter()
                .map(|v| rewrite(v, param_subs, local_renames))
                .collect(),
        },
        Microstatement::Assignment {
            mutable,
            name,
            value,
        } => Microstatement::Assignment {
            mutable: *mutable,
            name: local_renames.get(name).cloned().unwrap_or_else(|| name.clone()),
            value: Box::new(rewrite(value, param_subs, local_renames)),
        },
        Microstatement::Return { value } => Microstatement::Return {
            value: value
                .as_ref()
                .map(|v| Box::new(rewrite(v, param_subs, local_renames))),
        },
        _ => ms.clone(),
    }
}

/// If `function` is a `Normal` function whose body is argument declarations,
/// then one or more `Assignment`/`FnCall` statements, and finally a single
/// `return <expr>`, returns `(middle statements, return expression)`. These can
/// be inlined as a block expression. Returns `None` for single-statement bodies
/// (handled by `single_return_expr`) or unsupported statement shapes.
pub fn multi_statement_body(function: &Function) -> Option<(Vec<&Microstatement>, &Microstatement)> {
    if !matches!(function.kind, FnKind::Normal) {
        return None;
    }
    let (last, rest) = function.microstatements.split_last()?;
    let tail = match last {
        Microstatement::Return { value: Some(v) } => v,
        _ => return None,
    };
    let mut stmts = Vec::new();
    for ms in rest {
        match ms {
            Microstatement::Arg { .. } => {}
            Microstatement::Assignment { .. } | Microstatement::FnCall { .. } => stmts.push(ms),
            // Closures, indirect calls, mid-body returns, etc. are out of scope.
            _ => return None,
        }
    }
    if stmts.is_empty() {
        return None;
    }
    if !expr_is_substitutable(tail) || !stmts.iter().all(|s| expr_is_substitutable(s)) {
        return None;
    }
    Some((stmts, tail))
}

/// Builds the rewritten (middle statements, tail expression) for inlining a
/// multi-statement `function` called with `args` as a block expression, or
/// `None` if it would be unsafe. Inner locals are renamed to a unique prefix so
/// they cannot shadow caller variables introduced by the argument substitution.
pub fn build_multi_inline(
    function: &Function,
    args: &[Microstatement],
) -> Option<(Vec<Microstatement>, Microstatement)> {
    let (stmts, tail) = multi_statement_body(function)?;
    let params = function.args();
    if params.len() != args.len() {
        return None;
    }
    let locals: Vec<String> = stmts
        .iter()
        .filter_map(|s| match s {
            Microstatement::Assignment { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    // Guard against any parameter or local name baked into a raw-code `Value`
    // (which our structural substitution/renaming cannot rewrite).
    {
        let mut names: Vec<&str> = params.iter().map(|(p, _, _)| p.as_str()).collect();
        names.extend(locals.iter().map(|l| l.as_str()));
        let mut reprs = Vec::new();
        for s in &stmts {
            collect_value_reprs(s, &mut reprs);
        }
        collect_value_reprs(tail, &mut reprs);
        if reprs
            .iter()
            .any(|r| names.iter().any(|n| r != n && r.contains(n)))
        {
            return None;
        }
    }
    let mut param_subs = HashMap::new();
    for ((pname, _kind, ptypen), arg) in params.iter().zip(args.iter()) {
        // A parameter whose type can have a field/element moved out of it is
        // unsafe to inline (a `b.0`-style projection would move the caller's val).
        if type_has_movable_projection(ptypen) {
            return None;
        }
        // Every use across all statements and the tail must be borrow-only.
        let borrowed = stmts.iter().all(|s| param_only_borrowed(s, pname))
            && param_only_borrowed(tail, pname);
        if !borrowed {
            return None;
        }
        let uses: usize = stmts.iter().map(|s| count_var_uses(s, pname)).sum::<usize>()
            + count_var_uses(tail, pname);
        let trivial = matches!(
            arg,
            Microstatement::Value { .. } | Microstatement::Arg { .. }
        );
        if !trivial && uses != 1 {
            return None;
        }
        param_subs.insert(pname.clone(), arg.clone());
    }
    let id = next_inline_id();
    let local_renames: HashMap<String, String> = locals
        .iter()
        .map(|l| (l.clone(), format!("__inl{id}_{l}")))
        .collect();
    let rewritten_stmts = stmts
        .iter()
        .map(|s| rewrite(s, &param_subs, &local_renames))
        .collect();
    let rewritten_tail = rewrite(tail, &param_subs, &local_renames);
    Some((rewritten_stmts, rewritten_tail))
}
