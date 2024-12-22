use wasm_bindgen::prelude::*;

use alan_compiler::lntojs::lntojs;
use alan_compiler::program::Program;
use alan_compiler::program::Scope;

#[wasm_bindgen]
pub fn compile(src: &str) -> String {
    Program::set_target_lang_js();
    match Scope::from_src("program.ln", src.to_string()) {
        Err(e) => {
            return format!("{:?}", e);
        }
        Ok(_) => {}
    };
    match lntojs("program.ln".to_string()) {
        Err(e) => {
            return format!("{:?}", e);
        }
        Ok((js, _)) => js,
    }
}
