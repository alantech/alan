use std::cell::{Cell, RefCell};
use std::fs::read_to_string;
use std::pin::Pin;
use std::time::SystemTime;

use super::Scope;
use crate::parse;

use ordered_hash_map::OrderedHashMap;

/// A parsed and semantically-loaded source file.
#[derive(Debug)]
pub struct ParsedFile<'a> {
    pub src: Pin<Box<String>>,
    pub ast: parse::Ln,
    pub scope: Scope<'a>,
    pub path: String,
    pub mtime: Option<SystemTime>,
}

// This data structure should allow file-level reloading, which we can probably use as a rough
// approximation for iterative recompliation and language server support, and since Rust is fast,
// this might just be "good enough" assuming non-insane source file sizes.
#[derive(Debug, Default)]
pub struct Program<'a> {
    #[allow(clippy::box_collection)]
    pub scopes_by_file: OrderedHashMap<String, ParsedFile<'a>>,
    /// Deprecated: compile-time env is stored in [`COMPILE_ENV`]. Kept for API compatibility.
    pub env: OrderedHashMap<String, String>,
}

thread_local! {
    /// Compile-time environment (`Env{"KEY"}` in Alan). Separate from [`Program`] so nested
    /// `get_program`/`return_program` calls do not observe an empty env.
    static COMPILE_ENV: RefCell<Option<OrderedHashMap<String, String>>> = const { RefCell::new(None) };
}

thread_local!(pub static PROGRAM_RS: Cell<Program<'static>> = Cell::new(Program::default()));
thread_local!(pub static PROGRAM_JS: Cell<Program<'static>> = Cell::new(Program::default()));

thread_local!(static TARGET_LANG_RS: Cell<bool> = const { Cell::new(true) });

/// RAII guard that ensures the program is returned to thread-local storage on drop.
/// Use via `let guard = Program::get_program_guard()` instead of `Program::get_program()`
/// to guarantee the program is returned even if the function returns early or panics.
pub struct ProgramGuard {
    program: Option<Program<'static>>,
}

impl ProgramGuard {
    pub fn new(program: Program<'static>) -> Self {
        ProgramGuard {
            program: Some(program),
        }
    }

    /// Access the inner program as a reference.
    pub fn get_ref(&self) -> &Program<'static> {
        self.program.as_ref().expect("program already returned")
    }

    /// Access the inner program as a mutable reference.
    pub fn get_mut(&mut self) -> &mut Program<'static> {
        self.program.as_mut().expect("program already returned")
    }
}

impl Drop for ProgramGuard {
    fn drop(&mut self) {
        if let Some(p) = self.program.take() {
            Program::return_program(p);
        }
    }
}

fn platform_env() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        "what".to_string()
    }
}

fn arch_env() -> String {
    if cfg!(target_arch = "x86_64") {
        "x86_64".to_string()
    } else if cfg!(target_arch = "x86") {
        "x86_32".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "arm_64".to_string()
    } else if cfg!(target_arch = "arm") {
        "arm_32".to_string()
    } else {
        "what".to_string()
    }
}

impl<'a> Program<'a> {
    fn ensure_compile_env() {
        COMPILE_ENV.with(|cell| {
            if cell.borrow().is_some() {
                return;
            }
            let mut env = OrderedHashMap::new();
            if !cfg!(target_family = "wasm") {
                for (k, v) in std::env::vars() {
                    env.insert(k, v);
                }
            }
            if TARGET_LANG_RS.get() {
                env.insert("ALAN_OUTPUT_LANG".to_string(), "rs".to_string());
                env.insert("ALAN_PLATFORM".to_string(), platform_env());
                env.insert("ALAN_ARCH".to_string(), arch_env());
            } else {
                if cfg!(target_family = "wasm") {
                    env.insert("ALAN_TARGET".to_string(), "release".to_string());
                }
                env.insert("ALAN_OUTPUT_LANG".to_string(), "js".to_string());
                env.insert("ALAN_PLATFORM".to_string(), "browser".to_string());
                env.insert("ALAN_ARCH".to_string(), "what".to_string());
            }
            if !env.contains_key("ALAN_TARGET") {
                env.insert("ALAN_TARGET".to_string(), "release".to_string());
            }
            *cell.borrow_mut() = Some(env);
        });
    }

    /// Set a compile-time environment variable (visible to `Env{"KEY"}` in Alan source).
    pub fn set_compile_env(key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        Self::ensure_compile_env();
        COMPILE_ENV.with(|cell| {
            cell.borrow_mut()
                .as_mut()
                .expect("compile env initialized")
                .insert(key.clone(), value.into());
        });
        if key == "ALAN_TARGET" {
            Scope::clear_root_scope_cache();
        }
    }

    pub(crate) fn compile_env_get(key: &str) -> Option<String> {
        Self::ensure_compile_env();
        COMPILE_ENV.with(|cell| {
            cell.borrow()
                .as_ref()
                .expect("compile env initialized")
                .get(key)
                .cloned()
        })
    }

    pub(crate) fn compile_env_contains(key: &str) -> bool {
        Self::ensure_compile_env();
        COMPILE_ENV.with(|cell| {
            cell.borrow()
                .as_ref()
                .expect("compile env initialized")
                .contains_key(key)
        })
    }

    pub fn load(path: String) -> Result<(), Box<dyn std::error::Error>> {
        {
            let program = Program::get_program_guard();
            if program.get_ref().scopes_by_file.contains_key(&path) {
                return Ok(());
            }
        }
        let (ln_src, mtime) = if path.starts_with('@') {
            let src = match path.as_str() {
                "@std/fs" => include_str!("../std/fs.ln").to_string(),
                "@std/seq" => include_str!("../std/seq.ln").to_string(),
                "@std/window" => include_str!("../std/window.ln").to_string(),
                _ => {
                    return Err(format!("Unknown standard library named {}", &path).into());
                }
            };
            (src, None)
        } else {
            let metadata = std::fs::metadata(&path)?;
            let mtime = metadata.modified().ok();
            (read_to_string(&path)?, mtime)
        };
        Scope::from_src(&path, ln_src, mtime)
    }

    /// Returns the modification time recorded when the file was last parsed, if any.
    pub fn file_mtime(&self, path: &str) -> Option<SystemTime> {
        self.scopes_by_file.get(path).and_then(|f| f.mtime)
    }

    /// Returns whether the file on disk has changed since it was last parsed.
    pub fn file_needs_reparse(&self, path: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let Some(parsed) = self.scopes_by_file.get(path) else {
            return Ok(true);
        };
        if path.starts_with('@') {
            return Ok(false);
        }
        let current_mtime = std::fs::metadata(path)?.modified().ok();
        Ok(current_mtime != parsed.mtime)
    }

    pub fn scope_by_file(&self, path: &str) -> Result<&Scope<'a>, Box<dyn std::error::Error>> {
        match self.scopes_by_file.get(path) {
            Some(parsed) => Ok(&parsed.scope),
            None => Err(format!("Could not find a scope for file {path}").into()),
        }
    }

    pub fn get_program() -> Program<'static> {
        Self::ensure_compile_env();
        let mut program = if TARGET_LANG_RS.get() {
            PROGRAM_RS.take()
        } else {
            PROGRAM_JS.take()
        };
        program.env = COMPILE_ENV.with(|cell| {
            cell.borrow()
                .as_ref()
                .expect("compile env initialized")
                .clone()
        });
        program
    }

    /// RAII-safe alternative to `get_program()`. The program is automatically returned
    /// when the guard is dropped, even on early return or panic.
    pub fn get_program_guard() -> ProgramGuard {
        ProgramGuard::new(Self::get_program())
    }

    pub fn return_program(p: Program<'static>) {
        // COMPILE_ENV is the source of truth for compile-time env; do not mirror p.env back or
        // nested get/return pairs can clobber values set via set_compile_env.
        if TARGET_LANG_RS.get() {
            PROGRAM_RS.set(p);
        } else {
            PROGRAM_JS.set(p);
        }
    }

    pub fn set_target_lang_js() {
        if !TARGET_LANG_RS.get() {
            return;
        }
        TARGET_LANG_RS.set(false);
        COMPILE_ENV.with(|cell| *cell.borrow_mut() = None);
        Scope::clear_root_scope_cache();
    }

    pub fn set_target_lang_rs() {
        if TARGET_LANG_RS.get() {
            return;
        }
        TARGET_LANG_RS.set(true);
        COMPILE_ENV.with(|cell| *cell.borrow_mut() = None);
        Scope::clear_root_scope_cache();
    }

    pub fn is_target_lang_rs() -> bool {
        TARGET_LANG_RS.get()
    }
}
