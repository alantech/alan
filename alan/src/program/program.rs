use std::cell::Cell;
use std::fs::read_to_string;
use std::pin::Pin;
use std::sync::{LazyLock, Mutex};

use super::Scope;
use crate::parse;

use ordered_hash_map::OrderedHashMap;

// This data structure should allow file-level reloading, which we can probably use as a rough
// approximation for iterative recompliation and language server support, and since Rust is fast,
// this might just be "good enough" assuming non-insane source file sizes.
#[derive(Debug)]
pub struct Program<'a> {
    #[allow(clippy::box_collection)]
    pub scopes_by_file: OrderedHashMap<String, (Pin<Box<String>>, parse::Ln, Scope<'a>)>,
    pub env: OrderedHashMap<String, String>,
}

pub static PROGRAM_RS: LazyLock<Mutex<Program<'static>>> = LazyLock::new(|| {
    Mutex::new(Program {
        scopes_by_file: OrderedHashMap::new(),
        env: {
            let mut env = OrderedHashMap::new();
            for (k, v) in std::env::vars() {
                env.insert(k.to_string(), v.to_string());
            }
            env.insert("ALAN_OUTPUT_LANG".to_string(), "rs".to_string());
            env
        },
    })
});
pub static PROGRAM_JS: LazyLock<Mutex<Program<'static>>> = LazyLock::new(|| {
    Mutex::new(Program {
        scopes_by_file: OrderedHashMap::new(),
        env: {
            let mut env = OrderedHashMap::new();
            for (k, v) in std::env::vars() {
                env.insert(k.to_string(), v.to_string());
            }
            env.insert("ALAN_OUTPUT_LANG".to_string(), "js".to_string());
            env
        },
    })
});

thread_local!(static TARGET_LANG_RS: Cell<bool> = const { Cell::new(true) });

impl<'a> Program<'a> {
    pub fn load(path: String) -> Result<(), Box<dyn std::error::Error>> {
        let ln_src = if path.starts_with('@') {
            match path.as_str() {
                //"@std/app" => include_str!("../std/app.ln").to_string(),
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
            None => Err(format!("Could not find a scope for file {}", path).into()),
        }
    }

    pub fn get_program<'b>() -> &'b LazyLock<Mutex<Program<'static>>> {
        if TARGET_LANG_RS.get() {
            &PROGRAM_RS
        } else {
            &PROGRAM_JS
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
