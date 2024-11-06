// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.
use ordered_hash_map::OrderedHashMap;

use crate::lntors::typen;
use crate::program::{ArgKind, CType, FnKind, Function, Microstatement, Program, Scope};

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
        Microstatement::Arg { name, kind, typen } => {
            // TODO: Update the serialization logic to understand values vs references so we can
            // eliminate this useless (and harmful for mutable references) clone
            if let CType::Function { .. } = typen {
                Ok(("".to_string(), out, deps))
            } else {
                match &kind {
                    ArgKind::Mut => Ok(("".to_string(), out, deps)), // We actively want to mutate the argument, don't
                    // alias it
                    ArgKind::Own => Ok(("".to_string(), out, deps)), // We already own the value
                    ArgKind::Ref => Ok((
                        format!("let mut {} = {}.clone()", name, name), // TODO: not always mutable
                        out,
                        deps,
                    )), // TODO: Should these two be distinguished?
                    ArgKind::Deref => Ok((
                        format!("let mut {} = *{}", name, name), // TODO: not always mutable
                        out,
                        deps,
                    )),
                }
            }
        }
        Microstatement::Assignment {
            name,
            value,
            mutable: _,
        } => {
            let (val, o, d) = from_microstatement(value, parent_fn, scope, out, deps)?;
            // I wish I didn't have to write the following line because you can't re-assign a
            // variable in a let destructuring, afaict
            out = o;
            deps = d;
            Ok((
                format!(
                    "let {}{} = {}",
                    // TODO: Shouldn't always be mut
                    "mut ",
                    name,
                    match val.strip_prefix("&mut ") {
                        Some(s) => s,
                        None => &val,
                    }
                ),
                out,
                deps,
            ))
        }
        Microstatement::Closure { function } => {
            let arg_names = function
                .args()
                .into_iter()
                .map(|(n, _, _)| n)
                .collect::<Vec<String>>();
            let mut inner_statements = Vec::new();
            for ms in &function.microstatements {
                let (val, o, d) = from_microstatement(ms, parent_fn, scope, out, deps)?;
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
        } => match &typen {
            CType::Type(n, _) if n == "string" => Ok((
                format!("{}.to_string()", representation).to_string(),
                out,
                deps,
            )),
            CType::Binds(n, _) => match &**n {
                CType::TString(s) => {
                    if s == "String" {
                        Ok((
                            format!("{}.to_string()", representation).to_string(),
                            out,
                            deps,
                        ))
                    } else {
                        Ok((representation.clone(), out, deps))
                    }
                }
                CType::Import(n, d) => {
                    match &**d {
                        CType::Type(_, t) => match &**t {
                            CType::Rust(d) => match &**d {
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
                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                        }
                        CType::Rust(d) => match &**d {
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
                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                        }
                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                    }
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
                let f = scope.resolve_function_by_type(representation, typen);
                let f = match f {
                    None => {
                        // If the current scope isn't the original scope for the parent function, maybe the
                        // function we're looking for is in the original scope
                        if parent_fn.origin_scope_path != scope.path {
                            let program = Program::get_program();
                            let out = match program.scope_by_file(&parent_fn.origin_scope_path) {
                                Ok(original_scope) => original_scope
                                    .resolve_function_by_type(representation, typen)
                                    .cloned(),
                                Err(_) => None,
                            };
                            Program::return_program(program);
                            out
                        } else {
                            None
                        }
                    }
                    f => f.cloned(), // TODO: Can I avoid this?
                };
                match &f {
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
                    Some(fun) => {
                        match &fun.kind {
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
                                // Come up with a function name that is unique so Rust doesn't choke on
                                // duplicate function names that are allowed in Alan
                                let rustname = format!("{}_{}", fun.name, arg_strs.join("_"));
                                // Make the function we need, but with the name we're
                                let res = generate(rustname.clone(), fun, scope, out, deps)?;
                                out = res.0;
                                deps = res.1;
                                if let FnKind::External(d) = &fun.kind {
                                    match &*d {
                                        CType::Type(_, t) => match &**t {
                                            CType::Rust(d) => match &**d {
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
                                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                            }
                                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                                        }
                                        CType::Rust(d) => match &**d {
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
                                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                        }
                                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                                    }
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
                                    match &*d {
                                        CType::Type(_, t) => match &**t {
                                            CType::Rust(d) => match &**d {
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
                                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                            }
                                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                                        }
                                        CType::Rust(d) => match &**d {
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
                                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                        }
                                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                                    }
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
                let (rep, o, d) = from_microstatement(val, parent_fn, scope, out, deps)?;
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
            let mut arg_types = Vec::new();
            let mut arg_type_strs = Vec::new();
            for arg in args {
                let arg_type = arg.get_type();
                let (_, o, d) = typen::generate(&arg_type, out, deps)?;
                out = o;
                deps = d;
                arg_types.push(arg_type.clone());
                let res = typen::ctype_to_rtype(&arg_type, true, deps)?;
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
                    // Come up with a function name that is unique so Rust doesn't choke on
                    // duplicate function names that are allowed in Alan
                    let rustname = format!("{}_{}", function.name, arg_strs.join("_")).to_string();
                    // Make the function we need, but with the name we're
                    let res = generate(rustname.clone(), function, scope, out, deps)?;
                    out = res.0;
                    deps = res.1;
                    // Now call this function
                    let mut argstrs = Vec::new();
                    for (i, arg) in args.iter().enumerate() {
                        let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
                        out = o;
                        deps = d;
                        // If the argument is itself a function, this is the only place in Rust
                        // where you can't pass by reference, so we check the type and change
                        // the argument output accordingly.
                        let arg_type = arg.get_type();
                        match arg_type {
                            CType::Function(..) => argstrs.push(a.to_string()),
                            _ => match function.args()[i].1 {
                                ArgKind::Mut => {
                                    let mut prefix = "&mut ";
                                    for (name, kind, _) in &parent_fn.args() {
                                        if name == &a {
                                            if let ArgKind::Mut = kind {
                                                prefix = "";
                                            }
                                        }
                                    }
                                    argstrs.push(format!("{}{}", prefix, a));
                                }
                                // Because we create clones for these two right now, we always need
                                // this
                                ArgKind::Ref | ArgKind::Deref => argstrs.push(format!("&{}", a)),
                                ArgKind::Own => argstrs.push(a.clone()),
                            },
                        }
                    }
                    if let FnKind::External(d) = &function.kind {
                        match &*d {
                            CType::Type(_, t) => match &**t {
                                CType::Rust(d) => match &**d {
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
                                    _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                }
                                otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                            }
                            CType::Rust(d) => match &**d {
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
                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                        }
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
                        let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
                        out = o;
                        deps = d;
                        // If the argument is itself a function, this is the only place in Rust
                        // where you can't pass by reference, so we check the type and change
                        // the argument output accordingly.
                        let arg_type = arg.get_type();
                        match arg_type {
                            CType::Function(..) => argstrs.push(a.to_string()),
                            _ => match function.args()[i].1 {
                                ArgKind::Mut => {
                                    let mut prefix = "&mut ";
                                    for (name, kind, _) in &parent_fn.args() {
                                        if name == &a {
                                            if let ArgKind::Mut = kind {
                                                prefix = "";
                                            }
                                        }
                                    }
                                    argstrs.push(format!("{}{}", prefix, a));
                                }
                                // Because we create clones for these two right now, we always need
                                // this
                                ArgKind::Ref | ArgKind::Deref => argstrs.push(format!("&{}", a)),
                                ArgKind::Own => argstrs.push(a.clone()),
                            },
                        }
                    }
                    if let FnKind::ExternalBind(_, d) = &function.kind {
                        match &*d {
                            CType::Type(_, t) => match &**t {
                                CType::Rust(d) => match &**d {
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
                                    _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                }
                                otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                            }
                            CType::Rust(d) => match &**d {
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
                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                        }
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
                        } => match &typen {
                            CType::Type(n, _) if n == "string" => Ok((
                                format!("{}.to_string()", representation).to_string(),
                                out,
                                deps,
                            )),
                            CType::Binds(n, _) => match &**n {
                                CType::TString(s) => {
                                    if s == "String" {
                                        Ok((
                                            format!("{}.to_string()", representation).to_string(),
                                            out,
                                            deps,
                                        ))
                                    } else {
                                        Ok((representation.clone(), out, deps))
                                    }
                                }
                                CType::Import(n, d) => {
                                    match &**d {
                                        CType::Type(_, t) => match &**t {
                                            CType::Rust(d) => match &**d {
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
                                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                            }
                                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                                        }
                                        CType::Rust(d) => match &**d {
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
                                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                        }
                                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                                    }
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
                FnKind::Derived | FnKind::DerivedVariadic => {
                    // The initial work to get the values to construct the type is the same as
                    // with bound functions, though.
                    let (_, o, d) = typen::generate(&function.rettype(), out, deps)?;
                    out = o;
                    deps = d;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
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
                                    return Ok((format!("{}.{}", argstrs[0], i), out, deps));
                                }
                                let accessor_field = ts
                                    .iter()
                                    .filter(|t1| match t1 {
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
                                    .find(|(_, t)| match t {
                                        CType::Field(n, _) => *n == function.name,
                                        _ => false,
                                    });
                                if let Some((i, _)) = accessor_field {
                                    return Ok((format!("{}.{}", argstrs[0], i), out, deps));
                                }
                            }
                            CType::Field(..) => {
                                return Ok((format!("{}.0", argstrs[0]), out, deps));
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
                                // We're assuming the enum sub-type naming scheme also follows
                                // the convention of matching the type name or field name,
                                // which works because we're generating all of the code that
                                // defines the enums. We also need the name of the enum for
                                // this to work, so we're assuming we got it from the first
                                // function argument. We blow up here if the first argument is
                                // *not* a Type we can get an enum name from (it *shouldn't* be
                                // possible, but..)
                                let enum_type = function.args()[0].2.degroup();
                                let enum_name = enum_type.to_callable_string();
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
                                                    return Ok((format!("(match &{} {{ Err(e) => Some(e.clone()), _ => None }})", argstrs[0]), out, deps));
                                                } else {
                                                    return Ok((format!("(match &{} {{ Ok(v) => Some(v.clone()), _ => None }})", argstrs[0]), out, deps));
                                                }
                                            }
                                        }
                                    }
                                    return Ok((
                                        format!(
                                            "(match &{} {{ {}::{}(v) => Some(v.clone()), _ => None }})",
                                            argstrs[0], enum_name, function.name
                                        ),
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
                            return Ok(("None".to_string(), out, deps));
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
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                &ts[0], true, deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                &ts[1], true, deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
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
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                &ts[0], true, deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                &ts[1], true, deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                            CType::Either(ts) => {
                                if argstrs.len() > 1 {
                                    return Err(format!("Invalid arguments {} provided for Either constructor function, must be zero or one argument", argstrs.join(", ")).into());
                                }
                                let enum_type = match &function.args().first() {
                                    Some(t) => t.2.degroup(),
                                    None => CType::Void,
                                };
                                let enum_name = match enum_type {
                                    CType::Field(n, _) => Ok(n.clone()),
                                    CType::Type(n, _) => Ok(n.clone()),
                                    _ => Err(format!("Cannot generate an constructor function for {} type as the input type has no name?", function.name)),
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
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                &ts[0], true, deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                &ts[1], true, deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
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
                                                } else if let CType::Type(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        let (okrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                &ts[0], true, deps,
                                                            )?;
                                                        deps = d;
                                                        let (errrustname, d) =
                                                            typen::ctype_to_rtype(
                                                                &ts[1], true, deps,
                                                            )?;
                                                        deps = d;
                                                        if let CType::Binds(..) = t {
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
                                                    if let CType::Void = &ts[1] {
                                                        if let CType::Void = t {
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
                                                    } else if let CType::Type(name, _) = &ts[1] {
                                                        if name == "Error" {
                                                            let (okrustname, d) =
                                                                typen::ctype_to_rtype(
                                                                    &ts[0], true, deps,
                                                                )?;
                                                            deps = d;
                                                            let (errrustname, d) =
                                                                typen::ctype_to_rtype(
                                                                    &ts[1], true, deps,
                                                                )?;
                                                            deps = d;
                                                            if let CType::Binds(..) = t {
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
                                                    match &**d {
                                                        CType::Type(_, t) => match &**t {
                                                            CType::Rust(d) => match &**d {
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
                                                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                                            }
                                                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                                                        }
                                                        CType::Rust(d) => match &**d {
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
                                                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                                        }
                                                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {:?}", otherwise))
                                                    }
                                                    // Special-casing for Option and Result mapping. TODO:
                                                    // Make this more centralized
                                                    if ts.len() == 2 {
                                                        if let CType::Void = &ts[1] {
                                                            if let CType::Void = t {
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
                                                        } else if let CType::Type(name, _) = &ts[1]
                                                        {
                                                            if name == "Error" {
                                                                let (okrustname, d) =
                                                                    typen::ctype_to_rtype(
                                                                        &ts[0], true, deps,
                                                                    )?;
                                                                deps = d;
                                                                let (errrustname, d) =
                                                                    typen::ctype_to_rtype(
                                                                        &ts[1], true, deps,
                                                                    )?;
                                                                deps = d;
                                                                if let CType::Binds(..) = t {
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
                                        _ => {}
                                    }
                                }
                                return Err(format!("Cannot generate a constructor function for {} type as it is not part of the {} type", enum_name, function.name).into());
                            }
                            CType::Tuple(ts) => {
                                // TODO: Better type checking here, but it's *probably* being
                                // done at a higher layer
                                if argstrs.len()
                                    == ts
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
                                        .collect::<Vec<&CType>>()
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
                let (a, o, d) = from_microstatement(arg, parent_fn, scope, out, deps)?;
                out = o;
                deps = d;
                // If the argument is itself a function, this is the only place in Rust
                // where you can't pass by reference, so we check the type and change
                // the argument output accordingly.
                let arg_type = arg.get_type();
                match arg_type {
                    CType::Function(..) => argstrs.push(a.to_string()),
                    // TODO: How to figure out the arg kinds for a VarCall
                    _ => argstrs.push(if a.starts_with("&mut ") {
                        a
                    } else {
                        format!("&mut {}", a)
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
                let (retval, o, d) = from_microstatement(val, parent_fn, scope, out, deps)?;
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
    let mut fn_string = "".to_string();
    // First make sure all of the function argument types are defined
    let mut arg_strs = Vec::new();
    for arg in &function.args() {
        let (l, k, t) = arg;
        let (t_str, o, d) = typen::generate(t, out, deps)?;
        out = o;
        deps = d;
        if t_str.starts_with("impl") || t_str.starts_with("&") {
            arg_strs.push(format!("{}: {}", l, t_str));
        } else {
            match k {
                ArgKind::Mut => arg_strs.push(format!("{}: &mut {}", l, t_str)),
                ArgKind::Own => arg_strs.push(format!("{}: {}", l, t_str)),
                _ => arg_strs.push(format!("{}: &{}", l, t_str)),
            }
        }
    }
    let opt_ret_str = match &function.rettype().degroup() {
        CType::Void => None,
        CType::Type(n, _) if n == "void" => None,
        otherwise => {
            let (t_str, o, d) = typen::generate(otherwise, out, deps)?;
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
            Some(rettype) => format!(" -> {}", rettype).to_string(),
            None => "".to_string(),
        },
    )
    .to_string();
    for microstatement in &function.microstatements {
        let (stmt, o, d) = from_microstatement(microstatement, function, scope, out, deps)?;
        out = o;
        deps = d;
        fn_string = format!("{}    {};\n", fn_string, stmt);
    }
    fn_string = format!("{}}}", fn_string);
    out.insert(rustname, fn_string);
    Ok((out, deps))
}
