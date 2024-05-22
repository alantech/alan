// TODO: Generics/Interfaces resolution
use ordered_hash_map::OrderedHashMap;

use crate::program::{Program, Scope, Type, TypeType};

pub fn generate(
    typen: &Type,
    _scope: &Scope,
    _program: &Program,
    out: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match &typen.typetype {
        // The first value is the identifier, and the second is the generated source. For the
        // `Bind` and `Alias` types, these already exist in the Rust environment (or should exist,
        // assuming no bugs in the standard library) so they do not alter the generated source
        // output, while the `Structlike` type requires a new struct to be created and inserted
        // into the source definition, potentially inserting inner types as needed
        TypeType::Bind(s) => Ok((s.clone(), out)),
        // TODO: The "alias" type is just the normal type assignment path, now, no distinction so
        // this needs to be more capable going forward
        TypeType::Create(a) => Ok((a.to_string(), out)),
    }
}
