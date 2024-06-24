use super::ctype::{withtypeoperatorslist_to_ctype, CType};
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
        typen: CType,
    },
    FnCall {
        function: String, // TODO: It would be nice to make this a vector of pointers to function objects so we can narrow down the exact implementation sooner
        args: Vec<Microstatement>,
    },
    Value {
        typen: CType,
        representation: String, // TODO: Can we do better here?
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
    pub fn get_type(
        &self,
        scope: &Scope,
        program: &Program,
    ) -> Result<CType, Box<dyn std::error::Error>> {
        match self {
            Self::Value { typen, .. } => Ok(typen.clone()),
            Self::Array { typen, .. } => Ok(typen.clone()),
            Self::Arg { typen, .. } => Ok(typen.clone()),
            Self::Assignment { value, .. } => value.get_type(scope, program),
            Self::Return { value } => match value {
                Some(v) => v.get_type(scope, program),
                None => Ok(CType::Void),
            },
            Self::FnCall { function, args } => {
                let mut arg_types = Vec::new();
                for arg in args {
                    let arg_type = arg.get_type(scope, program)?;
                    arg_types.push(arg_type);
                }
                match program.resolve_function(scope, function, &arg_types) {
                    Some(function_object) => Ok(function_object.rettype.clone()),
                    None => Err(format!(
                        "Could not find function {}({})",
                        function,
                        arg_types
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                    .into()),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum BaseChunk<'a> {
    IIGE(
        &'a Option<Microstatement>,
        &'a parse::Functions,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    GFuncCall(
        &'a Option<Microstatement>,
        &'a String,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    IIFE(
        &'a Option<Microstatement>,
        &'a parse::Functions,
        Option<&'a parse::FnCall>,
    ),
    FuncCall(
        &'a Option<Microstatement>,
        &'a String,
        Option<&'a parse::FnCall>,
    ),
    TypeCall(
        &'a Option<Microstatement>,
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

pub fn baseassignablelist_to_microstatements(
    bal: &Vec<parse::BaseAssignable>,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
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
            prior_value.clone(),
            bal.get(i),
            bal.get(i + 1),
            bal.get(i + 2),
            bal.get(i + 3),
        ) {
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
            ) => (BaseChunk::IIGE(&prior_value, f, g, Some(h)), 4),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
            ) => (BaseChunk::GFuncCall(&prior_value, f, g, Some(h)), 4),
            (
                None,
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
                _,
            ) => (BaseChunk::IIGE(&prior_value, f, g, Some(h)), 3),
            (
                None,
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
                _,
            ) => (BaseChunk::GFuncCall(&prior_value, f, g, Some(h)), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                _,
            ) => (BaseChunk::IIGE(&prior_value, f, g, None), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                _,
            ) => (BaseChunk::GFuncCall(&prior_value, f, g, None), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::IIFE(&prior_value, f, Some(g)), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::FuncCall(&prior_value, f, Some(g)), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::GnCall(t)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::TypeCall(&prior_value, t, Some(g)), 3),
            (
                None,
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::IIFE(&prior_value, f, Some(g)), 2),
            (
                None,
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::FuncCall(&prior_value, f, Some(g)), 2),
            (
                None,
                Some(parse::BaseAssignable::GnCall(t)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::TypeCall(&prior_value, t, Some(g)), 2),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                _,
                _,
            ) => (BaseChunk::IIFE(&prior_value, f, None), 2),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                _,
                _,
            ) => (BaseChunk::FuncCall(&prior_value, f, None), 2),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::GnCall(t)),
                _,
                _,
            ) => (BaseChunk::TypeCall(&prior_value, t, None), 2),
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
        i = i + inc;
        // Now we just operate on our chunk and create a new prior_value to replace the old one, if
        // any exists. We'll start from the easier ones first and work our way up to the
        // complicated ones
        match chunk {
            // We know it has to be defined because we blew up earlier if not
            BaseChunk::Constant(c) => {
                match c {
                    parse::Constants::Bool(b) => {
                        prior_value = Some(Microstatement::Value {
                            typen: CType::Bound("bool".to_string(), "bool".to_string()),
                            representation: b.clone(),
                        });
                    }
                    parse::Constants::Strn(s) => {
                        prior_value = Some(Microstatement::Value {
                            typen: CType::Bound("string".to_string(), "String".to_string()),
                            representation: if s.starts_with('"') {
                                s.clone()
                            } else {
                                // TODO: Is there a cheaper way to do this conversion?
                                s.replace("\"", "\\\"")
                                    .replace("\\'", "\\\\\"")
                                    .replace("'", "\"")
                                    .replace("\\\\\"", "'")
                            },
                        });
                    }
                    parse::Constants::Num(n) => match n {
                        parse::Number::RealNum(r) => {
                            prior_value = Some(Microstatement::Value {
                                // TODO: Replace this with the `CType::Float` and have built-ins
                                // that accept them
                                typen: CType::Bound("f64".to_string(), "f64".to_string()),
                                representation: r.clone(),
                            });
                        }
                        parse::Number::IntNum(i) => {
                            prior_value = Some(Microstatement::Value {
                                // TODO: Replace this with `CType::Int` and have built-ins that
                                // accept them
                                typen: CType::Bound("i64".to_string(), "i64".to_string()),
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
                        Microstatement::Assignment { value, .. } => value.get_type(scope, program),
                        Microstatement::Arg { typen, .. } => Ok(typen.clone()),
                        _ => unreachable!(),
                    },
                    None => {
                        // It could be a function. TODO: Use `program.resolve_function` for better
                        // type safety. This requires getting the types of all of the involved
                        // arguments
                        match scope.functions.get(v) {
                            // TODO: the specific function chosen may be wrong, need to use
                            // `program.resolve_function` here
                            Some(f) => Ok(CType::Function(
                                Box::new(if f[0].args.len() == 0 {
                                    CType::Void
                                } else {
                                    CType::Tuple(
                                        f[0].args
                                            .iter()
                                            .map(|(l, t)| {
                                                CType::Field(l.clone(), Box::new(t.clone()))
                                            })
                                            .collect::<Vec<CType>>(),
                                    )
                                }),
                                Box::new(f[0].rettype.clone()),
                            )), // TODO: Convert the Function class to use a
                            // function type directly
                            None => {
                                // Check the root scope, too
                                match program.scopes_by_file.get("@root") {
                                    Some((_, _, s)) => match s.functions.get(v) {
                                        Some(f) => Ok(CType::Function(
                                            Box::new(if f[0].args.len() == 0 {
                                                CType::Void
                                            } else {
                                                CType::Tuple(
                                                    f[0].args
                                                        .iter()
                                                        .map(|(l, t)| {
                                                            CType::Field(
                                                                l.clone(),
                                                                Box::new(t.clone()),
                                                            )
                                                        })
                                                        .collect::<Vec<CType>>(),
                                                )
                                            }),
                                            Box::new(f[0].rettype.clone()),
                                        )), // TODO: Convert the Function class to use a
                                        // function type directly
                                        None => {
                                            let maybe_c = match program.resolve_const(scope, v) {
                                                Some(c) => Some(c.clone()),
                                                None => None,
                                            };
                                            match maybe_c {
                                                None => {
                                                    Err(format!("Couldn't find variable {}", v)
                                                        .into())
                                                }
                                                Some(c) => {
                                                    // TODO: Confirm the specified typename matches the
                                                    // actual typename of the value
                                                    let mut temp_scope = scope.temp_child();
                                                    microstatements =
                                                        withoperatorslist_to_microstatements(
                                                            &c.assignables,
                                                            &mut temp_scope,
                                                            program,
                                                            microstatements,
                                                        )?;
                                                    let cm = microstatements.pop().unwrap();
                                                    let typen = match &cm {
                                                        Microstatement::Value { typen, .. } | Microstatement::Array { typen, .. } => Ok(typen.clone()),
                                                        Microstatement::FnCall { function: _, args: _ } => Err("TODO: Support global constant function calls"),
                                                        _ => Err("This should be impossible, a constant has to be a value, array, or fncall"),
                                                    }?;
                                                    microstatements.push(
                                                        Microstatement::Assignment {
                                                            mutable: false,
                                                            name: v.clone(),
                                                            value: Box::new(cm),
                                                        },
                                                    );
                                                    Ok(typen)
                                                }
                                            }
                                        }
                                    },
                                    None => {
                                        let maybe_c = match program.resolve_const(scope, v) {
                                            Some(c) => Some(c.clone()),
                                            None => None,
                                        };
                                        match maybe_c {
                                            None => {
                                                Err(format!("Couldn't find variable {}", v).into())
                                            }
                                            Some(c) => {
                                                // TODO: Confirm the specified typename matches the
                                                // actual typename of the value
                                                let mut temp_scope = scope.temp_child();
                                                microstatements =
                                                    withoperatorslist_to_microstatements(
                                                        &c.assignables,
                                                        &mut temp_scope,
                                                        program,
                                                        microstatements,
                                                    )?;
                                                let cm = microstatements.pop().unwrap();
                                                let typen = match &cm {
                                                    Microstatement::Value { typen, .. } | Microstatement::Array { typen, .. } => Ok(typen.clone()),
                                                    Microstatement::FnCall { function: _, args: _ } => Err("TODO: Support global constant function calls"),
                                                    _ => Err("This should be impossible, a constant has to be a value, array, or fncall"),
                                                }?;
                                                microstatements.push(Microstatement::Assignment {
                                                    mutable: false,
                                                    name: v.clone(),
                                                    value: Box::new(cm),
                                                });
                                                Ok(typen)
                                            }
                                        }
                                    }
                                }
                            }
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
                if a.assignablelist.len() == 0 {
                    return Err("Cannot create an empty array with bracket syntax, use `Array{MyType}()` syntax instead".into());
                }
                let mut array_vals = Vec::new();
                for wol in &a.assignablelist {
                    microstatements =
                        withoperatorslist_to_microstatements(wol, scope, program, microstatements)?;
                    array_vals.push(microstatements.pop().unwrap());
                }
                // TODO: Currently assuming all array values are the same type, should check that
                // better
                let inner_type = array_vals[0].get_type(scope, program)?;
                let inner_type_str = inner_type.to_functional_string();
                let array_type_name = format!(
                    "Array_{}_",
                    inner_type_str
                        .replace(" ", "_")
                        .replace(",", "_")
                        .replace("{", "_")
                        .replace("}", "_")
                ); // Really bad
                let array_type = CType::Array(Box::new(inner_type));
                let type_str = format!("type {} = {}[];", array_type_name, inner_type_str);
                let parse_type = parse::types(&type_str);
                CType::from_ast(scope, program, &parse_type.unwrap().1, false)?;
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
                microstatements = withoperatorslist_to_microstatements(
                    &g.assignablelist[0],
                    scope,
                    program,
                    microstatements,
                )?;
                prior_value = microstatements.pop();
            }
            BaseChunk::Function(_f) => {
                return Err("TODO: Implement closure functions in Microstatement syntax".into());
            }
            BaseChunk::ArrayAccessor(a) => {
                if let Some(prior) = &prior_value {
                    let mut array_accessor_microstatements = vec![prior.clone()];
                    for wol in &a.assignablelist {
                        microstatements = withoperatorslist_to_microstatements(
                            wol,
                            scope,
                            program,
                            microstatements,
                        )?;
                        array_accessor_microstatements.push(microstatements.pop().unwrap());
                    }
                    // TODO: Check that this function actually exists with the arguments being provided
                    prior_value = Some(Microstatement::FnCall {
                        function: "get".to_string(),
                        args: array_accessor_microstatements,
                    });
                } else {
                    // This is impossible, but I'm having a hard time convincing Rust of that
                    panic!("Impossible to reach the ArrayAccessor path without a prior value");
                }
            }
            BaseChunk::ConstantAccessor(c) => {
                if let Some(prior) = &prior_value {
                    let mut constant_accessor_microstatements = vec![prior.clone()];
                    microstatements = baseassignablelist_to_microstatements(
                        &vec![parse::BaseAssignable::Constants(c.clone())],
                        scope,
                        program,
                        microstatements,
                    )?;
                    constant_accessor_microstatements.push(microstatements.pop().unwrap());
                    // TODO: Check that this function actually exists with the argument being provided
                    prior_value = Some(Microstatement::FnCall {
                        function: "get".to_string(),
                        args: constant_accessor_microstatements,
                    });
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
                            microstatements = withoperatorslist_to_microstatements(
                                &arg,
                                scope,
                                program,
                                microstatements,
                            )?;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type(scope, program)?);
                }
                // We create a type on-the-fly from the contents the GnCall block. It's given a
                // name based on the CType tree with all non-`a-zA-Z0-9_` chars replaced with `-`
                // TODO: Eliminate the duplication of CType generation logic by abstracting out the
                // automatic function creation into a reusable component
                let ctype = withtypeoperatorslist_to_ctype(&g.typecalllist, scope, program)?;
                let name = ctype
                    .to_functional_string()
                    .replace(" ", "_")
                    .replace(",", "_")
                    .replace("{", "_")
                    .replace("}", "_");
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
                    typedef: parse::TypeDef::TypeCreate(parse::TypeCreate {
                        a: "=".to_string(),
                        b: "".to_string(),
                        typeassignables: g.typecalllist.clone(),
                    }),
                    optsemicolon: ";".to_string(),
                };
                CType::from_ast(scope, program, &parse_type, false)?;
                // Now we are sure the type and function exist, and we know the name for the
                // function. It would be best if we could just pass it to ourselves and run the
                // `FuncCall` logic below, but it's easier at the moment to duplicate :( TODO
                prior_value = Some(Microstatement::FnCall {
                    function: name,
                    args: arg_microstatements,
                });
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
                            microstatements = withoperatorslist_to_microstatements(
                                &arg,
                                scope,
                                program,
                                microstatements,
                            )?;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type(scope, program)?);
                }
                // Now confirm that there's actually a function with this name that takes these
                // types
                let func = program.resolve_function(scope, f, &arg_types);
                match func {
                    Some(_) => {
                        // Success! Let's emit this
                        prior_value = Some(Microstatement::FnCall {
                            function: f.to_string(),
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
                            microstatements = withoperatorslist_to_microstatements(
                                &arg,
                                scope,
                                program,
                                microstatements,
                            )?;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type(scope, program)?);
                }
                // TODO: Be less sketchy here, this punts validation to later where we may produce
                // a nonsensical error message
                let generics = g
                    .to_string()
                    .replace("{", "")
                    .replace("}", "")
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                let mut temp_scope = scope.child(program);
                let maybe_type = match program.resolve_type(scope, f) {
                    None => None,
                    Some(t) => Some(t.clone()), // TODO: Kill the clone
                };
                let maybe_generic_function =
                    program.resolve_generic_function(&mut temp_scope, f, &generics, &arg_types);
                match (maybe_type, maybe_generic_function) {
                    (None, None) => {
                        return Err(format!(
                            "Generic type or function {}{} not found",
                            f,
                            g.to_string()
                        )
                        .into());
                    }
                    (Some(_), Some(_)) => {
                        // If this hits, it matched on the arguments
                    }
                    (None, Some(func)) => {
                        // Grab that realized generic function and shove it into the current scope
                        if scope.functions.contains_key(&func.name) {
                            let func_vec = scope.functions.get_mut(&func.name).unwrap();
                            func_vec.push(func.clone());
                        } else {
                            scope
                                .functions
                                .insert(func.name.clone(), vec![func.clone()]);
                        }
                        prior_value = Some(Microstatement::FnCall {
                            function: func.name.clone(),
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
                            g.to_string()
                                .replace(" ", "_")
                                .replace(",", "_")
                                .replace(":", "_")
                                .replace("{", "_")
                                .replace("}", "_")
                        )
                        .replace("|", "_")
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
                            typedef: parse::TypeDef::TypeCreate(parse::TypeCreate {
                                a: "=".to_string(),
                                b: "".to_string(),
                                typeassignables: vec![parse::WithTypeOperators::TypeBaseList(
                                    vec![
                                        parse::TypeBase::Variable(f.to_string()),
                                        parse::TypeBase::GnCall(g.clone()),
                                    ],
                                )],
                            }),
                            optsemicolon: ";".to_string(),
                        };
                        let t = CType::from_ast(scope, program, &parse_type, false)?;
                        let real_name = t
                            .to_functional_string()
                            .replace(" ", "_")
                            .replace(",", "_")
                            .replace("{", "_")
                            .replace("}", "_");
                        // Now we are sure the type and function exist, and we know the name for the
                        // function. It would be best if we could just pass it to ourselves and run the
                        // `FuncCall` logic below, but it's easier at the moment to duplicate :( TODO
                        prior_value = Some(Microstatement::FnCall {
                            function: real_name,
                            args: arg_microstatements,
                        });
                    }
                }
                scope.merge_child_functions(&mut temp_scope);
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
    Ok(microstatements)
}

pub fn withoperatorslist_to_microstatements(
    withoperatorslist: &Vec<parse::WithOperators>,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    // To properly linearize the operations here, we need to scan through all of the operators,
    // determine which is the highest precedence, whether it is infix or prefix (or maybe postfix
    // in the future?) and then process them and whichever of the baseassignables surrounding them
    // are associated, then put those results in the same "slot" as before and check again. Because
    // users can define these operators, that makes it theoretically possible for the same operator
    // to be used in both an infix or prefix manner, or with different precedence levels, depending
    // on the types of the data involved, which makes things *really* complicated here. TODO:
    // Actually implement that complexity, for now, just pretend operators have only one binding.
    let mut queue = withoperatorslist.clone();
    while queue.len() > 0 {
        let mut largest_operator_level: i8 = -1;
        let mut largest_operator_index: i64 = -1;
        for (i, assignable_or_operator) in queue.iter().enumerate() {
            match assignable_or_operator {
                parse::WithOperators::Operators(o) => {
                    let operatorname = o.trim();
                    let operator = match program.resolve_operator(scope, &operatorname.to_string())
                    {
                        Some(o) => Ok(o),
                        None => Err(format!("Operator {} not found", operatorname)),
                    }?;
                    let level = match &operator {
                        OperatorMapping::Prefix { level, .. } => level,
                        OperatorMapping::Infix { level, .. } => level,
                        OperatorMapping::Postfix { level, .. } => level,
                    };
                    if level > &largest_operator_level {
                        largest_operator_level = *level;
                        largest_operator_index = i as i64;
                    }
                }
                _ => {}
            }
        }
        if largest_operator_index > -1 {
            // We have at least one operator, and this is the one to dig into
            let operatorname = match &queue[largest_operator_index as usize] {
                parse::WithOperators::Operators(o) => o.trim(),
                _ => unreachable!(),
            };
            let operator = match program.resolve_operator(scope, &operatorname.to_string()) {
                Some(o) => Ok(o),
                None => Err(format!("Operator {} not found", operatorname)),
            }?;
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
                        operatorname
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => {
                        Ok(baseassignablelist)
                    }
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is an infix operator but preceded by another operator {}",
                        operatorname, o
                    )),
                }?;
                let second_arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!("Operator {} is an infix operator but missing a right-hand side value", operatorname)),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => Ok(baseassignablelist),
                    parse::WithOperators::Operators(o) => Err(format!("Operator{} is an infix operator but followed by a lower precedence operator {}", operatorname, o)),
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
                        operatorname
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => {
                        Ok(baseassignablelist)
                    }
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is an prefix operator but followed by another operator {}",
                        operatorname, o
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
                        operatorname
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(bal) => Ok(bal),
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is a postfix operator but preceded by another operator {}",
                        operatorname, o
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
            microstatements = baseassignablelist_to_microstatements(
                &baseassignablelist,
                scope,
                program,
                microstatements,
            )?;
        }
    }
    Ok(microstatements)
}

pub fn assignablestatement_to_microstatements(
    assignable: &parse::AssignableStatement,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    microstatements = withoperatorslist_to_microstatements(
        &assignable.assignables,
        scope,
        program,
        microstatements,
    )?;
    Ok(microstatements)
}

pub fn returns_to_microstatements(
    returns: &parse::Returns,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    if let Some(retval) = &returns.retval {
        // We get all of the microstatements involved in the return statement, then we pop
        // off the last one, if any exists, to get the final return value. Then we shove
        // the other microstatements into the array and the new Return microstatement with
        // that last value attached to it.
        microstatements = withoperatorslist_to_microstatements(
            &retval.assignables,
            scope,
            program,
            microstatements,
        )?;
        let value = match microstatements.pop() {
            None => None,
            Some(v) => Some(Box::new(v)),
        };
        microstatements.push(Microstatement::Return { value });
    } else {
        microstatements.push(Microstatement::Return { value: None });
    }
    Ok(microstatements)
}

pub fn declarations_to_microstatements(
    declarations: &parse::Declarations,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    let (name, assignables, mutable) = match &declarations {
        parse::Declarations::Const(c) => (c.variable.clone(), &c.assignables, false),
        parse::Declarations::Let(l) => (l.variable.clone(), &l.assignables, true),
    };
    // Get all of the assignable microstatements generated
    microstatements =
        withoperatorslist_to_microstatements(assignables, scope, program, microstatements)?;
    let value = match microstatements.pop() {
        None => Err("An assignment without a value should be impossible."),
        Some(v) => Ok(Box::new(v)),
    }?;
    microstatements.push(Microstatement::Assignment {
        name,
        value,
        mutable,
    });
    Ok(microstatements)
}

pub fn statement_to_microstatements(
    statement: &parse::Statement,
    scope: &mut Scope,
    program: &mut Program,
    microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    match statement {
        // This is just whitespace, so we do nothing here
        parse::Statement::A(_) => Ok(microstatements),
        parse::Statement::Assignables(assignable) => Ok(assignablestatement_to_microstatements(
            assignable,
            scope,
            program,
            microstatements,
        )?),
        parse::Statement::Returns(returns) => Ok(returns_to_microstatements(
            returns,
            scope,
            program,
            microstatements,
        )?),
        parse::Statement::Declarations(declarations) => Ok(declarations_to_microstatements(
            declarations,
            scope,
            program,
            microstatements,
        )?),
        parse::Statement::Assignments(_assignments) => Err("Implement me".into()),
        parse::Statement::Conditional(_condtitional) => Err("Implement me".into()),
    }
}
