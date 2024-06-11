use crate::parse;

#[derive(Clone, Debug)]
pub enum FnKind {
    Normal(Vec<parse::Statement>),
    Bind(String),
    Derived,
    DerivedVariadic,
}
