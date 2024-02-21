// TODO: Use the Program type to load up the code and all of the relevant data structures, then
// start manipulating them to produce Rust code. Because of the borrow checker, making
// idiomatic-looking Rust from Alan may be tough, so let's start off with something like the old
// lntoamm and just generate a crap-ton of simple statements with auto-generated variable names and
// let LLVM optimize it all away.

use crate::program::Program;
use crate::lntors::function::generate;

mod function;

pub fn lntors(entry_file: String) -> Result<String, Box<dyn std::error::Error>> {
    let program = Program::new(entry_file)?;
    // Assuming a single scope for now
    let scope = match program.scopes_by_file.get(&program.entry_file.clone()) {
        Some((_, _, s)) => s,
        None => {
            return Err("Somehow didn't find a scope for the entry file!?".into());
        }
    };
    // Without support for building shared libs yet, assume there is an `export fn main` in the
    // entry file or fail otherwise
    match scope.exports.get("main") {
        Some(_) => {},
        None => {
            return Err(
                "Entry file has no `main` function exported. This is not yet supported.".into(),
            );
        }
    };
    // Getting here *should* guarantee that the `main` function exists, so let's grab it.
    let func = match scope.functions.get("main") {
        Some(f) => f,
        None => {
            return Err(
                "An export has been found without a definition. This should be impossible.".into(),
            );
        }
    };
    // The `main` function takes no arguments, for now. It could have a return type, but we don't
    // support that, yet.
    assert_eq!(func.args.len(), 0);
    // Assertion proven, start emitting the Rust `main` function
    Ok(generate(&func, &scope, &program)?)
}
