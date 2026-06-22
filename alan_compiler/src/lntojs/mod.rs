use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use crate::lntojs::function::generate as fn_generate;
use crate::program::{CType, Program};

mod function;
mod typen;

fn is_promise_head(typen: Arc<CType>) -> bool {
    let mut t = typen.degroup();
    while matches!(&*t, CType::Type(..) | CType::Group(_)) {
        t = match &*t {
            CType::Type(_, inner) | CType::Group(inner) => inner.clone().degroup(),
            _ => unreachable!(),
        };
    }
    matches!(&*t, CType::Promise(_))
}

fn is_exitcode_type(typen: Arc<CType>) -> bool {
    let mut t = typen.degroup();
    loop {
        match &*t {
            CType::Type(n, _) if n == "ExitCode" => return true,
            CType::Type(_, inner) | CType::Group(inner) | CType::Promise(inner) => {
                t = inner.clone().degroup();
            }
            _ => return false,
        }
    }
}

pub(crate) fn register_nodejs_dependency(d: &CType, deps: &mut OrderedHashMap<String, String>) {
    match d {
        CType::Type(_, t) => match &**t {
            CType::Nodejs(d) => match &**d {
                CType::Dependency(n, v) => {
                    let name = match &**n {
                        CType::TString(s) => s.clone(),
                        _ => CType::fail("Dependency names must be strings"),
                    };
                    let version = match &**v {
                        CType::TString(s) => s.clone(),
                        _ => CType::fail("Dependency versions must be strings"),
                    };
                    deps.insert(name, version);
                }
                _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
            }
            otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Nodejs{{D}} dependencies: {otherwise:?}"))
        }
        CType::Nodejs(d) => match &**d {
            CType::Dependency(n, v) => {
                let name = match &**n {
                    CType::TString(s) => s.clone(),
                    _ => CType::fail("Dependency names must be strings"),
                };
                let version = match &**v {
                    CType::TString(s) => s.clone(),
                    _ => CType::fail("Dependency versions must be strings"),
                };
                deps.insert(name, version);
            }
            _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
        }
        otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Nodejs{{D}} dependencies: {otherwise:?}"))
    }
}

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
    // Determine which single-use functions can be inlined into their sole caller.
    crate::program::inline::set_inline_targets(crate::program::inline::compute_inline_targets(
        &func[0],
    ));
    // Assertion proven, start emitting the `main` function to run as an IIFE
    let (fns, deps) = fn_generate(
        "main".to_string(),
        &func[0],
        scope,
        OrderedHashMap::new(),
        OrderedHashMap::new(),
    )?;
    let main_is_async = is_promise_head(func[0].rettype());
    let main_is_exitcode = is_exitcode_type(func[0].rettype());
    let main_call = if main_is_async {
        if main_is_exitcode {
            "main().then(process.exit).catch(e => console.error(e));"
        } else {
            "main().catch(e => console.error(e));"
        }
    } else if main_is_exitcode {
        "try { process.exit(main()); } catch (e) { console.error(e); }"
    } else {
        "try { main(); } catch (e) { console.error(e); }"
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
