use ordered_hash_map::OrderedHashMap;

use crate::lntojs::function::generate as fn_generate;
use crate::program::{CType, Program};

mod function;
mod typen;

pub fn lntojs(
    entry_file: String,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    Program::set_target_lang_js();
    Program::load(entry_file.clone())?;
    let program = Program::get_program();
    let scope = program.scope_by_file(&entry_file)?;
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
    assert_eq!(func[0].args().len(), 0);
    // Assertion proven, start emitting the `main` function to run as an IIFE
    let (fns, deps) = fn_generate(
        "main".to_string(),
        &func[0],
        scope,
        OrderedHashMap::new(),
        OrderedHashMap::new(),
    )?;
    let main_call = if let CType::Type(n, _) = &*func[0].rettype() {
        if n == "ExitCode" {
            "main().then(process.exit);"
        } else {
            "main();"
        }
    } else {
        "main();"
    };
    Program::return_program(program);
    Ok((
        format!(
            "{}\n{}\n{}",
            deps.keys()
                .map(|k| format!("import * as {k} from \"{k}\";"))
                .collect::<Vec<String>>()
                .join("\n"),
            fns.into_values().collect::<Vec<String>>().join("\n"),
            main_call,
        ),
        deps,
    ))
}
