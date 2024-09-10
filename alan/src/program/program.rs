use std::fs::read_to_string;
use std::pin::Pin;

use super::Scope;
use crate::parse;

use ordered_hash_map::OrderedHashMap;

// This data structure should allow file-level reloading, which we can probably use as a rough
// approximation for iterative recompliation and language server support, and since Rust is fast,
// this might just be "good enough" assuming non-insane source file sizes.
#[derive(Debug)]
pub struct Program<'a> {
    pub entry_file: String,
    #[allow(clippy::box_collection)]
    pub scopes_by_file: OrderedHashMap<String, (Pin<Box<String>>, parse::Ln, Scope<'a>)>,
}

impl<'a> Program<'a> {
    pub fn new(entry_file: String) -> Result<Program<'a>, Box<dyn std::error::Error>> {
        let mut p = Program {
            entry_file: entry_file.clone(),
            scopes_by_file: OrderedHashMap::new(),
        };
        // Load the entry file
        match p.load(entry_file) {
            Ok(p) => p,
            Err(e) => {
                // Somehow, trying to print this error can crash Rust!? Really not good.
                // Will need to figure out how to make these errors clearer to users.
                return Err(format!("{}", e).into());
            }
        };
        Ok(p)
    }

    pub fn load(&mut self, path: String) -> Result<(), Box<dyn std::error::Error>> {
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
        Scope::from_src(self, &path, ln_src)
    }
}
