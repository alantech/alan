use std::cell::Cell;
use std::fs::read_to_string;
use std::pin::Pin;

use super::Scope;
use crate::parse;

use ordered_hash_map::OrderedHashMap;

// This data structure should allow file-level reloading, which we can probably use as a rough
// approximation for iterative recompliation and language server support, and since Rust is fast,
// this might just be "good enough" assuming non-insane source file sizes.
#[derive(Debug, Default)]
pub struct Program<'a> {
    #[allow(clippy::box_collection)]
    pub scopes_by_file: OrderedHashMap<String, (Pin<Box<String>>, parse::Ln, Scope<'a>)>,
    pub env: OrderedHashMap<String, String>,
}

thread_local!(pub static PROGRAM_RS: Cell<Program<'static>> = Cell::new(
Program {
    scopes_by_file: OrderedHashMap::new(),
    env: {
        let mut env = OrderedHashMap::new();
        for (k, v) in std::env::vars() {
            env.insert(k.to_string(), v.to_string());
        }
        env.insert("ALAN_OUTPUT_LANG".to_string(), "rs".to_string());
        env.insert("ALAN_PLATFORM".to_string(), if cfg!(target_os="windows") {
            "windows".to_string()
        } else if cfg!(target_os="macos") {
            "macos".to_string()
        } else if cfg!(target_os="linux") {
            "linux".to_string()
        } else {
            "what".to_string()
        });
        env
    },
}));
thread_local!(pub static PROGRAM_JS: Cell<Program<'static>> = Cell::new(
Program {
    scopes_by_file: OrderedHashMap::new(),
    env: {
        let mut env = OrderedHashMap::new();
        if !cfg!(target_family="wasm") {
            for (k, v) in std::env::vars() {
                env.insert(k.to_string(), v.to_string());
            }
        } else {
            env.insert("ALAN_TARGET".to_string(), "release".to_string());
        }
        env.insert("ALAN_OUTPUT_LANG".to_string(), "js".to_string());
        env.insert("ALAN_PLATFORM".to_string(), "browser".to_string());
        env
    },
}));

thread_local!(static TARGET_LANG_RS: Cell<bool> = const { Cell::new(true) });

impl<'a> Program<'a> {
    pub fn load(path: String) -> Result<(), Box<dyn std::error::Error>> {
        let program = Program::get_program();
        if program.scopes_by_file.contains_key(&path) {
            // Already loaded, let's get out of here
            Program::return_program(program);
            return Ok(());
        }
        Program::return_program(program);
        let ln_src = if path.starts_with('@') {
            match path.as_str() {
                "@std/fs" => include_str!("../std/fs.ln").to_string(),
                "@std/seq" => include_str!("../std/seq.ln").to_string(),
                _ => {
                    return Err(format!("Unknown standard library named {}", &path).into());
                }
            }
        } else {
            read_to_string(&path)?
        };
        Scope::from_src(&path, ln_src)
    }

    pub fn scope_by_file(&self, path: &str) -> Result<&Scope<'a>, Box<dyn std::error::Error>> {
        match self.scopes_by_file.get(path) {
            Some((_, _, s)) => Ok(s),
            None => Err(format!("Could not find a scope for file {path}").into()),
        }
    }

    pub fn get_program() -> Program<'static> {
        if TARGET_LANG_RS.get() {
            PROGRAM_RS.take()
        } else {
            PROGRAM_JS.take()
        }
    }

    pub fn return_program(p: Program<'static>) {
        if TARGET_LANG_RS.get() {
            PROGRAM_RS.set(p);
        } else {
            PROGRAM_JS.set(p);
        }
    }

    pub fn set_target_lang_js() {
        TARGET_LANG_RS.set(false);
    }

    pub fn set_target_lang_rs() {
        TARGET_LANG_RS.set(true);
    }

    pub fn is_target_lang_rs() -> bool {
        TARGET_LANG_RS.get()
    }
}
