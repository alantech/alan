// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.
use ordered_hash_map::OrderedHashMap;

use crate::lntors::typen;
use crate::program::{Function, Microstatement, Program, Scope};

pub fn from_microstatement(
    microstatement: &Microstatement,
    scope: &Scope,
    program: &Program,
    mut out: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match microstatement {
        Microstatement::Arg { .. } => Ok(("".to_string(), out)), // Skip arg microstatements that are just used for discovery during generation
        Microstatement::Assignment { name, value } => {
            let (val, o) = from_microstatement(value, scope, program, out)?;
            // I wish I didn't have to write the following line because you can't re-assign a
            // variable in a let destructuring, afaict
            out = o;
            Ok((format!("let {} = {}", name, val,).to_string(), out))
        }
        Microstatement::Value {
            typen,
            representation,
        } => match typen.as_str() {
            "String" => Ok((format!("{}.to_string()", representation).to_string(), out)),
            _ => Ok((representation.clone(), out)),
        },
        Microstatement::FnCall { function, args } => {
            let mut arg_types = Vec::new();
            for arg in args {
                let arg_type = arg.get_type(scope, program)?;
                arg_types.push(arg_type);
            }
            match program.resolve_function(scope, function, &arg_types) {
                None => Err(format!("Function {} not found", function).into()),
                Some((f, _s)) => match &f.bind {
                    None => {
                        // Come up with a function name that is unique so Rust doesn't choke on
                        // duplicate function names that are allowed in Alan
                        let rustname = format!("{}_{}", f.name, f.args.iter().map(|(_, typename)| { typename.clone() /* TODO: Handle generic types better, also type inference */ }).collect::<Vec<String>>().join("_")).to_string();
                        // Make the function we need, but with the name we're
                        out = generate(rustname.clone(), &f, scope, program, out)?;
                        // Now call this function
                        let mut argstrs = Vec::new();
                        for arg in args {
                            let (a, o) = from_microstatement(arg, scope, program, out)?;
                            out = o;
                            argstrs.push(a);
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
                            argstrs.push(a);
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
        Microstatement::Emit { event, value } => match value {
            Some(val) => {
                let (emitval, o) = from_microstatement(val, scope, program, out)?;
                out = o;
                Ok((format!("event::{}({})", event, emitval,).to_string(), out))
            }
            None => Ok((format!("event::{}()", event).to_string(), out)),
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
        if let Some((t, s)) = program.resolve_type(scope, &arg.1) {
            if let Ok(t_str) = typen::generate(t, s, program) {
                arg_strs.push(format!("{}: {}", arg.0, t_str).to_string());
            } else {
                return Err(format!("Failed to convert Alan type {} to Rust", &arg.1).into());
            }
        } else {
            return Err(format!("Could not find type {}", &arg.1).into());
        }
    }
    let opt_ret_str = match &function.rettype {
        Some(rettype) => match program.resolve_type(scope, rettype) {
            None => None,
            Some((t, s)) => match typen::generate(t, s, program) {
                Ok(t) => Some(t),
                Err(e) => return Err(e),
            },
        },
        None => None,
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
