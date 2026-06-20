// Ordered last-use / liveness analysis over a function body, plus the
// clone-elision rewrites it enables. This is backend-agnostic: it operates only
// on the `Microstatement` IR and returns a rewritten statement list. The Rust
// codegen (`lntors::function`) applies it before emitting a function body.
//
// The core observation: for a value with ordinary (non-`Shared`) value
// semantics, a deep `clone()` followed by never using the original again is
// observationally identical to *moving* the original. So when a `clone(x)` is
// the provable last use of `x`, we drop the clone and move `x` instead.
//
// Safety net: if this analysis is ever wrong about `x` being dead afterward,
// the move it introduces produces a `use of moved value` (or `cannot move out
// of borrow`) error from `rustc`, which surfaces in the test suite rather than
// miscompiling silently. The one case that *could* change behavior silently is
// a `Shared{T}` (an `Arc<RwLock<T>>`), where a deep clone and a handle move
// differ in aliasing; those are excluded explicitly.

use crate::program::{ArgKind, CType, CfnKind, FnKind, Function, Microstatement};

/// Count every reference to `name` within a microstatement subtree, including
/// indirect-call receivers and closure captures (capturing a variable in a
/// closure counts as a use, since the closure may run later or repeatedly).
pub fn count_uses(ms: &Microstatement, name: &str) -> usize {
    match ms {
        Microstatement::Value { representation, .. } => usize::from(representation == name),
        Microstatement::Assignment { value, .. } => count_uses(value, name),
        Microstatement::Return { value: Some(v) } => count_uses(v, name),
        Microstatement::Return { value: None } => 0,
        Microstatement::FnCall { args, .. } | Microstatement::NativeCall { args, .. } => {
            args.iter().map(|a| count_uses(a, name)).sum()
        }
        Microstatement::VarCall {
            name: callee, args, ..
        } => usize::from(callee == name) + args.iter().map(|a| count_uses(a, name)).sum::<usize>(),
        Microstatement::Array { vals, .. } => vals.iter().map(|v| count_uses(v, name)).sum(),
        Microstatement::Closure { function } => function
            .microstatements
            .iter()
            .map(|m| count_uses(m, name))
            .sum(),
        Microstatement::Arg { .. } => 0,
    }
}

/// Returns true if the type (after unwrapping `Type`/`Group`/`Shared`) supports
/// moving a field/element out of it via an accessor that borrows the whole value
/// (tuple `.N`, struct fields, fixed buffers, arrays). Keeping such a value as a
/// borrow — or inlining a parameter of such a type — is unsafe because the body
/// may project-and-move out of it (e.g. `b.0`), which is illegal behind a shared
/// reference and, once the value is the caller's own, moves out of the caller's
/// value.
///
/// `Either`/sum types are deliberately excluded: their variant access either
/// clones (`match &x { Some(v) => v.clone() }`) or takes ownership (an `Own`
/// argument, already caught by the escape analysis), so a borrowed sum value is
/// never moved out from behind the reference. Scalars, strings, and opaque bound
/// types likewise have no movable projections.
pub fn type_has_movable_projection(t: &CType) -> bool {
    match t {
        CType::Type(_, inner) | CType::Group(inner) | CType::Shared(inner) => {
            type_has_movable_projection(inner)
        }
        CType::Tuple(..) | CType::Buffer(..) | CType::Array(_) | CType::Field(..) => true,
        _ => false,
    }
}

/// Returns true if `value` (after unwrapping `Type`/`Group`) is a `Shared{T}`.
fn type_is_shared(t: &CType) -> bool {
    match t {
        CType::Shared(_) => true,
        CType::Type(_, inner) | CType::Group(inner) => type_is_shared(inner),
        _ => false,
    }
}

/// Rewrite a function body, replacing each `clone(x)` call that is the provable
/// last use of `x` with a plain move of `x`. `is_shared_name` reports whether a
/// named variable holds a `Shared` value even when its `get_type()` hides it
/// (e.g. a deep clone whose inferred type is the inner `T`); such variables are
/// never elided.
pub fn elide_last_use_clones(
    function: &Function,
    is_shared_name: &dyn Fn(&str) -> bool,
) -> Vec<Microstatement> {
    let stmts = &function.microstatements;
    // Parameters passed as borrows (`&T`/`&mut T`) cannot be moved out of, so a
    // clone whose source is such a parameter must be kept.
    let borrow_params: Vec<String> = function
        .args()
        .into_iter()
        .filter(|(_, k, _)| matches!(k, ArgKind::Ref | ArgKind::Mut))
        .map(|(n, _, _)| n)
        .collect();

    let mut result = Vec::with_capacity(stmts.len());
    for (i, ms) in stmts.iter().enumerate() {
        // Is variable `name` referenced in any statement strictly after `i`?
        let used_later = |name: &str| {
            stmts[i + 1..]
                .iter()
                .any(|m| count_uses(m, name) > 0)
        };
        // A `clone(x)` node in this statement may be elided when:
        //   - `x` is a plain identifier (a `Value`),
        //   - `x` is not a borrowed parameter (movability),
        //   - `x` is not `Shared` (aliasing-preserving),
        //   - `x` is referenced exactly once in this statement (only by this
        //     clone, so the move does not race another use), and
        //   - `x` is not used in any later statement.
        let can_elide = |x: &str, arg_typen: &CType| {
            !borrow_params.iter().any(|p| p == x)
                && !type_is_shared(arg_typen)
                && !is_shared_name(x)
                && count_uses(ms, x) == 1
                && !used_later(x)
        };
        result.push(rewrite_clone(ms, &can_elide));
    }
    result
}

/// Recursively replace eligible `clone(x)` calls with a move of `x`. Does not
/// descend into closures: a closure has its own (possibly repeated) execution
/// scope, so the enclosing statement's last-use reasoning does not apply inside
/// it.
fn rewrite_clone(ms: &Microstatement, can_elide: &dyn Fn(&str, &CType) -> bool) -> Microstatement {
    // A direct `clone(Value{x})` that is eligible collapses to the argument.
    if let Microstatement::FnCall { function, args } = ms {
        if matches!(function.kind, FnKind::CfnRealized(CfnKind::Clone)) && args.len() == 1 {
            if let Microstatement::Value { representation, typen } = &args[0] {
                if can_elide(representation, typen) {
                    return args[0].clone();
                }
            }
        }
    }
    // Otherwise recurse structurally, rewriting nested clones.
    match ms {
        Microstatement::Assignment {
            mutable,
            name,
            value,
        } => Microstatement::Assignment {
            mutable: *mutable,
            name: name.clone(),
            value: Box::new(rewrite_clone(value, can_elide)),
        },
        Microstatement::Return { value: Some(v) } => Microstatement::Return {
            value: Some(Box::new(rewrite_clone(v, can_elide))),
        },
        Microstatement::FnCall { function, args } => Microstatement::FnCall {
            function: function.clone(),
            args: args.iter().map(|a| rewrite_clone(a, can_elide)).collect(),
        },
        Microstatement::VarCall { name, typen, args } => Microstatement::VarCall {
            name: name.clone(),
            typen: typen.clone(),
            args: args.iter().map(|a| rewrite_clone(a, can_elide)).collect(),
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
            args: args.iter().map(|a| rewrite_clone(a, can_elide)).collect(),
        },
        Microstatement::Array { typen, vals } => Microstatement::Array {
            typen: typen.clone(),
            vals: vals.iter().map(|v| rewrite_clone(v, can_elide)).collect(),
        },
        // Closures, plain values, args, and `Return(None)` are left untouched.
        other => other.clone(),
    }
}
