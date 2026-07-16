use ordered_hash_map::OrderedHashMap;

use crate::codegen::{self, DepType};
use crate::lntors::function::generate as fn_generate;
use crate::program::Program;

mod function;
mod typen;

pub(crate) fn register_rust_dependency(
    d: &crate::program::CType,
    deps: &mut OrderedHashMap<String, String>,
) {
    codegen::register_dependency(DepType::Rust, d, deps);
}

pub fn lntors(
    entry_file: String,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    Program::set_target_lang_rs();
    codegen::bootstrap(&entry_file, |func, scope| {
        crate::program::inline::set_inline_targets(crate::program::inline::compute_inline_targets(
            &func[0],
        ));
        function::set_fn_value_refs(&func[0]);
        let (fns, deps) = fn_generate(
            "main".to_string(),
            &func[0],
            scope,
            OrderedHashMap::new(),
            OrderedHashMap::new(),
        )?;
        Ok((
            format!(
                "use std::io::Write;\n\n{}",
                fns.into_values().collect::<Vec<String>>().join("\n")
            ),
            deps,
        ))
    })
}
