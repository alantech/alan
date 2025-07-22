use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::Export;
use super::Scope;
use crate::parse;

#[derive(Clone, Debug)]
pub enum OperatorMapping {
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

impl OperatorMapping {
    pub fn from_ast<'a>(
        mut scope: Scope<'a>,
        operatormapping_ast: &parse::OperatorMapping,
        is_export: bool,
    ) -> Result<Scope<'a>, Box<dyn std::error::Error>> {
        let opmap = match operatormapping_ast.fix {
            parse::Fix::Prefix(_) => OperatorMapping::Prefix {
                level: operatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: operatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: operatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
            parse::Fix::Infix(_) => OperatorMapping::Infix {
                level: operatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: operatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: operatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
            parse::Fix::Postfix(_) => OperatorMapping::Postfix {
                level: operatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: operatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: operatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
        };
        let name = match &opmap {
            OperatorMapping::Prefix { operatorname, .. } => operatorname.clone(),
            OperatorMapping::Infix { operatorname, .. } => operatorname.clone(),
            OperatorMapping::Postfix { operatorname, .. } => operatorname.clone(),
        };
        if let Some(generics) = &operatormapping_ast.opttypegenerics {
            // We are going to conditionally compile this type declaration. If the we get true, we
            // continue, if we get false, we don't compile and return a Fail type that isn't added
            // to the scope to cause compilation to crash *if* something tries to use this, and if
            // we don't get a boolean at all or we get multiple inner values in the generic call,
            // we bail out immediately because of a syntax error.
            let generic_call = withtypeoperatorslist_to_ctype(&generics.typecalllist, &scope)?;
            match &*generic_call {
                CType::Bool(b) => match b {
                    false => return Ok(scope),
                    true => { /* Do nothing */ }
                },
                CType::Type(_, c) => match &**c {
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
            scope.exports.insert(
                match &opmap {
                    OperatorMapping::Prefix { .. } => format!("prefix{name}"),
                    OperatorMapping::Infix { .. } => format!("infix{name}"),
                    OperatorMapping::Postfix { .. } => format!("postfix{name}"),
                },
                Export::OpMap,
            );
        }
        scope.operatormappings.insert(
            match &opmap {
                OperatorMapping::Prefix { .. } => format!("prefix{name}"),
                OperatorMapping::Infix { .. } => format!("infix{name}"),
                OperatorMapping::Postfix { .. } => format!("postfix{name}"),
            },
            opmap,
        );
        Ok(scope)
    }
}
