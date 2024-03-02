// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.

use crate::lntors::typen;
use crate::program::{Function, Microstatement, Program, Scope};

pub fn from_microstatement(
    microstatement: &Microstatement,
    scope: &Scope,
    program: &Program,
) -> Result<String, Box<dyn std::error::Error>> {
    match microstatement {
        Microstatement::Arg { .. } => Ok("".to_string()), // Skip arg microstatements that are just used for discovery during generation
        Microstatement::Assignment { name, value } => Ok(format!(
            "let {} = {}",
            name,
            from_microstatement(value, scope, program)?
        )
        .to_string()),
        Microstatement::Value { typen, representation} => match typen.as_str() {
            "String" => Ok(format!("{}.to_string()", representation).to_string()),
            _ => Ok(representation.clone())
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
                    None => Err("Inlining user-defined functions not yet supported".into()),
                    Some(rustname) => {
                        let mut argstrs = Vec::new();
                        for arg in args {
                            let a = from_microstatement(arg, scope, program)?;
                            argstrs.push(a);
                        }
                        Ok(format!("{}({})", rustname, argstrs.join(", ")).to_string())
                    }
                },
            }
        }
        Microstatement::Return { value } => match value {
            Some(val) => {
                Ok(format!("return {}", from_microstatement(val, scope, program)?).to_string())
            }
            None => Ok("return".to_string()),
        },
        Microstatement::Emit { event, value } => match value {
            Some(val) => {
                Ok(format!("event::{}({})", event, from_microstatement(val, scope, program)?).to_string())
            }
            None => {
                Ok(format!("event::{}()", event).to_string())
            }
        },
    }
}

pub fn generate(
    function: &Function,
    scope: &Scope,
    program: &Program,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut out = "".to_string();
    // First make sure all of the function argument types are defined
    let mut arg_strs = Vec::new();
    for arg in &function.args {
        if let Some((t, s)) = program.resolve_type(scope, &arg.1) {
            if let Ok(t_str) = typen::generate(t, s, program) {
                arg_strs.push(t_str);
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
    out = format!(
        "{}fn {}({}){} {{\n",
        out,
        function.name.clone(),
        arg_strs.join(", "),
        match opt_ret_str {
            Some(rettype) => format!(" -> {}", rettype).to_string(),
            None => "".to_string(),
        },
    )
    .to_string();
    for microstatement in &function.microstatements {
        let stmt = from_microstatement(microstatement, scope, program)?;
        out = format!("{}    {};\n", out, stmt);
    }
    out = format!("{}}}", out);
    Ok(out)
}
