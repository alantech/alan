use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::microstatement::{statement_to_microstatements, Microstatement};
use super::scope::merge;
use super::Export;
use super::FnKind;
use super::Scope;
use crate::parse;

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, CType)>,
    pub rettype: CType,
    pub microstatements: Vec<Microstatement>,
    pub kind: FnKind,
}

impl Function {
    pub fn from_ast<'a>(
        scope: Scope<'a>,
        function_ast: &parse::Functions,
        is_export: bool,
    ) -> Result<Scope<'a>, Box<dyn std::error::Error>> {
        // In the top-level of a file, all functions *must* be named
        let name = match &function_ast.optname {
            Some(name) => name.clone(),
            None => {
                return Err("Top-level function without a name!".into());
            }
        };
        Function::from_ast_with_name(scope, function_ast, is_export, name)
    }

    pub fn from_ast_with_name<'a>(
        mut scope: Scope<'a>,
        function_ast: &parse::Functions,
        is_export: bool,
        name: String,
    ) -> Result<Scope<'a>, Box<dyn std::error::Error>> {
        if let Some(generics) = &function_ast.opttypegenerics {
            // We are going to conditionally compile this type declaration. If the we get true, we
            // continue, if we get false, we don't compile and return a Fail type that isn't added
            // to the scope to cause compilation to crash *if* something tries to use this, and if
            // we don't get a boolean at all or we get multiple inner values in the generic call,
            // we bail out immediately because of a syntax error.
            let generic_call = withtypeoperatorslist_to_ctype(&generics.typecalllist, &scope)?;
            match generic_call {
                CType::Bool(b) => match b {
                    false => return Ok(scope),
                    true => { /* Do nothing */ }
                },
                CType::Type(_, c) => match *c {
                    CType::Bool(b) => match b {
                        false => return Ok(scope),
                        true => { /* Do nothing */ }
                    },
                    _ => {
                        return Err(format!(
                        "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                        name,
                        generics.to_string()
                    )
                        .into())
                    }
                },
                _ => {
                    return Err(format!(
                    "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                    name,
                    generics.to_string()
                )
                    .into())
                }
            }
        }
        if let parse::FullFunctionBody::DecOnly(_) = &function_ast.fullfunctionbody {
            if let Some(fntype) = &function_ast.opttype {
                if let Some(g) = &function_ast.optgenerics {
                    let mut generics = Vec::new();
                    // TODO: The semantics in here are different, so we may want to make a new parser
                    // type here, but for now, just do some manual parsing and blow up if we encounter
                    // something unexpected
                    let mut i = 0;
                    while i < g.typecalllist.len() {
                        match (
                            g.typecalllist.get(i),
                            g.typecalllist.get(i + 1),
                            g.typecalllist.get(i + 2),
                            g.typecalllist.get(i + 3),
                        ) {
                            (Some(t1), Some(t2), Some(t3), Some(t4))
                                if t2.to_string().trim() == ":" && t4.to_string().trim() == "," =>
                            {
                                // TODO: This should be an interface type, instead
                                generics.push((
                                    t1.to_string().trim().to_string(),
                                    CType::Infer(
                                        t1.to_string().trim().to_string(),
                                        t3.to_string().trim().to_string(),
                                    ),
                                ));
                                i += 4;
                            }
                            (Some(t1), Some(t2), Some(t3), None)
                                if t2.to_string().trim() == ":" =>
                            {
                                // TODO: This should be an interface type, instead
                                generics.push((
                                    t1.to_string().trim().to_string(),
                                    CType::Infer(
                                        t1.to_string().trim().to_string(),
                                        t3.to_string().trim().to_string(),
                                    ),
                                ));
                                i += 3; // This should exit the loop
                            }
                            (Some(t1), Some(t2), _, _) if t2.to_string().trim() == "," => {
                                // TODO: This should be an interface type, instead
                                generics.push((
                                    t1.to_string().trim().to_string(),
                                    CType::Infer(
                                        t1.to_string().trim().to_string(),
                                        "Any".to_string(),
                                    ),
                                ));
                                i += 2;
                            }
                            (Some(t1), None, None, None) => {
                                // TODO: This should be an interface type, instead
                                generics.push((
                                    t1.to_string().trim().to_string(),
                                    CType::Infer(
                                        t1.to_string().trim().to_string(),
                                        "Any".to_string(),
                                    ),
                                ));
                                i += 1;
                            }
                            (a, b, c, d) => {
                                // Any other patterns are invalid
                                return Err(format!("Unexpected generic type definition, failure to parse at {:?} {:?} {:?} {:?}", a, b, c, d).into());
                            }
                        }
                    }
                    let mut temp_scope = scope.child();
                    // This lets us partially resolve the function argument and return types
                    for g in &generics {
                        temp_scope = CType::from_ctype(temp_scope, g.0.clone(), g.1.clone());
                    }
                    let ctype = withtypeoperatorslist_to_ctype(fntype, &temp_scope)?;
                    // If the `ctype` is a Function type, we have both the input and output defined. If
                    // it's any other type, we presume it's only the input type defined
                    let (rustfunc, input_type, rettype) = match ctype {
                        CType::Call(n, f) => match &*n {
                            CType::TString(s) => {
                                match &*f {
                                    CType::Function(i, o) => (s.clone(), *i.clone(), *o.clone()),
                                    otherwise => (s.clone(), otherwise.clone(), CType::Void), // TODO: Type inference signaling?
                                }
                            }
                            _ => CType::fail("TODO: Support more than bare function calls for generic function binding"),
                        },
                        otherwise => CType::fail(&format!("A declaration-only function must be a binding Call{{N, F}}: {:?}", otherwise)),
                    };
                    // In case there were any created functions (eg constructor or accessor
                    // functions) in that path, we need to merge the child's functions back up
                    // TODO: Why can't I box this up into a function?
                    merge!(scope, temp_scope);
                    // The input type will be interpreted in many different ways:
                    // If it's a Group, unwrap it and continue. Ideally after that it's a Tuple
                    // type containing Field types, that's a "conventional" function
                    // definition, where the label becomes an argument name and the type is the
                    // type. If the tuple doesn't have Fields inside of it, we auto-generate
                    // argument names, eg `arg0`, `arg1`, etc. If it is not a Tuple type but is
                    // a Field type, we have a single argument function with a specified
                    // variable name. If it's any other type, we just label it `arg0`
                    let degrouped_input = input_type.degroup();
                    let mut args = Vec::new();
                    match degrouped_input {
                        CType::Tuple(ts) => {
                            for (i, t) in ts.iter().enumerate() {
                                args.push(match t {
                                    CType::Field(argname, t) => (argname.clone(), *t.clone()),
                                    otherwise => (format!("arg{}", i), otherwise.clone()),
                                });
                            }
                        }
                        CType::Field(argname, t) => args.push((argname.clone(), *t.clone())),
                        CType::Void => {} // Do nothing so an empty set is properly
                        otherwise => args.push(("arg0".to_string(), otherwise.clone())),
                    }
                    let function = Function {
                        name,
                        args,
                        rettype,
                        microstatements: Vec::new(),
                        kind: FnKind::BoundGeneric(generics, rustfunc.clone()),
                    };
                    if is_export {
                        scope
                            .exports
                            .insert(function.name.clone(), Export::Function);
                    }
                    if scope.functions.contains_key(&function.name) {
                        let func_vec = scope.functions.get_mut(&function.name).unwrap();
                        func_vec.push(function);
                    } else {
                        scope
                            .functions
                            .insert(function.name.clone(), vec![function]);
                    }
                    // TODO: This is a hack, I shouldn't be digging into the `fntype` to get the
                    // function name
                    //FnKind::BoundGeneric(generics, b.rustfunc.clone())
                } else {
                    let ctype = withtypeoperatorslist_to_ctype(fntype, &scope)?;
                    if is_export {
                        scope.exports.insert(name.clone(), Export::Function);
                    }
                    if scope.functions.contains_key(&name) {
                        scope
                            .functions
                            .get_mut(&name)
                            .unwrap()
                            .append(&mut ctype.to_functions(name.clone()).1);
                    } else {
                        scope
                            .functions
                            .insert(name.clone(), ctype.to_functions(name.clone()).1);
                    }
                }
                return Ok(scope);
            } else {
                return Err("Declaration-only functions must have a declared function type".into());
            }
        }
        let statements = match &function_ast.fullfunctionbody {
            parse::FullFunctionBody::FunctionBody(body) => body.statements.clone(),
            parse::FullFunctionBody::AssignFunction(assign) => {
                vec![parse::Statement::Returns(parse::Returns {
                    returnn: "return".to_string(),
                    a: " ".to_string(),
                    retval: Some(parse::RetVal {
                        assignables: assign.assignables.clone(),
                        a: "".to_string(),
                    }),
                    semicolon: ";".to_string(),
                })]
            }
            parse::FullFunctionBody::BindFunction(_) => Vec::new(),
            parse::FullFunctionBody::DecOnly(_) => unreachable!(),
        };
        let kind = match (&function_ast.fullfunctionbody, &function_ast.optgenerics) {
            (parse::FullFunctionBody::DecOnly(_), _) => unreachable!(),
            (parse::FullFunctionBody::BindFunction(b), None) => FnKind::Bind(b.rustfunc.clone()),
            (parse::FullFunctionBody::BindFunction(b), Some(g)) => {
                let mut generics = Vec::new();
                // TODO: The semantics in here are different, so we may want to make a new parser
                // type here, but for now, just do some manual parsing and blow up if we encounter
                // something unexpected
                let mut i = 0;
                while i < g.typecalllist.len() {
                    match (
                        g.typecalllist.get(i),
                        g.typecalllist.get(i + 1),
                        g.typecalllist.get(i + 2),
                        g.typecalllist.get(i + 3),
                    ) {
                        (Some(t1), Some(t2), Some(t3), Some(t4))
                            if t2.to_string().trim() == ":" && t4.to_string().trim() == "," =>
                        {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                CType::Infer(
                                    t1.to_string().trim().to_string(),
                                    t3.to_string().trim().to_string(),
                                ),
                            ));
                            i += 4;
                        }
                        (Some(t1), Some(t2), Some(t3), None) if t2.to_string().trim() == ":" => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                CType::Infer(
                                    t1.to_string().trim().to_string(),
                                    t3.to_string().trim().to_string(),
                                ),
                            ));
                            i += 3; // This should exit the loop
                        }
                        (Some(t1), Some(t2), _, _) if t2.to_string().trim() == "," => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                CType::Infer(t1.to_string().trim().to_string(), "Any".to_string()),
                            ));
                            i += 2;
                        }
                        (Some(t1), None, None, None) => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                CType::Infer(t1.to_string().trim().to_string(), "Any".to_string()),
                            ));
                            i += 1;
                        }
                        (a, b, c, d) => {
                            // Any other patterns are invalid
                            return Err(format!("Unexpected generic type definition, failure to parse at {:?} {:?} {:?} {:?}", a, b, c, d).into());
                        }
                    }
                }
                FnKind::BoundGeneric(generics, b.rustfunc.clone())
            }
            (_, Some(g)) => {
                let mut generics = Vec::new();
                // TODO: The semantics in here are different, so we may want to make a new parser
                // type here, but for now, just do some manual parsing and blow up if we encounter
                // something unexpected
                let mut i = 0;
                while i < g.typecalllist.len() {
                    match (
                        g.typecalllist.get(i),
                        g.typecalllist.get(i + 1),
                        g.typecalllist.get(i + 2),
                        g.typecalllist.get(i + 3),
                    ) {
                        (Some(t1), Some(t2), Some(t3), Some(t4))
                            if t2.to_string().trim() == ":" && t4.to_string().trim() == "," =>
                        {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                CType::Infer(
                                    t1.to_string().trim().to_string(),
                                    t3.to_string().trim().to_string(),
                                ),
                            ));
                            i += 4;
                        }
                        (Some(t1), Some(t2), Some(t3), None) if t2.to_string().trim() == ":" => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                CType::Infer(
                                    t1.to_string().trim().to_string(),
                                    t3.to_string().trim().to_string(),
                                ),
                            ));
                            i += 3; // This should exit the loop
                        }
                        (Some(t1), Some(t2), _, _) if t2.to_string().trim() == "," => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                CType::Infer(t1.to_string().trim().to_string(), "Any".to_string()),
                            ));
                            i += 2;
                        }
                        (Some(t1), None, None, None) => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                CType::Infer(t1.to_string().trim().to_string(), "Any".to_string()),
                            ));
                            i += 1;
                        }
                        (a, b, c, d) => {
                            // Any other patterns are invalid
                            return Err(format!("Unexpected generic type definition, failure to parse at {:?} {:?} {:?} {:?}", a, b, c, d).into());
                        }
                    }
                }
                FnKind::Generic(generics, statements.clone())
            }
            _ => FnKind::Normal,
        };
        // TODO: Add code to properly convert the typeassignable vec into a CType tree and use it.
        // For now, just hardwire the parsing as before.
        let (args, rettype) = match &function_ast.opttype {
            None => Ok::<(Vec<(String, CType)>, CType), Box<dyn std::error::Error>>((
                Vec::new(),
                CType::Void,
            )), // TODO: Does this path *ever* trigger?
            Some(typeassignable) if typeassignable.is_empty() => Ok((Vec::new(), CType::Void)),
            Some(typeassignable) => match &kind {
                FnKind::Generic(gs, _) | FnKind::BoundGeneric(gs, _) => {
                    let mut temp_scope = scope.child();
                    // This lets us partially resolve the function argument and return types
                    for g in gs {
                        temp_scope = CType::from_ctype(temp_scope, g.0.clone(), g.1.clone());
                    }
                    let ctype = withtypeoperatorslist_to_ctype(typeassignable, &temp_scope)?;
                    // If the `ctype` is a Function type, we have both the input and output defined. If
                    // it's any other type, we presume it's only the input type defined
                    let (input_type, output_type) = match ctype {
                        CType::Function(i, o) => (*i.clone(), *o.clone()),
                        otherwise => (otherwise.clone(), CType::Void), // TODO: Type inference signaling?
                    };
                    // In case there were any created functions (eg constructor or accessor
                    // functions) in that path, we need to merge the child's functions back up
                    // TODO: Why can't I box this up into a function?
                    merge!(scope, temp_scope);
                    // The input type will be interpreted in many different ways:
                    // If it's a Group, unwrap it and continue. Ideally after that it's a Tuple
                    // type containing Field types, that's a "conventional" function
                    // definition, where the label becomes an argument name and the type is the
                    // type. If the tuple doesn't have Fields inside of it, we auto-generate
                    // argument names, eg `arg0`, `arg1`, etc. If it is not a Tuple type but is
                    // a Field type, we have a single argument function with a specified
                    // variable name. If it's any other type, we just label it `arg0`
                    let degrouped_input = input_type.degroup();
                    let mut out_args = Vec::new();
                    match degrouped_input {
                        CType::Tuple(ts) => {
                            for (i, t) in ts.iter().enumerate() {
                                out_args.push(match t {
                                    CType::Field(argname, t) => (argname.clone(), *t.clone()),
                                    otherwise => (format!("arg{}", i), otherwise.clone()),
                                });
                            }
                        }
                        CType::Field(argname, t) => out_args.push((argname.clone(), *t.clone())),
                        CType::Void => {} // Do nothing so an empty set is properly
                        otherwise => out_args.push(("arg0".to_string(), otherwise.clone())),
                    }
                    Ok((out_args, output_type.clone()))
                }
                _ => {
                    // TODO: Figure out how to drop this duplication
                    let ctype = withtypeoperatorslist_to_ctype(typeassignable, &scope)?;
                    // If the `ctype` is a Function type, we have both the input and output defined. If
                    // it's any other type, we presume it's only the input type defined
                    let (input_type, output_type) = match ctype {
                        CType::Function(i, o) => (*i.clone(), *o.clone()),
                        otherwise => (otherwise.clone(), CType::Void), // TODO: Type inference signaling?
                    };
                    // TODO: This is getting duplicated in a few different places. The CType creation
                    // should probably centralize creating these type names and constructor functions
                    // for us rather than this hackiness. Only adding the hackery to the output_type
                    // because that's all I need, and the input type would be much more convoluted.
                    if let CType::Void = output_type {
                        // Skip this
                    } else {
                        // This particular hackery assumes that the return type is not itself a
                        // function and that it is using the `->` operator syntax. These are terrible
                        // assumptions and this hacky code needs to die soon.
                        let mut lastfnop = None;
                        for (i, ta) in typeassignable.iter().enumerate() {
                            if ta.to_string().trim() == "->" {
                                lastfnop = Some(i);
                            }
                        }
                        if let Some(lastfnop) = lastfnop {
                            let returntypeassignables =
                                typeassignable[lastfnop + 1..typeassignable.len()].to_vec();
                            // TODO: Be more complete here
                            let name = output_type.to_callable_string();
                            // Don't recreate the exact same thing. It only causes pain
                            if scope.resolve_type(&name).is_none() {
                                let parse_type = parse::Types {
                                    typen: "type".to_string(),
                                    a: "".to_string(),
                                    opttypegenerics: None,
                                    b: "".to_string(),
                                    fulltypename: parse::FullTypename {
                                        typename: name.clone(),
                                        opttypegenerics: None,
                                    },
                                    c: "".to_string(),
                                    typedef: parse::TypeDef {
                                        a: "=".to_string(),
                                        b: "".to_string(),
                                        typeassignables: returntypeassignables,
                                    },
                                    optsemicolon: ";".to_string(),
                                };
                                let res = CType::from_ast(scope, &parse_type, false)?;
                                scope = res.0;
                            }
                        }
                    }
                    // The input type will be interpreted in many different ways:
                    // If it's a Group, unwrap it and continue. Ideally after that it's a Tuple
                    // type containing Field types, that's a "conventional" function
                    // definition, where the label becomes an argument name and the type is the
                    // type. If the tuple doesn't have Fields inside of it, we auto-generate
                    // argument names, eg `arg0`, `arg1`, etc. If it is not a Tuple type but is
                    // a Field type, we have a single argument function with a specified
                    // variable name. If it's any other type, we just label it `arg0`
                    let degrouped_input = input_type.degroup();
                    let mut out_args = Vec::new();
                    match degrouped_input {
                        CType::Tuple(ts) => {
                            for (i, t) in ts.iter().enumerate() {
                                out_args.push(match t {
                                    CType::Field(argname, t) => (argname.clone(), *t.clone()),
                                    otherwise => (format!("arg{}", i), otherwise.clone()),
                                });
                            }
                        }
                        CType::Field(argname, t) => out_args.push((argname.clone(), *t.clone())),
                        CType::Void => {} // Do nothing so an empty set is properly
                        otherwise => out_args.push(("arg0".to_string(), otherwise.clone())),
                    }
                    Ok((out_args, output_type.clone()))
                }
            },
        }?;
        let microstatements = {
            let mut ms = Vec::new();
            for (name, typen) in &args {
                ms.push(Microstatement::Arg {
                    name: name.clone(),
                    typen: typen.clone(),
                });
            }
            // We can't generate the rest of the microstatements while the generic function is
            // still generic
            if function_ast.optgenerics.is_none() {
                for statement in &statements {
                    let res = statement_to_microstatements(statement, scope, ms)?;
                    scope = res.0;
                    ms = res.1;
                }
            }
            ms
        };
        let function = Function {
            name,
            args,
            rettype,
            microstatements,
            kind,
        };
        if is_export {
            scope
                .exports
                .insert(function.name.clone(), Export::Function);
        }
        if scope.functions.contains_key(&function.name) {
            let func_vec = scope.functions.get_mut(&function.name).unwrap();
            func_vec.push(function);
        } else {
            scope
                .functions
                .insert(function.name.clone(), vec![function]);
        }
        Ok(scope)
    }

    pub fn from_generic_function<'a>(
        mut scope: Scope<'a>,
        generic_function: &Function,
        generic_types: Vec<CType>,
    ) -> Result<(Scope<'a>, Function), Box<dyn std::error::Error>> {
        match &generic_function.kind {
            FnKind::Normal
            | FnKind::Bind(_)
            | FnKind::Derived
            | FnKind::DerivedVariadic
            | FnKind::Static => {
                Err("Should be impossible. Attempted to realize a non-generic function".into())
            }
            FnKind::BoundGeneric(gen_args, generic_fn_string) => {
                let arg_strs = generic_types
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>();
                let mut bind_str = generic_fn_string.clone();
                for (i, arg_str) in arg_strs.iter().enumerate() {
                    let gen_str = &gen_args[i].0;
                    bind_str = bind_str.replace(gen_str, arg_str);
                }
                let kind = FnKind::Bind(bind_str);
                let args = generic_function
                    .args
                    .iter()
                    .map(|(name, argtype)| {
                        (name.clone(), {
                            let mut a = argtype.clone();
                            for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                                a = a.swap_subtype(o, n);
                            }
                            a
                        })
                    })
                    .collect::<Vec<(String, CType)>>();
                // Make sure all argument types exist within the generic function call scope
                for (_, arg) in &args {
                    scope = CType::from_ctype(scope, arg.to_callable_string(), arg.clone());
                }
                let rettype = {
                    let mut a = generic_function.rettype.clone();
                    for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                        a = a.swap_subtype(o, n);
                    }
                    a
                };
                let microstatements = {
                    let mut ms = Vec::new();
                    for (name, typen) in &args {
                        ms.push(Microstatement::Arg {
                            name: name.clone(),
                            typen: typen.clone(),
                        });
                    }
                    ms
                };
                let name = format!(
                    "{}_{}",
                    generic_function.name,
                    generic_types
                        .iter()
                        .map(|t| t.to_callable_string())
                        .collect::<Vec<String>>()
                        .join("_")
                ); // Really bad
                let f = Function {
                    name,
                    args,
                    rettype,
                    microstatements,
                    kind,
                };
                if scope.functions.contains_key(&f.name) {
                    let func_vec = scope.functions.get_mut(&f.name).unwrap();
                    func_vec.push(f.clone());
                } else {
                    scope.functions.insert(f.name.clone(), vec![f.clone()]);
                }
                let res = match scope.functions.get(&f.name) {
                    None => Err("This should be impossible. Cannot get the function we just added to the scope"),
                    Some(fs) => Ok(fs.last().unwrap().clone()), // We know it's the last one
                                                                // because we just put it there
                }?;
                Ok((scope, res))
            }
            FnKind::Generic(gen_args, statements) => {
                let kind = FnKind::Normal;
                let args = generic_function
                    .args
                    .iter()
                    .map(|(name, argtype)| {
                        (name.clone(), {
                            let mut a = argtype.clone();
                            for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                                a = a.swap_subtype(o, n);
                            }
                            a
                        })
                    })
                    .collect::<Vec<(String, CType)>>();
                // Make sure all argument types exist within the generic function call scope
                for (_, arg) in &args {
                    scope = CType::from_ctype(scope, arg.to_callable_string(), arg.clone());
                }
                let rettype = {
                    let mut a = generic_function.rettype.clone();
                    for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                        a = a.swap_subtype(o, n);
                    }
                    a
                };
                // Make the generic names aliases to these types during statement-to-microstatement
                // generation
                for (i, (n, _)) in gen_args.iter().enumerate() {
                    scope.types.insert(n.clone(), generic_types[i].clone());
                }
                let microstatements = {
                    let mut ms = Vec::new();
                    for (name, typen) in &args {
                        ms.push(Microstatement::Arg {
                            name: name.clone(),
                            typen: typen.clone(),
                        });
                    }
                    for statement in statements {
                        let res = statement_to_microstatements(statement, scope, ms)?;
                        scope = res.0;
                        ms = res.1;
                    }
                    ms
                };
                let name = format!(
                    "{}_{}",
                    generic_function.name,
                    generic_types
                        .iter()
                        .map(|t| t.to_callable_string())
                        .collect::<Vec<String>>()
                        .join("_")
                ); // Really bad
                let f = Function {
                    name,
                    args,
                    rettype,
                    microstatements,
                    kind,
                };
                if scope.functions.contains_key(&f.name) {
                    let func_vec = scope.functions.get_mut(&f.name).unwrap();
                    func_vec.push(f.clone());
                } else {
                    scope.functions.insert(f.name.clone(), vec![f.clone()]);
                }
                let res = match scope.functions.get(&f.name) {
                    None => Err("This should be impossible. Cannot get the function we just added to the scope"),
                    Some(fs) => Ok(fs.last().unwrap().clone()), // We know it's the last one
                                                                // because we just put it there
                }?;
                Ok((scope, res))
            }
        }
    }
}
