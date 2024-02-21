// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.

use crate::lntors::typen;
use crate::parse::{BaseAssignable, Constants, Statement, WithOperators};
use crate::program::{Function, Program, Scope};

pub fn from_statement(
    statement: &Statement,
    scope: &Scope,
    program: &Program,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // TODO: Need a proper root scope to define these mappings better, and a statement to
    // "microstatement" function to encapsulate all of the logic (and dynamic precedence logic
    // to construct a tree to depth-first traverse) For now, we're gonna wing it to have
    // something here.
    let mut stmt = "".to_string();
    match statement {
        Statement::A(_) => Ok(None),
        Statement::Assignables(assignable) => {
            for assignable_or_operator in assignable.assignables.iter() {
                match assignable_or_operator {
                    WithOperators::BaseAssignableList(baseassignablelist) => {
                        for (i, baseassignable) in baseassignablelist.iter().enumerate() {
                            match baseassignable {
                                BaseAssignable::Variable(var) => {
                                    // The behavior of a variable depends on if there's
                                    // anything following after it. Many things following are
                                    // invalid syntax, but `FnCall` and `MethodSep` are valid
                                    let next = baseassignablelist.get(i + 1);
                                    if let Some(otherbase) = next {
                                        if match otherbase {
                                            BaseAssignable::FnCall(_) => true,
                                            BaseAssignable::MethodSep(_) => false, // TODO
                                            _ => false,
                                        } {
                                            match program.resolve_function(scope, var) {
                                                None => {
                                                    return Err(format!(
                                                        "Function {} not found",
                                                        var
                                                    )
                                                    .into());
                                                }
                                                Some((f, s)) => match &f.bind {
                                                    None => {
                                                        return Err("Inlining user-defined functions not yet supported".into());
                                                    }
                                                    Some(rustname) => {
                                                        stmt = format!("{}{}", stmt, rustname)
                                                            .to_string();
                                                    }
                                                },
                                            }
                                        } else {
                                            return Err(
                                                format!("Invalid syntax after {}", var).into()
                                            );
                                        }
                                    } else {
                                        // It's just a variable, return it as-is
                                        stmt = format!("{}{}", stmt, var);
                                    }
                                }
                                BaseAssignable::FnCall(call) => {
                                    // TODO: This should be properly recursive, just going to
                                    // hardwire grabbing the constant from within it for now
                                    let arg = &call.assignablelist[0][0];
                                    let txt = match arg {
                                        WithOperators::BaseAssignableList(l) => match &l[0] {
                                            BaseAssignable::Constants(c) => match c {
                                                Constants::Strn(s) => s,
                                                _ => {
                                                    return Err("Unsupported constant type".into());
                                                }
                                            },
                                            _ => {
                                                return Err("Unsupported argument type".into());
                                            }
                                        },
                                        _ => {
                                            return Err("Unsupported argument type".into());
                                        }
                                    };
                                    stmt = format!("{}({})", stmt, txt);
                                }
                                _ => {
                                    return Err("Unsupported assignable type".into());
                                }
                            }
                        }
                    }
                    _ => {
                        return Err("Operators currently unsupported".into());
                    }
                }
            }
            Ok(Some(stmt))
        }
        _ => Err("Unsupported statement".into()),
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
    for statement in &function.statements {
        let stmt = from_statement(statement, scope, program)?;
        if let Some(s) = stmt {
            out = format!("{}  {};\n", out, s);
        }
    }
    out = format!("{}}}", out);
    Ok(out)
}
