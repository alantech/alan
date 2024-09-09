use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::microstatement::{statement_to_microstatements, Microstatement};
use super::scope::merge;
use super::ArgKind;
use super::Export;
use super::FnKind;
use super::Scope;
use crate::parse;

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub typen: CType,
    pub microstatements: Vec<Microstatement>,
    pub kind: FnKind,
}

pub fn type_to_args(t: &CType) -> Vec<(String, ArgKind, CType)> {
    match t {
        CType::Function(i, _) => {
            let mut args = Vec::new();
            match &**i {
                CType::Tuple(ts) => {
                    for (i, t) in ts.iter().enumerate() {
                        args.push(match t {
                            CType::Field(argname, t) => match &**t {
                                CType::Own(t) => (argname.clone(), ArgKind::Own, *t.clone()),
                                CType::Deref(t) => (argname.clone(), ArgKind::Deref, *t.clone()),
                                CType::Mut(t) => (argname.clone(), ArgKind::Mut, *t.clone()),
                                otherwise => (argname.clone(), ArgKind::Ref, otherwise.clone()),
                            },
                            CType::Own(t) => (format!("arg{}", i), ArgKind::Own, *t.clone()),
                            CType::Deref(t) => (format!("arg{}", i), ArgKind::Deref, *t.clone()),
                            CType::Mut(t) => (format!("arg{}", i), ArgKind::Mut, *t.clone()),
                            otherwise => (format!("arg{}", i), ArgKind::Ref, otherwise.clone()),
                        });
                    }
                }
                CType::Field(argname, t) => match &**t {
                    CType::Own(t) => args.push((argname.clone(), ArgKind::Own, *t.clone())),
                    CType::Deref(t) => args.push((argname.clone(), ArgKind::Deref, *t.clone())),
                    CType::Mut(t) => args.push((argname.clone(), ArgKind::Mut, *t.clone())),
                    otherwise => args.push((argname.clone(), ArgKind::Ref, otherwise.clone())),
                },
                CType::Void => { /* Do nothing */ }
                CType::Own(t) => args.push(("arg0".to_string(), ArgKind::Own, *t.clone())),
                CType::Deref(t) => args.push(("arg0".to_string(), ArgKind::Deref, *t.clone())),
                CType::Mut(t) => args.push(("arg0".to_string(), ArgKind::Mut, *t.clone())),
                otherwise => args.push(("arg0".to_string(), ArgKind::Ref, otherwise.clone())),
            }
            args
        }
        CType::Tuple(ts) => {
            let mut args = Vec::new();
            for (i, t) in ts.iter().enumerate() {
                args.push(match t {
                    CType::Field(argname, t) => match &**t {
                        CType::Own(t) => (argname.clone(), ArgKind::Own, *t.clone()),
                        CType::Deref(t) => (argname.clone(), ArgKind::Deref, *t.clone()),
                        CType::Mut(t) => (argname.clone(), ArgKind::Mut, *t.clone()),
                        otherwise => (argname.clone(), ArgKind::Ref, otherwise.clone()),
                    },
                    CType::Own(t) => (format!("arg{}", i), ArgKind::Own, *t.clone()),
                    CType::Deref(t) => (format!("arg{}", i), ArgKind::Deref, *t.clone()),
                    CType::Mut(t) => (format!("arg{}", i), ArgKind::Mut, *t.clone()),
                    otherwise => (format!("arg{}", i), ArgKind::Ref, otherwise.clone()),
                });
            }
            args
        }
        CType::Field(argname, t) => match &**t {
            CType::Own(t) => vec![(argname.clone(), ArgKind::Own, *t.clone())],
            CType::Deref(t) => vec![(argname.clone(), ArgKind::Deref, *t.clone())],
            CType::Mut(t) => vec![(argname.clone(), ArgKind::Mut, *t.clone())],
            otherwise => vec![(argname.clone(), ArgKind::Ref, otherwise.clone())],
        },
        CType::Void => Vec::new(),
        CType::Own(t) => vec![("arg0".to_string(), ArgKind::Own, *t.clone())],
        CType::Deref(t) => vec![("arg0".to_string(), ArgKind::Deref, *t.clone())],
        CType::Mut(t) => vec![("arg0".to_string(), ArgKind::Mut, *t.clone())],
        otherwise => vec![("arg0".to_string(), ArgKind::Ref, otherwise.clone())],
    }
}

pub fn type_to_rettype(t: &CType) -> CType {
    match t {
        CType::Function(_, o) => *o.clone(),
        _ => CType::Void,
    }
}

pub fn args_and_rettype_to_type(args: Vec<(String, ArgKind, CType)>, rettype: CType) -> CType {
    CType::Function(
        Box::new(if args.is_empty() {
            CType::Void
        } else {
            CType::Tuple(
                args.into_iter()
                    .map(|(n, k, t)| {
                        CType::Field(
                            n,
                            Box::new(match k {
                                ArgKind::Mut => CType::Mut(Box::new(t)),
                                ArgKind::Ref => t,
                                ArgKind::Own | ArgKind::Deref => CType::fail(
                                    "Somehow got an Own or Deref for a normal Alan function",
                                ),
                            }),
                        )
                    })
                    .collect::<Vec<CType>>(),
            )
        }),
        Box::new(rettype),
    )
}

impl Function {
    pub fn args(&self) -> Vec<(String, ArgKind, CType)> {
        type_to_args(&self.typen)
    }

    pub fn rettype(&self) -> CType {
        type_to_rettype(&self.typen)
    }

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
                    let (kind, input_type, rettype) = match ctype {
                        CType::Call(n, f) => match &*n {
                            CType::TString(s) => {
                                match &*f {
                                    CType::Function(i, o) => (FnKind::BoundGeneric(generics, s.clone()), *i.clone(), *o.clone()),
                                    otherwise => (FnKind::BoundGeneric(generics, s.clone()), otherwise.clone(), CType::Infer("unknown".to_string(), "unknown".to_string())),
                                }
                            }
                            CType::Import(n, d) => {
                                match &**n {
                                    CType::TString(s) => {
                                        match &*f {
                                            CType::Function(i, o) => (FnKind::ExternalGeneric(generics, s.clone(), *d.clone()), *i.clone(), *o.clone()),
                                            otherwise => (FnKind::ExternalGeneric(generics, s.clone(), *d.clone()), otherwise.clone(), CType::Infer("unknown".to_string(), "unknown".to_string())),
                                        }
                                    }
                                    _ => CType::fail("TODO: Support more than bare function imports for generic function binding"),
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
                    let degrouped_input = input_type.degroup();
                    let function = Function {
                        name,
                        typen: CType::Function(Box::new(degrouped_input), Box::new(rettype)),
                        microstatements: Vec::new(),
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
            parse::FullFunctionBody::DecOnly(_) => unreachable!(),
        };
        let kind = match (&function_ast.fullfunctionbody, &function_ast.optgenerics) {
            (parse::FullFunctionBody::DecOnly(_), _) => unreachable!(),
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
        let mut typen = match &function_ast.opttype {
            None => Ok::<CType, Box<dyn std::error::Error>>(CType::Function(
                Box::new(CType::Void),
                Box::new(CType::Infer("unknown".to_string(), "unknown".to_string())),
            )),
            Some(typeassignable) if typeassignable.is_empty() => Ok(CType::Function(
                Box::new(CType::Void),
                Box::new(CType::Infer("unknown".to_string(), "unknown".to_string())),
            )),
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
                        otherwise => (
                            otherwise.clone(),
                            CType::Infer("unknown".to_string(), "unknown".to_string()),
                        ),
                    };
                    // In case there were any created functions (eg constructor or accessor
                    // functions) in that path, we need to merge the child's functions back up
                    merge!(scope, temp_scope);
                    let degrouped_input = input_type.degroup();
                    Ok(CType::Function(
                        Box::new(degrouped_input),
                        Box::new(output_type),
                    ))
                }
                _ => {
                    let ctype = withtypeoperatorslist_to_ctype(typeassignable, &scope)?;
                    // If the `ctype` is a Function type, we have both the input and output defined. If
                    // it's any other type, we presume it's only the input type defined

                    let (input_type, output_type) = match ctype {
                        CType::Function(i, o) => (*i.clone(), *o.clone()),
                        otherwise => (
                            otherwise.clone(),
                            CType::Infer("unknown".to_string(), "unknown".to_string()),
                        ),
                    };
                    let degrouped_input = input_type.degroup();
                    Ok(CType::Function(
                        Box::new(degrouped_input),
                        Box::new(output_type),
                    ))
                }
            },
        }?;
        let microstatements = {
            let mut ms = Vec::new();
            for (name, kind, typen) in type_to_args(&typen) {
                ms.push(Microstatement::Arg { name, kind, typen });
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
        // Determine the actual return type of the function and check if it matches the specified
        // return type (or update that return type if it's to be inferred
        if let Some(ms) = microstatements.last() {
            if let Microstatement::Arg { .. } = ms {
                // Don't do anything in this path, this is probably a derived function
            } else {
                let current_rettype = type_to_rettype(&typen);
                let actual_rettype = match ms {
                    Microstatement::Return { value: Some(v) } => v.get_type(),
                    _ => CType::Void,
                };
                if let CType::Infer(..) = current_rettype {
                    // We're definitely replacing with the inferred type
                    let input_type = match &typen {
                        CType::Function(i, _) => *i.clone(),
                        _ => CType::Void,
                    };
                    typen = CType::Function(Box::new(input_type), Box::new(actual_rettype));
                } else if current_rettype.to_strict_string(false)
                    != actual_rettype.to_strict_string(false)
                {
                    CType::fail(&format!(
                        "Function {} specified to return {} but actually returns {}",
                        name,
                        current_rettype.to_strict_string(false),
                        actual_rettype.to_strict_string(false),
                    ));
                } else {
                    // Do nothing, they're the same
                }
            }
        }
        // TODO: This is getting duplicated in a few different places. The CType creation
        // should probably centralize creating these type names and constructor functions
        // for us rather than this hackiness. Only adding the hackery to the output_type
        // because that's all I need, and the input type would be much more convoluted.
        match &typen {
            CType::Function(i, o) => {
                match &**o {
                    CType::Void => { /* Do nothing */ }
                    CType::Infer(t, _) if t == "unknown" && function_ast.optgenerics.is_none() => {
                        CType::fail(&format!(
                            "The return type for {}({}) could not be inferred.",
                            name,
                            i.to_strict_string(false)
                        ));
                    }
                    CType::Infer(..) => { /* Do nothing */ }
                    otherwise => {
                        let name = otherwise.to_callable_string();
                        // Don't recreate the exact same thing. It only causes pain
                        if scope.resolve_type(&name).is_none() {
                            scope = CType::from_ctype(scope, name, otherwise.clone());
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        let function = Function {
            name,
            typen,
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
            | FnKind::External(_)
            | FnKind::Bind(_)
            | FnKind::ExternalBind(_, _)
            | FnKind::Derived
            | FnKind::DerivedVariadic
            | FnKind::Static => {
                Err("Should be impossible. Attempted to realize a non-generic function".into())
            }
            FnKind::BoundGeneric(gen_args, generic_fn_string)
            | FnKind::ExternalGeneric(gen_args, generic_fn_string, _) => {
                let arg_strs = generic_types
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>();
                let mut bind_str = generic_fn_string.clone();
                for (i, arg_str) in arg_strs.iter().enumerate() {
                    let gen_str = &gen_args[i].0;
                    bind_str = bind_str.replace(gen_str, arg_str);
                }
                let kind = match &generic_function.kind {
                    FnKind::BoundGeneric(..) => FnKind::Bind(bind_str),
                    FnKind::ExternalGeneric(_, _, d) => FnKind::ExternalBind(bind_str, d.clone()),
                    _ => unreachable!(),
                };
                let args = generic_function
                    .args()
                    .iter()
                    .map(|(name, kind, argtype)| {
                        (name.clone(), kind.clone(), {
                            let mut a = argtype.clone();
                            for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                                a = a.swap_subtype(o, n);
                            }
                            a
                        })
                    })
                    .collect::<Vec<(String, ArgKind, CType)>>();
                // Make sure all argument types exist within the generic function call scope
                for (_, _, arg) in &args {
                    scope = CType::from_ctype(scope, arg.to_callable_string(), arg.clone());
                }
                let mut rettype = {
                    let mut a = generic_function.rettype().clone();
                    for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                        a = a.swap_subtype(o, n);
                    }
                    a
                };
                let microstatements = {
                    let mut ms = Vec::new();
                    for (name, kind, typen) in &args {
                        ms.push(Microstatement::Arg {
                            name: name.clone(),
                            kind: kind.clone(),
                            typen: typen.clone(),
                        });
                    }
                    ms
                };
                // Determine the actual return type of the function and check if it matches the specified
                // return type (or update that return type if it's to be inferred
                if let Some(ms) = microstatements.last() {
                    if let Microstatement::Arg { .. } = ms {
                        // Don't do anything in this path, this is probably a derived function
                    } else {
                        let actual_rettype = match ms {
                            Microstatement::Return { value: Some(v) } => v.get_type(),
                            _ => CType::Void,
                        };
                        if let CType::Infer(..) = &rettype {
                            rettype = actual_rettype;
                        } else if rettype.to_strict_string(false)
                            != actual_rettype.to_strict_string(false)
                        {
                            CType::fail(&format!(
                                "Function {} specified to return {} but actually returns {}",
                                generic_function.name,
                                rettype.to_strict_string(false),
                                actual_rettype.to_strict_string(false),
                            ));
                        } else {
                            // Do nothing, they're the same
                        }
                    }
                }
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
                    // TODO: Can I eliminate this indirection?
                    typen: args_and_rettype_to_type(args, rettype),
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
                    .args()
                    .iter()
                    .map(|(name, kind, argtype)| {
                        (name.clone(), kind.clone(), {
                            let mut a = argtype.clone();
                            for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                                a = a.swap_subtype(o, n);
                            }
                            a
                        })
                    })
                    .collect::<Vec<(String, ArgKind, CType)>>();
                // Make sure all argument types exist within the generic function call scope
                for (_, _, arg) in &args {
                    scope = CType::from_ctype(scope, arg.to_callable_string(), arg.clone());
                }
                let mut rettype = {
                    let mut a = generic_function.rettype().clone();
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
                    for (name, kind, typen) in &args {
                        ms.push(Microstatement::Arg {
                            name: name.clone(),
                            kind: kind.clone(),
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
                // Determine the actual return type of the function and check if it matches the specified
                // return type (or update that return type if it's to be inferred
                if let Some(ms) = microstatements.last() {
                    if let Microstatement::Arg { .. } = ms {
                        // Don't do anything in this path, this is probably a derived function
                    } else {
                        let actual_rettype = match ms {
                            Microstatement::Return { value: Some(v) } => v.get_type(),
                            _ => CType::Void,
                        };
                        if let CType::Infer(..) = &rettype {
                            rettype = actual_rettype;
                        } else if rettype.to_strict_string(false)
                            != actual_rettype.to_strict_string(false)
                        {
                            CType::fail(&format!(
                                "Function {} specified to return {} but actually returns {}",
                                generic_function.name,
                                rettype.to_strict_string(false),
                                actual_rettype.to_strict_string(false),
                            ));
                        } else {
                            // Do nothing, they're the same
                        }
                    }
                }
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
                    // TODO: Can I eliminate this indirection?
                    typen: args_and_rettype_to_type(args, rettype),
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
