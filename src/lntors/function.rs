// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.

use crate::parse::{BaseAssignable, Constants, Statement, WithOperators};
use crate::program::{Program, Function, Scope};

pub fn generate(function: &Function, scope: &Scope, program: &Program) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: Support things beyond the "Hello, World" example
    let mut out = format!("fn {}() {{\n", function.name).to_string();
    for statement in &function.statements {
        // TODO: Need a proper root scope to define these mappings better, and a statement to
        // "microstatement" function to encapsulate all of the logic (and dynamic precedence logic
        // to construct a tree to depth-first traverse) For now, we're gonna wing it to have
        // something here.
        let mut stmt = "".to_string();
        match statement {
            Statement::A(_) => {
                continue;
            }
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
                                                // TODO: Real function name lookup goes here
                                                match var.as_str() {
                                                    "print" => {
                                                        stmt =
                                                            format!("{}println!", stmt).to_string();
                                                    }
                                                    _ => {
                                                        return Err(format!(
                                                            "Function {} not found",
                                                            var
                                                        )
                                                        .into());
                                                    }
                                                }
                                            } else {
                                                return Err(format!(
                                                    "Invalid syntax after {}",
                                                    var
                                                )
                                                .into());
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
                                                        return Err(
                                                            "Unsupported constant type".into()
                                                        );
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
            }
            _ => {
                return Err("Unsupported statement".into());
            }
        }
        out = format!("{}  {};\n", out, stmt);
    }
    out = format!("{}}}", out);
    Ok(out)
}
