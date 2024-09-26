use nom::combinator::all_consuming;
use ordered_hash_map::OrderedHashMap;

use crate::lntojs::typen;
use crate::parse::integer;
use crate::program::{ArgKind, CType, FnKind, Function, Microstatement, Scope};

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
                Ok((format!("let {} = {}", n, val), out, deps))
            } else {
                Ok((format!("const {} = {}", n, val), out, deps))
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
        } => match &typen {
            CType::Type(n, _) if n == "string" => {
                if representation.starts_with("\"") {
                    Ok((representation.replace("\n", "\\n"), out, deps))
                } else {
                    Ok((representation.clone(), out, deps))
                }
            }
            CType::Type(n, _) if n == "i64" || n == "u64" => {
                if all_consuming(integer)(representation).is_ok() {
                    if n == "i64" {
                        Ok((format!("new alan_std.I64({}n)", representation), out, deps))
                    } else {
                        Ok((format!("new alan_std.U64({}n)", representation), out, deps))
                    }
                } else {
                    Ok((representation.clone(), out, deps))
                }
            }
            CType::Binds(n, _) => match &**n {
                CType::TString(_) => Ok((representation.clone(), out, deps)),
                CType::Import(n, d) => {
                    match &**d {
                        CType::Type(_, t) => match &**t {
                            CType::Node(d) => match &**d {
                                CType::Dependency(n, v) => {
                                    let name = match &**n {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency names must be strings"),
                                    };
                                    let version = match &**v {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency versions must be strings"),
                                    };
                                    deps.insert(name, version);
                                }
                                _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                        }
                        CType::Node(d) => match &**d {
                            CType::Dependency(n, v) => {
                                let name = match &**n {
                                    CType::TString(s) => s.clone(),
                                    _ => CType::fail("Dependency names must be strings"),
                                };
                                let version = match &**v {
                                    CType::TString(s) => s.clone(),
                                    _ => CType::fail("Dependency versions must be strings"),
                                };
                                deps.insert(name, version);
                            }
                            _ => CType::fail("Node.js dependencies must be declared with the dependency syntax")
                        }
                        otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                    }
                    match &**n {
                        CType::TString(_) => { /* Do nothing */ }
                        _ => CType::fail("Native import names must be strings"),
                    }
                    Ok((representation.clone(), out, deps))
                }
                otherwise => CType::fail(&format!(
                    "Bound types must be strings or node.js imports: {:?}",
                    otherwise
                )),
            },
            CType::Function(..) => {
                let f = scope.resolve_function_by_type(representation, typen);
                match f {
                    None => {
                        let args = parent_fn.args();
                        for (name, _, typen) in args {
                            if &name == representation {
                                if let CType::Function(_, _) = typen {
                                    // TODO: Do we need better matching? The upper stage should
                                    // have taken care of this
                                    return Ok((representation.clone(), out, deps));
                                }
                            }
                        }
                        Err(format!(
                            "Somehow can't find a definition for function {}, {:?}",
                            representation, typen
                        )
                        .into())
                    }
                    Some(fun) => match &fun.kind {
                        FnKind::Normal
                        | FnKind::External(_)
                        | FnKind::Generic(..)
                        | FnKind::Derived
                        | FnKind::DerivedVariadic
                        | FnKind::Static => {
                            let mut arg_strs = Vec::new();
                            for arg in &fun.args() {
                                arg_strs.push(arg.2.to_callable_string());
                            }
                            let jsname = format!("{}_{}", fun.name, arg_strs.join("_"));
                            let (o, d) = generate(jsname.clone(), fun, scope, out, deps)?;
                            out = o;
                            deps = d;
                            if let FnKind::External(d) = &fun.kind {
                                match &*d {
                                        CType::Type(_, t) => match &**t {
                                            CType::Node(d) => match &**d {
                                                CType::Dependency(n, v) => {
                                                    let name = match &**n {
                                                        CType::TString(s) => s.clone(),
                                                        _ => CType::fail("Dependency names must be strings"),
                                                    };
                                                    let version = match &**v {
                                                        CType::TString(s) => s.clone(),
                                                        _ => CType::fail("Dependency versions must be strings"),
                                                    };
                                                    deps.insert(name, version);
                                                }
                                                _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                            }
                                            otherwise => CType::fail(&format!("Native imports compiled to Javascript *must* be declared Node{{D}} dependencies: {:?}", otherwise))
                                        }
                                        CType::Node(d) => match &**d {
                                            CType::Dependency(n, v) => {
                                                let name = match &**n {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency names must be strings"),
                                                };
                                                let version = match &**v {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency versions must be strings"),
                                                };
                                                deps.insert(name, version);
                                            }
                                            _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                        }
                                        otherwise => CType::fail(&format!("Native imports compiled to Javascript *must* be declared Node{{D}} dependencies: {:?}", otherwise))
                                    }
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
                                match &*d {
                                        CType::Type(_, t) => match &**t {
                                            CType::Node(d) => match &**d {
                                                CType::Dependency(n, v) => {
                                                    let name = match &**n {
                                                        CType::TString(s) => s.clone(),
                                                        _ => CType::fail("Dependency names must be strings"),
                                                    };
                                                    let version = match &**v {
                                                        CType::TString(s) => s.clone(),
                                                        _ => CType::fail("Dependency versions must be strings"),
                                                    };
                                                    deps.insert(name, version);
                                                }
                                                _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                            }
                                            otherwise => CType::fail(&format!("Native imports compiled to Javascript *must* be declared Node{{D}} dependencies: {:?}", otherwise))
                                        }
                                        CType::Node(d) => match &**d {
                                            CType::Dependency(n, v) => {
                                                let name = match &**n {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency names must be strings"),
                                                };
                                                let version = match &**v {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency versions must be strings"),
                                                };
                                                deps.insert(name, version);
                                            }
                                            _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                        }
                                        otherwise => CType::fail(&format!("Native imports compiled to Javascript *must* be declared Node{{D}} dependencies: {:?}", otherwise))
                                    }
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
                let (_, o, d) = typen::generate(&arg_type, out, deps)?;
                out = o;
                deps = d;
                arg_types.push(arg_type.clone());
                let res = typen::ctype_to_jtype(&arg_type, deps)?;
                arg_type_strs.push(res.0);
                deps = res.1;
            }
            match &function.kind {
                FnKind::Generic(..) | FnKind::BoundGeneric(..) | FnKind::ExternalGeneric(..) => {
                    Err("Generic functions should have been resolved before reaching here".into())
                }
                FnKind::Normal | FnKind::External(_) => {
                    let (_, o, d) = typen::generate(&function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut arg_strs = Vec::new();
                    for arg in &function.args() {
                        arg_strs.push(arg.2.to_callable_string());
                    }
                    // Come up with a function name that is unique so Javascript doesn't choke on
                    // duplicate function names that are allowed in Alan
                    let jsname = format!("{}_{}", function.name, arg_strs.join("_")).to_string();
                    // Make the function we need, but with the name we're
                    let res = generate(jsname.clone(), function, scope, out, deps)?;
                    out = res.0;
                    deps = res.1;
                    // Now call this function
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
                        match &*d {
                            CType::Type(_, t) => match &**t {
                                CType::Node(d) => match &**d {
                                    CType::Dependency(n, v) => {
                                        let name = match &**n {
                                            CType::TString(s) => s.clone(),
                                            _ => CType::fail("Dependency name must be a string"),
                                        };
                                        let version = match &**v {
                                            CType::TString(s) => s.clone(),
                                            _ => CType::fail("Dependency version must be a string"),
                                        };
                                        deps.insert(name, version);
                                    }
                                    _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                }
                                otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                            }
                            CType::Node(d) => match &**d {
                                CType::Dependency(n, v) => {
                                    let name = match &**n {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency name must be a string"),
                                    };
                                    let version = match &**v {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency version must be a string"),
                                    };
                                    deps.insert(name, version);
                                }
                                _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                        }
                    }
                    Ok((
                        format!("await {}({})", jsname, argstrs.join(", ")).to_string(),
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
                        argstrs.push(a);
                    }
                    if let FnKind::ExternalBind(_, d) = &function.kind {
                        match &*d {
                            CType::Type(_, t) => match &**t {
                                CType::Node(d) => match &**d {
                                    CType::Dependency(n, v) => {
                                        let name = match &**n {
                                            CType::TString(s) => s.clone(),
                                            _ => CType::fail("Dependency name must be a string"),
                                        };
                                        let version = match &**v {
                                            CType::TString(s) => s.clone(),
                                            _ => CType::fail("Dependency version must be a string"),
                                        };
                                        deps.insert(name, version);
                                    }
                                    _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                }
                                otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                            }
                            CType::Node(d) => match &**d {
                                CType::Dependency(n, v) => {
                                    let name = match &**n {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency name must be a string"),
                                    };
                                    let version = match &**v {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency version must be a string"),
                                    };
                                    deps.insert(name, version);
                                }
                                _ => CType::fail("Node dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                        }
                    }
                    Ok((
                        format!("await {}({})", jsname, argstrs.join(", ")).to_string(),
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
                        } => match &typen {
                            CType::Binds(n, _) => match &**n {
                                CType::TString(_) => Ok((representation.clone(), out, deps)),
                                CType::Import(n, d) => {
                                    match &**d {
                                        CType::Type(_, t) => match &**t {
                                            CType::Node(d) => match &**d {
                                                CType::Dependency(n, v) => {
                                                    let name = match &**n {
                                                        CType::TString(s) => s.clone(),
                                                        _ => CType::fail("Dependency name must be a string"),
                                                    };
                                                    let version = match &**v {
                                                        CType::TString(s) => s.clone(),
                                                        _ => CType::fail("Dependency version must be a string"),
                                                    };
                                                    deps.insert(name, version);
                                                }
                                                _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                            }
                                            otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                                        }
                                        CType::Node(d) => match &**d {
                                            CType::Dependency(n, v) => {
                                                let name = match &**n {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency name must be a string"),
                                                };
                                                let version = match &**v {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency version must be a string"),
                                                };
                                                deps.insert(name, version);
                                            }
                                            _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                        }
                                        otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                                    }
                                    match &**n {
                                        CType::TString(_) => { /* Do nothing */ }
                                        _ => CType::fail("Native import names must be strings"),
                                    }
                                    Ok((representation.clone(), out, deps))
                                }
                                otherwise => CType::fail(&format!(
                                    "2. Bound types must be strings or node.js imports: {:?}",
                                    otherwise
                                )),
                            },
                            _ => Ok((representation.clone(), out, deps)),
                        },
                        _ => unreachable!(),
                    }
                }
                FnKind::Derived | FnKind::DerivedVariadic => {
                    // The initial work to get the values to construct the type is the same as
                    // with bound functions, though.
                    let (_, o, d) = typen::generate(&function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) = from_microstatement(arg, scope, parent_fn, out, deps)?;
                        out = o;
                        deps = d;
                        argstrs.push(a.to_string());
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
                    if function.args().len() == 1 {
                        // This is a wacky unwrapping logic...
                        let mut input_type = &function.args()[0].2;
                        while matches!(input_type, CType::Type(..) | CType::Group(_)) {
                            input_type = match input_type {
                                CType::Type(_, t) => t,
                                CType::Group(t) => t,
                                t => t,
                            };
                        }
                        match input_type {
                            CType::Tuple(ts) => {
                                // Short-circuit for direct `<N>` function calls (which can only be
                                // generated by the internals of the compiler)
                                if let Ok(i) = function.name.parse::<i64>() {
                                    if let CType::Field(n, _) = &ts[i as usize] {
                                        return Ok((format!("{}.{}", argstrs[0], n), out, deps));
                                    } else {
                                        return Ok((format!("{}.arg{}", argstrs[0], i), out, deps));
                                    }
                                }
                                let mut accessor_field = None;
                                for (i, t) in ts.iter().enumerate() {
                                    match t {
                                        CType::Field(n, _) => {
                                            if n == &function.name {
                                                accessor_field = Some(n.clone());
                                            }
                                        }
                                        _ => {
                                            if format!("arg{}", i) == function.name {
                                                accessor_field = Some(function.name.clone());
                                            }
                                        }
                                    }
                                }
                                if let Some(n) = accessor_field {
                                    return Ok((format!("{}.{}", argstrs[0], n), out, deps));
                                }
                            }
                            CType::Field(..) => {
                                return Ok((format!("{}.arg0", argstrs[0]), out, deps));
                            }
                            CType::Either(ts) => {
                                // The kinds of types allowed here are `Type`, `Bound`, and
                                // `ResolvedBoundGeneric`, and `Field`. Other types don't have
                                // a string name we can match against the function name
                                let accessor_field = ts.iter().find(|t| match t {
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
                                        if let CType::Void = &ts[1] {
                                            return Ok((argstrs[0].clone(), out, deps));
                                        } else if let CType::Type(name, _) = &ts[1] {
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
                                                "(typeof {} === 'string' ? {} : null)",
                                                argstrs[0], argstrs[0]
                                            ),
                                            "f32" | "f64" | "ExitCode" => format!(
                                                "(typeof {} === 'number' ? {} : null)",
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
                                                "(typeof {} === 'boolean' ? {} : null)",
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
                        let inner_ret_type = match &function.rettype().degroup() {
                            CType::Field(_, t) => *t.clone(),
                            CType::Type(_, t) => *t.clone(),
                            t => t.clone(),
                        };
                        if let CType::Either(_) = inner_ret_type {
                            return Ok(("null".to_string(), out, deps));
                        }
                    }
                    let ret_type = &function.rettype().degroup();
                    let ret_name = ret_type.to_callable_string();
                    if function.name == "store" {
                        let inner_ret_type = match ret_type {
                            CType::Field(_, t) => *t.clone(),
                            CType::Type(_, t) => *t.clone(),
                            t => t.clone(),
                        };
                        match inner_ret_type {
                            CType::Either(ts) => {
                                if argstrs.len() != 2 {
                                    return Err(format!("Invalid arguments {} provided for Either re-assignment function, must be two arguments", argstrs.join(", ")).into());
                                }
                                let enum_type = &function.args()[1].2.degroup();
                                let enum_name = match enum_type {
                                    CType::Field(n, _) => Ok(n.clone()),
                                    CType::Type(n, _) => Ok(n.clone()),
                                    _ => Err(format!("Cannot generate an constructor function for {} type as the input type has no name?", ret_name)),
                                }?;
                                for t in &ts {
                                    let inner_type = t.degroup();
                                    match &inner_type {
                                        CType::Field(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
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
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[0], deps)?;
                                                        deps = d;
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[1], deps)?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
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
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[0], deps)?;
                                                        deps = d;
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[1], deps)?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                                return Err(format!("Cannot assign a value of the {} type as it is not part of the {} type", enum_name, ret_name).into());
                            }
                            _ => return Err("How did this path get triggered?".into()),
                        }
                    } else if function.name == ret_name {
                        let mut inner_ret_type = ret_type.clone();
                        while matches!(inner_ret_type, CType::Type(..)) {
                            inner_ret_type = match inner_ret_type {
                                CType::Type(_, t) => *t,
                                t => t,
                            };
                        }
                        match inner_ret_type {
                            CType::Buffer(_, s) => {
                                let size = match *s {
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
                            CType::Either(ts) => {
                                if argstrs.len() > 1 {
                                    return Err(format!("Invalid arguments {} provided for Either constructor function, must be zero or one argument", argstrs.join(", ")).into());
                                }
                                let enum_type = match &function.args().first() {
                                    Some(t) => t.2.degroup(),
                                    None => CType::Void,
                                };
                                let enum_name = match &enum_type {
                                    CType::Field(n, _) => Ok(n.clone()),
                                    CType::Type(n, _) => Ok(n.clone()),
                                    _ => Err(format!("Cannot generate an constructor function for {} type as the input type has no name?", function.name)),
                                }?;
                                for t in &ts {
                                    let mut inner_type = t.degroup();
                                    if let CType::Tuple(ts) = &inner_type {
                                        if ts.len() == 1 {
                                            inner_type = ts[0].clone();
                                        }
                                    }
                                    match &inner_type {
                                        CType::Field(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
                                                        return Ok(("null".to_string(), out, deps));
                                                    } else {
                                                        return Ok((argstrs[0].clone(), out, deps));
                                                    }
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[0], deps)?;
                                                        deps = d;
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[1], deps)?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                                        CType::Field(_, f)
                                            if f.to_functional_string()
                                                == enum_type.to_functional_string() =>
                                        {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
                                                        return Ok(("null".to_string(), out, deps));
                                                    } else {
                                                        return Ok((argstrs[0].clone(), out, deps));
                                                    }
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[0], deps)?;
                                                        deps = d;
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[1], deps)?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
                                                        return Ok(("null".to_string(), out, deps));
                                                    } else {
                                                        return Ok((argstrs[0].clone(), out, deps));
                                                    }
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[0], deps)?;
                                                        deps = d;
                                                        let (_, d) =
                                                            typen::ctype_to_jtype(&ts[1], deps)?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                                                    if let CType::Void = &ts[1] {
                                                        if let CType::Void = t {
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
                                                    } else if let CType::Type(name, _) = &ts[1] {
                                                        if name == "Error" {
                                                            let (_, d) = typen::ctype_to_jtype(
                                                                &ts[0], deps,
                                                            )?;
                                                            deps = d;
                                                            let (_, d) = typen::ctype_to_jtype(
                                                                &ts[1], deps,
                                                            )?;
                                                            deps = d;
                                                            if let CType::Binds(..) = t {
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
                                                    match &**d {
                                                        CType::Type(_, t) => match &**t {
                                                            CType::Node(d) => match &**d {
                                                                CType::Dependency(n, v) => {
                                                                    let name = match &**n {
                                                                        CType::TString(s) => s.clone(),
                                                                        _ => CType::fail("Dependency name must be a string"),
                                                                    };
                                                                    let version = match &**v {
                                                                        CType::TString(s) => s.clone(),
                                                                        _ => CType::fail("Dependency version must be a string"),
                                                                    };
                                                                    deps.insert(name, version);
                                                                }
                                                                _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                                            }
                                                            otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                                                        }
                                                        CType::Node(d) => match &**d {
                                                            CType::Dependency(n, v) => {
                                                                let name = match &**n {
                                                                    CType::TString(s) => s.clone(),
                                                                    _ => CType::fail("Dependency name must be a string"),
                                                                };
                                                                let version = match &**v {
                                                                    CType::TString(s) => s.clone(),
                                                                    _ => CType::fail("Dependency version must be a string"),
                                                                };
                                                                deps.insert(name, version);
                                                            }
                                                            _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                                        }
                                                        otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                                                    }
                                                    // Special-casing for Option and Result mapping. TODO:
                                                    // Make this more centralized
                                                    if ts.len() == 2 {
                                                        if let CType::Void = &ts[1] {
                                                            if let CType::Void = t {
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
                                                        } else if let CType::Type(name, _) = &ts[1]
                                                        {
                                                            if name == "Error" {
                                                                let (_, d) = typen::ctype_to_jtype(
                                                                    &ts[0], deps,
                                                                )?;
                                                                deps = d;
                                                                let (_, d) = typen::ctype_to_jtype(
                                                                    &ts[1], deps,
                                                                )?;
                                                                deps = d;
                                                                if let CType::Binds(..) = t {
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
                                return Err(format!("Cannot generate a constructor function for {} type as it is not part of the {} type", enum_name, function.name).into());
                            }
                            CType::Tuple(ts) => {
                                // TODO: Better type checking here, but it's *probably* being
                                // done at a higher layer
                                let filtered_ts = ts
                                    .iter()
                                    .filter(|t| match t {
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
                                    .collect::<Vec<&CType>>();
                                if argstrs.len() == filtered_ts.len() {
                                    if argstrs.len() == 1 {
                                        return Ok((
                                            format!(
                                                "{{ {}: {} }}",
                                                match filtered_ts[0] {
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
                                                        match t {
                                                            CType::Field(n, _) => n.clone(),
                                                            _ => format!("arg{}", i),
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
                            otherwise => {
                                return Err(format!("How did you get here? Trying to create a constructor function {:?} for {:?}", function, otherwise).into());
                            }
                        }
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
                format!("await {}({})", name, argstrs.join(", ")).to_string(),
                out,
                deps,
            ))
        }
        Microstatement::Return { value } => match value {
            Some(val) => {
                let (retval, o, d) = from_microstatement(val, scope, parent_fn, out, deps)?;
                out = o;
                deps = d;
                Ok((format!("return {}", retval), out, deps))
            }
            None => Ok(("return".to_string(), out, deps)),
        },
    }
}

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
    for arg in &function.args() {
        let (l, _, t) = arg;
        let (_, o, d) = typen::generate(t, out, deps)?;
        out = o;
        deps = d;
        arg_strs.push(l.clone());
    }
    match &function.rettype().degroup() {
        CType::Void => { /* Do nothing */ }
        CType::Type(n, _) if n == "void" => { /* Do nothing */ }
        otherwise => {
            let (_, o, d) = typen::generate(otherwise, out, deps)?;
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
        fn_string = format!("{}    {};\n", fn_string, stmt);
    }
    fn_string = format!("{}}}", fn_string);
    out.insert(jsname, fn_string);
    Ok((out, deps))
}
