use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use super::program::{CType, Function, Program, Scope};

mod common;
pub use common::*;

/// Identifies which CType variant holds a native dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DepType {
    Rust,
    Nodejs,
}

/// Shared helper for registering native dependencies.
/// Handles both `CType::Type(_, inner)` and direct `CType::Rust`/`CType::Nodejs` forms.
pub fn register_dependency(
    dep_type: DepType,
    d: &CType,
    deps: &mut OrderedHashMap<String, String>,
) {
    let variant_name = match dep_type {
        DepType::Rust => "Rust",
        DepType::Nodejs => "Nodejs",
    };
    let inner = match d {
        CType::Type(_, t) => &**t,
        _ => d,
    };
    let dep_match = match inner {
        CType::Rust(d) if dep_type == DepType::Rust => d,
        CType::Nodejs(d) if dep_type == DepType::Nodejs => d,
        _ => {
            CType::fail(&format!(
                "Native imports compiled to {} *must* be declared {}{{D}} dependencies: {:?}",
                match dep_type {
                    DepType::Rust => "Rust",
                    DepType::Nodejs => "Javascript",
                },
                variant_name,
                d,
            ));
        }
    };
    let (name, version) = match &**dep_match {
        CType::Dependency(n, v) => (
            match &**n {
                CType::TString(s) => s.clone(),
                _ => CType::fail("Dependency names must be strings"),
            },
            match &**v {
                CType::TString(s) => s.clone(),
                _ => CType::fail("Dependency versions must be strings"),
            },
        ),
        _ => CType::fail(&format!(
            "{} dependencies must be declared with the dependency syntax",
            match dep_type {
                DepType::Rust => "Rust",
                DepType::Nodejs => "Node.js",
            }
        )),
    };
    deps.insert(name, version);
}

/// Shared bootstrap: load program, validate `main`, run backend-specific work
/// while the program is borrowed, then return it.
pub fn bootstrap<T>(
    entry_file: &str,
    work: impl FnOnce(&[Arc<Function>], &Scope) -> Result<T, Box<dyn std::error::Error>>,
) -> Result<T, Box<dyn std::error::Error>> {
    Program::load(entry_file.to_string())?;
    let program = Program::get_program();
    let scope = program.scope_by_file(entry_file)?;
    match scope.exports.get("main") {
        Some(_) => {}
        None => {
            return Err(
                "Entry file has no `main` function exported. This is not yet supported.".into(),
            );
        }
    };
    let func = match scope.functions.get("main") {
        Some(f) => f,
        None => {
            return Err(
                "An export has been found without a definition. This should be impossible.".into(),
            );
        }
    };
    assert_eq!(func.len(), 1);
    assert_eq!(func[0].args().len(), 0);
    let result = work(func, scope)?;
    Program::return_program(program);
    Ok(result)
}

/// Backend-specific operations needed during codegen. The shared codegen helpers
/// and traversal call into this trait to produce backend-specific output.
pub trait Backend {
    /// Generate a function with the given name, emitting its body and all
    /// referenced callees recursively. Returns the updated `out` and `deps` maps.
    fn generate_function(
        name: String,
        function: &Function,
        scope: &Scope,
        out: OrderedHashMap<String, String>,
        deps: OrderedHashMap<String, String>,
    ) -> Result<
        (OrderedHashMap<String, String>, OrderedHashMap<String, String>),
        Box<dyn std::error::Error>,
    >;

    /// Called from the shared function-value resolution when the resolved
    /// function is a Normal/External/etc. (non-bind). The backend should handle
    /// name construction, generation, and any post-generation wrapping.
    fn render_function_value(
        fun: &Arc<Function>,
        out: OrderedHashMap<String, String>,
        deps: OrderedHashMap<String, String>,
    ) -> Result<
        (String, OrderedHashMap<String, String>, OrderedHashMap<String, String>),
        Box<dyn std::error::Error>,
    >;

    /// Called when the resolved function is a bind (Bind/BoundGeneric/ExternalBind/etc.).
    fn render_bind_value(
        fun: &Arc<Function>,
        out: OrderedHashMap<String, String>,
        deps: OrderedHashMap<String, String>,
    ) -> Result<
        (String, OrderedHashMap<String, String>, OrderedHashMap<String, String>),
        Box<dyn std::error::Error>,
    >;
}
