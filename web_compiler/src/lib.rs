use wasm_bindgen::prelude::*;

use alan_compiler::lntojs::lntojs;
use alan_compiler::program::Program;
use alan_compiler::program::Scope;

#[wasm_bindgen]
pub fn compile(src: &str) -> String {
    Program::set_target_lang_js();
    if let Err(e) = Scope::from_src("program.ln", src.to_string()) {
        return format!("{e:?}");
    }
    match lntojs("program.ln".to_string()) {
        Err(e) => format!("{e:?}"),
        Ok((js, _)) => js,
    }
}
