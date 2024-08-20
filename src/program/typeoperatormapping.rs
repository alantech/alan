use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::Export;
use super::Scope;
use crate::parse;

#[derive(Clone, Debug)]
pub enum TypeOperatorMapping {
    Prefix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
    Infix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
    Postfix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
}

impl TypeOperatorMapping {
    pub fn from_ast<'a>(
        mut scope: Scope<'a>,
        typeoperatormapping_ast: &parse::TypeOperatorMapping,
        is_export: bool,
    ) -> Result<Scope<'a>, Box<dyn std::error::Error>> {
        let opmap = match typeoperatormapping_ast.fix {
            parse::Fix::Prefix(_) => TypeOperatorMapping::Prefix {
                level: typeoperatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: typeoperatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: typeoperatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
            parse::Fix::Infix(_) => TypeOperatorMapping::Infix {
                level: typeoperatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: typeoperatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: typeoperatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
            parse::Fix::Postfix(_) => TypeOperatorMapping::Postfix {
                level: typeoperatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: typeoperatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: typeoperatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
        };
        let name = match &opmap {
            TypeOperatorMapping::Prefix { operatorname, .. } => operatorname.clone(),
            TypeOperatorMapping::Infix { operatorname, .. } => operatorname.clone(),
            TypeOperatorMapping::Postfix { operatorname, .. } => operatorname.clone(),
        };
        if let Some(generics) = &typeoperatormapping_ast.opttypegenerics {
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
        if is_export {
            scope.exports.insert(name.clone(), Export::TypeOpMap);
        }
        scope.typeoperatormappings.insert(name, opmap);
        Ok(scope)
    }
}
