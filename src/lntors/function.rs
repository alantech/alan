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
        Microstatement::Assignment { name, value } => Ok(format!(
            "let {} = {}",
            name,
            from_microstatement(value, scope, program)?
        )
        .to_string()),
        Microstatement::Value { representation, .. } => Ok(representation.clone()),
        Microstatement::FnCall { function, args } => {
            // TODO: Add logic to get the type from the args array of microstatements. For the sake
            // of keeping the hello world test working for now, adding some magic knowledge that
            // should not be hardcoded until the microstatement generation adds the type
            // information needed.
            match program.resolve_function(scope, function, &vec!["String".to_string()]) {
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
    }
}

pub fn generate(
    function: &Function,
    scope: &Scope,
    program: &Program,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut out = "".to_string();
    // First make sure all of the function argument types are defined
    for arg in &function.args {
        match program.resolve_type(scope, &arg.1) {
            None => continue,
            Some((t, s)) => {
                out = format!("{}{}", out, typen::generate(t, s, program)?);
            }
        }
    }
    match &function.rettype {
        Some(rettype) => match program.resolve_type(scope, rettype) {
            None => {}
            Some((t, s)) => {
                out = format!("{}{}", out, typen::generate(t, s, program)?);
            }
        },
        None => {}
    }
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
        function
            .args
            .iter()
            .map(|(argname, argtype)| format!("{}: {}", argname, argtype).to_string()) // TODO: Don't assume Rust and Alan types exactly match syntax
            .collect::<Vec<String>>()
            .join(", "),
        match &function.rettype {
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
