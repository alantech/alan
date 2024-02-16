// TODO: Use the Program type to load up the code and all of the relevant data structures, then
// start manipulating them to produce Rust code. Because of the borrow checker, making
// idiomatic-looking Rust from Alan may be tough, so let's start off with something like the old
// lntoamm and just generate a crap-ton of simple statements with auto-generated variable names and
// let LLVM optimize it all away.

use crate::program::Program;
use crate::parse::{Statement, WithOperators, BaseAssignable, Constants};

pub fn lntors(entry_file: String) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: Support things beyond the "Hello, World" example
    let mut out = "".to_string();
    let program = Program::new(entry_file)?;
    // Assuming a single scope for now
    let scope = match program.scopes_by_file.get(&program.entry_file.clone()) {
        Some((_, _, s)) => s,
        None => {
            return Err("Somehow didn't find a scope for the entry file!?".into());
        }
    };
    // Assuming the 'start' handler has been defined
    let start = match scope.handlers.get("start") {
        Some(h) => h,
        None => {
            return Err("Entry file has no handlers. This is not yet supported.".into());
        }
    };
    // A handler without a function should be impossible, so this part, at least, shouldn't change
    let func = match scope.functions.get(&start.functionname.clone()) {
        Some(f) => f,
        None => {
            return Err("A handler has been found without a function definition. This should be impossible.".into());
        }
    };
    // The `start` handler takes no arguments and returns no value
    assert_eq!(func.args.len(), 0);
    assert_eq!(func.rettype, None);
    // Assertion proven, start emitting the `start` handler as a `main` function
    out = "fn main() {\n".to_string();
    for statement in &func.statements {
        // TODO: Need a proper root scope to define these mappings better, and a statement to
        // "microstatement" function to encapsulate all of the logic (and dynamic precedence logic
        // to construct a tree to depth-first traverse) For now, we're gonna wing it to have
        // something here.
        let mut stmt = "".to_string();
        match statement {
            Statement::A(_) => { continue; },
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
                                                _ => false
                                            } {
                                                // TODO: Real function name lookup goes here
                                                match var.as_str() {
                                                    "print" => {
                                                      stmt = format!("{}println!", stmt).to_string();
                                                    },
                                                    _ => {
                                                        return Err(format!("Function {} not found", var).into());
                                                    }
                                                }
                                            } else {
                                                return Err(format!("Invalid syntax after {}", var).into());
                                            }
                                        } else {
                                            // It's just a variable, return it as-is
                                            stmt = format!("{}{}", stmt, var);
                                        }
                                    },
                                    BaseAssignable::FnCall(call) => {
                                        // TODO: This should be properly recursive, just going to
                                        // hardwire grabbing the constant from within it for now
                                        let arg = &call.assignablelist[0][0];
                                        let txt = match arg {
                                            WithOperators::BaseAssignableList(l) => {
                                                match &l[0] {
                                                    BaseAssignable::Constants(c) => {
                                                        match c {
                                                            Constants::Strn(s) => s,
                                                            _ => {
                                                                return Err("Unsupported constant type".into());
                                                            }
                                                        }
                                                    },
                                                    _ => {
                                                        return Err("Unsupported argument type".into());
                                                    }
                                                }
                                            },
                                            _ => {
                                                return Err("Unsupported argument type".into());
                                            }
                                        };
                                        stmt = format!("{}({})", stmt, txt);
                                    },
                                    _ => {
                                        return Err("Unsupported assignable type".into());
                                    }
                                }
                            }
                        },
                        _ => {
                            return Err("Operators currently unsupported".into());
                        }
                    }
                }
            },
            _ => {
              return Err("Unsupported statement".into());
            }
        }
        out = format!("{}  {};\n", out, stmt);
    }
    out = format!("{}\n}}", out);
    Ok(out)
}