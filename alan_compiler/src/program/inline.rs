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
//! For now we only handle "single return expression" bodies, which inline as a
//! pure expression substitution and therefore behave identically in Rust and JS
//! (no block-expressions / IIFEs required).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{ArgKind, FnKind, Function, Microstatement};

thread_local! {
    static INLINE_TARGETS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
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
        _ => false,
    }
}

/// Returns true if every occurrence of the parameter `name` within the body
/// expression `ms` is a *borrowing* use (passed as an `ArgKind::Ref` argument).
///
/// This is the key correctness gate for inlining: when we splice the body into
/// the caller and substitute a caller variable/expression for the parameter, any
/// position that *consumes* the value (move into an owned/mut/deref argument,
/// store into an array, or appear as the returned value itself) would move or
/// mutate the caller's value. The original function boundary protected against
/// that (it received `&T` and worked on its own copy), so inlining such uses
/// would change ownership semantics (and typically fails to compile in Rust).
/// Restricting to borrow-only uses preserves the original semantics.
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
                    if !matches!(params.get(i).map(|p| &p.1), Some(ArgKind::Ref)) {
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
        _ => true,
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
            let expr = single_return_expr(f)?;
            if expr_calls_identity(expr, &id) {
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
    for ((pname, _kind, _typen), arg) in params.iter().zip(args.iter()) {
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
    match ms {
        Microstatement::Value { representation, .. } => match subs.get(representation) {
            Some(replacement) => replacement.clone(),
            None => ms.clone(),
        },
        Microstatement::FnCall { function, args } => Microstatement::FnCall {
            function: function.clone(),
            args: args.iter().map(|a| substitute(a, subs)).collect(),
        },
        Microstatement::Array { typen, vals } => Microstatement::Array {
            typen: typen.clone(),
            vals: vals.iter().map(|v| substitute(v, subs)).collect(),
        },
        _ => ms.clone(),
    }
}
