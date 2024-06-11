use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::microstatement::{statement_to_microstatements, Microstatement};
use super::Export;
use super::FnKind;
use super::Program;
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
    pub fn from_ast(
        scope: &mut Scope,
        program: &mut Program,
        function_ast: &parse::Functions,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // In the top-level of a file, all functions *must* be named
        let name = match &function_ast.optname {
            Some(name) => name.clone(),
            None => {
                return Err("Top-level function without a name!".into());
            }
        };
        Function::from_ast_with_name(scope, program, function_ast, is_export, name)
    }

    pub fn from_ast_with_name(
        scope: &mut Scope,
        program: &mut Program,
        function_ast: &parse::Functions,
        is_export: bool,
        name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Add code to properly convert the typeassignable vec into a CType tree and use it.
        // For now, just hardwire the parsing as before.
        let (args, rettype) = match &function_ast.opttype {
            None => Ok::<(Vec<(String, CType)>, CType), Box<dyn std::error::Error>>((
                Vec::new(),
                CType::Void,
            )), // TODO: Does this path *ever* trigger?
            Some(typeassignable) if typeassignable.len() == 0 => Ok((Vec::new(), CType::Void)),
            Some(typeassignable) => {
                let ctype = withtypeoperatorslist_to_ctype(&typeassignable, scope, program)?;
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
                    for i in 0..typeassignable.len() {
                        if typeassignable[i].to_string().trim() == "->" {
                            lastfnop = Some(i);
                        }
                    }
                    if let Some(lastfnop) = lastfnop {
                        let returntypeassignables =
                            typeassignable[lastfnop + 1..typeassignable.len()].to_vec();
                        // TODO: Be more complete here
                        let name = output_type
                            .to_strict_string(false)
                            .replace(" ", "_")
                            .replace(",", "_")
                            .replace(":", "_")
                            .replace("{", "_")
                            .replace("}", "_")
                            .replace("|", "_")
                            .replace("()", "void"); // Really bad
                        if let Some(_) = program.resolve_type(scope, &name) {
                            // Don't recreate the exact same thing. It only causes pain
                        } else {
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
                                    typeassignables: returntypeassignables,
                                }),
                                optsemicolon: ";".to_string(),
                            };
                            CType::from_ast(scope, program, &parse_type, false)?;
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
                let degrouped_input = match input_type {
                    CType::Group(c) => *c.clone(),
                    otherwise => otherwise.clone(),
                };
                let mut out_args = Vec::new();
                match degrouped_input {
                    CType::Tuple(ts) => {
                        for i in 0..ts.len() {
                            out_args.push(match &ts[i] {
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
        }?;
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
        };
        let microstatements = {
            let mut ms = Vec::new();
            for (name, typen) in &args {
                ms.push(Microstatement::Arg {
                    name: name.clone(),
                    typen: typen.clone(),
                });
            }
            for statement in &statements {
                ms = statement_to_microstatements(statement, scope, program, ms)?;
            }
            ms
        };
        let kind = match &function_ast.fullfunctionbody {
            parse::FullFunctionBody::BindFunction(b) => FnKind::Bind(b.rustfunc.clone()),
            _ => FnKind::Normal(statements),
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
        Ok(())
    }
}
