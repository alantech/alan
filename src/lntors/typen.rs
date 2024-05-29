// TODO: Generics/Interfaces resolution
use ordered_hash_map::OrderedHashMap;

use crate::program::{CType, Program, Scope};

fn ctype_to_rtype(
    ctype: &CType,
    scope: &Scope,
    program: &Program,
    in_function_type: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    match ctype {
        CType::Type(name, _) => Ok(name.clone()), // TODO: Do we need to handle this recursively,
        // or will the syntax ordering save us here?
        CType::Generic(name, ..) => Ok(name.clone()),
        CType::Bound(name, _) => Ok(name.clone()),
        CType::BoundGeneric(name, ..) => Ok(name.clone()),
        CType::IntrinsicGeneric(name, _) => Ok(name.clone()), // How would this even be reached?
        CType::Int(i) => Ok(i.to_string()),
        CType::Float(f) => Ok(f.to_string()),
        CType::Bool(b) => Ok(b.to_string()),
        CType::TString(s) => Ok(s.clone()),
        CType::Group(g) => Ok(format!(
            "({})",
            ctype_to_rtype(g, scope, program, in_function_type)?
        )),
        CType::Function(i, o) => Ok(format!(
            "fn({}) -> {}",
            ctype_to_rtype(i, scope, program, true)?,
            ctype_to_rtype(o, scope, program, true)?
        )),
        CType::Tuple(ts) => {
            let mut out = Vec::new();
            for t in ts {
                out.push(ctype_to_rtype(t, scope, program, in_function_type)?);
            }
            Ok(format!("({})", out.join(", ")))
        }
        CType::Field(k, v) => {
            if in_function_type {
                Ok(ctype_to_rtype(v, scope, program, in_function_type)?)
            } else {
                Ok(format!(
                    "{}: {}",
                    k,
                    ctype_to_rtype(v, scope, program, in_function_type)?
                ))
            }
        }
        CType::Either(_ts) => Err("TODO: Implement either-to-enum logic".into()),
        CType::Buffer(t, s) => Ok(format!(
            "[{};{}]",
            ctype_to_rtype(t, scope, program, in_function_type)?,
            s
        )),
        CType::Array(t) => Ok(format!(
            "Vec<{}>",
            ctype_to_rtype(t, scope, program, in_function_type)?
        )),
    }
}

pub fn generate(
    typen: &CType,
    scope: &Scope,
    program: &Program,
    mut out: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match &typen {
        // The first value is the identifier, and the second is the generated source. For the
        // `Bind` and `Alias` types, these already exist in the Rust environment (or should exist,
        // assuming no bugs in the standard library) so they do not alter the generated source
        // output, while the `Structlike` type requires a new struct to be created and inserted
        // into the source definition, potentially inserting inner types as needed
        CType::Bound(_name, rtype) => Ok((rtype.clone(), out)),
        CType::Type(name, ctype) => {
            out.insert(
                name.clone(),
                format!(
                    "type {} = {};\n",
                    name.clone(),
                    ctype_to_rtype(ctype, scope, program, false)?
                ),
            );
            Ok((name.clone(), out))
        }
        _ => Ok(("".to_string(), out)), // Ignore all other types at the top-level for now?
    }
}
