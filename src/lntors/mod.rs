use ordered_hash_map::OrderedHashMap;

use crate::lntors::event::generate as evt_generate;
use crate::lntors::function::generate as fn_generate;
use crate::program::Program;

mod event;
mod function;
mod typen;

pub fn lntors(entry_file: String) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: Figure out a better way to include custom Rust functions that we may then bind
    let preamble = include_str!("../std/root.rs").to_string();
    let program = Program::new(entry_file)?;
    // Generate all of the events and their handlers defined across all scopes
    // TODO: Pruning unused events should be pursued eventually
    let mut fns = OrderedHashMap::new();
    let (event_fns, f) = evt_generate(&program, fns)?;
    fns = f;
    // Getting the entry scope, where the `main` function is expected
    let scope = match program.scopes_by_file.get(&program.entry_file.clone()) {
        Some((_, _, s)) => s,
        None => {
            return Err("Somehow didn't find a scope for the entry file!?".into());
        }
    };
    // Without support for building shared libs yet, assume there is an `export fn main` in the
    // entry file or fail otherwise
    match scope.exports.get("main") {
        Some(_) => {}
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
    // support that, yet. Also assert that there is only a single `main` function, since *usually*
    // you're allowed to have multiple functions with the same name as long as they have different
    // arguments.
    assert_eq!(func.len(), 1);
    assert_eq!(func[0].args.len(), 0);
    // Assertion proven, start emitting the Rust `main` function
    fns = fn_generate("main".to_string(), &func[0], &scope, &program, fns)?;
    Ok(format!("{}\n{}\n{}", preamble, event_fns, fns.into_values().collect::<Vec<String>>().join("\n")).to_string())
}
