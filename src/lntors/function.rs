// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.
use ordered_hash_map::OrderedHashMap;

use crate::lntors::typen;
use crate::program::{CType, Function, Microstatement, Program, Scope};

pub fn from_microstatement(
    microstatement: &Microstatement,
    scope: &Scope,
    program: &Program,
    mut out: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match microstatement {
        Microstatement::Arg { name, .. } => {
            // TODO: Update the serialization logic to understand values vs references so we can
            // eliminate this useless (and harmful for mutable references) clone
            Ok((
                format!("let mut {} = {}.clone()", name, name).to_string(), // TODO: not always mutable
                out,
            ))
        }
        Microstatement::Assignment {
            name,
            value,
            mutable,
        } => {
            let (val, o) = from_microstatement(value, scope, program, out)?;
            // I wish I didn't have to write the following line because you can't re-assign a
            // variable in a let destructuring, afaict
            out = o;
            Ok((
                format!(
                    "let {}{} = {}",
                    if *mutable {
                        "mut "
                    } else {
                        "mut " /* TODO: shouldn't be mut */
                    },
                    name,
                    val,
                )
                .to_string(),
                out,
            ))
        }
        Microstatement::Value {
            typen,
            representation,
        } => match &typen {
            CType::Bound(a, _) if a == "string" => {
                Ok((format!("{}.to_string()", representation).to_string(), out))
            }
            CType::Function(..) => {
                // We need to make sure this function we're referencing exists
                match scope.functions.get(representation) {
                    Some(fns) => {
                        let f = &fns[0]; // TODO: Proper implementation selection
                        let mut arg_strs = Vec::new();
                        for arg in &f.args {
                            match typen::ctype_to_rtype(&arg.1, scope, program, false) {
                                Err(e) => Err(e),
                                Ok(s) => {
                                    arg_strs.push(
                                        s.replace("<", "_")
                                            .replace(">", "_")
                                            .replace(",", "_")
                                            .replace(" ", ""),
                                    );
                                    /* TODO: Handle generic types better, also type inference */
                                    Ok(())
                                }
                            }?;
                        }
                        // Come up with a function name that is unique so Rust doesn't choke on
                        // duplicate function names that are allowed in Alan
                        let rustname = format!("{}_{}", f.name, arg_strs.join("_")).to_string();
                        // Make the function we need, but with the name we're
                        out = generate(rustname.clone(), &f, scope, program, out)?;
                        Ok((rustname, out))
                    }
                    None => Err(format!(
                        "Somehow can't find a definition for function {}",
                        representation
                    )
                    .into()),
                }
            }
            _ => Ok((representation.clone(), out)),
        },
        Microstatement::Array { vals, .. } => {
            let mut val_representations = Vec::new();
            for val in vals {
                let (rep, o) = from_microstatement(val, scope, program, out)?;
                val_representations.push(rep);
                out = o;
            }
            Ok((
                format!("vec!({})", val_representations.join(", ")).to_string(),
                out,
            ))
        }
        Microstatement::Type { typen, keyvals } => {
            // Need to make sure the struct is defined, first
            let (_, o) = typen::generate(typen, scope, program, out)?;
            out = o;
            // Now generating the representation
            let mut keyval_representations = Vec::new();
            for (key, val) in keyvals.iter() {
                let (rep, o) = from_microstatement(val, scope, program, out)?;
                keyval_representations.push(format!("{}: {},", key, rep));
                out = o;
            }
            Ok((
                format!(
                    r#"{} {{
    {}
}}"#,
                    typen::ctype_to_rtype(&typen, scope, program, false)?,
                    keyval_representations.join("\n")
                )
                .to_string(),
                out,
            ))
        }
        Microstatement::FnCall { function, args } => {
            let mut arg_types = Vec::new();
            let mut arg_type_strs = Vec::new();
            for arg in args {
                let arg_type = arg.get_type(scope, program)?;
                arg_types.push(arg_type.clone());
                match typen::ctype_to_rtype(&arg_type, scope, program, false) {
                    Err(e) => Err(e),
                    Ok(s) => {
                        arg_type_strs.push(s);
                        Ok(())
                    }
                }?
            }
            match program.resolve_function(scope, function, &arg_types) {
                None => Err(format!(
                    "Function {}({}) not found",
                    function,
                    arg_type_strs.join(", ")
                )
                .into()),
                Some((f, _s)) => match &f.bind {
                    None => {
                        let mut arg_strs = Vec::new();
                        for arg in &f.args {
                            match typen::ctype_to_rtype(&arg.1, scope, program, false) {
                                Err(e) => Err(e),
                                Ok(s) => {
                                    arg_strs.push(
                                        s.replace("<", "_")
                                            .replace(">", "_")
                                            .replace(",", "_")
                                            .replace(" ", ""),
                                    );
                                    /* TODO: Handle generic types better, also type inference */
                                    Ok(())
                                }
                            }?;
                        }
                        // Come up with a function name that is unique so Rust doesn't choke on
                        // duplicate function names that are allowed in Alan
                        let rustname = format!("{}_{}", f.name, arg_strs.join("_")).to_string();
                        // Make the function we need, but with the name we're
                        out = generate(rustname.clone(), &f, scope, program, out)?;
                        // Now call this function
                        let mut argstrs = Vec::new();
                        for arg in args {
                            let (a, o) = from_microstatement(arg, scope, program, out)?;
                            out = o;
                            // If the argument is itself a function, this is the only place in Rust
                            // where you can't pass by reference, so we check the type and change
                            // the argument output accordingly.
                            let arg_type = arg.get_type(scope, program)?;
                            match arg_type {
                                CType::Function(..) => argstrs.push(format!("{}", a)),
                                _ => argstrs.push(format!("&mut {}", a)),
                            }
                        }
                        Ok((
                            format!("{}({})", rustname, argstrs.join(", ")).to_string(),
                            out,
                        ))
                    }
                    Some(rustname) => {
                        let mut argstrs = Vec::new();
                        for arg in args {
                            let (a, o) = from_microstatement(arg, scope, program, out)?;
                            out = o;
                            // If the argument is itself a function, this is the only place in Rust
                            // where you can't pass by reference, so we check the type and change
                            // the argument output accordingly.
                            let arg_type = arg.get_type(scope, program)?;
                            match arg_type {
                                CType::Function(..) => argstrs.push(format!("{}", a)),
                                _ => argstrs.push(format!("&mut {}", a)),
                            }
                        }
                        Ok((
                            format!("{}({})", rustname, argstrs.join(", ")).to_string(),
                            out,
                        ))
                    }
                },
            }
        }
        Microstatement::Return { value } => match value {
            Some(val) => {
                let (retval, o) = from_microstatement(val, scope, program, out)?;
                out = o;
                Ok((format!("return {}", retval).to_string(), out))
            }
            None => Ok(("return".to_string(), out)),
        },
    }
}

pub fn generate(
    rustname: String,
    function: &Function,
    scope: &Scope,
    program: &Program,
    mut out: OrderedHashMap<String, String>,
) -> Result<OrderedHashMap<String, String>, Box<dyn std::error::Error>> {
    let mut fn_string = "".to_string();
    // First make sure all of the function argument types are defined
    let mut arg_strs = Vec::new();
    for arg in &function.args {
        let (l, t) = arg;
        let (t_str, o) = typen::generate(t, scope, program, out)?;
        out = o;
        arg_strs.push(format!("{}: &{}", l, t_str).to_string());
    }
    let opt_ret_str = match &function.rettype {
        CType::Void => None,
        otherwise => {
            let (t_str, o) = typen::generate(otherwise, scope, program, out)?;
            out = o;
            Some(t_str)
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
        let (stmt, o) = from_microstatement(microstatement, scope, program, out)?;
        out = o;
        fn_string = format!("{}    {};\n", fn_string, stmt);
    }
    fn_string = format!("{}}}", fn_string);
    out.insert(rustname, fn_string);
    Ok(out)
}
