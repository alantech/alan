use std::collections::HashSet;
use std::sync::Arc;

use nom::combinator::all_consuming;
use nom::Parser;
use ordered_hash_map::OrderedHashMap;

use crate::codegen;
use crate::lntojs::typen;
use crate::parse::{booln, integer, real};
use crate::program::{ArgKind, CType, CfnKind, FnKind, Function, Microstatement, Scope};

/// JavaScript-safe local binding for an Alan identifier (`var`, `void`, ...).
fn js_binding_name(name: &str) -> &str {
    match name {
        "var" => "__var__",
        "void" => "__void__",
        _ => name,
    }
}

/// If `representation` is a compile-time literal of a primitive native type
/// (`string`/`i64`/`u64`/`f64`/`bool`), returns it wrapped in the corresponding
/// `alan_std` boxing class (`new alan_std.I64(1n)`, `new alan_std.Str("x")`,
/// ...). Returns `None` for anything else — including a non-literal of those
/// types (e.g. a variable/parameter name), which is already a boxed value and
/// must be passed through untouched.
///
/// This is the single place that knows how primitive literals are boxed for the
/// JS runtime, so both the `Value` handler and the `NativeCall` serializer (which
/// may receive a substituted literal as a method receiver/argument once inlining
/// is enabled) produce identical, valid output.
fn box_native_value(typen: &CType, representation: &str) -> Option<String> {
    // A numeric literal whose type was never narrowed by context is an `AnyOf` candidate set;
    // box it as its FUI default (the last candidate), matching the type codegen renders for it.
    if let CType::AnyOf(ts) = typen {
        return ts
            .last()
            .and_then(|t| box_native_value(&t.clone().degroup(), representation));
    }
    match typen {
        CType::Type(n, _) if n == "string" => {
            if representation.starts_with('"') {
                Some(format!(
                    "new alan_std.Str({})",
                    representation.replace('\n', "\\n")
                ))
            } else {
                None
            }
        }
        // 64-bit integers wrap a `BigInt` (the `Int` base calls `BigInt(val)`), so the literal needs
        // an `n` suffix: `new alan_std.I64(1n)`.
        CType::Type(n, _) if n == "i64" || n == "u64" => {
            if all_consuming(integer).parse(representation).is_ok() {
                Some(format!(
                    "new alan_std.{}({representation}n)",
                    n.to_uppercase()
                ))
            } else {
                None
            }
        }
        // Sub-64-bit integers wrap a plain `Number`. These can now appear as bare literals (e.g.
        // `5.u8` or `let x: u8 = 5`) since numeric literals adopt the narrowed type directly, so they
        // must be boxed into their `alan_std` wrapper just like the 64-bit ones.
        CType::Type(n, _) if matches!(n.as_str(), "i8" | "i16" | "i32" | "u8" | "u16" | "u32") => {
            if all_consuming(integer).parse(representation).is_ok() {
                Some(format!(
                    "new alan_std.{}({representation})",
                    n.to_uppercase()
                ))
            } else {
                None
            }
        }
        // Both float widths wrap a `Number`. A literal narrowed to a float may be in integer form
        // (e.g. `5.f32`), so accept either spelling.
        CType::Type(n, _) if n == "f32" || n == "f64" => {
            if all_consuming(real).parse(representation).is_ok()
                || all_consuming(integer).parse(representation).is_ok()
            {
                Some(format!(
                    "new alan_std.{}({representation})",
                    n.to_uppercase()
                ))
            } else {
                None
            }
        }
        CType::Type(n, _) if n == "bool" => {
            if all_consuming(booln).parse(representation).is_ok() {
                Some(format!("new alan_std.Bool({representation})"))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_promise_head(typen: Arc<CType>) -> bool {
    let mut t = typen.degroup();
    while matches!(&*t, CType::Type(..) | CType::Group(_)) {
        t = match &*t {
            CType::Type(_, inner) | CType::Group(inner) => inner.clone().degroup(),
            _ => unreachable!(),
        };
    }
    matches!(&*t, CType::Promise(_))
}

fn function_codegen_is_async(function: Arc<Function>) -> bool {
    fn microstatement_awaits_for_codegen(ms: &Microstatement, seen: &mut HashSet<usize>) -> bool {
        match ms {
            Microstatement::Assignment { value, .. } => {
                microstatement_awaits_for_codegen(value, seen)
            }
            Microstatement::FnCall { function, args } => {
                function_codegen_is_async_inner(function.clone(), seen)
                    || args
                        .iter()
                        .any(|a| microstatement_awaits_for_codegen(a, seen))
            }
            Microstatement::VarCall { typen, args, .. } => {
                is_promise_head(typen.clone())
                    || args
                        .iter()
                        .any(|a| microstatement_awaits_for_codegen(a, seen))
            }
            Microstatement::Array { vals, .. } => vals
                .iter()
                .any(|v| microstatement_awaits_for_codegen(v, seen)),
            Microstatement::Return { value } => value
                .as_deref()
                .is_some_and(|v| microstatement_awaits_for_codegen(v, seen)),
            Microstatement::NativeCall { args, .. } => args
                .iter()
                .any(|a| microstatement_awaits_for_codegen(a, seen)),
            Microstatement::Closure { function } => {
                function_codegen_is_async_inner(function.clone(), seen)
            }
            Microstatement::Arg { .. } | Microstatement::Value { .. } => false,
        }
    }

    fn function_codegen_is_async_inner(function: Arc<Function>, seen: &mut HashSet<usize>) -> bool {
        let ptr = Arc::as_ptr(&function) as usize;
        if !seen.insert(ptr) {
            return false;
        }
        is_promise_head(function.rettype())
            || function
                .microstatements
                .iter()
                .any(|ms| microstatement_awaits_for_codegen(ms, seen))
    }

    let mut seen = HashSet::new();
    function_codegen_is_async_inner(function, &mut seen)
}

/// Render a conditional branch closure's body as a sequence of JS statements for inlining inside a
/// native `if (...) { ... } else { ... }` block.
///
/// When `discard` is set, the conditional's *value* is thrown away (the `if` call sits in a
/// discarded statement position), so the branch's terminal value-`return X` must become a bare
/// `X;` -- otherwise the inlined `return` would erroneously return from the *enclosing* function.
/// When `discard` is unset (value or return position), terminal `return`s are kept intact. A void
/// `if` is never rendered with `discard` (its branches' `return`s are genuine early-returns from
/// the enclosing function, e.g. from the `if`/`else` syntax rewrite), so those stay untouched.
#[allow(clippy::type_complexity)]
fn render_js_branch_body(
    microstatement: &Microstatement,
    parent_fn: &Function,
    scope: &Scope,
    discard: bool,
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
    if let Microstatement::Closure { function } = microstatement {
        let mss = &function.microstatements;
        let mut parts = Vec::new();
        for (i, ms) in mss.iter().enumerate() {
            let is_last = i + 1 == mss.len();
            // In discard position, strip the closure's terminal value-`return` to a bare
            // expression so the conditional doesn't return out of the enclosing function.
            if is_last && discard {
                if let Microstatement::Return { value: Some(v) } = ms {
                    let (val, o, d) = from_microstatement(v, parent_fn, scope, out, deps)?;
                    out = o;
                    deps = d;
                    if !val.trim().is_empty() {
                        parts.push(val);
                    }
                    continue;
                }
            }
            let (val, o, d) = from_microstatement(ms, parent_fn, scope, out, deps)?;
            out = o;
            deps = d;
            if !val.trim().is_empty() {
                parts.push(val);
            }
        }
        return Ok((parts.join(";\n        "), out, deps));
    }
    // A branch that is a closure-typed *value* (e.g. a `() -> T` parameter forwarded through a
    // delegating `if`, rather than a literal closure) is invoked here. In value/return position its
    // result is `return`ed (a void branch's returned `undefined` is harmless); in discard position
    // it is called purely for side effects. If the closure is async (its return type is
    // `Promise{T}`), the call is awaited.
    if branch_value_is_function(microstatement) {
        let (val, o, d) = from_microstatement(microstatement, parent_fn, scope, out, deps)?;
        out = o;
        deps = d;
        let awaited = if branch_value_is_async(microstatement) {
            format!("await {val}()")
        } else {
            format!("{val}()")
        };
        return Ok((
            if discard {
                awaited
            } else {
                format!("return {awaited}")
            },
            out,
            deps,
        ));
    }
    // Fallback: render normally (e.g. a non-closure expression).
    from_microstatement(microstatement, parent_fn, scope, out, deps)
}

/// Whether a branch microstatement is a closure-typed *value* (a `() -> T` function passed by
/// reference, not a literal `Closure`) -- the shape produced when a user-defined `if` overload
/// forwards its branch parameters directly to the underlying `if` cfn.
fn branch_value_is_function(ms: &Microstatement) -> bool {
    !matches!(ms, Microstatement::Closure { .. })
        && matches!(&*ms.get_type().degroup(), CType::Function(..))
}

/// Whether invoking a closure-typed value branch yields a `Promise` (i.e. the closure is async).
fn branch_value_is_async(ms: &Microstatement) -> bool {
    if let CType::Function(_, o) = &*ms.get_type().degroup() {
        is_promise_head(o.clone())
    } else {
        false
    }
}

/// Render a realized `if{T}` cfn call (`args[0]` condition, `args[1]`/`args[2]` branch closures) as
/// a native `if (cond) { ... } else { ... }` block, inlining each branch closure's body. Used in
/// statement positions (return / discarded-statement) where the closure overhead can be dropped
/// entirely so V8 sees a native conditional. `discard` is set when the conditional's value is
/// thrown away (see `render_js_branch_body`).
#[allow(clippy::type_complexity)]
fn render_js_native_ifelse(
    args: &[Microstatement],
    parent_fn: &Function,
    scope: &Scope,
    discard: bool,
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
    let (cond, o, d) = from_microstatement(&args[0], parent_fn, scope, out, deps)?;
    out = o;
    deps = d;
    // The condition is a boxed `alan_std.Bool`; a bare object is always truthy in JS, so unwrap
    // its `.val` (falling back to the value itself if it is already a primitive).
    let cond = format!("({cond})?.val ?? ({cond})");
    let (then_body, o, d) = render_js_branch_body(&args[1], parent_fn, scope, discard, out, deps)?;
    out = o;
    deps = d;
    let (else_body, o, d) = render_js_branch_body(&args[2], parent_fn, scope, discard, out, deps)?;
    out = o;
    deps = d;
    let then_block = if then_body.trim().is_empty() {
        "{}".to_string()
    } else {
        format!("{{\n        {then_body};\n    }}")
    };
    let else_block = if else_body.trim().is_empty() {
        "{}".to_string()
    } else {
        format!("{{\n        {else_body};\n    }}")
    };
    Ok((
        format!("if ({cond}) {then_block} else {else_block}"),
        out,
        deps,
    ))
}

/// JS has no block-scoped `let` redeclaration, but Alan permits shadowing (and the
/// conditional-assignment lowering in `microstatement.rs` emits a shadowing `let <var> = if(...)`).
/// Within a single rendered function body, the first time a name is bound it keeps its
/// `let`/`const` declaration; any later binding of the same name renders as a plain reassignment
/// (`<name> = ...`). `declared` tracks the names already bound in the current body and should be
/// seeded with the function's argument names.
fn js_dedup_declaration(
    stmt: String,
    ms: &Microstatement,
    declared: &mut std::collections::HashSet<String>,
) -> String {
    if let Microstatement::Assignment { name, .. } = ms {
        let n = js_binding_name(name).to_string();
        if !declared.insert(n) {
            if let Some(rest) = stmt.strip_prefix("let ") {
                return rest.to_string();
            }
            if let Some(rest) = stmt.strip_prefix("const ") {
                return rest.to_string();
            }
        }
    }
    stmt
}

/// Whether a microstatement is a realized `if{T}` cfn call.
fn is_ifelse_call(ms: &Microstatement) -> bool {
    matches!(
        ms,
        Microstatement::FnCall { function, .. }
            if matches!(&function.kind, FnKind::CfnRealized(CfnKind::IfElse))
    )
}

#[allow(clippy::type_complexity)]
pub fn from_microstatement(
    microstatement: &Microstatement,
    parent_fn: &Function,
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
        Microstatement::Arg {
            name: _,
            kind,
            typen: _,
        } => match kind {
            ArgKind::Ref | ArgKind::Mut => Ok(("".to_string(), out, deps)),
            _ => Err("Targeting JS does not allow for Own{T} or Deref{T}".into()),
        },
        Microstatement::Assignment {
            name,
            value,
            mutable,
        } => {
            let (val, o, d) = from_microstatement(value, parent_fn, scope, out, deps)?;
            out = o;
            deps = d;
            let n = js_binding_name(name);
            if *mutable {
                Ok((format!("let {n} = {val}"), out, deps))
            } else {
                Ok((format!("const {n} = {val}"), out, deps))
            }
        }
        Microstatement::Closure { function } => {
            let arg_names = function
                .args()
                .into_iter()
                .map(|(n, _, _)| n)
                .collect::<Vec<String>>();
            let async_prefix = if function_codegen_is_async(function.clone()) {
                "async "
            } else {
                ""
            };
            let mut inner_statements = Vec::new();
            let mut declared: std::collections::HashSet<String> =
                arg_names.iter().cloned().collect();
            for ms in &function.microstatements {
                let (val, o, d) = from_microstatement(ms, parent_fn, scope, out, deps)?;
                out = o;
                deps = d;
                let val = js_dedup_declaration(val, ms, &mut declared);
                if !val.trim().is_empty() {
                    inner_statements.push(val);
                }
            }
            let body = if inner_statements.is_empty() {
                "".to_string()
            } else {
                format!("\n        {};\n    ", inner_statements.join(";\n        "))
            };
            Ok((
                format!(
                    "{}function ({}) {{{}}}",
                    async_prefix,
                    arg_names.join(", "),
                    body,
                ),
                out,
                deps,
            ))
        }
        Microstatement::Value {
            typen,
            representation,
        } => {
            // A numeric literal whose type was never narrowed by context is an `AnyOf`; collapse it
            // to its FUI default so it boxes (`new alan_std.I64(..)`) like a concrete literal.
            let typen = &typen.clone().collapse_anyof_default();
            match &**typen {
                CType::Void | CType::DerivedVoid(..) if representation == "()" => {
                    Ok(("undefined".to_string(), out, deps))
                }
                CType::Type(n, inner)
                    if representation == "()"
                        && (n == "void"
                            || matches!(&**inner, CType::Void | CType::DerivedVoid(..))) =>
                {
                    Ok(("undefined".to_string(), out, deps))
                }
                CType::Group(inner)
                    if representation == "()"
                        && matches!(&**inner, CType::Void | CType::DerivedVoid(..)) =>
                {
                    Ok(("undefined".to_string(), out, deps))
                }
                CType::Type(n, _)
                    if matches!(
                        n.as_str(),
                        "string"
                            | "bool"
                            | "i8"
                            | "i16"
                            | "i32"
                            | "i64"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "f32"
                            | "f64"
                    ) =>
                {
                    Ok((
                        box_native_value(typen, representation)
                            .unwrap_or_else(|| representation.clone()),
                        out,
                        deps,
                    ))
                }
                CType::Binds(n, _) => match &**n {
                    CType::TString(_) => Ok((representation.clone(), out, deps)),
                    CType::Import(n, d) => {
                        super::register_nodejs_dependency(d, &mut deps);
                        match &**n {
                            CType::TString(_) => { /* Do nothing */ }
                            _ => CType::fail("Native import names must be strings"),
                        }
                        Ok((representation.clone(), out, deps))
                    }
                    otherwise => CType::fail(&format!(
                        "Bound types must be strings or node.js imports: {otherwise:?}"
                    )),
                },
                CType::Function(..) => {
                    codegen::resolve_function_value::<LnToJs>(
                        representation,
                        typen.clone(),
                        scope,
                        parent_fn,
                        out,
                        deps,
                    )
                }
                _ => Ok((js_binding_name(representation).to_string(), out, deps)),
            }
        }
        Microstatement::Array { vals, .. } => codegen::render_array(
            vals,
            out,
            deps,
            |val, out, deps| from_microstatement(val, parent_fn, scope, out, deps),
            |vals| format!("[{}]", vals.join(", ")),
        ),
        Microstatement::NativeCall {
            typen,
            kind,
            name,
            args,
        } => {
            // Serialize a native construct (function/method/property/operator) in
            // the codegen layer; `kind` selects the surface form. For the
            // receiver-based forms `args[0]` is the receiver.
            // A `Value` argument that is a variable/parameter name is emitted
            // directly (it is already a boxed runtime value); a `Value` that is a
            // compile-time literal is boxed into its `alan_std` class via
            // `box_native_value` so an inlined literal receiver/argument (e.g.
            // `new alan_std.I64(1n).wrappingAdd(...)`) is valid and type-correct
            // rather than a bare `1.wrappingAdd(...)`. Non-`Value` arguments (only
            // possible once these are inlined) render normally.
            let mut rendered = Vec::new();
            for a in args {
                let s = if let Microstatement::Value {
                    typen: at,
                    representation,
                } = a
                {
                    box_native_value(at, representation).unwrap_or_else(|| representation.clone())
                } else {
                    let (s, o, d) = from_microstatement(a, parent_fn, scope, out, deps)?;
                    out = o;
                    deps = d;
                    s
                };
                rendered.push(s);
            }
            let call = codegen::build_native_call_no_cast(kind, name, &rendered)?;
            // Apply the result type's serialization by rendering the assembled call
            // through the `Value` handler with the call's return type.
            from_microstatement(
                &Microstatement::Value {
                    typen: typen.clone(),
                    representation: call,
                },
                parent_fn,
                scope,
                out,
                deps,
            )
        }
        Microstatement::FnCall { function, args } => {
            let mut arg_types = Vec::new();
            let mut arg_type_strs = Vec::new();
            for arg in args {
                let arg_type = arg.get_type();
                let (_, o, d) = typen::generate(arg_type.clone(), out, deps)?;
                out = o;
                deps = d;
                arg_types.push(arg_type.clone());
                let res = typen::ctype_to_jtype(arg_type.clone(), deps)?;
                arg_type_strs.push(res.0);
                deps = res.1;
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
                    if let Some(inlined) = codegen::try_single_inline(function, args) {
                        return from_microstatement(&inlined, parent_fn, scope, out, deps);
                    }
                    let (_, o, d) = typen::generate(function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut arg_strs = Vec::new();
                    for arg in &function.args() {
                        let arg_str = arg.2.clone().to_callable_string();
                        arg_strs.push(js_binding_name(&arg_str).to_string());
                    }
                    let jsname = format!("{}_{}", function.name, arg_strs.join("_")).to_string();
                    let res = generate(jsname.clone(), function, scope, out, deps)?;
                    out = res.0;
                    deps = res.1;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
                        out = o;
                        deps = d;
                        argstrs.push(js_binding_name(&a).to_string());
                    }
                    if let FnKind::External(d) = &function.kind {
                        super::register_nodejs_dependency(d, &mut deps);
                    }
                    let call = format!("{}({})", jsname, argstrs.join(", "));
                    Ok((
                        if function_codegen_is_async(function.clone()) {
                            format!("(await {call})")
                        } else {
                            format!("({call})")
                        },
                        out,
                        deps,
                    ))
                }
                FnKind::Bind(jsname) | FnKind::ExternalBind(jsname, _) => {
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
                        out = o;
                        deps = d;
                        argstrs.push(js_binding_name(&a).to_string());
                    }
                    if let FnKind::ExternalBind(_, d) = &function.kind {
                        super::register_nodejs_dependency(d, &mut deps);
                    }
                    let call = format!("{}({})", jsname, argstrs.join(", "));
                    Ok((
                        if function_codegen_is_async(function.clone()) {
                            format!("(await {call})")
                        } else {
                            format!("({call})")
                        },
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
                            CType::Binds(n, _) => match &**n {
                                CType::TString(_) => Ok((representation.clone(), out, deps)),
                                CType::Import(n, d) => {
                                    super::register_nodejs_dependency(d, &mut deps);
                                    match &**n {
                                        CType::TString(_) => { /* Do nothing */ }
                                        _ => CType::fail("Native import names must be strings"),
                                    }
                                    Ok((representation.clone(), out, deps))
                                }
                                otherwise => CType::fail(&format!(
                                    "Bound types must be strings or node.js imports: {otherwise:?}"
                                )),
                            },
                            CType::Type(n, _) if n == "string" => {
                                Ok((format!("new alan_std.Str({representation})"), out, deps))
                            }
                            CType::Type(n, _) if n == "bool" => {
                                Ok((format!("new alan_std.Bool({representation})"), out, deps))
                            }
                            CType::Type(n, _) if n == "i64" => {
                                Ok((format!("new alan_std.I64({representation})"), out, deps))
                            }
                            CType::Type(n, _) if n == "f64" => {
                                Ok((format!("new alan_std.F64({representation})"), out, deps))
                            }
                            _ => Ok((representation.clone(), out, deps)),
                        },
                        _ => unreachable!(),
                    }
                }
                FnKind::CfnRealized(CfnKind::Clone) => {
                    // Inject a compiler-generated `clone` helper function into the output if not
                    // already present, then call it. This replaces the alan_std.clone dependency.
                    let (_, o, d) = typen::generate(function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
                        out = o;
                        deps = d;
                        argstrs.push(a.to_string());
                    }
                    // Inject the clone helper function if it doesn't exist yet
                    if !out.contains_key("clone") {
                        out.insert(
                            "clone".to_string(),
                            "function clone(v) {\n\
                             if (v === null) return null;\n\
                            if (typeof v === 'object' && v !== null && typeof GPUBuffer !== 'undefined' && (v instanceof GPUBuffer || v.rawBuffer instanceof GPUBuffer)) return v;\n\
                             if (v instanceof Map) return new Map([...v.entries()].map(kv => [clone(kv[0]), clone(kv[1])]));\n\
                             if (typeof v === 'object' && v !== null && typeof v.build === 'function') return v.build(clone(v.val));\n\
                             if (typeof v === 'object' && v !== null && typeof v.map === 'function') return v.map(clone);\n\
                             if (typeof v === 'object' && v !== null && typeof v.store === 'function') return new (Object.getPrototypeOf(v).constructor)(clone(v.map));\n\
                             if (typeof v === 'object' && v !== null && v.val !== undefined && typeof v.val !== 'object') return new (Object.getPrototypeOf(v).constructor)(v.val);\n\
                             if (typeof v === 'object' && v !== null) return Object.fromEntries([...Object.entries(v)].map(kv => [kv[0], clone(kv[1])]));\n\
                             return v;\n\
                             }".to_string(),
                        );
                    }
                    Ok((format!("clone({})", argstrs[0]), out, deps))
                }
                FnKind::CfnRealized(CfnKind::IfElse) => {
                    // Value position: emit an IIFE so the conditional stays an expression. The
                    // IIFE is colored by whether either branch closure awaits -- a pure (sync)
                    // conditional becomes a plain sync IIFE with no `await`, while an awaiting
                    // branch produces an awaited async IIFE. Statement positions (return /
                    // discarded) are special-cased elsewhere to drop the IIFE entirely.
                    let (inner, o, d) =
                        render_js_native_ifelse(args, parent_fn, scope, false, out, deps)?;
                    out = o;
                    deps = d;
                    let branch_async = |ms: &Microstatement| match ms {
                        Microstatement::Closure { function } => {
                            function_codegen_is_async(function.clone())
                        }
                        // A forwarded closure-typed value branch: async iff calling it yields a
                        // `Promise` (its return type is `Promise{T}`).
                        _ => branch_value_is_async(ms),
                    };
                    let is_async = branch_async(&args[1]) || branch_async(&args[2]);
                    if is_async {
                        Ok((format!("(await (async () => {{ {inner} }})())"), out, deps))
                    } else {
                        Ok((format!("(() => {{ {inner} }})()"), out, deps))
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
                        let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
                        out = o;
                        deps = d;
                        argstrs.push(js_binding_name(&a).to_string());
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
                        // This is a wacky unwrapping logic...
                        let mut input_type = function.args()[0].2.clone();
                        while matches!(&*input_type, CType::Type(..) | CType::Group(_)) {
                            input_type = match &*input_type {
                                CType::Type(_, t) => t.clone(),
                                CType::Group(t) => t.clone(),
                                _ => input_type,
                            };
                        }
                        match &*input_type {
                            CType::Tuple(ts, _) => {
                                // Short-circuit for direct `<N>` function calls (which can only be
                                // generated by the internals of the compiler)
                                if let Ok(i) = function.name.parse::<i64>() {
                                    if let CType::Field(n, _) = &*ts[i as usize] {
                                        return Ok((format!("{}.{}", argstrs[0], n), out, deps));
                                    } else {
                                        return Ok((format!("{}.arg{}", argstrs[0], i), out, deps));
                                    }
                                }
                                let mut accessor_field = None;
                                for (i, t) in ts.iter().enumerate() {
                                    match &**t {
                                        CType::Field(n, _) => {
                                            if n == &function.name {
                                                accessor_field = Some(n.clone());
                                            }
                                        }
                                        _ => {
                                            if format!("arg{i}") == function.name {
                                                accessor_field = Some(function.name.clone());
                                            }
                                        }
                                    }
                                }
                                if let Some(n) = accessor_field {
                                    return Ok((format!("{}.{}", argstrs[0], n), out, deps));
                                }
                            }
                            CType::Buffer(_, s) => {
                                // Similarly short-circuit for direct `<N>` function calls
                                if let Ok(i) = function.name.parse::<i64>() {
                                    if let CType::Int(l) = **s {
                                        if i128::from(i) < l {
                                            return Ok((
                                                format!("{}[{}]", argstrs[0], i),
                                                out,
                                                deps,
                                            ));
                                        }
                                    }
                                }
                            }
                            CType::Field(..) => {
                                return Ok((format!("{}.arg0", argstrs[0]), out, deps));
                            }
                            CType::Either(ts, _) => {
                                // The kinds of types allowed here are `Type`, `Bound`, and
                                // `ResolvedBoundGeneric`, and `Field`. Other types don't have
                                // a string name we can match against the function name
                                let accessor_field = ts.iter().find(|t| match &***t {
                                    CType::Field(n, _) => *n == function.name,
                                    CType::Type(n, _) => *n == function.name,
                                    _ => false,
                                });
                                // We pass through to the main path if we can't find a matching
                                // name
                                if accessor_field.is_some() {
                                    if let Some(kind) = codegen::enum_variant_kind(ts) {
                                        match kind {
                                            codegen::EnumVariantKind::Option => {
                                                return Ok((argstrs[0].clone(), out, deps));
                                            }
                                            codegen::EnumVariantKind::Result => {
                                                if function.name == "Error" {
                                                    return Ok((
                                                        format!(
                                                            "({} instanceof alan_std.AlanError ? {} : null)",
                                                            argstrs[0], argstrs[0]
                                                        ),
                                                        out,
                                                        deps,
                                                    ));
                                                } else {
                                                    return Ok((
                                                        format!(
                                                            "(!({} instanceof alan_std.AlanError) ? {} : null)",
                                                            argstrs[0], argstrs[0]
                                                        ),
                                                        out,
                                                        deps,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                    // TODO: This is some funky coupling to the root.ln file
                                    return Ok((
                                        match function.name.as_str() {
                                            "string" | "String" => format!(
                                                "({} instanceof alan_std.Str ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "ExitCode" => format!(
                                                "(typeof {} === 'number' ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "f32" => format!(
                                                "({} instanceof alan_std.F32 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "f64" => format!(
                                                "({} instanceof alan_std.F64 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "i8" => format!(
                                                "({} instanceof alan_std.I8 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "i16" => format!(
                                                "({} instanceof alan_std.I16 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "i32" => format!(
                                                "({} instanceof alan_std.I32 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "i64" => format!(
                                                "({} instanceof alan_std.I64 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "u8" => format!(
                                                "({} instanceof alan_std.U8 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "u16" => format!(
                                                "({} instanceof alan_std.U16 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "u32" => format!(
                                                "({} instanceof alan_std.U32 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "u64" => format!(
                                                "({} instanceof alan_std.U64 ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "bool" => format!(
                                                "({} instanceof alan_std.Bool ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            _ => format!(
                                                "({} instanceof {} ? {} : null)",
                                                argstrs[0], function.name, argstrs[0]
                                            ),
                                        },
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
                            return Ok(("null".to_string(), out, deps));
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
                                    CType::Array(_) => Ok(enum_type.to_callable_string()),
                                    CType::Binds(..) => Ok(enum_type.to_callable_string()),
                                    otherwise => Err(format!("Cannot generate an constructor function for {ret_name} type as the input type has no name? {otherwise:?}")),
                                }?;
                                for t in ts {
                                    let inner_type = t.clone().degroup();
                                    match &*inner_type {
                                        CType::Field(n, _) if *n == enum_name => {
                                            if let Some(res) =
                                                codegen::handle_option_result_symmetry(
                                                    ts,
                                                    &mut deps,
                                                    || codegen::is_empty_variant(ts, t),
                                                    |_kind, d| {
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        Ok(format!("{} = null", argstrs[0]))
                                                    },
                                                    |_kind, d| {
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        Ok(format!(
                                                            "{} = {}",
                                                            argstrs[0], argstrs[1]
                                                        ))
                                                    },
                                                )
                                            {
                                                return Ok((res?, out, deps));
                                            }
                                            return Ok((
                                                format!("{} = {}", argstrs[0], argstrs[1]),
                                                out,
                                                deps,
                                            ));
                                        }
                                        CType::Type(n, _) if *n == enum_name => {
                                            if let Some(res) =
                                                codegen::handle_option_result_symmetry(
                                                    ts,
                                                    &mut deps,
                                                    || codegen::is_empty_variant(ts, t),
                                                    |_kind, d| {
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        Ok(format!("{} = null", argstrs[0]))
                                                    },
                                                    |_kind, d| {
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        Ok(format!(
                                                            "{} = {}",
                                                            argstrs[0], argstrs[1]
                                                        ))
                                                    },
                                                )
                                            {
                                                return Ok((res?, out, deps));
                                            }
                                            return Ok((
                                                format!("{} = {}", argstrs[0], argstrs[1]),
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
                        while matches!(&*inner_ret_type, CType::Type(..) | CType::Promise(_)) {
                            inner_ret_type = match &*inner_ret_type {
                                CType::Type(_, t) => t.clone(),
                                CType::Promise(t) => t.clone(),
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
                                    return Ok((format!("[{}]", argstrs.join(", ")), out, deps));
                                } else if argstrs.len() == 1 {
                                    return Ok((
                                        format!("new Array({}).fill({})", size, argstrs[0]),
                                        out,
                                        deps,
                                    ));
                                } else {
                                    return Err(format!("Invalid arguments {} provided for Buffer constructor function, must be either 1 element to fill, or the full size of the buffer", argstrs.join(", ")).into());
                                }
                            }
                            CType::Array(_) => {
                                return Ok((format!("[{}]", argstrs.join(", ")), out, deps));
                            }
                            CType::Shared(_) | CType::Promise(_) => {
                                return Ok((argstrs[0].clone(), out, deps));
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
                                    CType::Field(n, _) | CType::Type(n, _) => Ok(n.clone()),
                                    CType::Array(_) | CType::Binds(..) | CType::Tuple(..) | CType::Either(..) => Ok(enum_type.clone().to_callable_string()),
                                    otherwise => Err(format!("Cannot generate an constructor function for {} type as the input type has no name?, {:?}", function.name, otherwise)),
                                }?;
                                for t in ts {
                                    let inner_type = t.clone().degroup();
                                    match &*inner_type {
                                        CType::Array(_) => {
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Tuple(ts, _)
                                            if inner_type.clone().to_callable_string()
                                                == enum_name =>
                                        {
                                            if let Some(res) =
                                                codegen::handle_option_result_symmetry(
                                                    ts,
                                                    &mut deps,
                                                    || codegen::is_empty_variant(ts, t),
                                                    |_, _| Ok("null".to_string()),
                                                    |_, d| {
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        Ok(argstrs[0].clone())
                                                    },
                                                )
                                            {
                                                return Ok((res?, out, deps));
                                            }
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Either(ts, _)
                                            if inner_type.clone().to_callable_string()
                                                == enum_name =>
                                        {
                                            if let Some(res) =
                                                codegen::handle_option_result_symmetry(
                                                    ts,
                                                    &mut deps,
                                                    || codegen::is_empty_variant(ts, t),
                                                    |_, _| Ok("null".to_string()),
                                                    |_, d| {
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        Ok(argstrs[0].clone())
                                                    },
                                                )
                                            {
                                                return Ok((res?, out, deps));
                                            }
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Field(n, _) if *n == enum_name => {
                                            if let Some(res) =
                                                codegen::handle_option_result_symmetry(
                                                    ts,
                                                    &mut deps,
                                                    || codegen::is_empty_variant(ts, t),
                                                    |_, _| Ok("null".to_string()),
                                                    |_, d| {
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        Ok(argstrs[0].clone())
                                                    },
                                                )
                                            {
                                                return Ok((res?, out, deps));
                                            }
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Type(n, _) if *n == enum_name => {
                                            if let Some(res) =
                                                codegen::handle_option_result_symmetry(
                                                    ts,
                                                    &mut deps,
                                                    || codegen::is_empty_variant(ts, t),
                                                    |_, _| Ok("null".to_string()),
                                                    |_, d| {
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        let (_, dd) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            d.clone(),
                                                        )?;
                                                        *d = dd;
                                                        Ok(argstrs[0].clone())
                                                    },
                                                )
                                            {
                                                return Ok((res?, out, deps));
                                            }
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Binds(n, ..) => {
                                            match &**n {
                                                CType::TString(s) if *s == enum_name => {
                                                    if let Some(res) =
                                                        codegen::handle_option_result_symmetry(
                                                            ts,
                                                            &mut deps,
                                                            || codegen::is_empty_variant(ts, t),
                                                            |_, _| {
                                                                Ok(format!("{} = null", argstrs[0]))
                                                            },
                                                            |_, d| {
                                                                let (_, dd) =
                                                                    typen::ctype_to_jtype(
                                                                        ts[0].clone(),
                                                                        d.clone(),
                                                                    )?;
                                                                *d = dd;
                                                                let (_, dd) =
                                                                    typen::ctype_to_jtype(
                                                                        ts[1].clone(),
                                                                        d.clone(),
                                                                    )?;
                                                                *d = dd;
                                                                Ok(argstrs[0].clone())
                                                            },
                                                        )
                                                    {
                                                        return Ok((res?, out, deps));
                                                    }
                                                    return Ok((argstrs[0].clone(), out, deps));
                                                }
                                                CType::Import(n, d) => {
                                                    match &**n {
                                                        CType::TString(s) if s == &enum_name => {
                                                            super::register_nodejs_dependency(
                                                                d, &mut deps,
                                                            );
                                                            if let Some(res) = codegen::handle_option_result_symmetry(
                                                             ts,
                                                             &mut deps,
                                                             || matches!(&**t, CType::Void),
                                                             |_, _| Ok("null".to_string()),
                                                             |_, dd| {
                                                                 let (_, ddd) = typen::ctype_to_jtype(ts[0].clone(), dd.clone())?;
                                                                 *dd = ddd;
                                                                 let (_, ddd) = typen::ctype_to_jtype(ts[1].clone(), dd.clone())?;
                                                                 *dd = ddd;
                                                                 Ok(argstrs[0].clone())
                                                             },
                                                         ) {
                                                             return Ok((res?, out, deps));
                                                         }
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                        CType::TString(_)
                                                            if enum_name
                                                                == inner_type
                                                                    .clone()
                                                                    .to_callable_string() =>
                                                        {
                                                            if let Some(res) = codegen::handle_option_result_symmetry(
                                                             ts,
                                                             &mut deps,
                                                             || matches!(&**t, CType::Void),
                                                             |_, _| Ok("null".to_string()),
                                                             |_, dd| {
                                                                 let (_, ddd) = typen::ctype_to_jtype(ts[0].clone(), dd.clone())?;
                                                                 *dd = ddd;
                                                                 let (_, ddd) = typen::ctype_to_jtype(ts[1].clone(), dd.clone())?;
                                                                 *dd = ddd;
                                                                 Ok(argstrs[0].clone())
                                                             },
                                                         ) {
                                                             return Ok((res?, out, deps));
                                                         }
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                // Check for parent constructor: single argument whose type is an
                                // Either containing a superset of the child's variants
                                if argstrs.len() == 1 {
                                    let single_arg_type = match &function.args().first() {
                                        Some(t) => t.2.clone().degroup(),
                                        None => Arc::new(CType::Void),
                                    };
                                    if let CType::Either(parent_variants, _) = &*single_arg_type {
                                        // Find which parent variants match child variants
                                        let mut matched_types: Vec<String> = Vec::new();
                                        for pv in parent_variants.iter() {
                                            let parent_key =
                                                pv.clone().degroup().to_callable_string();
                                            for cv in ts {
                                                if cv.clone().degroup().to_callable_string()
                                                    == parent_key
                                                {
                                                    matched_types.push(parent_key);
                                                    break;
                                                }
                                            }
                                        }
                                        if !matched_types.is_empty()
                                            && matched_types.len() < parent_variants.len()
                                        {
                                            let parent_arg = match argstrs[0].strip_prefix("&mut ")
                                            {
                                                Some(s) => s.to_string(),
                                                None => argstrs[0].clone(),
                                            };
                                            // Generate type check: if (typeof arg === 'type') return arg; else return null;
                                            let type_checks: Vec<String> = matched_types
                                                .iter()
                                                .map(|t| match t.as_str() {
                                                    "i64" => format!(
                                                        "{} instanceof alan_std.I64",
                                                        parent_arg
                                                    ),
                                                    "u64" => format!(
                                                        "{} instanceof alan_std.U64",
                                                        parent_arg
                                                    ),
                                                    "f64" => format!(
                                                        "{} instanceof alan_std.F64",
                                                        parent_arg
                                                    ),
                                                    "i32" => format!(
                                                        "{} instanceof alan_std.I32",
                                                        parent_arg
                                                    ),
                                                    "u32" => format!(
                                                        "{} instanceof alan_std.U32",
                                                        parent_arg
                                                    ),
                                                    "i16" => format!(
                                                        "{} instanceof alan_std.I16",
                                                        parent_arg
                                                    ),
                                                    "u16" => format!(
                                                        "{} instanceof alan_std.U16",
                                                        parent_arg
                                                    ),
                                                    "i8" => format!(
                                                        "{} instanceof alan_std.I8",
                                                        parent_arg
                                                    ),
                                                    "u8" => format!(
                                                        "{} instanceof alan_std.U8",
                                                        parent_arg
                                                    ),
                                                    "f32" => format!(
                                                        "{} instanceof alan_std.F32",
                                                        parent_arg
                                                    ),
                                                    "string" => format!(
                                                        "{} instanceof alan_std.Str",
                                                        parent_arg
                                                    ),
                                                    "bool" => format!(
                                                        "{} instanceof alan_std.Bool",
                                                        parent_arg
                                                    ),
                                                    _ => format!("{}.type === '{}'", parent_arg, t),
                                                })
                                                .collect();
                                            let condition = type_checks.join(" || ");
                                            return Ok((
                                                format!("({} ? {} : null)", condition, parent_arg),
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
                                    }                                     {
                                        let child_fields: Vec<&Arc<CType>> =
                                            codegen::filter_static_fields(ts.iter()).collect();
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
                                            // Build { childField: parent.childField, ... }
                                            let field_assigns: Vec<String> = child_fields
                                                .iter()
                                                .zip(parent_indices.iter())
                                                .map(|(cf, pi)| {
                                                    let child_name = match &***cf {
                                                        CType::Field(n, _) => n.clone(),
                                                        _ => format!(
                                                            "arg{}",
                                                            (**cf)
                                                                .clone()
                                                                .to_callable_string()
                                                                .len()
                                                        ),
                                                    };
                                                    let parent_name = match &*parent_fields[*pi] {
                                                        CType::Field(n, _) => n.clone(),
                                                        _ => format!("arg{}", pi),
                                                    };
                                                    format!(
                                                        "{}: {}.{}",
                                                        child_name, parent_arg, parent_name
                                                    )
                                                })
                                                .collect();
                                            return Ok((
                                                format!("{{\n{}\n}}", field_assigns.join(",\n")),
                                                out,
                                                deps,
                                            ));
                                        }
                                    }
                                }
                                let filtered_ts =
                                    codegen::filter_static_fields(ts.iter()).collect::<Vec<&Arc<CType>>>();
                                if argstrs.len() == filtered_ts.len() {
                                    if argstrs.len() == 1 {
                                        return Ok((
                                            format!(
                                                "{{ {}: {} }}",
                                                match &**filtered_ts[0] {
                                                    CType::Field(n, _) => &n,
                                                    _ => "arg0",
                                                },
                                                argstrs[0],
                                            ),
                                            out,
                                            deps,
                                        ));
                                    } else {
                                        return Ok((
                                            format!(
                                                "{{\n{}\n}}",
                                                argstrs
                                                    .iter()
                                                    .zip(filtered_ts.iter())
                                                    .enumerate()
                                                    .map(|(i, (a, t))| format!(
                                                        "  {}: {}",
                                                        match &***t {
                                                            CType::Field(n, _) => n.clone(),
                                                            _ => format!("arg{i}"),
                                                        },
                                                        a
                                                    ))
                                                    .collect::<Vec<String>>()
                                                    .join(",\n")
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
                            CType::Field(n, _) => {
                                return Ok((format!("{{ {}: {} }}", n, argstrs[0]), out, deps));
                            }
                            CType::Binds(..) => {
                                return Ok((argstrs.join(", "), out, deps));
                            }
                            CType::Void | CType::DerivedVoid(..) => {
                                // DerivedVoid parent constructor: takes parent, returns void
                                return Ok(("undefined".to_string(), out, deps));
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
                            let mut child_type = child_type_raw.clone().degroup();
                            loop {
                                child_type = match &*child_type {
                                    CType::Type(_, t) => t.clone(),
                                    CType::Group(t) => t.clone(),
                                    CType::Promise(t) => t.clone(),
                                    _ => break,
                                };
                            }
                            let child_name = child_type.clone().to_callable_string();
                            if function.name == child_name {
                                let arg_type = function.args()[0].2.clone().degroup();
                                if let CType::Either(parent_variants, _) = &*arg_type {
                                    if let CType::Either(child_variants, _) = &*child_type {
                                        let mut matched_types: Vec<String> = Vec::new();
                                        for pv in parent_variants.iter() {
                                            let parent_key =
                                                pv.clone().degroup().to_callable_string();
                                            for cv in child_variants {
                                                if cv.clone().degroup().to_callable_string()
                                                    == parent_key
                                                {
                                                    matched_types.push(parent_key);
                                                    break;
                                                }
                                            }
                                        }
                                        if !matched_types.is_empty()
                                            && matched_types.len() < parent_variants.len()
                                        {
                                            let parent_arg = match argstrs[0].strip_prefix("&mut ")
                                            {
                                                Some(s) => s.to_string(),
                                                None => argstrs[0].clone(),
                                            };
                                            let type_checks: Vec<String> = matched_types
                                                .iter()
                                                .map(|t| match t.as_str() {
                                                    "i64" => format!(
                                                        "{} instanceof alan_std.I64",
                                                        parent_arg
                                                    ),
                                                    "u64" => format!(
                                                        "{} instanceof alan_std.U64",
                                                        parent_arg
                                                    ),
                                                    "f64" => format!(
                                                        "{} instanceof alan_std.F64",
                                                        parent_arg
                                                    ),
                                                    "i32" => format!(
                                                        "{} instanceof alan_std.I32",
                                                        parent_arg
                                                    ),
                                                    "u32" => format!(
                                                        "{} instanceof alan_std.U32",
                                                        parent_arg
                                                    ),
                                                    "i16" => format!(
                                                        "{} instanceof alan_std.I16",
                                                        parent_arg
                                                    ),
                                                    "u16" => format!(
                                                        "{} instanceof alan_std.U16",
                                                        parent_arg
                                                    ),
                                                    "i8" => format!(
                                                        "{} instanceof alan_std.I8",
                                                        parent_arg
                                                    ),
                                                    "u8" => format!(
                                                        "{} instanceof alan_std.U8",
                                                        parent_arg
                                                    ),
                                                    "f32" => format!(
                                                        "{} instanceof alan_std.F32",
                                                        parent_arg
                                                    ),
                                                    "string" => format!(
                                                        "{} instanceof alan_std.Str",
                                                        parent_arg
                                                    ),
                                                    "bool" => format!(
                                                        "{} instanceof alan_std.Bool",
                                                        parent_arg
                                                    ),
                                                    _ => format!("{}.type === '{}'", parent_arg, t),
                                                })
                                                .collect();
                                            let condition = type_checks.join(" || ");
                                            return Ok((
                                                format!("({} ? {} : null)", condition, parent_arg),
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
                    while matches!(&*fallback_ret, CType::Type(..) | CType::Promise(_)) {
                        fallback_ret = match &*fallback_ret {
                            CType::Type(_, t) => t.clone(),
                            CType::Promise(t) => t.clone(),
                            _ => fallback_ret,
                        };
                    }
                    if matches!(&*fallback_ret, CType::Shared(_) | CType::Promise(_)) {
                        return Ok((argstrs[0].clone(), out, deps));
                    }
                    Err(format!(
                        "Trying to create an automatic function for {} but the return type is {}",
                        function.name, ret_name
                    )
                    .into())
                }
            }
        }
        Microstatement::VarCall { name, typen, args } => {
            let mut argstrs = Vec::new();
            for arg in args {
                let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
                out = o;
                deps = d;
                argstrs.push(a);
            }
            let call = format!("{}({})", name, argstrs.join(", "));
            Ok((
                if is_promise_head(typen.clone()) {
                    format!("(await {call})")
                } else {
                    format!("({call})")
                },
                out,
                deps,
            ))
        }
        Microstatement::Return { value } => match value {
            Some(val) => {
                // Return position: when the returned value is a cfn-`if`, emit the native
                // `if (c) { ...A... } else { ...B... }` directly with the branches' internal
                // `return`s intact -- no `return ...;` wrapper, no closure/IIFE overhead.
                if let Microstatement::FnCall { function, args } = &**val {
                    if matches!(&function.kind, FnKind::CfnRealized(CfnKind::IfElse)) {
                        return render_js_native_ifelse(args, parent_fn, scope, false, out, deps);
                    }
                }
                let (retval, o, d) = from_microstatement(val, parent_fn, scope, out, deps)?;
                out = o;
                deps = d;
                Ok((format!("return {retval}"), out, deps))
            }
            None => Ok(("return".to_string(), out, deps)),
        },
    }
}

#[allow(clippy::type_complexity)]
pub fn generate(
    jsname: String,
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
    let mut fn_string = "".to_string();
    // First make sure all of the function argument types are defined
    let mut arg_strs = Vec::new();
    for arg in function.args() {
        let (l, _, t) = arg;
        let (_, o, d) = typen::generate(t, out, deps)?;
        out = o;
        deps = d;
        arg_strs.push(l.clone());
    }
    let ret = function.rettype().degroup();
    match &*ret {
        CType::Void | CType::DerivedVoid(..) => { /* Do nothing */ }
        CType::Type(n, _) if n == "void" => { /* Do nothing */ }
        _otherwise => {
            let (_, o, d) = typen::generate(ret, out, deps)?;
            out = o;
            deps = d;
        }
    };
    // Start generating the function output. We can do this eagerly like this because, at least for
    // now, we inline all other function calls within an "entry" function (the main function, or
    // any function that's attached to an event, or any function that's part of an exported set in
    // a shared library). LLVM *probably* doesn't deduplicate this redundancy, so this will need to
    // be revisited, but it eliminates a whole host of generation problems that I can come back to
    // later.
    let fn_async = if function_codegen_is_async(Arc::new(function.clone())) {
        "async "
    } else {
        ""
    };
    fn_string = format!(
        "{}{}function {}({}) {{\n",
        fn_string,
        fn_async,
        jsname.clone(),
        arg_strs.join(", "),
    )
    .to_string();
    let mut declared: std::collections::HashSet<String> = arg_strs.iter().cloned().collect();
    for microstatement in &function.microstatements {
        // Discarded statement position: a top-level cfn-`if` emits the native `if/else` form
        // directly rather than a pointless IIFE statement. The conditional's value is thrown away
        // here (`discard`), so a branch closure's terminal *value*-`return X` becomes a bare `X;`
        // rather than returning from this enclosing function. A bare `return;` (a genuine void
        // early-return synthesized by the `if`/`else` rewrite) is kept by `render_js_branch_body`.
        if is_ifelse_call(microstatement) {
            if let Microstatement::FnCall { args, .. } = microstatement {
                let (stmt, o, d) = render_js_native_ifelse(args, function, scope, true, out, deps)?;
                out = o;
                deps = d;
                fn_string = format!("{fn_string}    {stmt}\n");
                continue;
            }
        }
        let (stmt, o, d) = from_microstatement(microstatement, function, scope, out, deps)?;
        out = o;
        deps = d;
        let stmt = js_dedup_declaration(stmt, microstatement, &mut declared);
        if stmt.trim().is_empty() {
            continue;
        }
        fn_string = format!("{fn_string}    {stmt};\n");
    }
    fn_string = format!("{fn_string}}}");
    out.insert(jsname, fn_string);
    Ok((out, deps))
}

/// JavaScript backend implementation of the shared `Backend` trait.
pub struct LnToJs;

impl codegen::Backend for LnToJs {
    fn generate_function(
        name: String,
        function: &Function,
        scope: &Scope,
        out: OrderedHashMap<String, String>,
        deps: OrderedHashMap<String, String>,
    ) -> codegen::CodegenMapsResult {
        generate(name, function, scope, out, deps)
    }

    fn render_function_value(
        fun: &Arc<Function>,
        scope: &Scope,
        out: OrderedHashMap<String, String>,
        deps: OrderedHashMap<String, String>,
    ) -> codegen::CodegenResult<String> {
        let jsname = codegen::mangled_function_name(fun);
        let (o, d) = generate(jsname.clone(), fun, scope, out, deps)?;
        let (out, mut deps) = (o, d);
        if let FnKind::External(d) = &fun.kind {
            super::register_nodejs_dependency(d, &mut deps);
        }
        Ok((jsname, out, deps))
    }

    fn render_bind_value(
        fun: &Arc<Function>,
        out: OrderedHashMap<String, String>,
        deps: OrderedHashMap<String, String>,
    ) -> codegen::CodegenResult<String> {
        let mut deps = deps;
        if let FnKind::ExternalGeneric(_, _, d) | FnKind::ExternalBind(_, d) = &fun.kind {
            super::register_nodejs_dependency(d, &mut deps);
        }
        match &fun.kind {
            FnKind::Bind(name)
            | FnKind::BoundGeneric(_, name)
            | FnKind::ExternalBind(name, _)
            | FnKind::ExternalGeneric(_, name, _) => Ok((name.clone(), out, deps)),
            _ => Err("render_bind_value called on non-bind function kind".into()),
        }
    }
}
