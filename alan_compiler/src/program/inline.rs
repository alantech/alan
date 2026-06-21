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
        // `NativeCall` bodies (native method/property leaf wrappers, e.g.
        // `arg0.unwrap()`) are substitutable. The receiver may be *consumed* by
        // the native method, but that is sound to inline whenever the wrapper's
        // corresponding parameter is declared `Own`: the caller has already
        // relinquished ownership at the call site, so moving the substituted
        // argument in the body happens at the same program point. The per-param
        // `ArgKind` gating in `build_inline_substitution`/`build_multi_inline`
        // enforces that (see `param_borrowed_or_clone_protected`).
        Microstatement::NativeCall { args, .. } => args.iter().all(expr_is_substitutable),
        _ => false,
    }
}

/// Returns true if every occurrence of the parameter `name` within the body
/// expression `ms` is safe to inline: either a *non-consuming* use (passed as a
/// `Ref`/`Deref` argument, or a native-call argument), or a *clone-protected*
/// consume (passed as an `ArgKind::Own` argument of a real function call).
///
/// This is the key correctness gate for inlining: when we splice the body into
/// the caller and substitute a caller variable/expression for the parameter, a
/// bare-value/array/return position would move the caller's value outright, and
/// a `Mut` argument would mutate it -- neither is protected, so they are
/// rejected. Borrowing/copying preserve the value, and an `Own` argument is
/// rendered by the Rust backend with clone-protected ownership (it moves the
/// substituted value at its last use, otherwise clones it), so a value
/// substituted there is never double-moved.
///
/// A `NativeCall` carries no per-argument `ArgKind` (the receiver's `Own`/`Ref`
/// kind is lost when lowered from the bind signature). A native method *can*
/// consume its receiver (e.g. `unwrap`), but this function is only ever evaluated
/// for a `Ref`/`Deref` parameter (see `param_is_inlinable`): a wrapper
/// `fn(arg0: &T) = arg0.method(..)` could not have compiled if `method` moved out
/// of the `&T` receiver, so a direct appearance of the parameter as a native-call
/// argument is provably a borrow. Only nested sub-expressions need recursion.
fn param_borrowed_or_clone_protected(ms: &Microstatement, name: &str) -> bool {
    let is_named = |m: &Microstatement| matches!(m, Microstatement::Value { representation, .. } if representation == name);
    match ms {
        Microstatement::Value { representation, .. } => representation != name,
        Microstatement::FnCall { function, args } => {
            let params = function.args();
            for (i, arg) in args.iter().enumerate() {
                if is_named(arg) {
                    if !matches!(
                        params.get(i).map(|p| &p.1),
                        Some(ArgKind::Ref) | Some(ArgKind::Deref) | Some(ArgKind::Own)
                    ) {
                        return false;
                    }
                } else if !param_borrowed_or_clone_protected(arg, name) {
                    return false;
                }
            }
            true
        }
        Microstatement::Array { vals, .. } => vals
            .iter()
            .all(|v| !is_named(v) && param_borrowed_or_clone_protected(v, name)),
        Microstatement::Assignment { value, .. } => {
            !is_named(value) && param_borrowed_or_clone_protected(value, name)
        }
        Microstatement::Return { value: Some(v) } => {
            !is_named(v) && param_borrowed_or_clone_protected(v, name)
        }
        Microstatement::NativeCall { args, .. } => args
            .iter()
            .all(|a| is_named(a) || param_borrowed_or_clone_protected(a, name)),
        _ => true,
    }
}

/// Strict variant: returns true only if every occurrence of `name` is a
/// genuinely *non-consuming* use -- a `Ref`/`Deref` function argument, or a
/// native-call argument (provably a borrow for a `Ref`/`Deref` parameter; see
/// the note on `param_borrowed_or_clone_protected`). Unlike that function this
/// does *not* accept `Own` arguments, so it identifies the consuming uses that
/// require the caller's value to be movable at the spliced site.
fn param_only_borrowed(ms: &Microstatement, name: &str) -> bool {
    let is_named = |m: &Microstatement| matches!(m, Microstatement::Value { representation, .. } if representation == name);
    match ms {
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
        Microstatement::Array { vals, .. } => vals
            .iter()
            .all(|v| !is_named(v) && param_only_borrowed(v, name)),
        Microstatement::Assignment { value, .. } => {
            !is_named(value) && param_only_borrowed(value, name)
        }
        Microstatement::Return { value: Some(v) } => !is_named(v) && param_only_borrowed(v, name),
        Microstatement::NativeCall { args, .. } => args
            .iter()
            .all(|a| is_named(a) || param_only_borrowed(a, name)),
        _ => true,
    }
}

/// Returns true if inlining `function` would move (consume) the value supplied
/// for parameter `idx`, so the caller's argument must be a temporary or provably
/// movable at the spliced call site. An `Own`/`Mut` parameter is owned (treat as
/// consuming); a `Ref`/`Deref` parameter consumes only if some use is not a pure
/// borrow (e.g. it flows into an `Own` argument, the clone-protected case).
pub fn param_consumes_value(function: &Function, idx: usize) -> bool {
    let params = function.args();
    let Some((name, kind, _)) = params.get(idx) else {
        return false;
    };
    match kind {
        ArgKind::Own | ArgKind::Mut => true,
        ArgKind::Ref | ArgKind::Deref => !function
            .microstatements
            .iter()
            .filter(|m| !matches!(m, Microstatement::Arg { .. }))
            .all(|e| param_only_borrowed(e, name)),
    }
}

/// Returns true if `s` is a plain identifier path (a variable/parameter
/// reference such as `v` or `n.foo`), as opposed to a compile-time literal
/// (`1`, `3.14`, `true`, `"x"`). Used to decide whether substituting an argument
/// into a native-call position (which bypasses call-site conversion) is safe.
fn is_plain_identifier(s: &str) -> bool {
    if s == "true" || s == "false" {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == ':')
}

/// Returns true if substituting `arg` into a `NativeCall` position is safe.
/// `NativeCall` arguments render by their raw representation (a method receiver
/// or native argument), bypassing the conversions a normal call boundary applies
/// — e.g. the Rust backend leaves an integer literal type-ambiguous (`{integer}`,
/// so `1.wrapping_mul(..)` does not resolve), and a string literal would render
/// as `&str` rather than the `.to_string()`-converted `String` the position may
/// expect. A plain variable/parameter reference is already the converted,
/// concretely-typed value (and primitive bindings are now annotated), so it is
/// safe; any non-`Value` expression renders through normal codegen.
fn arg_is_conversion_free(arg: &Microstatement) -> bool {
    match arg {
        Microstatement::Value { representation, .. } => is_plain_identifier(representation),
        _ => true,
    }
}

/// Returns true if the parameter `name` appears as a *direct argument* of a
/// `NativeCall` anywhere in `ms` (the receiver or another native argument).
/// Such a position renders the substituted argument by its raw representation,
/// bypassing call-site conversion, so a literal argument there is unsafe.
fn param_used_as_native_arg(ms: &Microstatement, name: &str) -> bool {
    let is_named = |m: &Microstatement| matches!(m, Microstatement::Value { representation, .. } if representation == name);
    match ms {
        Microstatement::NativeCall { args, .. } => args
            .iter()
            .any(|a| is_named(a) || param_used_as_native_arg(a, name)),
        Microstatement::FnCall { args, .. } | Microstatement::VarCall { args, .. } => {
            args.iter().any(|a| param_used_as_native_arg(a, name))
        }
        Microstatement::Array { vals, .. } => {
            vals.iter().any(|v| param_used_as_native_arg(v, name))
        }
        Microstatement::Assignment { value, .. } => param_used_as_native_arg(value, name),
        Microstatement::Return { value: Some(v) } => param_used_as_native_arg(v, name),
        _ => false,
    }
}

/// Decides whether a single parameter of an inline candidate is safe to
/// substitute, given how it is used across the body expression(s) `exprs` and
/// its declared `ArgKind`/type.
///
/// - `Own`: the caller already moved the argument into the call at this site, so
///   consuming (or projecting/moving out of) the parameter in the body happens
///   at the same program point and preserves semantics. No borrow-only or
///   movable-projection restriction applies. (The duplication rule in the caller
///   still prevents re-evaluating a non-trivial argument.)
/// - `Ref`/`Deref`: the caller passed a borrow (`&T`)/copy, so the body may only
///   borrow the parameter — moving or projecting-and-moving out of it would
///   move/alias the caller's value. Enforced via
///   `param_borrowed_or_clone_protected` plus the
///   movable-projection check.
/// - `Mut`: not inlined (mutating through a `&mut` parameter is out of scope).
fn param_is_inlinable(
    exprs: &[&Microstatement],
    name: &str,
    kind: &ArgKind,
    ptypen: &CType,
) -> bool {
    match kind {
        ArgKind::Own => true,
        ArgKind::Ref | ArgKind::Deref => {
            !type_has_movable_projection(ptypen)
                && exprs
                    .iter()
                    .all(|e| param_borrowed_or_clone_protected(e, name))
        }
        ArgKind::Mut => false,
    }
}

use crate::program::liveness::type_has_movable_projection;

/// Counts the number of times the variable `name` is referenced in `ms`.
fn count_var_uses(ms: &Microstatement, name: &str) -> usize {
    match ms {
        Microstatement::Value { representation, .. } => usize::from(representation == name),
        Microstatement::FnCall { args, .. } => args.iter().map(|a| count_var_uses(a, name)).sum(),
        Microstatement::Array { vals, .. } => vals.iter().map(|v| count_var_uses(v, name)).sum(),
        Microstatement::Assignment { value, .. } => count_var_uses(value, name),
        Microstatement::Return { value: Some(v) } => count_var_uses(v, name),
        Microstatement::NativeCall { args, .. } => {
            args.iter().map(|a| count_var_uses(a, name)).sum()
        }
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
        Microstatement::NativeCall { args, .. } => {
            for a in args {
                walk(a, counts, bodies, visited);
            }
        }
        Microstatement::Value { .. }
        | Microstatement::Arg { .. }
        | Microstatement::Return { .. } => {}
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
        .filter_map(|(id, n)| {
            let f = bodies.get(&id)?;
            // A single-use function may inline as a single return expression or a
            // multi-statement (block) body. A *multi*-use function may also be
            // folded into every call site, but only when it is a trivial
            // single-statement wrapper whose parameters are all pure borrows, so
            // duplicating it adds neither code bulk nor clones.
            if n != 1 && !is_trivially_foldable(f) {
                return None;
            }
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

/// Returns true if `function` is a trivial single-statement wrapper that is
/// cheap and clone-free to fold into *every* call site (even when called more
/// than once): its body is a single `return <expr>` where `<expr>` is one
/// function call or native method/property/operator access whose arguments are
/// all leaves (parameter references or literals, no nested calls), and every
/// parameter is used only by borrow. The pure-borrow requirement guarantees that
/// duplicating the body across call sites introduces no additional clones.
fn is_trivially_foldable(function: &Function) -> bool {
    let Some(expr) = single_return_expr(function) else {
        return false;
    };
    // The body must be a single call/access whose arguments are leaves.
    let args_are_leaves = |args: &[Microstatement]| {
        args.iter()
            .all(|a| matches!(a, Microstatement::Value { .. } | Microstatement::Arg { .. }))
    };
    let shallow = match expr {
        Microstatement::FnCall { args, .. } | Microstatement::NativeCall { args, .. } => {
            args_are_leaves(args)
        }
        _ => false,
    };
    if !shallow {
        return false;
    }
    // Every parameter must be used only by borrow (no consumption), so folding
    // into multiple sites cannot add clones.
    function
        .args()
        .iter()
        .all(|(name, _, _)| param_only_borrowed(expr, name))
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
    for ((pname, kind, ptypen), arg) in params.iter().zip(args.iter()) {
        if !param_is_inlinable(&[expr], pname, kind, ptypen) {
            return None;
        }
        // A parameter feeding a native-call position must be substituted by a
        // conversion-free argument (rendered raw there, so a literal would lose
        // its call-site coercion / be type-ambiguous).
        if param_used_as_native_arg(expr, pname) && !arg_is_conversion_free(arg) {
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
            name: local_renames
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.clone()),
            value: Box::new(rewrite(value, param_subs, local_renames)),
        },
        Microstatement::Return { value } => Microstatement::Return {
            value: value
                .as_ref()
                .map(|v| Box::new(rewrite(v, param_subs, local_renames))),
        },
        Microstatement::NativeCall {
            typen,
            kind,
            name,
            args,
        } => Microstatement::NativeCall {
            typen: typen.clone(),
            kind: kind.clone(),
            name: name.clone(),
            args: args
                .iter()
                .map(|a| rewrite(a, param_subs, local_renames))
                .collect(),
        },
        _ => ms.clone(),
    }
}

/// If `function` is a `Normal` function whose body is argument declarations,
/// then one or more `Assignment`/`FnCall` statements, and finally a single
/// `return <expr>`, returns `(middle statements, return expression)`. These can
/// be inlined as a block expression. Returns `None` for single-statement bodies
/// (handled by `single_return_expr`) or unsupported statement shapes.
pub fn multi_statement_body(
    function: &Function,
) -> Option<(Vec<&Microstatement>, &Microstatement)> {
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
    let mut exprs: Vec<&Microstatement> = stmts.clone();
    exprs.push(tail);
    for ((pname, kind, ptypen), arg) in params.iter().zip(args.iter()) {
        if !param_is_inlinable(&exprs, pname, kind, ptypen) {
            return None;
        }
        // A parameter feeding a native-call position must be substituted by a
        // conversion-free argument (see `build_inline_substitution`).
        if exprs.iter().any(|e| param_used_as_native_arg(e, pname)) && !arg_is_conversion_free(arg)
        {
            return None;
        }
        let uses: usize = stmts
            .iter()
            .map(|s| count_var_uses(s, pname))
            .sum::<usize>()
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
