use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use crate::codegen::{self, DepType};
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
    codegen::register_dependency(DepType::Nodejs, d, deps);
}

pub fn lntojs(
    entry_file: String,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    Program::set_target_lang_js();
    codegen::bootstrap(&entry_file, |func, scope| {
        crate::program::inline::set_inline_targets(crate::program::inline::compute_inline_targets(
            &func[0],
        ));
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
    })
}
