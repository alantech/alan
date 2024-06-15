use super::Program;
use super::Scope;
use crate::parse;

#[derive(Clone, Debug)]
pub enum ImportType {
    // For both of these, the first string is the original name, and the second is the rename.
    // To simplify later logic, there's always a rename even if the user didn't rename anything, it
    // will just make a copy of the module or field name in those cases
    Standard(String, String),
    Fields(Vec<(String, String)>),
}

#[derive(Clone, Debug)]
pub struct Import {
    pub source_scope_name: String,
    pub import_type: ImportType,
}

impl Import {
    pub fn from_ast(
        program: &mut Program,
        path: String,
        scope: &mut Scope,
        import_ast: &parse::ImportStatement,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &import_ast {
            parse::ImportStatement::Standard(s) => {
                // First, get the path for the code
                let ln_file = s.dependency.resolve(path)?;
                let exists = match &program.scopes_by_file.get(&ln_file) {
                    Some(_) => true,
                    None => false,
                };
                if !exists {
                    // Need to load this file into the program first
                    program.load(ln_file.clone())?;
                }
                let import_name = if let Some(rename) = &s.renamed {
                    rename.varop.to_string()
                } else {
                    ln_file.clone()
                };
                let i = Import {
                    source_scope_name: ln_file.clone(),
                    import_type: ImportType::Standard(ln_file.clone(), import_name),
                };
                scope.imports.insert(ln_file, i);
                Ok(())
            }
            parse::ImportStatement::From(f) => {
                let ln_file = f.dependency.resolve(path)?;
                let exists = match &program.scopes_by_file.get(&ln_file) {
                    Some(_) => true,
                    None => false,
                };
                if !exists {
                    // Need to load this file into the program first
                    program.load(ln_file.clone())?;
                }
                let field_vec = f
                    .varlist
                    .iter()
                    .map(|v| {
                        if let Some(rename) = &v.optrenamed {
                            (v.varop.to_string(), rename.varop.to_string())
                        } else {
                            (v.varop.to_string(), v.varop.to_string())
                        }
                    })
                    .collect::<Vec<(String, String)>>();
                let i = Import {
                    source_scope_name: ln_file.clone(),
                    import_type: ImportType::Fields(field_vec),
                };
                scope.imports.insert(ln_file, i);
                Ok(())
            }
        }
    }
}
