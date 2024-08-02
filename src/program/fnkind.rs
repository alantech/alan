use crate::parse;

use super::CType;

#[derive(Clone, Debug)]
pub enum FnKind {
    Normal,
    Bind(String),
    Generic(Vec<(String, CType)>, Vec<parse::Statement>),
    BoundGeneric(Vec<(String, CType)>, String),
    Derived,
    DerivedVariadic,
    Static,
}
