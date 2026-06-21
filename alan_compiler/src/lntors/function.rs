// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use crate::lntors::typen;
use crate::program::liveness;
use crate::program::{
    ArgKind, CType, CfnKind, FnKind, Function, Microstatement, NativeCallKind, Program, Scope,
};

thread_local! {
    // Index of the top-level statement of the function currently being emitted.
    // `usize::MAX` means "unknown" (we are not directly inside the `generate`
    // statement loop), which disables the move optimization conservatively.
    static STMT_IDX: Cell<usize> = const { Cell::new(usize::MAX) };
    // Greater than zero while rendering a nested scope (closure body or an
    // inlined callee body) where the enclosing function's per-statement liveness
    // no longer applies. Moves are disabled (we clone instead) while untrusted.
    static UNTRUSTED_DEPTH: Cell<usize> = const { Cell::new(0) };
    // Names of functions referenced as first-class values (callbacks, stored
    // function pointers) anywhere in the reachable program. Such a function must
    // keep the `&T` parameter signature the higher-order call site expects (our
    // callbacks are `impl Fn(&T, ...)`), so its parameters are never promoted to
    // owned. Populated once per codegen run from the entry function.
    static FN_VALUE_REFS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Recursively collect the names of functions referenced as first-class values
/// (a `Value` whose type is a `CType::Function`) reachable from `ms`, descending
/// through called function bodies and closures (guarded by `visited` function
/// identities to terminate on recursion).
fn collect_fn_value_refs(
    ms: &Microstatement,
    refs: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) {
    match ms {
        Microstatement::Value {
            typen,
            representation,
        } if matches!(&**typen, CType::Function(..)) => {
            refs.insert(representation.clone());
        }
        Microstatement::FnCall { function, args } => {
            let id = crate::program::inline::fn_identity(function);
            if visited.insert(id) {
                for m in &function.microstatements {
                    collect_fn_value_refs(m, refs, visited);
                }
            }
            for a in args {
                collect_fn_value_refs(a, refs, visited);
            }
        }
        Microstatement::Closure { function } => {
            for m in &function.microstatements {
                collect_fn_value_refs(m, refs, visited);
            }
        }
        Microstatement::VarCall { args, .. } | Microstatement::NativeCall { args, .. } => {
            for a in args {
                collect_fn_value_refs(a, refs, visited);
            }
        }
        Microstatement::Array { vals, .. } => {
            for v in vals {
                collect_fn_value_refs(v, refs, visited);
            }
        }
        Microstatement::Assignment { value, .. } => collect_fn_value_refs(value, refs, visited),
        Microstatement::Return { value: Some(v) } => collect_fn_value_refs(v, refs, visited),
        _ => {}
    }
}

/// Installs the set of first-class-referenced function names for this codegen
/// run by walking the reachable call graph from `entry`.
pub fn set_fn_value_refs(entry: &Arc<Function>) {
    let mut refs = HashSet::new();
    let mut visited = HashSet::new();
    for m in &entry.microstatements {
        collect_fn_value_refs(m, &mut refs, &mut visited);
    }
    FN_VALUE_REFS.with(|r| *r.borrow_mut() = refs);
}

/// RAII guard that marks the current rendering scope as "untrusted" for the
/// ownership-move optimization (a closure body or an inlined callee body, where
/// the enclosing function's per-statement liveness no longer applies). While
/// alive, `caller_can_move` returns false, so promoted-owned arguments are
/// cloned rather than moved. The decrement on `Drop` is `?`-early-return-safe.
struct UntrustedGuard;
impl UntrustedGuard {
    fn new() -> Self {
        UNTRUSTED_DEPTH.with(|d| d.set(d.get() + 1));
        UntrustedGuard
    }
}
impl Drop for UntrustedGuard {
    fn drop(&mut self) {
        UNTRUSTED_DEPTH.with(|d| d.set(d.get() - 1));
    }
}

/// RAII guard installed at each function-emission boundary (`generate`). A
/// function body is rendered in its own trusted statement context, so we reset
/// the untrusted depth to zero for its duration; on `Drop` (including a `?`
/// early return) it restores the caller's statement index and depth, so that
/// emitting a callee inline while processing a caller's arguments does not
/// clobber the caller's per-statement liveness context.
struct StmtCtxGuard {
    idx: usize,
    depth: usize,
}
impl StmtCtxGuard {
    fn enter_function() -> Self {
        let g = StmtCtxGuard {
            idx: STMT_IDX.with(|c| c.get()),
            depth: UNTRUSTED_DEPTH.with(|c| c.get()),
        };
        UNTRUSTED_DEPTH.with(|c| c.set(0));
        g
    }
}
impl Drop for StmtCtxGuard {
    fn drop(&mut self) {
        STMT_IDX.with(|c| c.set(self.idx));
        UNTRUSTED_DEPTH.with(|c| c.set(self.depth));
    }
}

/// Returns true if a use of an argument named `name` anywhere in `function`'s
/// body requires *owning* the value (per `ref_arg_escapes`), i.e. the parameter
/// would otherwise be defensively cloned at function entry.
fn requires_ownership(function: &Function, name: &str) -> bool {
    function
        .microstatements
        .iter()
        .any(|ms| ref_arg_escapes(ms, name))
}

/// Returns true if `name` appears as a *non-receiver* argument of any native
/// call in `ms` (recursively): every argument of a `Function`-kind native call,
/// or `args[1..]` of a `Method`/`Property` call. Such positions are rendered
/// *raw* and must match the native construct's expected borrow form (e.g.
/// `vec.join(sep)` needs `sep: &str`), so a parameter used there cannot be
/// promoted to an owned value. A native *receiver* (`args[0]` of a
/// `Method`/`Property`) is exempt: Rust method-call syntax auto-refs/auto-muts
/// or moves the receiver as the method requires, so an owned receiver is always
/// valid.
fn used_as_native_value_arg(ms: &Microstatement, name: &str) -> bool {
    let is_named = |m: &Microstatement| matches!(m, Microstatement::Value { representation, .. } if representation == name);
    match ms {
        Microstatement::NativeCall { kind, args, .. } => {
            let nonreceiver = match kind {
                // No receiver: every argument is a raw value position.
                NativeCallKind::Function
                | NativeCallKind::Infix
                | NativeCallKind::Prefix
                | NativeCallKind::Cast => &args[..],
                // Receiver is args[0]; only args[1..] are raw value arguments.
                NativeCallKind::Method | NativeCallKind::Property => {
                    args.split_first().map(|(_, rest)| rest).unwrap_or(&[])
                }
            };
            if nonreceiver.iter().any(is_named) {
                return true;
            }
            // Recurse into all args (a nested native call may also qualify).
            args.iter().any(|a| used_as_native_value_arg(a, name))
        }
        Microstatement::FnCall { args, .. } | Microstatement::VarCall { args, .. } => {
            args.iter().any(|a| used_as_native_value_arg(a, name))
        }
        Microstatement::Array { vals, .. } => {
            vals.iter().any(|v| used_as_native_value_arg(v, name))
        }
        Microstatement::Assignment { value, .. } => used_as_native_value_arg(value, name),
        Microstatement::Return { value: Some(v) } => used_as_native_value_arg(v, name),
        Microstatement::Closure { function } => function
            .microstatements
            .iter()
            .any(|m| used_as_native_value_arg(m, name)),
        _ => false,
    }
}

/// Decide whether parameter `idx` of `function` should be emitted as an owned
/// (`T`) parameter instead of a borrow (`&T`). We promote a `Ref` parameter to
/// `Own` exactly when the body needs to own it (so it is currently defensively
/// cloned at entry); the caller then supplies ownership by moving the value when
/// it is at its last use, or cloning it otherwise. This is a pure function of
/// the callee, so the definition site (`generate`) and every call site agree
/// without any whole-program bookkeeping.
///
/// Restricted to `Normal` functions (the only ones we emit *and* whose call
/// sites we render via `render_arg`), non-`Shared`, non-function-typed value
/// parameters.
/// Returns true if `function` is (or may be) referenced as a first-class value,
/// so its signature must stay the form a higher-order call site expects
/// (`impl Fn(&T, ...)`, with the element type the callback is invoked with --
/// e.g. `&String`, not `&str`, for `alan_std`'s `String`-monomorphized generics).
/// User functions are matched by name via the reachable-graph pre-pass;
/// compiler-synthesized platform-call wrappers (the `Call{N, F}` references,
/// named `Call_*`) exist specifically to be passed as function values, so they
/// are matched by name prefix. Both are pure (the set is fixed before any
/// rendering), so the definition site and every call site agree.
fn is_value_referenced(function: &Function) -> bool {
    function.name.starts_with("Call_")
        || FN_VALUE_REFS.with(|r| r.borrow().contains(&function.name))
}

fn promote_param_to_own(function: &Function, idx: usize) -> bool {
    if !matches!(function.kind, FnKind::Normal) {
        return false;
    }
    // A first-class-referenced function must keep its `&T` signature.
    if is_value_referenced(function) {
        return false;
    }
    let args = function.args();
    let Some((name, kind, t)) = args.get(idx) else {
        return false;
    };
    matches!(kind, ArgKind::Ref)
        && !matches!(&**t, CType::Shared(_))
        && !matches!(&**t, CType::Function(..))
        && requires_ownership(function, name)
        // A parameter rendered raw in a native value-argument position must keep
        // the borrow form that position expects; do not take it by value.
        && !function
            .microstatements
            .iter()
            .any(|ms| used_as_native_value_arg(ms, name))
}

/// Returns true if the parameter named `name` of `function` is promoted to an
/// owned parameter (see `promote_param_to_own`).
fn param_name_promoted(function: &Function, name: &str) -> bool {
    function
        .args()
        .iter()
        .position(|(n, _, _)| n == name)
        .map(|i| promote_param_to_own(function, i))
        .unwrap_or(false)
}

/// Returns true if `name` names a value the function body being emitted
/// (`parent_fn`) owns and can move from: an `Own`/`Deref` parameter, a `Ref`
/// parameter that was promoted to owned (see `promote_param_to_own`), or a local
/// whose defining assignment produces a fresh owned value (anything other than a
/// bare alias of another variable, which could be a borrow). Borrowed
/// (`Ref`/`Mut`) parameters that were not promoted are never movable.
fn is_movable_owned(parent_fn: &Function, name: &str) -> bool {
    for (n, k, _) in parent_fn.args() {
        if n == name {
            return matches!(k, ArgKind::Own | ArgKind::Deref)
                || param_name_promoted(parent_fn, name);
        }
    }
    for ms in &parent_fn.microstatements {
        if let Microstatement::Assignment { name: n, value, .. } = ms {
            if n == name {
                return !matches!(&**value, Microstatement::Value { .. });
            }
        }
    }
    false
}

/// Returns true if `name` is a parameter or a locally-assigned variable of
/// `parent_fn` (as opposed to a literal/constant rendered as a `Value`). Only
/// known variables risk aliasing a value used elsewhere; an unrecognized `Value`
/// representation is a literal/temporary that is always safe to move.
fn is_known_variable(parent_fn: &Function, name: &str) -> bool {
    parent_fn.args().iter().any(|(n, _, _)| n == name)
        || parent_fn
            .microstatements
            .iter()
            .any(|ms| matches!(ms, Microstatement::Assignment { name: n, .. } if n == name))
}

/// Returns true if, at the current statement of `parent_fn`, the variable `name`
/// can be *moved* into a call (rather than cloned) to satisfy a promoted owned
/// parameter: we must be in a trusted top-level-statement context, the value
/// must be owned/movable, used exactly once in this statement, and never again
/// afterward.
fn caller_can_move(parent_fn: &Function, name: &str) -> bool {
    if UNTRUSTED_DEPTH.with(|d| d.get()) > 0 {
        return false;
    }
    let idx = STMT_IDX.with(|c| c.get());
    let stmts = &parent_fn.microstatements;
    if idx >= stmts.len() {
        return false;
    }
    if !is_movable_owned(parent_fn, name) {
        return false;
    }
    let here = liveness::count_uses(&stmts[idx], name);
    let after: usize = stmts[idx + 1..]
        .iter()
        .map(|m| liveness::count_uses(m, name))
        .sum();
    here == 1 && after == 0
}

/// Returns true if a single argument is safe to *consume* (move) at the current
/// call site: a temporary/literal (any non-`Value`, or a `Value` that is not a
/// known caller variable) is always movable, and a known variable is movable
/// only when `caller_can_move` permits. Used to gate inlining a function that
/// consumes the value supplied for a parameter.
fn arg_safe_to_consume(arg: &Microstatement, parent_fn: &Function) -> bool {
    match arg {
        Microstatement::Value { representation, .. }
            if is_known_variable(parent_fn, representation) =>
        {
            caller_can_move(parent_fn, representation)
        }
        _ => true,
    }
}

/// Returns true if inlining `function` at this call site is safe with respect to
/// ownership: for every parameter the body would *consume*, the corresponding
/// caller argument must be safe to move here. Inlining splices the body in,
/// bypassing the call boundary's clone-protection (and any cascade of further
/// inlines), so a consumed argument that is a still-live caller variable would be
/// moved illegally. Pure-borrow parameters impose no constraint.
fn inline_consumes_are_safe(
    function: &Function,
    args: &[Microstatement],
    parent_fn: &Function,
) -> bool {
    args.iter().enumerate().all(|(i, arg)| {
        !crate::program::inline::param_consumes_value(function, i)
            || arg_safe_to_consume(arg, parent_fn)
    })
}

/// Build a map of variable names to the inner type of their Shared{T} wrapper,
/// by tracing variable assignments back to their origin. A variable is considered
/// Shared if it was assigned from a Shared constructor call, or from another variable
/// that is Shared. This avoids relying on get_type() which may transparently unwrap Shared.
fn build_shared_vars(parent_fn: &Function) -> OrderedHashMap<String, Arc<CType>> {
    let mut shared_vars: OrderedHashMap<String, Arc<CType>> = OrderedHashMap::new();

    // First pass: scan microstatements for assignments
    for ms in &parent_fn.microstatements {
        if let Microstatement::Assignment { name, value, .. } = ms {
            match value.as_ref() {
                // Any FnCall that returns a Shared type or is a .clone on a Shared
                Microstatement::FnCall {
                    function,
                    args: fn_args,
                    ..
                } => {
                    let mut rt = function.rettype();
                    if let CType::Type(_, t) = rt.as_ref() {
                        rt = t.clone();
                    }
                    if let CType::Shared(inner) = rt.as_ref() {
                        shared_vars.insert(name.clone(), inner.clone());
                    }
                    // .clone on a Shared produces a Shared (deep clone), even though
                    // the inferred return type is the inner type
                    if matches!(function.kind, FnKind::CfnRealized(CfnKind::Clone))
                        && fn_args.len() == 1
                    {
                        let arg_type = fn_args[0].get_type();
                        if matches!(&*arg_type, CType::Shared(_)) {
                            if let CType::Shared(inner) = arg_type.as_ref() {
                                shared_vars.insert(name.clone(), inner.clone());
                            }
                        } else if let Microstatement::Value { representation, .. } = &fn_args[0] {
                            if let Some(inner) = shared_vars.get(representation) {
                                shared_vars.insert(name.clone(), inner.clone());
                            }
                        }
                    }
                }
                // Assigned from another variable — trace later
                Microstatement::Value { .. } => {
                    // Will be resolved in the trace pass below
                }
                _ => {}
            }
        }
    }

    // Second pass: trace variable chains to resolve indirect Shared assignments
    let mut changed = true;
    while changed {
        changed = false;
        for ms in &parent_fn.microstatements {
            if let Microstatement::Assignment { name, value, .. } = ms {
                if let Microstatement::Value { representation, .. } = value.as_ref() {
                    if representation != name
                        && !shared_vars.contains_key(name)
                        && shared_vars.contains_key(representation)
                    {
                        shared_vars.insert(
                            name.clone(),
                            shared_vars.get(representation).unwrap().clone(),
                        );
                        changed = true;
                    }
                }
            }
        }
    }

    shared_vars
}

/// Returns true if `name` is referenced anywhere within `ms` (recursively).
fn references_var(ms: &Microstatement, name: &str) -> bool {
    match ms {
        Microstatement::Value { representation, .. } => representation == name,
        Microstatement::Assignment { value, .. } => references_var(value, name),
        Microstatement::Return { value } => match value {
            Some(v) => references_var(v, name),
            None => false,
        },
        Microstatement::FnCall { args, .. } | Microstatement::VarCall { args, .. } => {
            args.iter().any(|a| references_var(a, name))
        }
        Microstatement::Array { vals, .. } => vals.iter().any(|v| references_var(v, name)),
        Microstatement::Closure { function } => function
            .microstatements
            .iter()
            .any(|m| references_var(m, name)),
        Microstatement::NativeCall { args, .. } => args.iter().any(|a| references_var(a, name)),
        Microstatement::Arg { .. } => false,
    }
}

/// Returns true if the argument `name` is used anywhere in `ms` in a position
/// that requires *owning* the value (move, mutate, deref-copy, return-by-value,
/// alias, store-in-array, or capture-by-closure). Such uses make the
/// "keep it as a borrow (`&T`) and elide the defensive clone" optimization
/// unsafe. A use purely as an `ArgKind::Ref` argument is safe because the
/// generated expression for it is a `&T` either way.
fn ref_arg_escapes(ms: &Microstatement, name: &str) -> bool {
    let is_named = |m: &Microstatement| matches!(m, Microstatement::Value { representation, .. } if representation == name);
    match ms {
        Microstatement::FnCall { function, args } => {
            let params = function.args();
            for (i, arg) in args.iter().enumerate() {
                if is_named(arg) {
                    // A direct use of the arg in call position is only safe when
                    // the corresponding parameter takes it by reference (`&T`).
                    match params.get(i).map(|p| &p.1) {
                        Some(ArgKind::Ref) => {}
                        _ => return true,
                    }
                } else if ref_arg_escapes(arg, name) {
                    return true;
                }
            }
            false
        }
        Microstatement::VarCall { args, .. } => {
            // We don't have parameter kinds for an indirect call, so be conservative.
            args.iter().any(|a| is_named(a) || ref_arg_escapes(a, name))
        }
        Microstatement::Return { value: Some(v) } => is_named(v) || ref_arg_escapes(v, name),
        Microstatement::Return { value: None } => false,
        Microstatement::Assignment { value, .. } => is_named(value) || ref_arg_escapes(value, name),
        Microstatement::Array { vals, .. } => {
            vals.iter().any(|v| is_named(v) || ref_arg_escapes(v, name))
        }
        // Conservatively treat any capture of the arg by a closure as an escape.
        Microstatement::Closure { function } => function
            .microstatements
            .iter()
            .any(|m| references_var(m, name)),
        // Conservatively treat use as a native method/property receiver/arg as an
        // escape (a method may consume its receiver, e.g. `unwrap`).
        Microstatement::NativeCall { args, .. } => {
            args.iter().any(|a| is_named(a) || ref_arg_escapes(a, name))
        }
        Microstatement::Value { .. } | Microstatement::Arg { .. } => false,
    }
}

use crate::program::liveness::type_has_movable_projection;

/// Returns true if `name` is a non-`Shared`, `ArgKind::Ref` parameter of
/// `parent_fn` that can be left as a borrow (`&T`) in the generated body instead
/// of being defensively cloned into an owned local. This requires that:
///   - its type has no movable field/element projections (see above), and
///   - no use of it requires ownership (see `ref_arg_escapes`).
/// We also restrict to non-`Shared` types to keep the (already subtle)
/// `Shared{T}` deref/locking logic untouched for now.
fn is_borrowable_ref_arg(name: &str, parent_fn: &Function) -> bool {
    let is_ref_value_arg = parent_fn.args().iter().any(|(n, k, t)| {
        n == name
            && matches!(k, ArgKind::Ref)
            && !matches!(&**t, CType::Shared(_))
            && !type_has_movable_projection(t)
    });
    is_ref_value_arg
        && !parent_fn
            .microstatements
            .iter()
            .any(|ms| ref_arg_escapes(ms, name))
}

/// Returns true if the rendered Rust type string is a primitive scalar whose
/// literal form is type-ambiguous (`{integer}`/`{float}`) without an annotation.
fn is_primitive_scalar_rtype(s: &str) -> bool {
    matches!(
        s,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
    )
}

/// Renders `t` to a Rust type string and returns it only if it is a clean
/// *single-type reference* suitable to nest inside `Option<..>`/`Vec<..>`: not
/// empty, not a borrow/`impl`/`Fn` type, and free of the punctuation that marks
/// tuples or multi-argument generics (`,` `(` `{` `#`, newlines). This guarantees
/// the constructed annotation matches the value's rendered type.
#[allow(clippy::type_complexity)]
fn clean_element_rtype(
    t: Arc<CType>,
    deps: OrderedHashMap<String, String>,
) -> Result<(Option<String>, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    let (s, deps) = typen::ctype_to_rtype(t, deps)?;
    let clean = !s.is_empty()
        && !s.starts_with('&')
        && !s.starts_with("impl")
        && !s.contains("Fn")
        && !s.contains(['{', '}', '#', '\n', ',', '(', ')']);
    Ok((if clean { Some(s) } else { None }, deps))
}

/// Builds the type annotation (`: T`, or empty) for a `let` binding of
/// `value_type`. Only the ambiguous-but-reliable shapes are annotated (see the
/// call site): primitive scalars, `Option<T>` (an `Either` with a `void`
/// variant), and `Vec<T>` (arrays). The element type must render as a clean
/// reference; anything else yields no annotation.
#[allow(clippy::type_complexity)]
fn let_binding_annotation(
    value_type: Arc<CType>,
    out: OrderedHashMap<String, String>,
    mut deps: OrderedHashMap<String, String>,
) -> Result<
    (
        String,
        OrderedHashMap<String, String>,
        OrderedHashMap<String, String>,
    ),
    Box<dyn std::error::Error>,
> {
    // Unwrap named `Type`/`Group` wrappers to the structural type the value
    // actually renders as (e.g. `Maybe{i64}` -> `Either([i64, void])`).
    let mut structural = value_type;
    while let CType::Type(_, inner) | CType::Group(inner) = &*structural {
        structural = inner.clone();
    }
    let anno = match &*structural {
        CType::Array(inner) => {
            let (el, d) = clean_element_rtype(inner.clone(), deps)?;
            deps = d;
            el.map(|s| format!(": Vec<{s}>"))
        }
        // `Option<T>`: an `Either` of exactly two variants, one of which is
        // `void`. The other variant is the payload `T`.
        CType::Either(variants, _)
            if variants.len() == 2 && variants.iter().any(|v| matches!(&**v, CType::Void)) =>
        {
            let payload = variants
                .iter()
                .find(|v| !matches!(&***v, CType::Void))
                .cloned();
            match payload {
                Some(p) => {
                    let (el, d) = clean_element_rtype(p, deps)?;
                    deps = d;
                    el.map(|s| format!(": Option<{s}>"))
                }
                None => None,
            }
        }
        _ => {
            let (s, d) = typen::ctype_to_rtype(structural.clone(), deps)?;
            deps = d;
            if is_primitive_scalar_rtype(&s) {
                Some(format!(": {s}"))
            } else {
                None
            }
        }
    };
    Ok((anno.unwrap_or_default(), out, deps))
}

/// Returns true if `t` is the alan `string` type (which lowers to Rust `String`),
/// after unwrapping `Type`/`Group` wrappers. A borrowed `string` is rendered as
/// `&str` and "make an owned copy" (`clone`/defensive clone) is `.to_string()`.
fn is_string_type(t: &CType) -> bool {
    match t {
        CType::Type(n, inner) => n == "string" || is_string_type(inner),
        CType::Group(inner) => is_string_type(inner),
        CType::Binds(n, _) => matches!(&**n, CType::TString(s) if s == "String"),
        _ => false,
    }
}

/// Serialize a `string`-typed value's representation to an owned `String`.
/// A `string` result is normally normalized to an owned `String` by appending
/// `.to_string()` (the native expression may yield a borrowed `&str` -- e.g. a
/// string literal or a `&str`-returning method). When the expression already
/// produces an owned `String` (a `format!(...)`, which is how every `string`
/// conversion/`concat` is bound), that `.to_string()` is a redundant clone, so
/// emit the expression as-is.
fn owned_string_repr(representation: &str) -> String {
    if representation.starts_with("format!(") {
        representation.to_string()
    } else {
        format!("{representation}.to_string()")
    }
}

/// If `arg` is a string *literal* (a `Value` whose representation is a quoted
/// string rather than an identifier), return that literal. It is already a
/// `&'static str`, so it can be passed to a borrowed `&str` parameter directly
/// instead of through the allocate-then-borrow `&"...".to_string()`.
fn string_literal_arg(arg: &Microstatement) -> Option<String> {
    match arg {
        Microstatement::Value {
            representation,
            typen,
        } if is_string_type(typen) && representation.starts_with('"') => {
            Some(representation.clone())
        }
        _ => None,
    }
}

fn render_arg(
    a: &str,
    arg_is_shared: bool,
    needs_deref: bool,
    arg_kind: &ArgKind,
    parent_fn: &Function,
) -> String {
    match arg_kind {
        ArgKind::Mut => {
            if arg_is_shared {
                format!("&mut (*({a}).write().unwrap())")
            } else {
                let mut prefix = "&mut ";
                for (name, kind, _) in &parent_fn.args() {
                    if name == a {
                        if let ArgKind::Mut = kind {
                            prefix = "";
                        }
                    }
                }
                format!("{prefix}{a}")
            }
        }
        ArgKind::Ref | ArgKind::Deref => {
            if needs_deref {
                format!("&(*({a}).read().unwrap())")
            } else if is_borrowable_ref_arg(a, parent_fn) {
                // `a` is already a `&T` binding (its defensive clone was elided),
                // so pass it straight through rather than re-borrowing it.
                a.to_string()
            } else {
                format!("&{a}")
            }
        }
        ArgKind::Own => {
            if needs_deref {
                format!("(*({a}).read().unwrap()).clone()")
            } else if arg_is_shared {
                format!("{}.clone()", a)
            } else {
                a.to_string()
            }
        }
    }
}

/// Render a microstatement that is expected to be a no-argument closure as a
/// bare Rust block (`{ ... }`), suitable for inlining into `if`/`while` control
/// flow. This replaces fragile `replacen("|| {", "{", 1)` string surgery with a
/// structural render that mirrors the `Microstatement::Closure` arm. Falls back
/// to rendering the value and stripping the closure prefix for any non-closure
/// or argument-bearing input, preserving the previous behavior exactly.
#[allow(clippy::type_complexity)]
fn render_inline_block(
    microstatement: &Microstatement,
    parent_fn: &Function,
    shared_vars: &OrderedHashMap<String, Arc<CType>>,
    scope: &Scope,
    mut out: OrderedHashMap<String, String>,
    mut deps: OrderedHashMap<String, String>,
) -> Result<
    (
        String,
        OrderedHashMap<String, String>,
        OrderedHashMap<String, String>,
    ),
    Box<dyn std::error::Error>,
> {
    // An inlined control-flow block (`if`/`while` body) executes conditionally
    // or repeatedly, so the enclosing statement's last-use reasoning does not
    // hold inside it: disable the move optimization here.
    let _untrusted = UntrustedGuard::new();
    if let Microstatement::Closure { function } = microstatement {
        if function.args().is_empty() {
            let mut inner_statements = Vec::new();
            for ms in &function.microstatements {
                let (val, o, d) =
                    from_microstatement(ms, parent_fn, shared_vars, scope, out, deps)?;
                out = o;
                deps = d;
                inner_statements.push(val);
            }
            return Ok((
                format!(
                    "{{\n        {};\n    }}",
                    inner_statements.join(";\n        ")
                ),
                out,
                deps,
            ));
        }
    }
    // Fallback: render normally and strip the closure prefix textually.
    let (val, o, d) =
        from_microstatement(microstatement, parent_fn, shared_vars, scope, out, deps)?;
    Ok((val.replacen("|| {", "{", 1), o, d))
}

#[allow(clippy::type_complexity)]
pub fn from_microstatement(
    microstatement: &Microstatement,
    parent_fn: &Function,
    shared_vars: &OrderedHashMap<String, Arc<CType>>,
    scope: &Scope,
    mut out: OrderedHashMap<String, String>,
    mut deps: OrderedHashMap<String, String>,
) -> Result<
    (
        String,
        OrderedHashMap<String, String>,
        OrderedHashMap<String, String>,
    ),
    Box<dyn std::error::Error>,
> {
    match microstatement {
        Microstatement::Arg { name, kind, typen } => {
            // TODO: Update the serialization logic to understand values vs references so we can
            // eliminate this useless (and harmful for mutable references) clone
            if let CType::Function { .. } = &**typen {
                Ok(("".to_string(), out, deps))
            } else {
                match &kind {
                    ArgKind::Mut => Ok(("".to_string(), out, deps)), // We actively want to mutate the argument, don't
                    // alias it
                    ArgKind::Own => Ok(("".to_string(), out, deps)), // We already own the value
                    ArgKind::Ref => {
                        if param_name_promoted(parent_fn, name) {
                            // The parameter is taken by value (`mut name: T`), so we
                            // already own it — no defensive clone, and the `mut` binding
                            // provides the mutable owned local the clone used to.
                            Ok(("".to_string(), out, deps))
                        } else if is_borrowable_ref_arg(name, parent_fn) {
                            // The argument is only ever used by reference, so keep it as the
                            // incoming `&T` borrow instead of defensively cloning it into an
                            // owned local. `render_arg` knows to pass it through directly.
                            Ok(("".to_string(), out, deps))
                        } else if is_string_type(typen) {
                            // A borrowed `string` is `&str`; `.to_string()` (not
                            // `.clone()`, which would stay `&str`) gives the owned
                            // `String` copy the body works on.
                            Ok((format!("let mut {name} = {name}.to_string()"), out, deps))
                        } else {
                            Ok((
                                format!("let mut {name} = {name}.clone()"), // TODO: not always mutable
                                out,
                                deps,
                            ))
                        }
                    } // TODO: Should these two be distinguished?
                    ArgKind::Deref => Ok((
                        format!("let mut {name} = *{name}"), // TODO: not always mutable
                        out,
                        deps,
                    )),
                }
            }
        }
        Microstatement::Assignment {
            name,
            value,
            mutable,
        } => {
            let (val, o, d) = from_microstatement(value, parent_fn, shared_vars, scope, out, deps)?;
            out = o;
            deps = d;
            let final_val = match val.strip_prefix("&mut ") {
                Some(s) => s.to_string(),
                None => val,
            };
            // Clone Shared (Arc) assignments so they can be moved into closures safely
            let value_type = value.get_type();
            let is_shared = match &*value_type {
                CType::Shared(_) => true,
                CType::Type(_, t) => matches!(&**t, CType::Shared(_)),
                _ => false,
            };
            // Also check if the assigned value is a variable that originates from Shared
            let is_shared_var = if !is_shared {
                if let Microstatement::Value { representation, .. } = value.as_ref() {
                    shared_vars.contains_key(representation)
                } else {
                    false
                }
            } else {
                false
            };
            let assigned = if is_shared || is_shared_var {
                format!("{}.clone()", final_val)
            } else {
                final_val
            };
            // Annotate the binding's type (`let x: T = ...`) so Rust does not
            // leave it ambiguous. The function-call boundary used to pin these
            // types; folding/inlining the call away can remove the only
            // constraint, leaving e.g. a `{integer}`/`{float}` literal or an
            // unbound generic (`None`, `vec![]`) un-inferable. We annotate only a
            // *fresh owned* value (a constructor/call/array result, not a bare
            // `Value` alias of another variable -- whose rendered form could be a
            // borrow `&T` that a `T` annotation would reject), and skip reference,
            // `impl`, and function (`Fn`) types for the same reason.
            // `shared_vars` also captures bindings whose RHS is a deep `.clone()`
            // of a `Shared` value (rendered as `Arc<RwLock<T>>`) even though the
            // value's `get_type()` reports the inner `T`; annotating those with the
            // inner type would mismatch the `Arc` they actually hold.
            // A bare `Value` that names a known caller variable is an *alias*
            // whose rendered form could be a borrow (`&T`); annotating it `T`
            // would be wrong. A `Value` that is a literal (numeric/string/bool)
            // still needs annotation (a float/int literal is otherwise ambiguous).
            let is_alias = matches!(
                value.as_ref(),
                Microstatement::Value { representation, .. }
                    if is_known_variable(parent_fn, representation)
            );
            let annotation =
                if is_shared || is_shared_var || shared_vars.contains_key(name) || is_alias {
                    String::new()
                } else {
                    // Annotate only the specific ambiguous-but-reliable shapes whose
                    // value renders without a type pin, constructing each string so it
                    // provably matches the rendered value (folding away the call that
                    // used to pin the type can otherwise leave these un-inferable):
                    //   - a primitive scalar (`{integer}`/`{float}` literal),
                    //   - `Option<T>` (an `Either` with a `void` variant -> `None`/
                    //     `Some(..)`), and
                    //   - `Vec<T>` (an array literal, possibly empty).
                    // `T` must itself render as a clean single-type reference. Other
                    // shapes (tuples, `Result` -- which already carries a turbofish --
                    // structs, etc.) are left un-annotated, as before.
                    let (anno, o, d) = let_binding_annotation(value_type.clone(), out, deps)?;
                    out = o;
                    deps = d;
                    anno
                };
            Ok((
                format!(
                    "let {}{}{} = {}",
                    if *mutable { "mut " } else { "" },
                    name,
                    annotation,
                    assigned
                ),
                out,
                deps,
            ))
        }
        Microstatement::Closure { function } => {
            let arg_names = function
                .args()
                .into_iter()
                .map(|(n, k, _)| match k {
                    ArgKind::Mut => format!("mut {n}"),
                    _ => n,
                })
                .collect::<Vec<String>>();
            // A closure body may run later or repeatedly, so the enclosing
            // statement's last-use reasoning does not apply: disable moves.
            let _untrusted = UntrustedGuard::new();
            let mut inner_statements = Vec::new();
            for ms in &function.microstatements {
                let (val, o, d) =
                    from_microstatement(ms, parent_fn, shared_vars, scope, out, deps)?;
                out = o;
                deps = d;
                inner_statements.push(val);
            }
            Ok((
                format!(
                    "|{}| {{\n        {};\n    }}",
                    arg_names.join(", "),
                    inner_statements.join(";\n        "),
                ),
                out,
                deps,
            ))
        }
        Microstatement::Value {
            typen,
            representation,
        } => match &**typen {
            CType::Type(n, _) if n == "string" => {
                Ok((owned_string_repr(representation), out, deps))
            }
            CType::Binds(n, _) => match &**n {
                CType::TString(s) => {
                    if s == "String" {
                        Ok((owned_string_repr(representation), out, deps))
                    } else {
                        Ok((representation.clone(), out, deps))
                    }
                }
                CType::Import(n, d) => {
                    super::register_rust_dependency(d, &mut deps);
                    match &**n {
                        CType::TString(_) => { /* Do nothing */ }
                        _ => CType::fail("Native import names must be strings"),
                    }
                    Ok((representation.clone(), out, deps))
                }
                _ => CType::fail("Bound types must be strings or rust imports"),
            },
            CType::Function(..) => {
                // We need to make sure this function we're referencing exists
                let f = scope.resolve_function_by_type(representation, typen.clone());
                let f = match f {
                    None => {
                        // If the current scope isn't the original scope for the parent function, maybe the
                        // function we're looking for is in the original scope
                        if parent_fn.origin_scope_path != scope.path {
                            let program = Program::get_program();
                            let out = match program.scope_by_file(&parent_fn.origin_scope_path) {
                                Ok(original_scope) => original_scope
                                    .resolve_function_by_type(representation, typen.clone()),
                                Err(_) => None,
                            };
                            Program::return_program(program);
                            out
                        } else {
                            None
                        }
                    }
                    f => f,
                };
                match &f {
                    None => {
                        let args = parent_fn.args();
                        for (name, _, typen) in args {
                            if &name == representation {
                                if let CType::Function(_, _) = &*typen {
                                    // TODO: Do we need better matching? The upper stage should
                                    // have taken care of this
                                    return Ok((representation.clone(), out, deps));
                                }
                            }
                        }
                        Err(format!(
                            "Somehow can't find a definition for function {representation}, {typen:?}"
                        )
                        .into())
                    }
                    Some(fun) => {
                        match &fun.kind {
                            FnKind::Normal
                            | FnKind::External(_)
                            | FnKind::Generic(..)
                            | FnKind::Derived
                            | FnKind::DerivedVariadic
                            | FnKind::Static
                            | FnKind::Cfn(..)
                            | FnKind::CfnRealized(_) => {
                                let mut arg_strs = Vec::new();
                                for arg in &fun.args() {
                                    arg_strs.push(arg.2.clone().to_callable_string());
                                }
                                // Come up with a function name that is unique so Rust doesn't choke on
                                // duplicate function names that are allowed in Alan
                                let rustname = format!("{}_{}", fun.name, arg_strs.join("_"));
                                // Make the function we need, but with the name we're
                                let res = generate(rustname.clone(), fun, scope, out, deps)?;
                                out = res.0;
                                deps = res.1;
                                if let FnKind::External(d) = &fun.kind {
                                    super::register_rust_dependency(d, &mut deps);
                                }
                                // A borrowed-`string` parameter is rendered as `&str`,
                                // but a higher-order call site that monomorphizes the
                                // element type to `String` invokes the callback with
                                // `&String` (e.g. `alan_std`'s generic buffer/array
                                // reducers). A bare `fn(&str)` path does not satisfy
                                // `Fn(&String)`, so wrap the reference in a closure: the
                                // closure's parameter types are *inferred* from the
                                // expected bound (`&String`) and the forwarded call
                                // coerces `&String` -> `&str`. For any other callback
                                // shape this is an identity wrapper.
                                let has_borrowed_string = fun.args().iter().any(|(_, k, t)| {
                                    matches!(k, ArgKind::Ref) && is_string_type(t)
                                });
                                if has_borrowed_string {
                                    let params = (0..fun.args().len())
                                        .map(|i| format!("arg{i}"))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    Ok((format!("|{params}| {rustname}({params})"), out, deps))
                                } else {
                                    Ok((rustname, out, deps))
                                }
                            }
                            FnKind::Bind(rustname)
                            | FnKind::BoundGeneric(_, rustname)
                            | FnKind::ExternalBind(rustname, _)
                            | FnKind::ExternalGeneric(_, rustname, _) => {
                                if let FnKind::ExternalGeneric(_, _, d)
                                | FnKind::ExternalBind(_, d) = &fun.kind
                                {
                                    super::register_rust_dependency(d, &mut deps);
                                }
                                Ok((rustname.clone(), out, deps))
                            }
                        }
                    }
                }
            }
            _ => Ok((representation.clone(), out, deps)),
        },
        Microstatement::Array { vals, .. } => {
            let mut val_representations = Vec::new();
            for val in vals {
                let (rep, o, d) =
                    from_microstatement(val, parent_fn, shared_vars, scope, out, deps)?;
                val_representations.push(rep);
                out = o;
                deps = d;
            }
            Ok((
                format!("vec![{}]", val_representations.join(", ")),
                out,
                deps,
            ))
        }
        Microstatement::NativeCall {
            typen,
            kind,
            name,
            args,
        } => {
            // Serialize a native construct (function/method/property/operator/cast)
            // in the codegen layer; `kind` selects the surface form. For the
            // receiver-based forms `args[0]` is the receiver. A `Value` argument is
            // emitted by its raw representation (a parameter name or an inlined
            // literal) so the native construct operates on the value directly.
            // Non-`Value` arguments (only possible once these are inlined) render
            // normally.
            let mut rendered = Vec::new();
            for a in args {
                let s = if let Microstatement::Value { representation, .. } = a {
                    representation.clone()
                } else {
                    let (s, o, d) =
                        from_microstatement(a, parent_fn, shared_vars, scope, out, deps)?;
                    out = o;
                    deps = d;
                    s
                };
                rendered.push(s);
            }
            let call = match kind {
                NativeCallKind::Function => format!("{}({})", name, rendered.join(", ")),
                NativeCallKind::Method => {
                    let (recv, rest) = rendered
                        .split_first()
                        .expect("a Method NativeCall always has a receiver argument");
                    format!("{}.{}({})", recv, name, rest.join(", "))
                }
                NativeCallKind::Property => {
                    let (recv, _) = rendered
                        .split_first()
                        .expect("a Property NativeCall always has a receiver argument");
                    format!("{}.{}", recv, name)
                }
                NativeCallKind::Infix => {
                    // `(lhs op rhs)` — exactly two arguments (enforced at bind realization).
                    format!("({} {} {})", rendered[0], name, rendered[1])
                }
                NativeCallKind::Prefix => format!("({} {})", name, rendered[0]),
                NativeCallKind::Cast => format!("({} as {})", rendered[0], name),
            };
            // Apply the result type's serialization (e.g. wrapping a `&str` result
            // in `.to_string()` for a `string` return) by rendering the assembled
            // call through the `Value` handler with the call's return type.
            from_microstatement(
                &Microstatement::Value {
                    typen: typen.clone(),
                    representation: call,
                },
                parent_fn,
                shared_vars,
                scope,
                out,
                deps,
            )
        }
        Microstatement::FnCall { function, args } => {
            // Hackery to inline `if` calls *if* it's safe to do so.
            if let FnKind::Bind(fname) = &function.kind {
                if fname == "ifstatementhack" {
                    let res =
                        from_microstatement(&args[0], parent_fn, shared_vars, scope, out, deps)?;
                    let conditional = res.0;
                    out = res.1;
                    deps = res.2;
                    let res =
                        render_inline_block(&args[1], parent_fn, shared_vars, scope, out, deps)?;
                    let successblock = res.0;
                    out = res.1;
                    deps = res.2;
                    return Ok((
                        format!("if {conditional} {successblock}").to_string(),
                        out,
                        deps,
                    ));
                } else if fname == "ifelsestatementhack" {
                    let res =
                        from_microstatement(&args[0], parent_fn, shared_vars, scope, out, deps)?;
                    let conditional = res.0;
                    out = res.1;
                    deps = res.2;
                    let res =
                        render_inline_block(&args[1], parent_fn, shared_vars, scope, out, deps)?;
                    let successblock = res.0;
                    out = res.1;
                    deps = res.2;
                    let res =
                        render_inline_block(&args[2], parent_fn, shared_vars, scope, out, deps)?;
                    let failblock = res.0;
                    out = res.1;
                    deps = res.2;
                    return Ok((
                        format!("if {conditional} {successblock} else {failblock}").to_string(),
                        out,
                        deps,
                    ));
                } else if fname == "whileloophack" {
                    // The condition closure ends in `return <expr>;`. We flatten it into a
                    // block-expression (`{ setup; <expr> }`) by splitting off the trailing
                    // return. This remains string-based because reproducing the exact
                    // whitespace structurally is not worth the churn; the loop body, however,
                    // is rendered structurally via `render_inline_block` below.
                    let res =
                        from_microstatement(&args[0], parent_fn, shared_vars, scope, out, deps)?;
                    let conditionalparts = res.0.split("return").collect::<Vec<&str>>();
                    let conditional = [conditionalparts[0], &conditionalparts[1].replace(";", "")]
                        .join("")
                        .replacen("|| {", "{", 1);
                    out = res.1;
                    deps = res.2;
                    let res =
                        render_inline_block(&args[1], parent_fn, shared_vars, scope, out, deps)?;
                    let loopblock = res.0;
                    out = res.1;
                    deps = res.2;
                    return Ok((
                        format!("while {conditional} {loopblock}").to_string(),
                        out,
                        deps,
                    ));
                }
            }
            match &function.kind {
                FnKind::Generic(..)
                | FnKind::BoundGeneric(..)
                | FnKind::ExternalGeneric(..)
                | FnKind::Cfn(..) => {
                    Err("Generic functions should have been resolved before reaching here".into())
                }
                FnKind::Normal | FnKind::External(_) => {
                    // If this function is called from exactly one site and is a single
                    // `return <expr>`, inline it here by substituting its parameters for our
                    // argument expressions, so the function itself is never emitted.
                    // Skip inlining when any argument is a `Shared{T}` value. The
                    // inlined body may use a parameter in a position (e.g. a native
                    // call argument) that renders it raw, but a `Shared` argument
                    // requires the deref/lock conversion (`&(*x.read().unwrap())`)
                    // that the function-call boundary applies via `render_arg`.
                    // Reproducing that structurally is involved, and `Shared`
                    // arguments are rare, so we conservatively keep the real call.
                    // `shared_vars` also captures variables whose `get_type()` hides
                    // their `Shared`-ness (e.g. a deep clone), which a type check
                    // alone would miss.
                    let any_arg_shared = args.iter().any(|arg| {
                        if matches!(&*arg.get_type(), CType::Shared(_)) {
                            return true;
                        }
                        match arg {
                            Microstatement::Value { representation, .. } => {
                                shared_vars.contains_key(representation)
                            }
                            Microstatement::FnCall { function, .. } => {
                                shared_vars.contains_key(&function.name)
                            }
                            _ => false,
                        }
                    });
                    if !any_arg_shared
                        && matches!(function.kind, FnKind::Normal)
                        && crate::program::inline::is_inline_target(
                            &crate::program::inline::fn_identity(function),
                        )
                        && inline_consumes_are_safe(function, args, parent_fn)
                    {
                        // Single `return <expr>` body: inline as a pure expression.
                        if let Some(subs) =
                            crate::program::inline::build_inline_substitution(function, args)
                        {
                            if let Some(expr) = crate::program::inline::single_return_expr(function)
                            {
                                let inlined = crate::program::inline::substitute(expr, &subs);
                                // The inlined expression splices the callee's body
                                // into this statement; the enclosing per-statement
                                // last-use reasoning does not model its internal
                                // ownership, so render it with moves disabled.
                                let _untrusted = UntrustedGuard::new();
                                return from_microstatement(
                                    &inlined,
                                    parent_fn,
                                    shared_vars,
                                    scope,
                                    out,
                                    deps,
                                );
                            }
                        }
                        // Multi-statement body: inline as a block expression (which also lets
                        // values drop early). Skip when the callee has `Shared` locals, whose
                        // deref/clone rendering depends on a `shared_vars` map we don't thread in.
                        if build_shared_vars(function).is_empty() {
                            if let Some((stmts, tail)) =
                                crate::program::inline::build_multi_inline(function, args)
                            {
                                // The inlined block splices the callee's body into
                                // this statement; render it with moves disabled.
                                let _untrusted = UntrustedGuard::new();
                                let mut block = "{\n".to_string();
                                for s in &stmts {
                                    let (rendered, o, d) = from_microstatement(
                                        s,
                                        parent_fn,
                                        shared_vars,
                                        scope,
                                        out,
                                        deps,
                                    )?;
                                    out = o;
                                    deps = d;
                                    block.push_str(&format!("        {rendered};\n"));
                                }
                                let (tail_str, o, d) = from_microstatement(
                                    &tail,
                                    parent_fn,
                                    shared_vars,
                                    scope,
                                    out,
                                    deps,
                                )?;
                                out = o;
                                deps = d;
                                // Match the `Return` handler: the tail is a value, not a `&mut`.
                                let tail_str = match tail_str.strip_prefix("&mut ") {
                                    Some(s) => s.to_string(),
                                    None => tail_str,
                                };
                                block.push_str(&format!("        {tail_str}\n    }}"));
                                return Ok((block, out, deps));
                            }
                        }
                    }
                    let (_, o, d) = typen::generate(function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut arg_strs = Vec::new();
                    for arg in &function.args() {
                        arg_strs.push(arg.2.clone().to_callable_string());
                    }
                    let rustname = format!("{}_{}", function.name, arg_strs.join("_")).to_string();
                    let res = generate(rustname.clone(), function, scope, out, deps)?;
                    out = res.0;
                    deps = res.1;
                    let mut argstrs = Vec::new();
                    for (i, arg) in args.iter().enumerate() {
                        let (a, o, d) =
                            from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
                        out = o;
                        deps = d;
                        let arg_type = arg.get_type();
                        // Check if this arg variable is a Shared type
                        let arg_name = match arg {
                            Microstatement::Value { representation, .. } => representation.clone(),
                            Microstatement::FnCall {
                                function: arg_fn, ..
                            } => arg_fn.name.clone(),
                            _ => String::new(),
                        };
                        let arg_is_shared = matches!(&*arg_type, CType::Shared(_))
                            || (!arg_name.is_empty() && shared_vars.contains_key(&arg_name));
                        let param_is_shared = match &*function.args()[i].2 {
                            CType::Shared(_) => true,
                            CType::Type(_, t) => matches!(&**t, CType::Shared(_)),
                            _ => false,
                        };
                        let needs_deref = arg_is_shared && !param_is_shared;
                        match &*arg_type {
                            CType::Function(..) => argstrs.push(a.to_string()),
                            // A string literal passed to a borrowed `&str`
                            // parameter (one we render, i.e. `Ref` and not
                            // promoted to an owned `String`) is already a
                            // `&'static str`: emit it directly rather than
                            // `&"...".to_string()` (allocate then borrow).
                            _ if matches!(function.args()[i].1, ArgKind::Ref)
                                && !promote_param_to_own(function, i)
                                && string_literal_arg(arg).is_some() =>
                            {
                                argstrs.push(string_literal_arg(arg).unwrap());
                            }
                            // A parameter taken by value -- either promoted (see
                            // `promote_param_to_own`) or already declared `Own` --
                            // needs an owned argument. `Shared`/deref arguments keep
                            // the existing owning conversion. Otherwise move the
                            // value when it is at its last use here, else clone to
                            // keep it available (clone-protected ownership, which
                            // also makes inlining consuming-parameter wrappers safe).
                            _ if (promote_param_to_own(function, i)
                                || matches!(function.args()[i].1, ArgKind::Own))
                                && !arg_is_shared
                                && !needs_deref =>
                            {
                                let owned = match arg {
                                    // A known variable that cannot be moved here
                                    // (still live, borrowed, or in an untrusted
                                    // scope) must be cloned to stay available.
                                    Microstatement::Value { representation, .. }
                                        if is_known_variable(parent_fn, representation)
                                            && !caller_can_move(parent_fn, representation) =>
                                    {
                                        format!("{a}.clone()")
                                    }
                                    // A non-`Value` argument is a fresh temporary
                                    // (already owned), a literal `Value`, or a
                                    // movable variable: pass/move it directly.
                                    _ => a.to_string(),
                                };
                                argstrs.push(owned);
                            }
                            _ => argstrs.push(render_arg(
                                &a,
                                arg_is_shared,
                                needs_deref,
                                &function.args()[i].1,
                                parent_fn,
                            )),
                        }
                    }
                    if let FnKind::External(d) = &function.kind {
                        super::register_rust_dependency(d, &mut deps);
                    }
                    Ok((
                        format!("{}({})", rustname, argstrs.join(", ")).to_string(),
                        out,
                        deps,
                    ))
                }
                FnKind::Bind(rustname) | FnKind::ExternalBind(rustname, _) => {
                    let mut argstrs = Vec::new();
                    for (i, arg) in args.iter().enumerate() {
                        let (a, o, d) =
                            from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
                        out = o;
                        deps = d;
                        let arg_type = arg.get_type();
                        let arg_name = match arg {
                            Microstatement::Value { representation, .. } => representation.clone(),
                            Microstatement::FnCall {
                                function: arg_fn, ..
                            } => arg_fn.name.clone(),
                            _ => String::new(),
                        };
                        let arg_is_shared = matches!(&*arg_type, CType::Shared(_))
                            || (!arg_name.is_empty() && shared_vars.contains_key(&arg_name));
                        let param_is_shared = match &*function.args()[i].2 {
                            CType::Shared(_) => true,
                            CType::Type(_, t) => matches!(&**t, CType::Shared(_)),
                            _ => false,
                        };
                        let needs_deref = arg_is_shared && !param_is_shared;
                        match &*arg_type {
                            CType::Function(..) => argstrs.push(a.to_string()),
                            _ => argstrs.push(render_arg(
                                &a,
                                arg_is_shared,
                                needs_deref,
                                &function.args()[i].1,
                                parent_fn,
                            )),
                        }
                    }
                    if let FnKind::ExternalBind(_, d) = &function.kind {
                        super::register_rust_dependency(d, &mut deps);
                    }
                    Ok((
                        format!("{}({})", rustname, argstrs.join(", ")).to_string(),
                        out,
                        deps,
                    ))
                }
                FnKind::Static => {
                    // Static functions just replace the function call with their static value
                    // calculated at compile time.
                    match &function.microstatements[0] {
                        Microstatement::Value {
                            representation,
                            typen,
                        } => match &**typen {
                            CType::Type(n, _) if n == "string" => Ok((
                                format!("{representation}.to_string()").to_string(),
                                out,
                                deps,
                            )),
                            CType::Binds(n, _) => match &**n {
                                CType::TString(s) => {
                                    if s == "String" {
                                        Ok((
                                            format!("{representation}.to_string()").to_string(),
                                            out,
                                            deps,
                                        ))
                                    } else {
                                        Ok((representation.clone(), out, deps))
                                    }
                                }
                                CType::Import(n, d) => {
                                    super::register_rust_dependency(d, &mut deps);
                                    match &**n {
                                        CType::TString(_) => { /* Do nothing */ }
                                        _ => CType::fail("Native import names must be strings"),
                                    }
                                    Ok((representation.clone(), out, deps))
                                }
                                _ => CType::fail("Bound types must be strings or rust imports"),
                            },
                            _ => Ok((representation.clone(), out, deps)),
                        },
                        _ => unreachable!(),
                    }
                }
                FnKind::CfnRealized(CfnKind::Clone) => {
                    // Generate .clone() for the argument. Handles Shared{T} specially.
                    let (_, o, d) = typen::generate(function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) =
                            from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
                        out = o;
                        deps = d;
                        let arg_type = arg.get_type();
                        match &*arg_type {
                            CType::Function(..) => argstrs.push(a.to_string()),
                            _ => argstrs.push(a.to_string()),
                        }
                    }
                    let arg_type = args[0].get_type();
                    let is_shared = matches!(&*arg_type, CType::Shared(_))
                        || (!matches!(&*arg_type, CType::Function(..))
                            && match &args[0] {
                                Microstatement::Value { representation, .. } => {
                                    shared_vars.contains_key(representation)
                                }
                                Microstatement::FnCall { function, .. } => {
                                    matches!(&*function.rettype(), CType::Shared(_))
                                }
                                _ => false,
                            });
                    match is_shared {
                        true => {
                            // Deep clone: unwrap Arc<RwLock<T>>, clone inner T, rewrap
                            Ok((
                                 format!("std::sync::Arc::new(std::sync::RwLock::new(({}).read().unwrap().clone()))", argstrs[0]),
                                out,
                                deps,
                            ))
                        }
                        // A borrowed `string` is `&str`, whose `.clone()` is still
                        // `&str`; `.to_string()` produces the owned `String` copy
                        // that `clone` is meant to yield (and also works for an
                        // owned `String`/`&String` argument).
                        _ if is_string_type(&arg_type) => {
                            Ok((format!("{}.to_string()", argstrs[0]), out, deps))
                        }
                        _ => Ok((format!("{}.clone()", argstrs[0]), out, deps)),
                    }
                }
                FnKind::Derived | FnKind::DerivedVariadic => {
                    // The initial work to get the values to construct the type is the same as
                    // with bound functions, though.
                    let (_, o, d) = typen::generate(function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) =
                            from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
                        out = o;
                        deps = d;
                        let arg_type = arg.get_type();
                        if matches!(&*arg_type, CType::Shared(_)) {
                            argstrs.push(format!("{}.clone()", a));
                        } else {
                            argstrs.push(a.to_string());
                        }
                    }
                    // The behavior of the generated code depends on the structure of the
                    // return type and the input types. We also do some logic based on the name
                    // of the function.
                    // 1) If the name of the function matches the name of return type, it's a
                    //    constructor function, and will interpret the arguments in different
                    //    ways:
                    //    a) If the return type is a Buffer, the arg count must be either the
                    //       size of the buffer with all args having the same type *or* it must
                    //       be exactly 1, with the arg matching the buffer's primary type that
                    //       the buffer will be filled with. In case someone creates a
                    //       one-element buffer, well, those two definitions are the same so it
                    //       will use the first implementation (as it will be faster).
                    //    b) If the return type is an Array, any number of values can be
                    //       provided and it will pre-populate the array with those values.
                    //    c) If the return type is an Either, it will expect only *one*
                    //       argument, and fail otherwise. The argument needs to be one of the
                    //       possibilities, which it will then put into the correct enum. An
                    //       earlier stage of the compiler should have generated function
                    //       definitions for each type in the Either.
                    //    d) If the return type is a tuple type, each argument of the function
                    //       needs to match, in the same order, the tuple's types. It doesn't
                    //       matter if the type itself has fields with names, those are ignored
                    //       and they're all turned into tuples.
                    //    e) If the return type is a group type or "type" type, it's unwrapped
                    //       and checked if it is one of the types above.
                    //    f) If it's any other type, it's a compiler error. There's no way to
                    //       derive an implementation for them that would be sensical.
                    // 2) If the input type is a tuple and the name of the function matches the
                    //    name of a field in the tuple, it's an accessor function.
                    // 3) If the input type is an either and the name of the function matches
                    //    the name of a sub-type, it returns a Maybe{T} for the type in
                    //    question. (This conflicts with (1) so it's checked first.)
                    if function.args().len() == 1 && !argstrs.is_empty() {
                        // Auto-deref Shared{T}: unwrap Arc<RwLock<T>> before accessing inner properties
                        let arg0_type = args[0].get_type();
                        let mut is_shared = false;
                        // Check actual argument type for Shared
                        {
                            let mut arg0_inner = &*arg0_type.clone().degroup();
                            while matches!(
                                arg0_inner,
                                CType::Type(..) | CType::Group(_) | CType::Shared(_)
                            ) {
                                if matches!(arg0_inner, CType::Shared(_)) {
                                    is_shared = true;
                                }
                                arg0_inner = match arg0_inner {
                                    CType::Type(_, t) => t,
                                    CType::Group(t) => t,
                                    CType::Shared(t) => t,
                                    _ => arg0_inner,
                                };
                            }
                        }
                        // Check if arg0 type is Shared (from type info)
                        is_shared = is_shared || matches!(&*args[0].get_type(), CType::Shared(_));
                        // Also check if the variable is a Shared by tracing to origin
                        if !is_shared {
                            let arg_name = match &args[0] {
                                Microstatement::FnCall {
                                    function: arg_fn, ..
                                } => arg_fn.name.clone(),
                                Microstatement::Value { representation, .. } => {
                                    representation.clone()
                                }
                                _ => String::new(),
                            };
                            if !arg_name.is_empty() {
                                is_shared = shared_vars.contains_key(&arg_name);
                            }
                        }
                        let shared_prefix = if is_shared {
                            format!("({}).write().unwrap()", argstrs[0])
                        } else {
                            argstrs[0].clone()
                        };
                        let mut input_type = &function.args()[0].2;
                        while matches!(
                            &**input_type,
                            CType::Type(..) | CType::Group(_) | CType::Shared(_)
                        ) {
                            input_type = match &**input_type {
                                CType::Type(_, t) => t,
                                CType::Group(t) => t,
                                CType::Shared(t) => t,
                                _ => input_type,
                            };
                        }
                        match &**input_type {
                            CType::Tuple(ts, _) => {
                                // Short-circuit for direct `<N>` function calls (which can only be
                                // generated by the internals of the compiler)
                                if let Ok(i) = function.name.parse::<i64>() {
                                    let clone_suffix = "";
                                    return Ok((
                                        format!("{}.{}{}", shared_prefix, i, clone_suffix),
                                        out,
                                        deps,
                                    ));
                                }
                                let accessor_field = ts
                                    .iter()
                                    .filter(|t1| match &***t1 {
                                        CType::Field(_, t2) => !matches!(
                                            &**t2,
                                            CType::TString(_)
                                                | CType::Int(_)
                                                | CType::Float(_)
                                                | CType::Bool(_)
                                        ),
                                        CType::TString(_)
                                        | CType::Int(_)
                                        | CType::Float(_)
                                        | CType::Bool(_) => false,
                                        _ => true,
                                    })
                                    .enumerate()
                                    .find(|(_, t)| match &***t {
                                        CType::Field(n, _) => *n == function.name,
                                        _ => false,
                                    });
                                if let Some((i, _)) = accessor_field {
                                    let clone_suffix = "";
                                    return Ok((
                                        format!("{}.{}{}", shared_prefix, i, clone_suffix),
                                        out,
                                        deps,
                                    ));
                                }
                            }
                            CType::Buffer(_, s) => {
                                // Similarly short-circuit for direct `<N>` function calls
                                if let Ok(i) = function.name.parse::<i64>() {
                                    if let CType::Int(l) = **s {
                                        if i128::from(i) < l {
                                            return Ok((
                                                format!(
                                                    "{}[{}]{}",
                                                    shared_prefix,
                                                    i,
                                                    if is_shared { ".clone()" } else { "" }
                                                ),
                                                out,
                                                deps,
                                            ));
                                        }
                                    }
                                }
                            }
                            CType::Field(..) => {
                                return Ok((
                                    format!(
                                        "{}.0{}",
                                        shared_prefix,
                                        if is_shared { ".clone()" } else { "" }
                                    ),
                                    out,
                                    deps,
                                ));
                            }
                            CType::Either(ts, _) => {
                                let enum_type = function.args()[0].2.clone().degroup();
                                let enum_name = enum_type.to_callable_string();
                                let accessor_field = ts.iter().find(|t| match &***t {
                                    CType::Field(n, _) => *n == function.name,
                                    CType::Type(n, _) => *n == function.name,
                                    _ => false,
                                });
                                if accessor_field.is_some() {
                                    if ts.len() == 2 {
                                        if let CType::Void = &*ts[1] {
                                            return Ok((shared_prefix, out, deps));
                                        } else if let CType::Type(name, _) = &*ts[1] {
                                            if name == "Error" {
                                                if function.name == "Error" {
                                                    return Ok((format!("(match &{} {{ Err(e) => Some(e.clone()), _ => None }})", shared_prefix), out, deps));
                                                } else {
                                                    return Ok((format!("(match &{} {{ Ok(v) => Some(v.clone()), _ => None }})", shared_prefix), out, deps));
                                                }
                                            }
                                        }
                                    }
                                    return Ok((
                                        format!(
                                            "(match &{} {{ {}::{}(v) => Some(v.clone()), _ => None }})",
                                            shared_prefix, enum_name, function.name
                                        ),
                                        out,
                                        deps,
                                    ));
                                }
                            }
                            _ => {}
                        }
                    } else if function.args().is_empty() {
                        let inner_ret_type = function.rettype().degroup();
                        let inner_ret_type = match &*inner_ret_type {
                            CType::Field(_, t) => t.clone(),
                            CType::Type(_, t) => t.clone(),
                            _ => inner_ret_type,
                        };
                        if let CType::Either(_, _) = &*inner_ret_type {
                            return Ok(("None".to_string(), out, deps));
                        }
                    }
                    let ret_type = function.rettype().degroup();
                    let ret_name = ret_type.clone().to_callable_string();
                    if function.name == "store" {
                        let inner_ret_type = match &*ret_type {
                            CType::Field(_, t) => t.clone(),
                            CType::Type(_, t) => t.clone(),
                            _ => ret_type,
                        };
                        match &*inner_ret_type {
                            CType::Either(ts, _) => {
                                if argstrs.len() != 2 {
                                    return Err(format!("Invalid arguments {} provided for Either re-assignment function, must be two arguments", argstrs.join(", ")).into());
                                }
                                let enum_type = function.args()[1].2.clone().degroup();
                                let enum_name = match &*enum_type {
                                    CType::Field(n, _) => Ok(n.clone()),
                                    CType::Type(n, _) => Ok(n.clone()),
                                    _ => Err(format!("Cannot generate an constructor function for {ret_name} type as the input type has no name?")),
                                }?;
                                for t in ts {
                                    let inner_type = t.clone().degroup();
                                    match &*inner_type {
                                        CType::Field(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok((
                                                            format!(
                                                                "{} = None",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "{} = Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                },
                                                                match argstrs[1]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[1],
                                                                }
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[0].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[1].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                format!(
                                                                    "{} = Err::<{}, {}>({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    },
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[1]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[1],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "{} = Ok::<{}, {}>({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    },
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[1]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[1],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{} = {}::{}({})",
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                    ret_name,
                                                    enum_name,
                                                    match argstrs[1].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[1],
                                                    },
                                                ),
                                                out,
                                                deps,
                                            ));
                                        }
                                        CType::Type(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok((
                                                            format!(
                                                                "{} = None",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "{} = Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                },
                                                                match argstrs[1]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[1],
                                                                }
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[0].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[1].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                format!(
                                                                    "{} = Err::<{}, {}>({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    },
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[1]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[1],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "{} = Ok::<{}, {}>({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    },
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[1]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[1],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{} = {}::{}({})",
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                    ret_name,
                                                    enum_name,
                                                    match argstrs[1].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[1],
                                                    },
                                                ),
                                                out,
                                                deps,
                                            ));
                                        }
                                        _ => {}
                                    }
                                }
                                return Err(format!("Cannot assign a value of the {enum_name} type as it is not part of the {ret_name} type").into());
                            }
                            _ => return Err("How did this path get triggered?".into()),
                        }
                    } else if function.name == ret_name {
                        let mut inner_ret_type = ret_type.clone();
                        while matches!(&*inner_ret_type, CType::Type(..)) {
                            inner_ret_type = match &*inner_ret_type {
                                CType::Type(_, t) => t.clone(),
                                _ => inner_ret_type,
                            };
                        }
                        match &*inner_ret_type {
                            CType::Buffer(_, s) => {
                                let size = match **s {
                                    CType::Int(s) => Ok(s as usize),
                                    _ => Err("Somehow received a buffer with a non-integer size"
                                        .to_string()),
                                }?;
                                if argstrs.len() == size {
                                    return Ok((
                                        format!(
                                            "[{}]",
                                            argstrs
                                                .iter()
                                                .map(|a| match a.strip_prefix("&mut ") {
                                                    Some(v) => v,
                                                    None => a,
                                                })
                                                .collect::<Vec<&str>>()
                                                .join(", ")
                                        ),
                                        out,
                                        deps,
                                    ));
                                } else if argstrs.len() == 1 {
                                    return Ok((
                                        format!(
                                            "[{};{}]",
                                            match argstrs[0].strip_prefix("&mut ") {
                                                Some(v) => v,
                                                None => &argstrs[0],
                                            },
                                            size
                                        ),
                                        out,
                                        deps,
                                    ));
                                } else {
                                    return Err(format!("Invalid arguments {} provided for Buffer constructor function, must be either 1 element to fill, or the full size of the buffer", argstrs.join(", ")).into());
                                }
                            }
                            CType::Array(_) => {
                                return Ok((
                                    format!(
                                        "vec![{}]",
                                        argstrs
                                            .iter()
                                            .map(|a| match a.strip_prefix("&mut ") {
                                                Some(v) => v.to_string(),
                                                None => a.clone(),
                                            })
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                    ),
                                    out,
                                    deps,
                                ));
                            }
                            CType::Shared(_) => {
                                return Ok((
                                    format!(
                                        "std::sync::Arc::new(std::sync::RwLock::new({}))",
                                        match argstrs[0].strip_prefix("&mut ") {
                                            Some(v) => v.to_string(),
                                            None => argstrs[0].clone(),
                                        }
                                    ),
                                    out,
                                    deps,
                                ));
                            }
                            CType::Either(ts, _) => {
                                if argstrs.len() > 1 {
                                    return Err(format!("Invalid arguments {} provided for Either constructor function, must be zero or one argument", argstrs.join(", ")).into());
                                }
                                let enum_type = match &function.args().first() {
                                    Some(t) => t.2.clone().degroup(),
                                    None => Arc::new(CType::Void),
                                };
                                let enum_name = match &*enum_type {
                                    CType::Field(n, _) => n.clone(),
                                    CType::Type(n, _) => n.clone(),
                                    _ => enum_type.clone().to_callable_string(),
                                };
                                // Check for parent constructor: single argument whose type is an
                                // Either containing a superset of the child's variants
                                if argstrs.len() == 1 {
                                    let single_arg_type = match &function.args().first() {
                                        Some(t) => t.2.clone().degroup(),
                                        None => Arc::new(CType::Void),
                                    };
                                    let parent_type_name =
                                        single_arg_type.clone().to_callable_string();
                                    if let CType::Either(parent_variants, _) = &*single_arg_type {
                                        // Find which parent variants match child variants
                                        let mut matched_indices: Vec<usize> = Vec::new();
                                        for (idx, pv) in parent_variants.iter().enumerate() {
                                            let parent_key =
                                                pv.clone().degroup().to_callable_string();
                                            for cv in ts {
                                                if cv.clone().degroup().to_callable_string()
                                                    == parent_key
                                                {
                                                    matched_indices.push(idx);
                                                    break;
                                                }
                                            }
                                        }
                                        if !matched_indices.is_empty()
                                            && matched_indices.len() < parent_variants.len()
                                        {
                                            let parent_arg = match argstrs[0].strip_prefix("&mut ")
                                            {
                                                Some(s) => s.to_string(),
                                                None => argstrs[0].clone(),
                                            };
                                            // Generate match expression: Some(Child::Variant(v)) for matched, None for excluded
                                            let mut arms = Vec::new();
                                            for (idx, pv) in parent_variants.iter().enumerate() {
                                                let variant_name =
                                                    pv.clone().degroup().to_callable_string();
                                                if matched_indices.contains(&idx) {
                                                    arms.push(format!(
                                                        "{}::{}(v) => Some({}::{}(v))",
                                                        parent_type_name,
                                                        variant_name,
                                                        function.name,
                                                        variant_name
                                                    ));
                                                } else {
                                                    arms.push(format!(
                                                        "{}::{}(_) => None",
                                                        parent_type_name, variant_name
                                                    ));
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "match {} {{\n    {}\n}}",
                                                    parent_arg,
                                                    arms.join(",\n    ")
                                                ),
                                                out,
                                                deps,
                                            ));
                                        }
                                    }
                                }
                                for t in ts {
                                    let inner_type = t.clone().degroup();
                                    match &*inner_type {
                                        CType::Field(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok(("None".to_string(), out, deps));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[0].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[1].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                format!(
                                                                    "Err::<{}, {}>({})",
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "Ok::<{}, {}>({})",
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{}::{}({})",
                                                    function.name,
                                                    enum_name,
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                ),
                                                out,
                                                deps,
                                            ));
                                        }
                                        CType::Type(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok(("None".to_string(), out, deps));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[0].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[1].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                format!(
                                                                    "Err::<{}, {}>({})",
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "Ok::<{}, {}>({})",
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{}::{}({})",
                                                    function.name,
                                                    enum_name,
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                ),
                                                out,
                                                deps,
                                            ));
                                        }
                                        CType::Binds(n, ..) => match &**n {
                                            CType::TString(s) if s == &enum_name => {
                                                // Special-casing for Option and Result mapping. TODO:
                                                // Make this more centralized
                                                if ts.len() == 2 {
                                                    if let CType::Void = &*ts[1] {
                                                        if let CType::Void = &**t {
                                                            return Ok((
                                                                "None".to_string(),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "Some({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    } else if let CType::Type(name, _) = &*ts[1] {
                                                        if name == "Error" {
                                                            let (okrustname, d) =
                                                                typen::ctype_to_rtype(
                                                                    ts[0].clone(),
                                                                    deps,
                                                                )?;
                                                            deps = d;
                                                            let (errrustname, d) =
                                                                typen::ctype_to_rtype(
                                                                    ts[1].clone(),
                                                                    deps,
                                                                )?;
                                                            deps = d;
                                                            if let CType::Binds(..) = &**t {
                                                                return Ok((
                                                                    format!(
                                                                        "Err::<{}, {}>({})",
                                                                        okrustname,
                                                                        errrustname,
                                                                        match argstrs[0]
                                                                            .strip_prefix("&mut ")
                                                                        {
                                                                            Some(s) => s,
                                                                            None => &argstrs[0],
                                                                        }
                                                                    ),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            } else {
                                                                return Ok((
                                                                    format!(
                                                                        "Ok::<{}, {}>({})",
                                                                        okrustname,
                                                                        errrustname,
                                                                        match argstrs[0]
                                                                            .strip_prefix("&mut ")
                                                                        {
                                                                            Some(s) => s,
                                                                            None => &argstrs[0],
                                                                        }
                                                                    ),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            }
                                                        }
                                                    }
                                                }
                                                return Ok((
                                                    format!(
                                                        "{}::{}({})",
                                                        function.name,
                                                        enum_name,
                                                        match argstrs[0].strip_prefix("&mut ") {
                                                            Some(s) => s,
                                                            None => &argstrs[0],
                                                        },
                                                    ),
                                                    out,
                                                    deps,
                                                ));
                                            }
                                            CType::Import(n, d) => match &**n {
                                                CType::TString(s) if s == &enum_name => {
                                                    super::register_rust_dependency(d, &mut deps);
                                                    // Special-casing for Option and Result mapping. TODO:
                                                    // Make this more centralized
                                                    if ts.len() == 2 {
                                                        if let CType::Void = &*ts[1] {
                                                            if let CType::Void = &**t {
                                                                return Ok((
                                                                    "None".to_string(),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            } else {
                                                                return Ok((
                                                                    format!(
                                                                        "Some({})",
                                                                        match argstrs[0]
                                                                            .strip_prefix("&mut ")
                                                                        {
                                                                            Some(s) => s,
                                                                            None => &argstrs[0],
                                                                        }
                                                                    ),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            }
                                                        } else if let CType::Type(name, _) = &*ts[1]
                                                        {
                                                            if name == "Error" {
                                                                let (okrustname, d) =
                                                                    typen::ctype_to_rtype(
                                                                        ts[0].clone(),
                                                                        deps,
                                                                    )?;
                                                                deps = d;
                                                                let (errrustname, d) =
                                                                    typen::ctype_to_rtype(
                                                                        ts[1].clone(),
                                                                        deps,
                                                                    )?;
                                                                deps = d;
                                                                if let CType::Binds(..) = &**t {
                                                                    return Ok((
                                                                        format!(
                                                                            "Err::<{}, {}>({})",
                                                                            okrustname,
                                                                            errrustname,
                                                                            match argstrs[0]
                                                                                .strip_prefix(
                                                                                    "&mut "
                                                                                ) {
                                                                                Some(s) => s,
                                                                                None => &argstrs[0],
                                                                            }
                                                                        ),
                                                                        out,
                                                                        deps,
                                                                    ));
                                                                } else {
                                                                    return Ok((
                                                                        format!(
                                                                            "Ok::<{}, {}>({})",
                                                                            okrustname,
                                                                            errrustname,
                                                                            match argstrs[0]
                                                                                .strip_prefix(
                                                                                    "&mut "
                                                                                ) {
                                                                                Some(s) => s,
                                                                                None => &argstrs[0],
                                                                            }
                                                                        ),
                                                                        out,
                                                                        deps,
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    return Ok((
                                                        format!(
                                                            "{}::{}({})",
                                                            function.name,
                                                            enum_name,
                                                            match argstrs[0].strip_prefix("&mut ") {
                                                                Some(s) => s,
                                                                None => &argstrs[0],
                                                            },
                                                        ),
                                                        out,
                                                        deps,
                                                    ));
                                                }
                                                _ => {}
                                            },
                                            _ => {}
                                        },
                                        _ => {
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok(("None".to_string(), out, deps));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[0].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                ts[1].clone(),
                                                                deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                format!(
                                                                    "Err::<{}, {}>({})",
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "Ok::<{}, {}>({})",
                                                                    okrustname,
                                                                    errrustname,
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{}::{}({})",
                                                    function.name,
                                                    enum_name,
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                ),
                                                out,
                                                deps,
                                            ));
                                        }
                                    }
                                }
                                return Err(format!("Cannot generate a constructor function for {} type as it is not part of the {} type", enum_name, function.name).into());
                            }
                            CType::Tuple(ts, _) => {
                                // TODO: Better type checking here, but it's *probably* being
                                // done at a higher layer
                                // Check for parent constructor: single argument whose type is a
                                // Tuple/Either containing a superset of the child's fields
                                if argstrs.len() == 1 {
                                    let single_arg_type = match &function.args().first() {
                                        Some(t) => t.2.clone().degroup(),
                                        None => Arc::new(CType::Void),
                                    };
                                    if let Some(parent_fields) = match &*single_arg_type {
                                        CType::Tuple(pf, _) => Some(pf.clone()),
                                        CType::Either(pf, _) => Some(pf.clone()),
                                        _ => None,
                                    } {
                                        // Filter out static fields from the child's expected fields
                                        let child_fields: Vec<&Arc<CType>> = ts
                                            .iter()
                                            .filter(|t| match &***t {
                                                CType::Field(_, t) => !matches!(
                                                    &**t,
                                                    CType::Int(_)
                                                        | CType::Float(_)
                                                        | CType::Bool(_)
                                                        | CType::TString(_),
                                                ),
                                                CType::Int(_)
                                                | CType::Float(_)
                                                | CType::Bool(_)
                                                | CType::TString(_) => false,
                                                _ => true,
                                            })
                                            .collect();
                                        // Find matching field indices in the parent
                                        let mut parent_indices: Vec<usize> = Vec::new();
                                        let mut all_matched = true;
                                        for child_field in &child_fields {
                                            let child_key = (*child_field)
                                                .clone()
                                                .degroup()
                                                .to_callable_string();
                                            let mut found = false;
                                            for (idx, pf) in parent_fields.iter().enumerate() {
                                                if pf.clone().degroup().to_callable_string()
                                                    == child_key
                                                {
                                                    parent_indices.push(idx);
                                                    found = true;
                                                    break;
                                                }
                                            }
                                            if !found {
                                                all_matched = false;
                                                break;
                                            }
                                        }
                                        if all_matched && !parent_indices.is_empty() {
                                            let parent_arg = match argstrs[0].strip_prefix("&mut ")
                                            {
                                                Some(s) => s.to_string(),
                                                None => argstrs[0].clone(),
                                            };
                                            let field_accesses: Vec<String> = parent_indices
                                                .iter()
                                                .map(|i| format!("{}.{}", parent_arg, i))
                                                .collect();
                                            // Tuples in Rust use (a, b, ...) syntax directly.
                                            // Single-element tuples need a trailing comma: (v.1,)
                                            return Ok((
                                                if field_accesses.len() == 1 {
                                                    format!("({},)", field_accesses[0])
                                                } else {
                                                    format!("({})", field_accesses.join(", "))
                                                },
                                                out,
                                                deps,
                                            ));
                                        }
                                    }
                                }
                                if argstrs.len()
                                    == ts
                                        .iter()
                                        .filter(|t| match &***t {
                                            CType::Field(_, t) => !matches!(
                                                &**t,
                                                CType::Int(_)
                                                    | CType::Float(_)
                                                    | CType::Bool(_)
                                                    | CType::TString(_),
                                            ),
                                            CType::Int(_)
                                            | CType::Float(_)
                                            | CType::Bool(_)
                                            | CType::TString(_) => false,
                                            _ => true,
                                        })
                                        .collect::<Vec<&Arc<CType>>>()
                                        .len()
                                {
                                    if argstrs.len() == 1 {
                                        return Ok((
                                            format!(
                                                "({},)",
                                                match argstrs[0].strip_prefix("&mut ") {
                                                    Some(s) => s.to_string(),
                                                    None => argstrs[0].clone(),
                                                }
                                            ),
                                            out,
                                            deps,
                                        ));
                                    } else {
                                        return Ok((
                                            format!(
                                                "({})",
                                                argstrs
                                                    .iter()
                                                    .map(|a| match a.strip_prefix("&mut ") {
                                                        Some(s) => s.to_string(),
                                                        None => a.to_string(),
                                                    })
                                                    .collect::<Vec<String>>()
                                                    .join(", ")
                                            ),
                                            out,
                                            deps,
                                        ));
                                    }
                                } else {
                                    return Err(format!(
                                        "{} has {} fields but {} provided",
                                        function.name,
                                        ts.len(),
                                        argstrs.len()
                                    )
                                    .into());
                                }
                            }
                            CType::Field(..) => {
                                return Ok((
                                    format!(
                                        "({},)",
                                        match argstrs[0].strip_prefix("&mut ") {
                                            Some(s) => s.to_string(),
                                            None => argstrs[0].clone(),
                                        }
                                    ),
                                    out,
                                    deps,
                                ));
                            }
                            CType::Binds(..) => {
                                return Ok((argstrs.join(", "), out, deps));
                            }
                            CType::Void | CType::DerivedVoid(..) => {
                                // DerivedVoid parent constructor: takes parent, returns void
                                return Ok(("()".to_string(), out, deps));
                            }
                            otherwise => {
                                return Err(format!("How did you get here? Trying to create a constructor function {function:?} for {otherwise:?}").into());
                            }
                        }
                    } else if function.args().len() == 1 {
                        // Check for parent constructor: return type is Maybe{Child} and arg is a superset Either
                        let ret_inner = match &*ret_type {
                            CType::Either(ts, _) if ts.len() == 2 => {
                                if let CType::Void = &*ts[1] {
                                    Some(ts[0].clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };
                        if let Some(child_type_raw) = ret_inner {
                            // Unwrap Type and Group wrappers to get to the inner type
                            let mut child_type = child_type_raw.clone().degroup();
                            loop {
                                child_type = match &*child_type {
                                    CType::Type(_, t) => t.clone(),
                                    CType::Group(t) => t.clone(),
                                    _ => break,
                                };
                            }
                            let child_name = child_type.clone().to_callable_string();
                            if function.name == child_name {
                                let arg_type = function.args()[0].2.clone().degroup();
                                let parent_type_name = arg_type.clone().to_callable_string();
                                if let CType::Either(parent_variants, _) = &*arg_type {
                                    if let CType::Either(child_variants, _) = &*child_type {
                                        let mut matched_indices: Vec<usize> = Vec::new();
                                        for (idx, pv) in parent_variants.iter().enumerate() {
                                            let parent_key =
                                                pv.clone().degroup().to_callable_string();
                                            for cv in child_variants {
                                                if cv.clone().degroup().to_callable_string()
                                                    == parent_key
                                                {
                                                    matched_indices.push(idx);
                                                    break;
                                                }
                                            }
                                        }
                                        if !matched_indices.is_empty()
                                            && matched_indices.len() < parent_variants.len()
                                        {
                                            let parent_arg = match argstrs[0].strip_prefix("&mut ")
                                            {
                                                Some(s) => s.to_string(),
                                                None => argstrs[0].clone(),
                                            };
                                            let mut arms = Vec::new();
                                            for (idx, pv) in parent_variants.iter().enumerate() {
                                                let variant_name =
                                                    pv.clone().degroup().to_callable_string();
                                                if matched_indices.contains(&idx) {
                                                    arms.push(format!(
                                                        "{}::{}(v) => Some({}::{}(v))",
                                                        parent_type_name,
                                                        variant_name,
                                                        function.name,
                                                        variant_name
                                                    ));
                                                } else {
                                                    arms.push(format!(
                                                        "{}::{}(_) => None",
                                                        parent_type_name, variant_name
                                                    ));
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "match {} {{\n    {}\n}}",
                                                    parent_arg,
                                                    arms.join(",\n    ")
                                                ),
                                                out,
                                                deps,
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Fallback: Check if return type is Shared for constructor generation
                    // when function name doesn't match (e.g., inferred generic constructors)
                    let mut fallback_ret = ret_type.clone();
                    while matches!(&*fallback_ret, CType::Type(..)) {
                        fallback_ret = match &*fallback_ret {
                            CType::Type(_, t) => t.clone(),
                            _ => fallback_ret,
                        };
                    }
                    if let CType::Shared(_) = &*fallback_ret {
                        return Ok((
                            format!(
                                "std::sync::Arc::new(std::sync::RwLock::new({}))",
                                match argstrs[0].strip_prefix("&mut ") {
                                    Some(v) => v.to_string(),
                                    None => argstrs[0].clone(),
                                }
                            ),
                            out,
                            deps,
                        ));
                    }
                    Err(format!(
                        "Trying to create an automatic function for {} but the return type is {}",
                        function.name, ret_name
                    )
                    .into())
                }
            }
        }
        Microstatement::VarCall { name, args, .. } => {
            let mut argstrs = Vec::new();
            for arg in args {
                let (a, o, d) = from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
                out = o;
                deps = d;
                // If the argument is itself a function, this is the only place in Rust
                // where you can't pass by reference, so we check the type and change
                // the argument output accordingly.
                let arg_type = arg.get_type();
                match &*arg_type {
                    CType::Function(..) => argstrs.push(a.to_string()),
                    // TODO: How to figure out the arg kinds for a VarCall
                    _ => argstrs.push(if a.starts_with("&mut ") {
                        a
                    } else {
                        format!("&mut {a}")
                    }),
                }
            }
            Ok((
                format!("{}({})", name, argstrs.join(", ")).to_string(),
                out,
                deps,
            ))
        }
        Microstatement::Return { value } => match value {
            Some(val) => {
                let (retval, o, d) =
                    from_microstatement(val, parent_fn, shared_vars, scope, out, deps)?;
                out = o;
                deps = d;
                Ok((
                    format!(
                        "return {}",
                        match retval.strip_prefix("&mut ") {
                            Some(v) => v,
                            None => &retval,
                        }
                    ),
                    out,
                    deps,
                ))
            }
            None => Ok(("return".to_string(), out, deps)),
        },
    }
}

#[allow(clippy::type_complexity)]
pub fn generate(
    rustname: String,
    function: &Function,
    scope: &Scope,
    mut out: OrderedHashMap<String, String>,
    mut deps: OrderedHashMap<String, String>,
) -> Result<
    (
        OrderedHashMap<String, String>,
        OrderedHashMap<String, String>,
    ),
    Box<dyn std::error::Error>,
> {
    let _ctx = StmtCtxGuard::enter_function();
    let shared_vars = build_shared_vars(function);
    let mut fn_string = "".to_string();
    // First make sure all of the function argument types are defined
    let mut arg_strs = Vec::new();
    for (idx, arg) in function.args().iter().enumerate() {
        let (l, k, t) = arg;
        // Re-add Mut{} for closure function arguments but then mark it as a reference
        let ty = if let ArgKind::Mut = k {
            if let CType::Function(..) = &**t {
                Arc::new(CType::Mut(t.clone()))
            } else {
                t.clone()
            }
        } else {
            t.clone()
        };
        let (t_str, o, d) = typen::generate(ty, out, deps)?;
        out = o;
        deps = d;
        if t_str.starts_with("impl") || t_str.starts_with("&") {
            if t_str.contains("FnMut") {
                arg_strs.push(format!("mut {l}: {t_str}"));
            } else {
                arg_strs.push(format!("{l}: {t_str}"));
            }
        } else if promote_param_to_own(function, idx) {
            // Take this parameter by value (owned) instead of `&T`, eliminating
            // the defensive entry clone. Declared `mut` to preserve the mutable
            // owned-local semantics the defensive clone used to provide.
            arg_strs.push(format!("mut {l}: {t_str}"));
        } else {
            match k {
                ArgKind::Mut => arg_strs.push(format!("{l}: &mut {t_str}")),
                ArgKind::Own => arg_strs.push(format!("{l}: {t_str}")),
                // A borrowed `string` is taken as `&str` (idiomatic, and accepts a
                // `&String` argument by deref coercion) rather than `&String`.
                _ if t_str == "String" => arg_strs.push(format!("{l}: &str")),
                _ => arg_strs.push(format!("{l}: &{t_str}")),
            }
        }
    }
    let ret = function.rettype().degroup();
    let opt_ret_str = match &*ret {
        CType::Void | CType::DerivedVoid(..) => None,
        CType::Type(n, _) if n == "void" => None,
        _otherwise => {
            let (t_str, o, d) = typen::generate(ret, out, deps)?;
            out = o;
            deps = d;
            match t_str.strip_prefix("&") {
                Some(s) => Some(s.to_string()),
                None => Some(t_str),
            }
        }
    };
    // Start generating the function output. We can do this eagerly like this because, at least for
    // now, we inline all other function calls within an "entry" function (the main function, or
    // any function that's attached to an event, or any function that's part of an exported set in
    // a shared library). LLVM *probably* doesn't deduplicate this redundancy, so this will need to
    // be revisited, but it eliminates a whole host of generation problems that I can come back to
    // later.
    fn_string = format!(
        "{}fn {}({}){} {{\n",
        fn_string,
        rustname.clone(),
        arg_strs.join(", "),
        match opt_ret_str {
            Some(rettype) => format!(" -> {rettype}").to_string(),
            None => "".to_string(),
        },
    )
    .to_string();
    // Elide `clone(x)` calls that are the provable last use of `x`, moving the
    // original instead. `shared_vars` (plus the value's own type) identifies the
    // `Shared` values whose deep-clone-vs-handle-move aliasing must be preserved.
    let is_shared_name = |name: &str| shared_vars.contains_key(name);
    let body = crate::program::liveness::elide_last_use_clones(function, &is_shared_name);
    for (idx, microstatement) in body.iter().enumerate() {
        STMT_IDX.with(|c| c.set(idx));
        let (stmt, o, d) =
            from_microstatement(microstatement, function, &shared_vars, scope, out, deps)?;
        out = o;
        deps = d;
        // Skip no-op statements (e.g. an argument-prologue binding that was
        // elided), which would otherwise emit a stray empty `;`.
        if !stmt.is_empty() {
            fn_string = format!("{fn_string}    {stmt};\n");
        }
    }
    fn_string = format!("{fn_string}}}");
    out.insert(rustname, fn_string);
    Ok((out, deps))
}
