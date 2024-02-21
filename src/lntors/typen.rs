// TODO: Everything in here

use crate::program::{Program, Scope, Type};

pub fn generate(
    typen: &Type,
    scope: &Scope,
    program: &Program,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok("".to_string())
}
