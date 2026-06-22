use std::sync::Arc;

use crate::parse;

use super::CType;

/// Kinds of compiler-provided (cfn) functions. Each variant tells the codegen
/// exactly what to emit, with no name-based matching required.
#[derive(Clone, Debug)]
pub enum CfnKind {
    Clone,
    /// Block-level / value `if`/`else` conditional. Realized as `if{T}(bool, () -> T, () -> T) -> T`
    /// where `T = ()` is the void/side-effect form. Codegen renders native control flow.
    IfElse,
}

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
    /// Compiler-provided generic function (from `cfn` syntax). Generic parameters
    /// are listed first; the `CfnKind` tells codegen what to emit.
    Cfn(CfnKind, Vec<(String, Arc<CType>)>),
    /// Realized instance of a compiler-provided function. The `CfnKind` survives
    /// realization so codegen can match on it directly.
    CfnRealized(CfnKind),
}
