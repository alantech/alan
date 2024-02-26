// TODO: Generics/Interfaces resolution

use crate::program::{Program, Scope, Type, TypeType};

pub fn generate(
    typen: &Type,
    scope: &Scope,
    program: &Program,
) -> Result<String, Box<dyn std::error::Error>> {
    match &typen.typetype {
        TypeType::Bind(s) => Ok(s.clone()),
        TypeType::Alias(a) => Ok(a.to_string()),
        TypeType::Structlike(_) => Ok(typen.typename.to_string()),
    }
}
