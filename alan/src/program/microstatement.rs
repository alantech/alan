use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::function::{type_to_args, type_to_rettype};
use super::scope::merge;
use super::ArgKind;
use super::FnKind;
use super::Function;
use super::OperatorMapping;
use super::Program;
use super::Scope;
use crate::parse;

/// Microstatements are a reduced syntax that doesn't have operators, methods, or reassigning to
/// the same variable. (We'll rely on LLVM to dedupe variables that are never used again.) This
/// syntax reduction will make generating the final output easier and also simplifies the work
/// needed to determine the actual types of a function's arguments and return type.
#[derive(Clone, Debug)]
pub enum Microstatement {
    Assignment {
        mutable: bool,
        name: String,
        value: Box<Microstatement>,
    },
    Arg {
        name: String,
        kind: ArgKind,
        typen: CType,
    },
    FnCall {
        function: Function,
        args: Vec<Microstatement>,
    },
    Closure {
        function: Function,
    },
    VarCall {
        name: String,
        typen: CType,
        args: Vec<Microstatement>,
    },
    Value {
        typen: CType,
        representation: String,
    },
    Array {
        typen: CType,
        vals: Vec<Microstatement>,
    },
    Return {
        value: Option<Box<Microstatement>>,
    }, // TODO: Conditionals
}

impl Microstatement {
    pub fn get_type(&self) -> CType {
        match self {
            Self::Value { typen, .. } => typen.clone(),
            Self::Array { typen, .. } => typen.clone(),
            Self::Arg { typen, .. } => typen.clone(),
            Self::Assignment { value, .. } => value.get_type(),
            Self::Return { value } => match value {
                Some(v) => v.get_type(),
                None => CType::Void,
            },
            Self::FnCall { function, args: _ } => function.rettype(),
            Self::Closure { function } => function.typen.clone(),
            Self::VarCall { typen, .. } => typen.clone(),
        }
    }
}

#[derive(Clone, Debug)]
enum BaseChunk<'a> {
    #[allow(clippy::upper_case_acronyms)]
    IIGE(
        Option<&'a Microstatement>,
        &'a parse::Functions,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    GFuncCall(
        Option<&'a Microstatement>,
        &'a String,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    #[allow(clippy::upper_case_acronyms)]
    IIFE(
        Option<&'a Microstatement>,
        &'a parse::Functions,
        Option<&'a parse::FnCall>,
    ),
    FuncCall(
        Option<&'a Microstatement>,
        &'a String,
        Option<&'a parse::FnCall>,
    ),
    TypeCall(
        Option<&'a Microstatement>,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    ConstantAccessor(&'a parse::Constants),
    ArrayAccessor(&'a parse::ArrayBase),
    Function(&'a parse::Functions),
    Group(&'a parse::FnCall),
    Array(&'a parse::ArrayBase),
    Variable(&'a String),
    Constant(&'a parse::Constants),
}

pub fn baseassignablelist_to_microstatements<'a>(
    bal: &[parse::BaseAssignable],
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    let mut i = 0;
    let mut prior_value: Option<Microstatement> = None;
    let l = bal.len();
    while i < l {
        // First find a chunk of the baseassignable list that we can work with and then perform the
        // operation afterwards. Fail with an error message if no valid path forward can be found.
        // I recognize that this could be done with `nom` at a higher level, but I don't think it
        // will buy me much for this little bit of parsing logic, and I am still not satisfied with
        // the lack of metadata tracking with my usage of `nom`.
        let (chunk, inc) = match (
            &prior_value,
            bal.get(i),
            bal.get(i + 1),
            bal.get(i + 2),
            bal.get(i + 3),
        ) {
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
            ) => (BaseChunk::IIGE(Some(p), f, g, Some(h)), 4),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
            ) => (BaseChunk::GFuncCall(Some(p), f, g, Some(h)), 4),
            (
                None,
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
                _,
            ) => (BaseChunk::IIGE(None, f, g, Some(h)), 3),
            (
                None,
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
                _,
            ) => (BaseChunk::GFuncCall(None, f, g, Some(h)), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                _,
            ) => (BaseChunk::IIGE(Some(p), f, g, None), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                _,
            ) => (BaseChunk::GFuncCall(Some(p), f, g, None), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::IIFE(Some(p), f, Some(g)), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::FuncCall(Some(p), f, Some(g)), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::GnCall(t)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::TypeCall(Some(p), t, Some(g)), 3),
            (
                None,
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::IIFE(None, f, Some(g)), 2),
            (
                None,
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::FuncCall(None, f, Some(g)), 2),
            (
                None,
                Some(parse::BaseAssignable::GnCall(t)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::TypeCall(None, t, Some(g)), 2),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                _,
                _,
            ) => (BaseChunk::IIFE(Some(p), f, None), 2),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                _,
                _,
            ) => (BaseChunk::FuncCall(Some(p), f, None), 2),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::GnCall(t)),
                _,
                _,
            ) => (BaseChunk::TypeCall(Some(p), t, None), 2),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Constants(c)),
                _,
                _,
            ) => (BaseChunk::ConstantAccessor(c), 2),
            (None, Some(parse::BaseAssignable::Functions(f)), _, _, _) => {
                (BaseChunk::Function(f), 1)
            }
            (None, Some(parse::BaseAssignable::FnCall(g)), _, _, _) => (BaseChunk::Group(g), 1),
            (None, Some(parse::BaseAssignable::Array(a)), _, _, _) => (BaseChunk::Array(a), 1),
            (None, Some(parse::BaseAssignable::Variable(v)), _, _, _) => {
                (BaseChunk::Variable(v), 1)
            }
            (None, Some(parse::BaseAssignable::Constants(c)), _, _, _) => {
                (BaseChunk::Constant(c), 1)
            }
            (Some(_), Some(parse::BaseAssignable::Array(a)), _, _, _) => {
                (BaseChunk::ArrayAccessor(a), 1)
            }
            _ => {
                return Err(format!(
                    "Invalid syntax: {}\n Cannot parse after {}, l - i = {}",
                    bal.iter()
                        .map(|ba| ba.to_string())
                        .collect::<Vec<String>>()
                        .join(""),
                    bal[i - 1].to_string(),
                    l - i,
                )
                .into());
            }
        };
        i += inc;
        // Now we just operate on our chunk and create a new prior_value to replace the old one, if
        // any exists. We'll start from the easier ones first and work our way up to the
        // complicated ones
        match chunk {
            // We know it has to be defined because we blew up earlier if not
            BaseChunk::Constant(c) => {
                match c {
                    parse::Constants::Bool(b) => {
                        prior_value = Some(Microstatement::Value {
                            typen: CType::bool(),
                            representation: b.clone(),
                        });
                    }
                    parse::Constants::Strn(s) => {
                        prior_value = Some(Microstatement::Value {
                            typen: CType::string(),
                            representation: if s.starts_with('"') {
                                s.clone()
                            } else {
                                // TODO: Is there a cheaper way to do this conversion?
                                s.replace('\"', "\\\"")
                                    .replace("\\'", "\\\\\"")
                                    .replace('\'', "\"")
                                    .replace("\\\\\"", "'")
                            },
                        });
                    }
                    parse::Constants::Num(n) => match n {
                        parse::Number::RealNum(r) => {
                            prior_value = Some(Microstatement::Value {
                                // TODO: Replace this with the `CType::Float` and have built-ins
                                // that accept them
                                typen: CType::f64(),
                                representation: r.clone(),
                            });
                        }
                        parse::Number::IntNum(i) => {
                            prior_value = Some(Microstatement::Value {
                                // TODO: Replace this with `CType::Int` and have built-ins that
                                // accept them
                                typen: CType::i64(),
                                representation: i.clone(),
                            });
                        }
                    },
                }
            }
            BaseChunk::Variable(v) => {
                let typen = match microstatements.iter().find(|m| match m {
                    Microstatement::Assignment { name, .. } => v == name,
                    Microstatement::Arg { name, .. } => v == name,
                    _ => false,
                }) {
                    // Reaching the `Some` path requires it to be of type
                    // Microstatment::Assignment, but Rust doesn't seem to know that, so force
                    // it.
                    Some(m) => match m {
                        Microstatement::Assignment { value, .. } => {
                            Ok::<CType, Box<dyn std::error::Error>>(value.get_type())
                        }
                        Microstatement::Arg { typen, .. } => Ok(typen.clone()),
                        _ => unreachable!(),
                    },
                    None => {
                        // It could be a function.
                        let mut function_types = scope.resolve_function_types(v);
                        if parent_fn.is_some() && parent_fn.unwrap().origin_scope_path != scope.path
                        {
                            let program = Program::get_program();
                            if let Ok(origin_scope) =
                                program.scope_by_file(&parent_fn.unwrap().origin_scope_path)
                            {
                                let other_function_types = origin_scope.resolve_function_types(v);
                                function_types = match (function_types, other_function_types) {
                                    (CType::Void, CType::Void) => CType::Void,
                                    (CType::Void, t) => t,
                                    (t, CType::Void) => t,
                                    (CType::AnyOf(t1), CType::AnyOf(t2)) => CType::AnyOf({
                                        let mut v = Vec::new();
                                        v.append(&mut t1.clone());
                                        v.append(&mut t2.clone());
                                        v
                                    }),
                                    (t, CType::AnyOf(t2)) => CType::AnyOf({
                                        let mut v = Vec::new();
                                        v.push(t.clone());
                                        v.append(&mut t2.clone());
                                        v
                                    }),
                                    (CType::AnyOf(t1), t) => CType::AnyOf({
                                        let mut v = Vec::new();
                                        v.append(&mut t1.clone());
                                        v.push(t.clone());
                                        v
                                    }),
                                    (t1, t2) => CType::AnyOf(vec![t1, t2]),
                                };
                            }
                            Program::return_program(program);
                        }
                        match function_types {
                            CType::Void => {
                                // It could be a constant
                                let maybe_c = scope.resolve_const(v);
                                match maybe_c {
                                    None => Err(format!("Couldn't find variable {}", v).into()),
                                    Some(c) => {
                                        // TODO: Confirm the specified typename matches the
                                        // actual typename of the value
                                        let mut temp_scope = scope.child();
                                        let res = withoperatorslist_to_microstatements(
                                            &c.assignables,
                                            parent_fn,
                                            temp_scope,
                                            microstatements,
                                        )?;
                                        temp_scope = res.0;
                                        microstatements = res.1;
                                        let cm = microstatements.pop().unwrap();
                                        let typen = match &cm {
                                            Microstatement::Value { typen, .. } | Microstatement::Array { typen, .. } => Ok(typen.clone()),
                                            Microstatement::FnCall { function: _, args: _ } => Err("TODO: Support global constant function calls"),
                                            _ => Err("This should be impossible, a constant has to be a value, array, or fncall"),
                                        }?;
                                        merge!(scope, temp_scope);
                                        microstatements.push(Microstatement::Assignment {
                                            mutable: false,
                                            name: v.clone(),
                                            value: Box::new(cm),
                                        });
                                        Ok(typen)
                                    }
                                }
                            }
                            f => Ok(f.clone()),
                        }
                    }
                }?;
                prior_value = Some(Microstatement::Value {
                    typen,
                    representation: v.to_string(),
                });
            }
            BaseChunk::Array(a) => {
                // We don't allow `[]` syntax, so blow up if the assignablelist is empty
                if a.assignablelist.is_empty() {
                    return Err("Cannot create an empty array with bracket syntax, use `Array{MyType}()` syntax instead".into());
                }
                let mut array_vals = Vec::new();
                for wol in &a.assignablelist {
                    let res = withoperatorslist_to_microstatements(
                        wol,
                        parent_fn,
                        scope,
                        microstatements,
                    )?;
                    scope = res.0;
                    microstatements = res.1;
                    array_vals.push(microstatements.pop().unwrap());
                }
                // TODO: Currently assuming all array values are the same type, should check that
                // better
                let inner_type = array_vals[0].get_type();
                let inner_type_str = inner_type.to_callable_string();
                let array_type_name = format!("Array_{}_", inner_type_str);
                let array_type = CType::Array(Box::new(inner_type));
                let type_str = format!("type {} = {}[];", array_type_name, inner_type_str);
                let parse_type = parse::types(&type_str);
                let res = CType::from_ast(scope, &parse_type.unwrap().1, false)?;
                scope = res.0;
                prior_value = Some(Microstatement::Array {
                    typen: array_type,
                    vals: array_vals,
                });
            }
            BaseChunk::Group(g) => {
                // TODO: Add support for anonymous tuples with this syntax, for now break if the
                // group's inner length is greater that one record
                if g.assignablelist.len() != 1 {
                    return Err("Anonymous tuple support not yet implemented".into());
                }
                let res = withoperatorslist_to_microstatements(
                    &g.assignablelist[0],
                    parent_fn,
                    scope,
                    microstatements,
                )?;
                scope = res.0;
                microstatements = res.1;
                prior_value = microstatements.pop();
            }
            BaseChunk::Function(f) => {
                // TODO: Move a lot of this into `Function`
                // First, some restrictions on closure function syntax (at least for now)
                if f.opttypegenerics.is_some() {
                    return Err(
                        "Conditional compilation not supported for closure functions".into(),
                    );
                }
                if f.optgenerics.is_some() {
                    return Err("Generics not supported for closure functions".into());
                }
                // If we got here, we know we're making a "normal" function
                let kind = FnKind::Normal;
                let mut inner_scope = scope.child();
                let original_len = microstatements.len();
                let statements = match &f.fullfunctionbody {
                    parse::FullFunctionBody::DecOnly(_) => Vec::new(), // TODO: Explode instead?
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
                };
                // TODO: A big blob of crap copied from Function that should really live there
                // *and* needs refactoring
                // TODO: Add code to properly convert the typeassignable vec into a CType tree and use it.
                // For now, just hardwire the parsing as before.
                let mut typen = match &f.opttype {
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
                            // This lets us partially resolve the function argument and return types
                            let mut temp_scope = inner_scope.child();
                            for g in gs {
                                temp_scope =
                                    CType::from_ctype(temp_scope, g.0.clone(), g.1.clone());
                            }
                            let ctype =
                                withtypeoperatorslist_to_ctype(typeassignable, &temp_scope)?;
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
                            merge!(inner_scope, temp_scope);
                            // The input type will be interpreted in many different ways:
                            // If it's a Group, unwrap it and continue. Ideally after that it's a Tuple
                            // type containing Field types, that's a "conventional" function
                            // definition, where the label becomes an argument name and the type is the
                            // type. If the tuple doesn't have Fields inside of it, we auto-generate
                            // argument names, eg `arg0`, `arg1`, etc. If it is not a Tuple type but is
                            // a Field type, we have a single argument function with a specified
                            // variable name. If it's any other type, we just label it `arg0`
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
                                    CType::Infer("unknown".to_string(), "unknonw".to_string()),
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
                for (name, kind, typen) in type_to_args(&typen) {
                    microstatements.push(Microstatement::Arg { name, kind, typen });
                }
                for statement in &statements {
                    let res = statement_to_microstatements(
                        statement,
                        parent_fn,
                        inner_scope,
                        microstatements,
                    )?;
                    inner_scope = res.0;
                    microstatements = res.1;
                }
                let ms = microstatements.split_off(original_len);
                if let Some(m) = ms.last() {
                    if let Microstatement::Arg { .. } = m {
                        // Don't do anything in this path, this is probably a derived function
                    } else {
                        let current_rettype = type_to_rettype(&typen);
                        let actual_rettype = match m {
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
                                match &f.optname {
                                    Some(name) => name,
                                    None => "closure",
                                },
                                current_rettype.to_strict_string(false),
                                actual_rettype.to_strict_string(false),
                            ));
                        } else {
                            // Do nothing, they're the same
                        }
                    }
                }
                match &typen {
                    CType::Function(i, o) => {
                        match &**o {
                            CType::Void => { /* Do nothing */ }
                            CType::Infer(t, _) if t == "unknown" => {
                                CType::fail(&format!(
                                    "The return type for {}({}) could not be inferred.",
                                    match &f.optname {
                                        Some(name) => name,
                                        None => "closure",
                                    },
                                    i.to_strict_string(false)
                                ));
                            }
                            CType::Infer(..) => { /* Do nothing */ }
                            otherwise => {
                                let name = otherwise.to_callable_string();
                                if scope.resolve_type(&name).is_none() {
                                    scope = CType::from_ctype(scope, name, otherwise.clone());
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                let function = Function {
                    name: match &f.optname {
                        Some(name) => name.clone(),
                        None => "closure".to_string(),
                    },
                    typen,
                    microstatements: ms,
                    kind,
                    origin_scope_path: scope.path.clone(),
                };
                prior_value = Some(Microstatement::Closure { function });
            }
            BaseChunk::ArrayAccessor(a) => {
                if let Some(prior) = &prior_value {
                    let mut temp_scope = scope.child();
                    let mut array_accessor_microstatements = vec![prior.clone()];
                    for wol in &a.assignablelist {
                        let res = withoperatorslist_to_microstatements(
                            wol,
                            parent_fn,
                            temp_scope,
                            microstatements,
                        )?;
                        temp_scope = res.0;
                        microstatements = res.1;
                        array_accessor_microstatements.push(microstatements.pop().unwrap());
                    }
                    let mut arg_types = Vec::new();
                    for m in &array_accessor_microstatements {
                        arg_types.push(m.get_type());
                    }
                    let res = temp_scope.resolve_function(&"get".to_string(), &arg_types);
                    match res {
                        Some((mut temp_scope, f)) => {
                            temp_scope
                                .functions
                                .insert("get".to_string(), vec![f.clone()]);
                            merge!(scope, temp_scope);
                            prior_value = Some(Microstatement::FnCall {
                                function: f,
                                args: array_accessor_microstatements,
                            })
                        }
                        None => {
                            return Err(format!(
                                "A function with the signature get({}) does not exist",
                                arg_types
                                    .iter()
                                    .map(|a| a.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            )
                            .into());
                        }
                    }
                } else {
                    // This is impossible, but I'm having a hard time convincing Rust of that
                    panic!("Impossible to reach the ArrayAccessor path without a prior value");
                }
            }
            BaseChunk::ConstantAccessor(c) => {
                if let Some(prior) = &prior_value {
                    let mut temp_scope = scope.child();
                    let constant_accessor_microstatements = vec![prior.clone()];
                    let mut arg_types = Vec::new();
                    for m in &constant_accessor_microstatements {
                        arg_types.push(m.get_type());
                        // In case the type constructor has not already been created
                        let t = m.get_type();
                        temp_scope = CType::from_ctype(temp_scope, t.to_callable_string(), t);
                    }
                    let res = temp_scope.resolve_function(&c.to_string(), &arg_types);
                    match res {
                        Some((mut temp_scope, f)) => {
                            temp_scope.functions.insert(c.to_string(), vec![f.clone()]);
                            merge!(scope, temp_scope);
                            prior_value = Some(Microstatement::FnCall {
                                function: f,
                                args: constant_accessor_microstatements,
                            })
                        }
                        None => {
                            return Err(format!(
                                "A function with the signature {}({}) does not exist",
                                c.to_string(),
                                arg_types
                                    .iter()
                                    .map(|a| a.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            )
                            .into());
                        }
                    }
                } else {
                    // This is impossible, but I'm having a hard time convincing Rust of that
                    panic!("Impossible to reach the ConstantAccessor path without a prior value");
                }
            }
            BaseChunk::TypeCall(prior, g, f) => {
                // Get all of the arguments for the function into an array. If there's a prior
                // value it becomes the first argument.
                let mut arg_microstatements = match prior {
                    Some(p) => vec![p.clone()],
                    None => Vec::new(),
                };
                match f {
                    None => {}
                    Some(fncall) => {
                        for arg in &fncall.assignablelist {
                            let res = withoperatorslist_to_microstatements(
                                arg,
                                parent_fn,
                                scope,
                                microstatements,
                            )?;
                            scope = res.0;
                            microstatements = res.1;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type());
                }
                // We create a type on-the-fly from the contents the GnCall block. It's given a
                // name based on the CType tree with all non-`a-zA-Z0-9_` chars replaced with `-`
                // TODO: Eliminate the duplication of CType generation logic by abstracting out the
                // automatic function creation into a reusable component
                let ctype = withtypeoperatorslist_to_ctype(&g.typecalllist, &scope)?;
                let name = ctype.to_callable_string();
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
                        a: Some("=".to_string()),
                        b: "".to_string(),
                        typeassignables: g.typecalllist.clone(),
                    },
                    optsemicolon: ";".to_string(),
                };
                let res = CType::from_ast(scope, &parse_type, false)?;
                scope = res.0;
                let temp_scope = scope.child();
                // Now we are sure the type and function exist, and we know the name for the
                // function. It would be best if we could just pass it to ourselves and run the
                // `FuncCall` logic below, but it's easier at the moment to duplicate :( TODO
                let res = temp_scope.resolve_function(&name, &arg_types);
                match res {
                    Some((mut temp_scope, f)) => {
                        temp_scope.functions.insert(name.clone(), vec![f.clone()]);
                        merge!(scope, temp_scope);
                        prior_value = Some(Microstatement::FnCall {
                            function: f.clone(),
                            args: arg_microstatements,
                        })
                    }
                    None => {
                        return Err(format!(
                            "A function with the signature {}({}) does not exist",
                            name,
                            arg_types
                                .iter()
                                .map(|a| a.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                        .into());
                    }
                }
            }
            BaseChunk::FuncCall(prior, f, g) => {
                // Get all of the arguments for the function into an array. If there's a prior
                // value it becomes the first argument.
                let mut arg_microstatements = match prior {
                    Some(p) => vec![p.clone()],
                    None => Vec::new(),
                };
                match g {
                    None => {}
                    Some(fncall) => {
                        for arg in &fncall.assignablelist {
                            let res = withoperatorslist_to_microstatements(
                                arg,
                                parent_fn,
                                scope,
                                microstatements,
                            )?;
                            scope = res.0;
                            microstatements = res.1;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type());
                }
                // Look for closure functions in the microstatement array first to see if that's
                // what should be called, scanning in reverse order to find the most recent
                // definition that matches, if multiple match
                let mut closure_fn = None;
                let mut var_fn = None;
                for ms in microstatements.iter().rev() {
                    match ms {
                        Microstatement::Closure { function } => {
                            if &function.name == f && function.args().len() == arg_types.len() {
                                let mut works = true;
                                for ((_, _, a), b) in function.args().iter().zip(&arg_types) {
                                    if !a.accepts(b) {
                                        works = false;
                                    }
                                }
                                if works {
                                    closure_fn = Some(function.clone());
                                    break;
                                }
                            }
                        }
                        Microstatement::Arg {
                            name,
                            kind: _,
                            typen,
                        } => {
                            if name == f {
                                if let CType::Function(i, o) = typen {
                                    let mut works = true;
                                    // TODO: Really need to just have the Function use the Function
                                    // CType instead of this stuff
                                    let farg_types = match &**i {
                                        CType::Void => Vec::new(),
                                        CType::Tuple(ts) => ts.clone(),
                                        other => vec![other.clone()],
                                    };
                                    for (a, b) in farg_types.iter().zip(&arg_types) {
                                        if !a.accepts(b) {
                                            works = false;
                                        }
                                    }
                                    if works {
                                        var_fn = Some((name.clone(), (**o).clone()));
                                        break;
                                    }
                                }
                            }
                        }
                        Microstatement::Assignment { .. } => {
                            // TODO
                        }
                        _ => { /* Do nothing */ }
                    }
                }
                if let Some(func) = closure_fn {
                    prior_value = Some(Microstatement::FnCall {
                        function: func,
                        args: arg_microstatements,
                    });
                } else if let Some((name, typen)) = var_fn {
                    prior_value = Some(Microstatement::VarCall {
                        name,
                        typen,
                        args: arg_microstatements,
                    });
                } else {
                    // Now confirm that there's actually a function with this name that takes these
                    // types
                    let temp_scope = scope.child();
                    let res = temp_scope.resolve_function(f, &arg_types);
                    match res {
                        Some((mut temp_scope, fun)) => {
                            // Success! Let's emit this
                            // TODO: Do a better job at type rewriting here
                            #[allow(clippy::needless_range_loop)]
                            for i in 0..fun.args().len() {
                                match &arg_microstatements[i] {
                                    Microstatement::Value {
                                        typen,
                                        representation,
                                    } => {
                                        let actual_typen = &fun.args()[i].2;
                                        if typen != actual_typen {
                                            if matches!(actual_typen, CType::Function(..)) {
                                                let temp_scope_2 = temp_scope.child();
                                                match temp_scope_2.resolve_function(
                                                    representation,
                                                    &type_to_args(actual_typen)
                                                        .into_iter()
                                                        .map(|(_, _, t)| t)
                                                        .collect::<Vec<CType>>(),
                                                ) {
                                                    None => {
                                                        arg_microstatements[i] =
                                                            Microstatement::Value {
                                                                typen: actual_typen.clone(),
                                                                representation: representation
                                                                    .clone(),
                                                            };
                                                    }
                                                    Some((s, func)) => {
                                                        if temp_scope
                                                            .functions
                                                            .contains_key(&func.name)
                                                        {
                                                            arg_microstatements[i] =
                                                                Microstatement::Value {
                                                                    typen: actual_typen.clone(),
                                                                    representation: func
                                                                        .name
                                                                        .clone(),
                                                                };
                                                        } else {
                                                            arg_microstatements[i] =
                                                                Microstatement::Value {
                                                                    typen: actual_typen.clone(),
                                                                    representation: representation
                                                                        .clone(),
                                                                };
                                                        }
                                                        merge!(temp_scope, s);
                                                    }
                                                }
                                            } else {
                                                arg_microstatements[i] = Microstatement::Value {
                                                    typen: actual_typen.clone(),
                                                    representation: representation.clone(),
                                                };
                                            }
                                        }
                                    }
                                    _ => { /* Do nothing */ }
                                }
                            }
                            merge!(scope, temp_scope);

                            prior_value = Some(Microstatement::FnCall {
                                function: fun.clone(), // TODO: Drop the clone
                                args: arg_microstatements,
                            });
                        }
                        None => {
                            return Err(format!(
                                "Could not find a function with a call signature of {}({})",
                                f,
                                arg_types
                                    .iter()
                                    .map(|a| a.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            )
                            .into());
                        }
                    }
                }
            }
            BaseChunk::IIFE(_prior, _f, _g) => {
                // TODO: This may just be some simple microstatement generation here compared to
                // actual closure creation
                return Err("TODO: Implement IIFE support".into());
            }
            BaseChunk::GFuncCall(prior, f, g, h) => {
                // TODO: Actually implement generic functions, for now this is just another way to
                // do a `TypeCall`
                // Get all of the arguments for the function into an array. If there's a prior
                // value it becomes the first argument.
                let mut arg_microstatements = match prior {
                    Some(p) => vec![p.clone()],
                    None => Vec::new(),
                };
                match h {
                    None => {}
                    Some(fncall) => {
                        for arg in &fncall.assignablelist {
                            let res = withoperatorslist_to_microstatements(
                                arg,
                                parent_fn,
                                scope,
                                microstatements,
                            )?;
                            scope = res.0;
                            microstatements = res.1;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type());
                }
                let generics = {
                    let mut generic_string = g.to_string();
                    match generic_string.strip_prefix('{') {
                        Some(s) => generic_string = s.to_string(),
                        None => { /* Do nothing */ }
                    }
                    match generic_string.strip_suffix('}') {
                        Some(s) => generic_string = s.to_string(),
                        None => { /* Do nothing */ }
                    }
                    // TODO: This is still sketchy, but a bit less so? It will fail with a sub-type
                    // being a generic with multiple args. Do this the right way, later.
                    generic_string
                        .replace(['{', '}'], "_")
                        .split(',')
                        .map(|s| s.to_string().trim().to_string())
                        .collect::<Vec<String>>()
                };
                let mut generic_types = Vec::new();
                for g in generics {
                    let t = match scope.resolve_type(&g) {
                        Some(t) => Ok(t.clone()), // TODO: Drop the cloning
                        None => {
                            // TODO: This should be inside of `resolve_type`, but that requires it
                            // to mutate scope and *that* is a whole refactoring can of worms
                            match g.parse::<i128>() {
                                Ok(i) => Ok(CType::Int(i)),
                                Err(_) => match g.parse::<f64>() {
                                    Ok(f) => Ok(CType::Float(f)),
                                    Err(_) => match g.as_str() {
                                        "true" => Ok(CType::Bool(true)),
                                        "false" => Ok(CType::Bool(false)),
                                        _ => {
                                            // TODO: Add string support
                                            Err(format!("Could not find type {}", g))
                                        }
                                    },
                                },
                            }
                        }
                    }?;
                    generic_types.push(t);
                }
                let maybe_type = scope.resolve_type(f);
                let temp_scope = scope.child();
                let maybe_generic_function =
                    temp_scope.resolve_generic_function(f, &generic_types, &arg_types);
                match (maybe_type, maybe_generic_function) {
                    (None, None) => {
                        return Err(format!(
                            "Generic type or function {}{} not found",
                            f,
                            g.to_string()
                        )
                        .into());
                    }
                    (_, Some((temp_scope, func))) => {
                        merge!(scope, temp_scope);
                        prior_value = Some(Microstatement::FnCall {
                            function: func.clone(), // TODO: Drop the clone
                            args: arg_microstatements,
                        });
                    }
                    (Some(_), None) => {
                        // Confirmed that this type exists, we now need to generate a realized
                        // generic type for this specified type and shove it into the non-exported
                        // scope, then we can be sure we can call it.
                        let name = format!(
                            "{}{}",
                            f,
                            g.to_string().replace([' ', ',', ':', '{', '}'], "_")
                        )
                        .replace('|', "_")
                        .replace("()", "void"); // Really bad
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
                                a: Some("=".to_string()),
                                b: "".to_string(),
                                typeassignables: vec![parse::WithTypeOperators::TypeBaseList(
                                    vec![
                                        parse::TypeBase::Variable(f.to_string()),
                                        parse::TypeBase::GnCall(g.clone()),
                                    ],
                                )],
                            },
                            optsemicolon: ";".to_string(),
                        };
                        let res = CType::from_ast(scope, &parse_type, false)?;
                        scope = res.0;
                        let t = res.1;
                        let real_name = t.to_callable_string();
                        // Now we are sure the type and function exist, and we know the name for the
                        // function. It would be best if we could just pass it to ourselves and run the
                        // `FuncCall` logic below, but it's easier at the moment to duplicate :( TODO
                        let temp_scope = scope.child();
                        let res = temp_scope.resolve_function(&real_name, &arg_types);
                        match res {
                            Some((mut temp_scope, func)) => {
                                temp_scope.functions.insert(f.clone(), vec![func.clone()]);
                                merge!(scope, temp_scope);
                                let res = CType::from_ast(scope, &parse_type, false)?; // TODO: Remove this
                                                                                       // duplicate
                                scope = res.0;
                                prior_value = Some(Microstatement::FnCall {
                                    function: func.clone(), // TODO: Drop the clone?
                                    args: arg_microstatements,
                                })
                            }
                            None => {
                                return Err(format!(
                                    "A function with the signature {}({}) does not exist",
                                    real_name,
                                    arg_types
                                        .iter()
                                        .map(|a| a.to_string())
                                        .collect::<Vec<String>>()
                                        .join(", ")
                                )
                                .into());
                            }
                        }
                    }
                }
            }
            BaseChunk::IIGE(_prior, _f, _g, _h) => {
                // TODO: This may similarly be just some simple microstatement generation here
                return Err("TODO: Implement IIGE support".into());
            }
        }
    }
    // Push the generated statement that *probably* exists into the microstatements array
    if let Some(prior) = prior_value {
        microstatements.push(prior);
    }
    Ok((scope, microstatements))
}

pub fn withoperatorslist_to_microstatements<'a>(
    withoperatorslist: &Vec<parse::WithOperators>,
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    // To properly linearize the operations here, we need to scan through all of the operators,
    // determine which is the highest precedence, whether it is infix or prefix (or maybe postfix
    // in the future?) and then process them and whichever of the baseassignables surrounding them
    // are associated, then put those results in the same "slot" as before and check again. Because
    // users can define these operators, that makes it theoretically possible for the same operator
    // to be used in both an infix or prefix manner, or with different precedence levels, depending
    // on the types of the data involved, which makes things *really* complicated here.
    let mut queue = withoperatorslist.clone();
    while !queue.is_empty() {
        let mut largest_operator_level: i8 = -1;
        let mut largest_operator_index: i64 = -1;
        let mut op = None;
        for (i, assignable_or_operator) in queue.iter().enumerate() {
            // This can sometimes be ambiguous on the symbol, `-` is both an infix subtract and a
            // prefix negate operation, and they have different precedence levels. If and only if
            // it might have the highest precedence do we check if it could reasonably resolve in
            // that way. (For a prefix, there must either be nothing before it or what's before it
            // needs to be an operator and what's after it must be an assignable, for a postfix
            // there must be nothing after it or what's after it is an operator and what's before
            // it is an assignable, and for an infix there must be an assignable before and after
            // it.) If it doesn't match those criteria we skip over that possibility and move on to
            // others.
            if let parse::WithOperators::Operators(o) = assignable_or_operator {
                let operatorname = o.trim();
                let prefix_op = scope.resolve_operator(&format!("prefix{}", operatorname));
                let infix_op = scope.resolve_operator(&format!("infix{}", operatorname));
                let postfix_op = scope.resolve_operator(&format!("postfix{}", operatorname));
                let mut level = -1;
                let mut operator = None;
                for local_op in [&prefix_op, &infix_op, &postfix_op] {
                    let local_level = match local_op {
                        Some(o) => match o {
                            OperatorMapping::Prefix { level, .. } => {
                                match (queue.get(i.wrapping_sub(1)), queue.get(i.wrapping_add(1))) {
                                    (None, Some(parse::WithOperators::BaseAssignableList(_)))
                                    | (
                                        Some(parse::WithOperators::Operators(_)),
                                        Some(parse::WithOperators::BaseAssignableList(_)),
                                    ) => *level,
                                    _ => -1,
                                }
                            }
                            OperatorMapping::Infix { level, .. } => {
                                match (queue.get(i.wrapping_sub(1)), queue.get(i.wrapping_add(1))) {
                                    (
                                        Some(parse::WithOperators::BaseAssignableList(_)),
                                        Some(parse::WithOperators::BaseAssignableList(_)),
                                    ) => *level,
                                    _ => -1,
                                }
                            }
                            OperatorMapping::Postfix { level, .. } => {
                                match (queue.get(i.wrapping_sub(1)), queue.get(i.wrapping_add(1))) {
                                    (Some(parse::WithOperators::BaseAssignableList(_)), None)
                                    | (
                                        Some(parse::WithOperators::BaseAssignableList(_)),
                                        Some(parse::WithOperators::Operators(_)),
                                    ) => *level,
                                    _ => -1,
                                }
                            }
                        },
                        _ => -1,
                    };
                    if local_level > level {
                        level = local_level;
                        operator = *local_op;
                    }
                }
                if level > largest_operator_level {
                    largest_operator_level = level;
                    largest_operator_index = i as i64;
                    op = operator;
                }
            }
        }
        if largest_operator_index > -1 {
            let operator = op.unwrap(); // Should be guaranteed to exist
            let functionname = match operator {
                OperatorMapping::Prefix { functionname, .. } => functionname.clone(),
                OperatorMapping::Infix { functionname, .. } => functionname.clone(),
                OperatorMapping::Postfix { functionname, .. } => functionname.clone(),
            };
            let is_infix = match operator {
                OperatorMapping::Prefix { .. } => false,
                OperatorMapping::Postfix { .. } => false,
                OperatorMapping::Infix { .. } => true,
            };
            let is_prefix = match operator {
                OperatorMapping::Prefix { .. } => true,
                OperatorMapping::Postfix { .. } => false,
                OperatorMapping::Infix { .. } => false,
            };
            if is_infix {
                // Confirm that we have records before and after the operator and that they are
                // baseassignables.
                let first_arg = match match queue.get(largest_operator_index as usize - 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {} is an infix operator but missing a left-hand side value",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        }
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => {
                        Ok(baseassignablelist)
                    }
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is an infix operator but preceded by another operator {}",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                        o
                    )),
                }?;
                let second_arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!("Operator {} is an infix operator but missing a right-hand side value",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        })),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => Ok(baseassignablelist),
                    parse::WithOperators::Operators(o) => Err(format!("Operator{} is an infix operator but followed by a lower precedence operator {}",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        }, o)),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `a + b` and turn it into `add(a, b)`
                let rewrite = parse::WithOperators::BaseAssignableList(vec![
                    parse::BaseAssignable::Variable(functionname),
                    parse::BaseAssignable::FnCall(parse::FnCall {
                        openparen: "(".to_string(),
                        a: "".to_string(),
                        assignablelist: vec![
                            vec![parse::WithOperators::BaseAssignableList(first_arg.to_vec())],
                            vec![parse::WithOperators::BaseAssignableList(
                                second_arg.to_vec(),
                            )],
                        ],
                        b: "".to_string(),
                        closeparen: ")".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithOperators> = queue
                    .splice(
                        (largest_operator_index as usize - 1)
                            ..(largest_operator_index as usize + 2),
                        vec![rewrite],
                    )
                    .collect();
            } else if is_prefix {
                // Confirm that we have a record after the operator and that it's a baseassignables
                let arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {} is a prefix operator but missing a right-hand side value",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => {
                        Ok(baseassignablelist)
                    }
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is an prefix operator but followed by another operator {}",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                        o
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `#array` and turn it into `len(array)`
                let rewrite = parse::WithOperators::BaseAssignableList(vec![
                    parse::BaseAssignable::Variable(functionname),
                    parse::BaseAssignable::FnCall(parse::FnCall {
                        openparen: "(".to_string(),
                        a: "".to_string(),
                        assignablelist: vec![vec![parse::WithOperators::BaseAssignableList(
                            arg.to_vec(),
                        )]],
                        b: "".to_string(),
                        closeparen: ")".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithOperators> = queue
                    .splice(
                        (largest_operator_index as usize)..(largest_operator_index as usize + 2),
                        vec![rewrite],
                    )
                    .collect();
            } else {
                let arg = match match queue.get(largest_operator_index as usize - 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {} is a postfix operator but missing a left-hand side value",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(bal) => Ok(bal),
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is a postfix operator but preceded by another operator {}",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                        o
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `var?` and turn it into `Maybe(var)`
                let rewrite = parse::WithOperators::BaseAssignableList(vec![
                    parse::BaseAssignable::Variable(functionname),
                    parse::BaseAssignable::FnCall(parse::FnCall {
                        openparen: "(".to_string(),
                        a: "".to_string(),
                        assignablelist: vec![vec![parse::WithOperators::BaseAssignableList(
                            arg.to_vec(),
                        )]],
                        b: "".to_string(),
                        closeparen: ")".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithOperators> = queue
                    .splice(
                        (largest_operator_index as usize - 1)
                            ..(largest_operator_index as usize + 1),
                        vec![rewrite],
                    )
                    .collect();
            }
        } else {
            // We have no more operators, there should only be one reworked baseassignablelist now
            if queue.len() != 1 {
                // No idea how such a wonky thing could occur. TODO: Improve error message
                return Err(format!("Invalid syntax: {:?}", withoperatorslist).into());
            }
            let baseassignablelist = match match queue.pop() {
                Some(v) => Ok(v),
                None => Err(format!(
                    "Somehow we collapsed the statement into nothing? {:?}",
                    withoperatorslist
                )),
            }? {
                parse::WithOperators::BaseAssignableList(b) => Ok(b),
                _ => Err(format!(
                    "Somehow we collapse the statement into a solitary operator? {:?}",
                    withoperatorslist
                )),
            }?;
            let res = baseassignablelist_to_microstatements(
                &baseassignablelist,
                parent_fn,
                scope,
                microstatements,
            )?;
            scope = res.0;
            microstatements = res.1;
        }
    }
    Ok((scope, microstatements))
}

pub fn assignablestatement_to_microstatements<'a>(
    assignable: &parse::AssignableStatement,
    parent_fn: Option<&Function>,
    scope: Scope<'a>,
    microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    let res = withoperatorslist_to_microstatements(
        &assignable.assignables,
        parent_fn,
        scope,
        microstatements,
    )?;
    Ok(res)
}

pub fn returns_to_microstatements<'a>(
    returns: &parse::Returns,
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    if let Some(retval) = &returns.retval {
        // We get all of the microstatements involved in the return statement, then we pop
        // off the last one, if any exists, to get the final return value. Then we shove
        // the other microstatements into the array and the new Return microstatement with
        // that last value attached to it.
        let res = withoperatorslist_to_microstatements(
            &retval.assignables,
            parent_fn,
            scope,
            microstatements,
        )?;
        scope = res.0;
        microstatements = res.1;
        let value = microstatements.pop().map(Box::new);
        microstatements.push(Microstatement::Return { value });
    } else {
        microstatements.push(Microstatement::Return { value: None });
    }
    Ok((scope, microstatements))
}

pub fn declarations_to_microstatements<'a>(
    declarations: &parse::Declarations,
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    let (name, assignables, mutable) = match &declarations {
        parse::Declarations::Const(c) => (c.variable.clone(), &c.assignables, false),
        parse::Declarations::Let(l) => (l.variable.clone(), &l.assignables, true),
    };
    // Get all of the assignable microstatements generated
    let res = withoperatorslist_to_microstatements(assignables, parent_fn, scope, microstatements)?;
    scope = res.0;
    microstatements = res.1;
    let value = match microstatements.pop() {
        None => Err("An assignment without a value should be impossible."),
        Some(v) => Ok(Box::new(v)),
    }?;
    microstatements.push(Microstatement::Assignment {
        name,
        value,
        mutable,
    });
    Ok((scope, microstatements))
}

pub fn statement_to_microstatements<'a>(
    statement: &parse::Statement,
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    match statement {
        // This is just whitespace, so we do nothing here
        parse::Statement::A(_) => Ok((scope, microstatements)),
        parse::Statement::Declarations(declarations) => Ok(declarations_to_microstatements(
            declarations,
            parent_fn,
            scope,
            microstatements,
        )?),
        parse::Statement::ArrayAssignment(arrayassignment) => {
            let mut args = Vec::new();
            let res = baseassignablelist_to_microstatements(
                &[arrayassignment.name.clone()],
                parent_fn,
                scope,
                microstatements,
            )?;
            scope = res.0;
            let mut ms = res.1;
            args.push(ms.pop().unwrap());
            for arg in &arrayassignment.array.assignablelist {
                let res = withoperatorslist_to_microstatements(arg, parent_fn, scope, ms)?;
                scope = res.0;
                ms = res.1;
                args.push(ms.pop().unwrap());
            }
            let res = withoperatorslist_to_microstatements(
                &arrayassignment.assignables,
                parent_fn,
                scope,
                ms,
            )?;
            scope = res.0;
            ms = res.1;
            args.push(ms.pop().unwrap());
            let arg_types = args.iter().map(|a| a.get_type()).collect::<Vec<CType>>();
            let store_fn = {
                // TODO: Do we really need this temp_scope?
                let temp_scope = scope.child();
                match temp_scope.resolve_function(&"store".to_string(), &arg_types) {
                    Some((_, f)) => Ok(f),
                    None => Err(format!(
                        "Could not find store function with arguments {}",
                        arg_types
                            .iter()
                            .map(|a| a.to_strict_string(false))
                            .collect::<Vec<String>>()
                            .join(", "),
                    )),
                }?
            };
            ms.push(Microstatement::FnCall {
                function: store_fn,
                args,
            });
            Ok((scope, ms))
        }
        parse::Statement::Assignables(assignable) => Ok(assignablestatement_to_microstatements(
            assignable,
            parent_fn,
            scope,
            microstatements,
        )?),
        parse::Statement::Returns(returns) => Ok(returns_to_microstatements(
            returns,
            parent_fn,
            scope,
            microstatements,
        )?),
        parse::Statement::Conditional(_condtitional) => Err("Implement me".into()),
    }
}
