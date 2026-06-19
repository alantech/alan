// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.
use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use crate::lntors::typen;
use crate::program::{ArgKind, CType, CfnKind, FnKind, Function, Microstatement, Program, Scope};

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
    let is_named = |m: &Microstatement| {
        matches!(m, Microstatement::Value { representation, .. } if representation == name)
    };
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
        Microstatement::Value { .. } | Microstatement::Arg { .. } => false,
    }
}

/// Returns true if the type (after unwrapping `Type`/`Group`/`Shared` wrappers)
/// supports moving a field/element out of it via an accessor that borrows the
/// whole value (tuple `.N`, struct fields, fixed buffers, arrays). Keeping such a
/// value as a borrow is unsafe because the body may project-and-move out of it
/// (e.g. `b.0`), which is illegal behind a shared reference, and the escape
/// analysis treats the accessor's `&self` argument as a harmless borrow.
///
/// `Either`/sum types are deliberately *not* included: their variant access
/// either clones (`match &x { Some(v) => v.clone() }`) or takes ownership (e.g.
/// `unwrap`, which appears as an `Own` argument and is already caught by the
/// escape analysis), so a borrowed sum value is never moved out from behind the
/// reference. Scalars, strings, and opaque bound types likewise have no movable
/// projections.
fn type_has_movable_projection(t: &CType) -> bool {
    match t {
        CType::Type(_, inner) | CType::Group(inner) | CType::Shared(inner) => {
            type_has_movable_projection(inner)
        }
        CType::Tuple(..) | CType::Buffer(..) | CType::Array(_) | CType::Field(..) => true,
        _ => false,
    }
}

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
                format!("{{\n        {};\n    }}", inner_statements.join(";\n        ")),
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
                        if is_borrowable_ref_arg(name, parent_fn) {
                            // The argument is only ever used by reference, so keep it as the
                            // incoming `&T` borrow instead of defensively cloning it into an
                            // owned local. `render_arg` knows to pass it through directly.
                            Ok(("".to_string(), out, deps))
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
            Ok((
                format!(
                    "let {}{} = {}",
                    if *mutable { "mut " } else { "" },
                    name,
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
            let mut inner_statements = Vec::new();
            for ms in &function.microstatements {
                let (val, o, d) = from_microstatement(ms, parent_fn, shared_vars, scope, out, deps)?;
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
                    super::register_rust_dependency(&**d, &mut deps);
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
                                    super::register_rust_dependency(&**d, &mut deps);
                                }
                                Ok((rustname, out, deps))
                            }
                            FnKind::Bind(rustname)
                            | FnKind::BoundGeneric(_, rustname)
                            | FnKind::ExternalBind(rustname, _)
                            | FnKind::ExternalGeneric(_, rustname, _) => {
                                if let FnKind::ExternalGeneric(_, _, d)
                                | FnKind::ExternalBind(_, d) = &fun.kind
                                {
                                    super::register_rust_dependency(&**d, &mut deps);
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
                let (rep, o, d) = from_microstatement(val, parent_fn, shared_vars, scope, out, deps)?;
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
        Microstatement::FnCall { function, args } => {
            // Hackery to inline `if` calls *if* it's safe to do so.
            if let FnKind::Bind(fname) = &function.kind {
                if fname == "ifstatementhack" {
                    let res = from_microstatement(&args[0], parent_fn, shared_vars, scope, out, deps)?;
                    let conditional = res.0;
                    out = res.1;
                    deps = res.2;
                    let res = render_inline_block(&args[1], parent_fn, shared_vars, scope, out, deps)?;
                    let successblock = res.0;
                    out = res.1;
                    deps = res.2;
                    return Ok((
                        format!("if {conditional} {successblock}").to_string(),
                        out,
                        deps,
                    ));
                } else if fname == "ifelsestatementhack" {
                    let res = from_microstatement(&args[0], parent_fn, shared_vars, scope, out, deps)?;
                    let conditional = res.0;
                    out = res.1;
                    deps = res.2;
                    let res = render_inline_block(&args[1], parent_fn, shared_vars, scope, out, deps)?;
                    let successblock = res.0;
                    out = res.1;
                    deps = res.2;
                    let res = render_inline_block(&args[2], parent_fn, shared_vars, scope, out, deps)?;
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
                    let res = from_microstatement(&args[0], parent_fn, shared_vars, scope, out, deps)?;
                    let conditionalparts = res.0.split("return").collect::<Vec<&str>>();
                    let conditional = [conditionalparts[0], &conditionalparts[1].replace(";", "")]
                        .join("")
                        .replacen("|| {", "{", 1);
                    out = res.1;
                    deps = res.2;
                    let res = render_inline_block(&args[1], parent_fn, shared_vars, scope, out, deps)?;
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
                    if matches!(function.kind, FnKind::Normal)
                        && crate::program::inline::is_inline_target(
                            &crate::program::inline::fn_identity(function),
                        )
                    {
                        // Single `return <expr>` body: inline as a pure expression.
                        if let Some(subs) =
                            crate::program::inline::build_inline_substitution(function, args)
                        {
                            if let Some(expr) = crate::program::inline::single_return_expr(function)
                            {
                                let inlined = crate::program::inline::substitute(expr, &subs);
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
                                let mut block = "{\n".to_string();
                                for s in &stmts {
                                    let (rendered, o, d) = from_microstatement(
                                        s, parent_fn, shared_vars, scope, out, deps,
                                    )?;
                                    out = o;
                                    deps = d;
                                    block.push_str(&format!("        {rendered};\n"));
                                }
                                let (tail_str, o, d) = from_microstatement(
                                    &tail, parent_fn, shared_vars, scope, out, deps,
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
                        let (a, o, d) = from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
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
                        super::register_rust_dependency(&**d, &mut deps);
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
                        let (a, o, d) = from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
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
                        super::register_rust_dependency(&**d, &mut deps);
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
                                    super::register_rust_dependency(&**d, &mut deps);
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
                        let (a, o, d) = from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
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
                        let (a, o, d) = from_microstatement(arg, parent_fn, shared_vars, scope, out, deps)?;
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
                                                    super::register_rust_dependency(&**d, &mut deps);
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
                let (retval, o, d) = from_microstatement(val, parent_fn, shared_vars, scope, out, deps)?;
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
    let shared_vars = build_shared_vars(function);
    let mut fn_string = "".to_string();
    // First make sure all of the function argument types are defined
    let mut arg_strs = Vec::new();
    for arg in &function.args() {
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
        } else {
            match k {
                ArgKind::Mut => arg_strs.push(format!("{l}: &mut {t_str}")),
                ArgKind::Own => arg_strs.push(format!("{l}: {t_str}")),
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
    for microstatement in &function.microstatements {
        let (stmt, o, d) = from_microstatement(microstatement, function, &shared_vars, scope, out, deps)?;
        out = o;
        deps = d;
        fn_string = format!("{fn_string}    {stmt};\n");
    }
    fn_string = format!("{fn_string}}}");
    out.insert(rustname, fn_string);
    Ok((out, deps))
}
