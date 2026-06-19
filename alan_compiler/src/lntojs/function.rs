use std::sync::Arc;

use nom::combinator::all_consuming;
use nom::Parser;
use ordered_hash_map::OrderedHashMap;

use crate::lntojs::typen;
use crate::parse::{booln, integer, real};
use crate::program::{ArgKind, CType, CfnKind, FnKind, Function, Microstatement, Program, Scope};

#[allow(clippy::type_complexity)]
pub fn from_microstatement(
    microstatement: &Microstatement,
    scope: &Scope,
    parent_fn: &Function,
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
            let (val, o, d) = from_microstatement(value, scope, parent_fn, out, deps)?;
            out = o;
            deps = d;
            let n = if name.as_str() == "var" {
                "__var__"
            } else {
                name.as_str()
            };
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
            let mut inner_statements = Vec::new();
            for ms in &function.microstatements {
                let (val, o, d) = from_microstatement(ms, scope, parent_fn, out, deps)?;
                out = o;
                deps = d;
                inner_statements.push(val);
            }
            Ok((
                format!(
                    "async function ({}) {{\n        {};\n    }}",
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
                if representation.starts_with("\"") {
                    Ok((
                        format!("new alan_std.Str({})", representation.replace("\n", "\\n")),
                        out,
                        deps,
                    ))
                } else {
                    Ok((representation.clone(), out, deps))
                }
            }
            CType::Type(n, _) if n == "i64" || n == "u64" => {
                if all_consuming(integer).parse(representation).is_ok() {
                    if n == "i64" {
                        Ok((format!("new alan_std.I64({representation}n)"), out, deps))
                    } else {
                        Ok((format!("new alan_std.U64({representation}n)"), out, deps))
                    }
                } else {
                    Ok((representation.clone(), out, deps))
                }
            }
            CType::Type(n, _) if n == "f64" => {
                if all_consuming(real).parse(representation).is_ok() {
                    Ok((format!("new alan_std.F64({representation})"), out, deps))
                } else {
                    Ok((representation.clone(), out, deps))
                }
            }
            CType::Type(n, _) if n == "bool" => {
                if all_consuming(booln).parse(representation).is_ok() {
                    Ok((format!("new alan_std.Bool({representation})"), out, deps))
                } else {
                    Ok((representation.clone(), out, deps))
                }
            }
            CType::Binds(n, _) => match &**n {
                CType::TString(_) => Ok((representation.clone(), out, deps)),
                CType::Import(n, d) => {
                    super::register_nodejs_dependency(&**d, &mut deps);
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
                    Some(fun) => match &fun.kind {
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
                            let jsname = format!("{}_{}", fun.name, arg_strs.join("_"));
                            let (o, d) = generate(jsname.clone(), fun, scope, out, deps)?;
                            out = o;
                            deps = d;
                            if let FnKind::External(d) = &fun.kind {
                                super::register_nodejs_dependency(&**d, &mut deps);
                            }
                            Ok((jsname, out, deps))
                        }
                        FnKind::Bind(jsname)
                        | FnKind::BoundGeneric(_, jsname)
                        | FnKind::ExternalBind(jsname, _)
                        | FnKind::ExternalGeneric(_, jsname, _) => {
                            if let FnKind::ExternalGeneric(_, _, d) | FnKind::ExternalBind(_, d) =
                                &fun.kind
                            {
                                super::register_nodejs_dependency(&**d, &mut deps);
                            }
                            Ok((jsname.clone(), out, deps))
                        }
                    },
                }
            }
            _ => {
                if representation.as_str() == "var" {
                    Ok(("__var__".to_string(), out, deps))
                } else {
                    Ok((representation.clone(), out, deps))
                }
            }
        },
        Microstatement::Array { vals, .. } => {
            let mut val_representations = Vec::new();
            for val in vals {
                let (rep, o, d) = from_microstatement(val, scope, parent_fn, out, deps)?;
                val_representations.push(rep);
                out = o;
                deps = d;
            }
            Ok((format!("[{}]", val_representations.join(", ")), out, deps))
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
                    let (_, o, d) = typen::generate(function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut arg_strs = Vec::new();
                    for arg in &function.args() {
                        let arg_str = arg.2.clone().to_callable_string();
                        if &arg_str == "var" {
                            arg_strs.push("__var__".to_string());
                        } else {
                            arg_strs.push(arg_str);
                        }
                    }
                    let jsname = format!("{}_{}", function.name, arg_strs.join("_")).to_string();
                    let res = generate(jsname.clone(), function, scope, out, deps)?;
                    out = res.0;
                    deps = res.1;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) = from_microstatement(arg, scope, parent_fn, out, deps)?;
                        out = o;
                        deps = d;
                        if a.as_str() == "var" {
                            argstrs.push("__var__".to_string());
                        } else {
                            argstrs.push(a);
                        }
                    }
                    if let FnKind::External(d) = &function.kind {
                        super::register_nodejs_dependency(&**d, &mut deps);
                    }
                    Ok((
                        format!("(await {}({}))", jsname, argstrs.join(", ")).to_string(),
                        out,
                        deps,
                    ))
                }
                FnKind::Bind(jsname) | FnKind::ExternalBind(jsname, _) => {
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) = from_microstatement(arg, scope, parent_fn, out, deps)?;
                        out = o;
                        deps = d;
                        if a.as_str() == "var" {
                            argstrs.push("__var__".to_string());
                        } else {
                            argstrs.push(a);
                        }
                    }
                    if let FnKind::ExternalBind(_, d) = &function.kind {
                        super::register_nodejs_dependency(&**d, &mut deps);
                    }
                    Ok((
                        format!("(await {}({}))", jsname, argstrs.join(", ")).to_string(),
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
                                    super::register_nodejs_dependency(&**d, &mut deps);
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
                        let (a, o, d) = from_microstatement(arg, scope, parent_fn, out, deps)?;
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
                FnKind::Derived | FnKind::DerivedVariadic => {
                    // The initial work to get the values to construct the type is the same as
                    // with bound functions, though.
                    let (_, o, d) = typen::generate(function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) = from_microstatement(arg, scope, parent_fn, out, deps)?;
                        out = o;
                        deps = d;
                        if a.as_str() == "var" {
                            argstrs.push("__var__".to_string());
                        } else {
                            argstrs.push(a);
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
                                    // Special-casing for Option and Result mapping. TODO:
                                    // Make this more centralized
                                    if ts.len() == 2 {
                                        if let CType::Void = &*ts[1] {
                                            return Ok((argstrs[0].clone(), out, deps));
                                        } else if let CType::Type(name, _) = &*ts[1] {
                                            if name == "Error" {
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
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok((
                                                            format!("{} = null", argstrs[0]),
                                                            out,
                                                            deps,
                                                        ));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "{} = {}",
                                                                argstrs[0], argstrs[1]
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                format!(
                                                                    "{} = {}",
                                                                    argstrs[0], argstrs[1],
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "{} = {}",
                                                                    argstrs[0], argstrs[1],
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!("{} = {}", argstrs[0], argstrs[1],),
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
                                                            format!("{} = null", argstrs[0],),
                                                            out,
                                                            deps,
                                                        ));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "{} = {}",
                                                                argstrs[0], argstrs[1],
                                                            ),
                                                            out,
                                                            deps,
                                                        ));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                format!(
                                                                    "{} = {}",
                                                                    argstrs[0], argstrs[1],
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "{} = {}",
                                                                    argstrs[0], argstrs[1],
                                                                ),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
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
                            CType::Shared(_) => {
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
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok(("null".to_string(), out, deps));
                                                    } else {
                                                        return Ok((argstrs[0].clone(), out, deps));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Either(ts, _)
                                            if inner_type.clone().to_callable_string()
                                                == enum_name =>
                                        {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok(("null".to_string(), out, deps));
                                                    } else {
                                                        return Ok((argstrs[0].clone(), out, deps));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Field(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok(("null".to_string(), out, deps));
                                                    } else {
                                                        return Ok((argstrs[0].clone(), out, deps));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Type(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &*ts[1] {
                                                    if let CType::Void = &**t {
                                                        return Ok(("null".to_string(), out, deps));
                                                    } else {
                                                        return Ok((argstrs[0].clone(), out, deps));
                                                    }
                                                } else if let CType::Type(name, _) = &*ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[0].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        let (_, d) = typen::ctype_to_jtype(
                                                            ts[1].clone(),
                                                            deps,
                                                        )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = &**t {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((argstrs[0].clone(), out, deps));
                                        }
                                        CType::Binds(n, ..) => match &**n {
                                            CType::TString(s) if *s == enum_name => {
                                                // Special-casing for Option and Result mapping. TODO:
                                                // Make this more centralized
                                                if ts.len() == 2 {
                                                    if let CType::Void = &*ts[1] {
                                                        if let CType::Void = &**t {
                                                            return Ok((
                                                                "null".to_string(),
                                                                out,
                                                                deps,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                argstrs[0].clone(),
                                                                out,
                                                                deps,
                                                            ));
                                                        }
                                                    } else if let CType::Type(name, _) = &*ts[1] {
                                                        if name == "Error" {
                                                            let (_, d) = typen::ctype_to_jtype(
                                                                ts[0].clone(),
                                                                deps,
                                                            )?;
                                                            deps = d;
                                                            let (_, d) = typen::ctype_to_jtype(
                                                                ts[1].clone(),
                                                                deps,
                                                            )?;
                                                            deps = d;
                                                            if let CType::Binds(..) = &**t {
                                                                return Ok((
                                                                    argstrs[0].clone(),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            } else {
                                                                return Ok((
                                                                    argstrs[0].clone(),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            }
                                                        }
                                                    }
                                                }
                                                return Ok((argstrs[0].clone(), out, deps));
                                            }
                                            CType::Import(n, d) => match &**n {
                                                CType::TString(s) if s == &enum_name => {
                                                    super::register_nodejs_dependency(&**d, &mut deps);
                                                    // Special-casing for Option and Result mapping. TODO:
                                                    // Make this more centralized
                                                    if ts.len() == 2 {
                                                        if let CType::Void = &*ts[1] {
                                                            if let CType::Void = &**t {
                                                                return Ok((
                                                                    "null".to_string(),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            } else {
                                                                return Ok((
                                                                    argstrs[0].clone(),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            }
                                                        } else if let CType::Type(name, _) = &*ts[1]
                                                        {
                                                            if name == "Error" {
                                                                let (_, d) = typen::ctype_to_jtype(
                                                                    ts[0].clone(),
                                                                    deps,
                                                                )?;
                                                                deps = d;
                                                                let (_, d) = typen::ctype_to_jtype(
                                                                    ts[1].clone(),
                                                                    deps,
                                                                )?;
                                                                deps = d;
                                                                if let CType::Binds(..) = &**t {
                                                                    return Ok((
                                                                        argstrs[0].clone(),
                                                                        out,
                                                                        deps,
                                                                    ));
                                                                } else {
                                                                    return Ok((
                                                                        argstrs[0].clone(),
                                                                        out,
                                                                        deps,
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    return Ok((argstrs[0].clone(), out, deps));
                                                }
                                                CType::TString(_)
                                                    if enum_name
                                                        == inner_type
                                                            .clone()
                                                            .to_callable_string() =>
                                                {
                                                    // Special-casing for Option and Result mapping. TODO:
                                                    // Make this more centralized
                                                    if ts.len() == 2 {
                                                        if let CType::Void = &*ts[1] {
                                                            if let CType::Void = &**t {
                                                                return Ok((
                                                                    "null".to_string(),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            } else {
                                                                return Ok((
                                                                    argstrs[0].clone(),
                                                                    out,
                                                                    deps,
                                                                ));
                                                            }
                                                        } else if let CType::Type(name, _) = &*ts[1]
                                                        {
                                                            if name == "Error" {
                                                                let (_, d) = typen::ctype_to_jtype(
                                                                    ts[0].clone(),
                                                                    deps,
                                                                )?;
                                                                deps = d;
                                                                let (_, d) = typen::ctype_to_jtype(
                                                                    ts[1].clone(),
                                                                    deps,
                                                                )?;
                                                                deps = d;
                                                                if let CType::Binds(..) = &**t {
                                                                    return Ok((
                                                                        argstrs[0].clone(),
                                                                        out,
                                                                        deps,
                                                                    ));
                                                                } else {
                                                                    return Ok((
                                                                        argstrs[0].clone(),
                                                                        out,
                                                                        deps,
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    return Ok((argstrs[0].clone(), out, deps));
                                                }
                                                _ => {}
                                            },
                                            _ => {}
                                        },
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
                                    } {
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
                                let filtered_ts = ts
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
                                    .collect::<Vec<&Arc<CType>>>();
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
                    while matches!(&*fallback_ret, CType::Type(..)) {
                        fallback_ret = match &*fallback_ret {
                            CType::Type(_, t) => t.clone(),
                            _ => fallback_ret,
                        };
                    }
                    if let CType::Shared(_) = &*fallback_ret {
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
        Microstatement::VarCall { name, args, .. } => {
            let mut argstrs = Vec::new();
            for arg in args {
                let (a, o, d) = from_microstatement(arg, scope, parent_fn, out, deps)?;
                out = o;
                deps = d;
                argstrs.push(a);
            }
            Ok((
                format!("(await {}({}))", name, argstrs.join(", ")).to_string(),
                out,
                deps,
            ))
        }
        Microstatement::Return { value } => match value {
            Some(val) => {
                let (retval, o, d) = from_microstatement(val, scope, parent_fn, out, deps)?;
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
    fn_string = format!(
        "{}async function {}({}) {{\n",
        fn_string,
        jsname.clone(),
        arg_strs.join(", "),
    )
    .to_string();
    for microstatement in &function.microstatements {
        let (stmt, o, d) = from_microstatement(microstatement, scope, function, out, deps)?;
        out = o;
        deps = d;
        fn_string = format!("{fn_string}    {stmt};\n");
    }
    fn_string = format!("{fn_string}}}");
    out.insert(jsname, fn_string);
    Ok((out, deps))
}
