use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::Export;
use super::Scope;
use crate::parse;

#[derive(Clone, Debug)]
pub struct Const {
    pub name: String,
    pub typename: Option<String>,
    pub assignables: Vec<parse::WithOperators>,
}

impl Const {
    pub fn from_ast<'a>(
        mut scope: Scope<'a>,
        const_ast: &parse::ConstDeclaration,
        is_export: bool,
    ) -> Result<Scope<'a>, Box<dyn std::error::Error>> {
        let name = const_ast.variable.clone();
        if let Some(generics) = &const_ast.opttypegenerics {
            // We are going to conditionally compile this type declaration. If the we get true, we
            // continue, if we get false, we don't compile and return a Fail type that isn't added
            // to the scope to cause compilation to crash *if* something tries to use this, and if
            // we don't get a boolean at all or we get multiple inner values in the generic call,
            // we bail out immediately because of a syntax error.
            let generic_call = withtypeoperatorslist_to_ctype(&generics.typecalllist, &scope)?;
            match generic_call {
                CType::Bool(b) => match b {
                    false => return Ok(scope),
                    true => { /* Do nothing */ }
                },
                CType::Type(_, c) => match *c {
                    CType::Bool(b) => match b {
                        false => return Ok(scope),
                        true => { /* Do nothing */ }
                    },
                    _ => {
                        return Err(format!(
                        "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                        name,
                        generics.to_string()
                    )
                        .into())
                    }
                },
                _ => {
                    return Err(format!(
                    "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                    name,
                    generics.to_string()
                )
                    .into())
                }
            }
        }
        let typename = const_ast
            .typedec
            .as_ref()
            .map(|td| td.fulltypename.to_string());
        let assignables = const_ast.assignables.clone();
        let c = Const {
            name,
            typename,
            assignables,
        };
        if is_export {
            scope.exports.insert(c.name.clone(), Export::Const);
        }
        scope.consts.insert(c.name.clone(), c);
        Ok(scope)
    }
}
