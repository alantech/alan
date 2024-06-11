#[derive(Clone, Debug)]
pub struct Const {
    pub name: String,
    pub typename: Option<String>,
    pub assignables: Vec<parse::WithOperators>,
}

impl Const {
    fn from_ast(
        scope: &mut Scope,
        const_ast: &parse::ConstDeclaration,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = const_ast.variable.clone();
        let typename = match &const_ast.typedec {
            Some(typedec) => Some(typedec.fulltypename.to_string()),
            None => None,
        };
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
        Ok(())
    }
}
