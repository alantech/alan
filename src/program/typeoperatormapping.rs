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
    pub fn from_ast(
        scope: &mut Scope,
        typeoperatormapping_ast: &parse::TypeOperatorMapping,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        if is_export {
            scope.exports.insert(name.clone(), Export::TypeOpMap);
        }
        scope.typeoperatormappings.insert(name, opmap);
        Ok(())
    }
}
