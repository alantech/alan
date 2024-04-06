// TODO: Generics/Interfaces resolution
use ordered_hash_map::OrderedHashMap;

use crate::program::{Program, Scope, Type, TypeType};

pub fn generate(
    typen: &Type,
    scope: &Scope,
    program: &Program,
    mut out: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match &typen.typetype {
        // The first value is the identifier, and the second is the generated source. For the
        // `Bind` and `Alias` types, these already exist in the Rust environment (or should exist,
        // assuming no bugs in the standard library) so they do not alter the generated source
        // output, while the `Structlike` type requires a new struct to be created and inserted
        // into the source definition, potentially inserting inner types as needed
        TypeType::Bind(s) => Ok((s.clone(), out)),
        TypeType::Alias(a) => Ok((a.to_string(), out)),
        TypeType::Structlike(s) => {
            let mut typestring = format!(r#"#[derive(Debug)]
struct {} {{
"#, typen.typename.to_string());
            for typeline in &s.typelist {
                let typename = typeline.fulltypename.to_string();
                let (subtypen, subtypescope) = match program.resolve_type(scope, &typename) {
                    None => Err(format!("Type {} not found", typename)),
                    Some((t, s)) => Ok((t, s)),
                }?;
                let res = generate(subtypen, subtypescope, program, out)?;
                let subtypename = res.0;
                out = res.1;
                typestring = format!(r#"{}
    pub {}: {},"#, typestring, typeline.variable, subtypename);
            }
            typestring = format!(r#"{}
}}"#, typestring);
            out.insert(typen.typename.to_string(), typestring.to_string());
            Ok((typen.typename.to_string(), out))
        },
    }
}
