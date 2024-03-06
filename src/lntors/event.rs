// Builds the event functions using the code from their handlers. Currently an O(n^2) algo for
// simplicity.

use crate::lntors::function::from_microstatement;
use crate::program::{Function, Microstatement, Program};

pub fn generate(program: &Program) -> Result<String, Box<dyn std::error::Error>> {
    // The events will all go into an `event` sub-module with every function marked public. These
    // event functions are what the `emit <eventName> <optionalValue>;` statements will call, so
    // something like:
    // emit foo "bar";
    // becomes something like:
    // event::foo("bar");
    //
    // The event functions copy their input argument for each handler, which is run within a
    // thread, looking something like:
    // let arg_copy = arg.clone();
    // std::thread::spawn(move || {
    //     <generated code that references arg_copy>
    // });
    //
    // With the generated code coming from the handler functions, in the order they're discovered
    // across the scopes.
    let mut out = "mod event {\n".to_string();
    // First scan through all scopes for each defined event
    for (_, (_, _, eventscope)) in program.scopes_by_file.iter() {
        for (eventname, event) in eventscope.events.iter() {
            // Define the function for this event
            out = format!(
                "{}    pub fn {}{} {{\n",
                out,
                eventname,
                match event.typename.as_str() {
                    "void" => "".to_string(),
                    x => format!("(arg: {})", x).to_string(),
                },
            );
            let expected_arg_type = match event.typename.as_str() {
                "void" => None,
                x => Some(x.to_string()),
            };
            // Next scan through all scopes for handlers that bind to this particular event
            for (_, (_, _, handlerscope)) in program.scopes_by_file.iter() {
                for (handlereventname, handler) in handlerscope.handlers.iter() {
                    if eventname == handlereventname {
                        // We have a match, grab the function(s) from this scope
                        let fns: &Vec<Function> =
                            match handlerscope.functions.get(&handler.functionname) {
                                Some(res) => Ok(res),
                                None => Err("Somehow unable to find function for handler"),
                            }?;
                        // Determine the correct function from the vector by finding the last one
                        // that implements the correct interface (if the event has an argument,
                        // then exactly one argument of the same type and no return type, if no
                        // argument then no arguments to the function)
                        let mut handler_pos = None;
                        for (i, possible_fn) in fns.iter().enumerate() {
                            if let Some(_) = &possible_fn.rettype {
                                continue;
                            }
                            match expected_arg_type {
                                None => {
                                    if possible_fn.args.len() == 0 {
                                        handler_pos = Some(i);
                                    }
                                }
                                Some(ref arg_type) => {
                                    if possible_fn.args.len() == 1
                                        && &possible_fn.args[0].1 == arg_type
                                    {
                                        handler_pos = Some(i);
                                    }
                                }
                            }
                        }
                        let handler_fn = match handler_pos {
                            None => Err("No function properly matches the event signature"),
                            Some(i) => Ok(&fns[i]),
                        }?;
                        // Because of what we validated above, we *know* that if this event takes
                        // an argument, then the first microstatement is an Arg microstatement that
                        // the normal code generation path is going to ignore. We're going to peek
                        // at the first microstatement of the function and decide if we need to
                        // insert the special `let <argname> = arg.clone();` statement or not
                        match &handler_fn.microstatements[0] {
                            Microstatement::Arg { name, .. } => {
                                // TODO: guard against valid alan variable names that are no valid
                                // rust variable names
                                out = format!("{}        let {} = arg.clone();\n", out, name)
                                    .to_string();
                            }
                            _ => {}
                        }
                        // Now we generate the thread to run this event handler on
                        out = format!("{}        std::thread::spawn(move || {{\n", out).to_string();
                        if let Some(b) = &handler_fn.bind {
                            // If it's a bound function, just call it with the argument, if there
                            // is one
                            let arg_str = match &handler_fn.microstatements[0] {
                                Microstatement::Arg { name, .. } => name.clone(),
                                _ => "".to_string(),
                            };
                            out = format!("{}            super::{}({});\n", out, b, arg_str);
                        } else {
                            // Inline the microstatements if it's an Alan function
                            for microstatement in &handler_fn.microstatements {
                                let stmt =
                                    from_microstatement(microstatement, handlerscope, program)?;
                                if stmt != "" {
                                    out = format!("{}            {};\n", out, stmt);
                                }
                            }
                        }
                        // And close out the thread
                        out = format!("{}        }});\n", out).to_string();
                    }
                }
            }
            // Now we finally close out this function
            out = format!("{}   }}\n", out).to_string();
        }
    }
    // And close out the event module
    out = format!("{}}}\n", out).to_string();
    Ok(out)
}
