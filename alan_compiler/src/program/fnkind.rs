use std::sync::Arc;

use crate::parse;

use super::CType;

#[derive(Clone, Debug)]
pub enum FnKind {
    Normal,
    Bind(String),
    Generic(Vec<(String, Arc<CType>)>, Vec<parse::Statement>),
    BoundGeneric(Vec<(String, Arc<CType>)>, String),
    Derived,
    DerivedVariadic,
    Static,
    External(Arc<CType>),
    ExternalBind(String, Arc<CType>),
    ExternalGeneric(Vec<(String, Arc<CType>)>, String, Arc<CType>),
}
