use std::sync::{Arc, LazyLock, Mutex, OnceLock, Weak};

use weak_table::PtrWeakKeyHashMap;

mod compatibility;
mod construction;
mod display;
mod generics;
mod operators;

use super::ArgKind;
use super::Export;
use super::FnKind;
use super::Function;
use super::Microstatement;
use super::Program;
use super::Scope;
use super::TypeOperatorMapping;
use crate::parse;

/// For a native bind argument, returns its rendered representation and whether it
/// was "trimmed" — i.e. a compile-time literal (`Int`/`Float`/`Bool`/`TString`)
/// inlined directly into the generated code rather than passed as a runtime
/// parameter. Non-literal arguments render as their parameter name (which keeps
/// them structurally substitutable downstream).
fn native_arg_repr(arg: &(String, ArgKind, Arc<CType>)) -> (String, bool) {
    match &*arg.2 {
        CType::Int(i) => (format!("{i}"), true),
        CType::Float(f) => (format!("{f}"), true),
        CType::Bool(b) => ((if *b { "true" } else { "false" }).to_string(), true),
        CType::TString(s) => (format!("\"{}\"", s.replace('"', "\\\"")), true),
        _ => (arg.0.clone(), false),
    }
}

/// Lower the arguments of a native bind into structural `Value` microstatements
/// for a `NativeCall`. A compile-time literal argument is inlined as its literal
/// representation (and flips `trimmed` so the caller drops it from the wrapper's
/// signature); any other argument is referenced by its parameter name. Shared by
/// every native bind shape (function/method/property/operator/cast) so the
/// literal handling lives in exactly one place.
fn native_call_args(
    args: &[(String, ArgKind, Arc<CType>)],
    trimmed: &mut bool,
) -> Vec<Microstatement> {
    args.iter()
        .map(|arg| {
            let (repr, was_trimmed) = native_arg_repr(arg);
            if was_trimmed {
                *trimmed = true;
            }
            Microstatement::Value {
                typen: arg.2.clone(),
                representation: repr,
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub enum CType {
    Void,
    DerivedVoid(Vec<Arc<CType>>), // void with parent types for Exclude{...} constructors
    Infer(String, String),        // TODO: Switch to an Interface here once they exist
    Type(String, Arc<CType>),
    Generic(String, Vec<String>, Arc<CType>),
    Binds(Arc<CType>, Vec<Arc<CType>>),
    Shared(Arc<CType>),
    Promise(Arc<CType>),
    IntrinsicGeneric(String, usize),
    IntCast(Arc<CType>),
    Int(i128),
    FloatCast(Arc<CType>),
    Float(f64),
    BoolCast(Arc<CType>),
    Bool(bool),
    StringCast(Arc<CType>),
    TString(String),
    Group(Arc<CType>),
    Unwrap(Arc<CType>),
    Function(Arc<CType>, Arc<CType>),
    Call(Arc<CType>, Arc<CType>),
    Infix(Arc<CType>),
    Prefix(Arc<CType>),
    Method(Arc<CType>),
    Property(Arc<CType>),
    Cast(Arc<CType>),
    Own(Arc<CType>),
    Deref(Arc<CType>),
    Mut(Arc<CType>),
    Dependency(Arc<CType>, Arc<CType>),
    Rust(Arc<CType>),
    Nodejs(Arc<CType>),
    From(Arc<CType>),
    Import(Arc<CType>, Arc<CType>),
    Tuple(Vec<Arc<CType>>, Vec<Arc<CType>>),
    Field(String, Arc<CType>),
    Either(Vec<Arc<CType>>, Vec<Arc<CType>>),
    Prop(Arc<CType>, Arc<CType>),
    Exclude(Arc<CType>, Arc<CType>),
    AnyOf(Vec<Arc<CType>>),
    Buffer(Arc<CType>, Arc<CType>),
    Array(Arc<CType>),
    Fail(String),
    Add(Vec<Arc<CType>>),
    Sub(Vec<Arc<CType>>),
    Mul(Vec<Arc<CType>>),
    Div(Vec<Arc<CType>>),
    Mod(Vec<Arc<CType>>),
    Pow(Vec<Arc<CType>>),
    Min(Vec<Arc<CType>>),
    Max(Vec<Arc<CType>>),
    Neg(Arc<CType>),
    Len(Arc<CType>),

    Size(Arc<CType>),
    FileStr(Arc<CType>),
    Concat(Arc<CType>, Arc<CType>),
    Env(Vec<Arc<CType>>),
    EnvExists(Arc<CType>),
    TIf(Arc<CType>, Vec<Arc<CType>>),
    And(Vec<Arc<CType>>),
    Or(Vec<Arc<CType>>),
    Xor(Vec<Arc<CType>>),
    Not(Arc<CType>),
    Nand(Vec<Arc<CType>>),
    Nor(Vec<Arc<CType>>),
    Xnor(Vec<Arc<CType>>),
    TEq(Vec<Arc<CType>>),
    Neq(Vec<Arc<CType>>),
    Lt(Vec<Arc<CType>>),
    Lte(Vec<Arc<CType>>),
    Gt(Vec<Arc<CType>>),
    Gte(Vec<Arc<CType>>),
}

pub(super) static CLOSE_BRACE: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static CLOSE_PAREN: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static COMMA: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static FNARROW: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static FNCALL: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static DEPAT: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static IMARROW: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static OR: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static DOT: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static AND: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static OPEN_BRACKET: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static CLOSE_BRACKET: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static ADD: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static SUB: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static MUL: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static DIV: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static MOD: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static POW: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static BAND: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static BOR: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static XOR: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static NAND: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static NOR: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static XNOR: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static EQ: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static NEQ: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static LT: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static LTE: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static GT: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static GTE: OnceLock<Arc<CType>> = OnceLock::new();
pub(super) static FUNCTIONAL_STRINGS: LazyLock<Mutex<PtrWeakKeyHashMap<Weak<CType>, String>>> =
    LazyLock::new(|| Mutex::new(PtrWeakKeyHashMap::<Weak<CType>, String>::new()));
pub(super) static STRICT_STRINGS: LazyLock<Mutex<PtrWeakKeyHashMap<Weak<CType>, String>>> =
    LazyLock::new(|| Mutex::new(PtrWeakKeyHashMap::<Weak<CType>, String>::new()));
pub(super) static LOOSE_STRINGS: LazyLock<Mutex<PtrWeakKeyHashMap<Weak<CType>, String>>> =
    LazyLock::new(|| Mutex::new(PtrWeakKeyHashMap::<Weak<CType>, String>::new()));
pub(super) static CALLABLE_STRINGS: LazyLock<Mutex<PtrWeakKeyHashMap<Weak<CType>, String>>> =
    LazyLock::new(|| Mutex::new(PtrWeakKeyHashMap::<Weak<CType>, String>::new()));

// TODO: I really hoped these two would share more code. Figure out how to DRY this out later, if
// possible
pub fn withtypeoperatorslist_to_ctype(
    withtypeoperatorslist: &Vec<parse::WithTypeOperators>,
    scope: &Scope,
) -> Result<Arc<CType>, Box<dyn std::error::Error>> {
    // To properly linearize the operations here, we need to scan through all of the operators,
    // determine which is the highest precedence, whether it is infix or prefix (or maybe postfix
    // in the future?) and then process them and whichever of the baseassignables surrounding them
    // are associated, then put those results in the same "slot" as before and check again. Because
    // users can define these operators, that makes it theoretically possible for the same operator
    // to be used in both an infix or prefix manner, or with different precedence levels, depending
    // on the types of the data involved, which makes things *really* complicated here. TODO:
    // Actually implement that complexity, for now, just pretend operators have only one binding.
    let mut queue = withtypeoperatorslist.clone();
    let mut out_ctype = None;
    while !queue.is_empty() {
        let mut largest_operator_level: i8 = -1;
        let mut largest_operator_index: i64 = -1;
        for (i, assignable_or_operator) in queue.iter().enumerate() {
            if let parse::WithTypeOperators::Operators(o) = assignable_or_operator {
                let operatorname = &o.op;
                let operator = match scope.resolve_typeoperator(operatorname) {
                    Some(o) => Ok(o),
                    None => Err(format!("Operator {operatorname} not found")),
                }?;
                let level = match &operator {
                    TypeOperatorMapping::Prefix { level, .. } => level,
                    TypeOperatorMapping::Infix { level, .. } => level,
                    TypeOperatorMapping::Postfix { level, .. } => level,
                };
                if level > &largest_operator_level {
                    largest_operator_level = *level;
                    largest_operator_index = i as i64;
                }
            }
        }
        if largest_operator_index > -1 {
            // We have at least one operator, and this is the one to dig into
            let operatorname = match &queue[largest_operator_index as usize] {
                parse::WithTypeOperators::Operators(o) => &o.op,
                _ => unreachable!(),
            };
            let operator = match scope.resolve_typeoperator(operatorname) {
                Some(o) => Ok(o),
                None => Err(format!("Operator {operatorname} not found")),
            }?;
            let functionname = match operator {
                TypeOperatorMapping::Prefix { functionname, .. } => functionname.clone(),
                TypeOperatorMapping::Infix { functionname, .. } => functionname.clone(),
                TypeOperatorMapping::Postfix { functionname, .. } => functionname.clone(),
            };
            let is_infix = match operator {
                TypeOperatorMapping::Prefix { .. } => false,
                TypeOperatorMapping::Postfix { .. } => false,
                TypeOperatorMapping::Infix { .. } => true,
            };
            let is_prefix = match operator {
                TypeOperatorMapping::Prefix { .. } => true,
                TypeOperatorMapping::Postfix { .. } => false,
                TypeOperatorMapping::Infix { .. } => false,
            };
            if is_infix {
                // Confirm that we have records before and after the operator and that they are
                // baseassignables.
                let first_arg = match match queue.get(largest_operator_index as usize - 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {operatorname} is an infix operator but missing a left-hand side value"
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is an infix operator but preceded by another operator {}",
                        operatorname, o.op
                    )),
                }?;
                let second_arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!("Operator {operatorname} is an infix operator but missing a right-hand side value")),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!("Operator{} is an infix operator but followed by a lower precedence operator {}", operatorname, o.op)),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `a + b` and turn it into `add(a, b)`
                let rewrite = parse::WithTypeOperators::TypeBaseList(vec![
                    parse::TypeBase::Variable(functionname),
                    parse::TypeBase::GnCall(parse::GnCall {
                        opencurly: "{".to_string(),
                        a: "".to_string(),
                        typecalllist: vec![
                            parse::WithTypeOperators::TypeBaseList(first_arg.to_vec()),
                            parse::WithTypeOperators::Operators(
                                parse::TypeOperatorsWithWhitespace {
                                    a: " ".to_string(),
                                    op: ",".to_string(),
                                    b: " ".to_string(),
                                },
                            ),
                            parse::WithTypeOperators::TypeBaseList(second_arg.to_vec()),
                        ],
                        b: "".to_string(),
                        closecurly: "}".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithTypeOperators> = queue
                    .splice(
                        (largest_operator_index as usize - 1)
                            ..(largest_operator_index as usize + 2),
                        vec![rewrite],
                    )
                    .collect();
            } else if is_prefix {
                // Confirm that we have a record after the operator and that it's a baseassignables
                let arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {operatorname} is a prefix operator but missing a right-hand side value"
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is a prefix operator but followed by another operator {}",
                        operatorname, o.op
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `#array` and turn it into `len(array)`
                let rewrite = parse::WithTypeOperators::TypeBaseList(vec![
                    parse::TypeBase::Variable(functionname),
                    parse::TypeBase::GnCall(parse::GnCall {
                        opencurly: "{".to_string(),
                        a: "".to_string(),
                        typecalllist: vec![parse::WithTypeOperators::TypeBaseList(arg.to_vec())],
                        b: "".to_string(),
                        closecurly: "}".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithTypeOperators> = queue
                    .splice(
                        (largest_operator_index as usize)..(largest_operator_index as usize + 2),
                        vec![rewrite],
                    )
                    .collect();
            } else {
                let arg = match match queue.get(largest_operator_index as usize - 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {operatorname} is a postfix operator but missing a left-hand side value"
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is a postfix operator but preceded by another operator {}",
                        operatorname, o.op
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `type?` and turn it into `Maybe{type}`
                let rewrite = parse::WithTypeOperators::TypeBaseList(vec![
                    parse::TypeBase::Variable(functionname),
                    parse::TypeBase::GnCall(parse::GnCall {
                        opencurly: "{".to_string(),
                        a: "".to_string(),
                        typecalllist: vec![parse::WithTypeOperators::TypeBaseList(arg.to_vec())],
                        b: "".to_string(),
                        closecurly: "}".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithTypeOperators> = queue
                    .splice(
                        (largest_operator_index as usize - 1)
                            ..(largest_operator_index as usize + 1),
                        vec![rewrite],
                    )
                    .collect();
            }
        } else {
            // We have no more typeoperators, there should only be one reworked typebaselist now
            if queue.len() != 1 {
                // No idea how such a wonky thing could occur. TODO: Improve error message
                return Err(format!("Invalid syntax: {withtypeoperatorslist:?}").into());
            }
            let typebaselist = match match queue.pop() {
                Some(v) => Ok(v),
                None => Err(format!(
                    "Somehow we collapsed the statement into nothing? {withtypeoperatorslist:?}"
                )),
            }? {
                parse::WithTypeOperators::TypeBaseList(b) => Ok(b),
                _ => Err(format!(
                    "Somehow we collapse the statement into a solitary operator? {withtypeoperatorslist:?}"
                )),
            }?;
            out_ctype = Some(typebaselist_to_ctype(&typebaselist, scope)?);
        }
    }
    match out_ctype {
        Some(ctype) => Ok(ctype),
        None => Err(format!("Never resolved a type from {withtypeoperatorslist:?}").into()),
    }
}

/// Strip user-facing `Type(name, inner)` alias wrappers, leaving the structural type.
fn strip_type_alias(typen: Arc<CType>) -> Arc<CType> {
    match &*typen {
        CType::Type(_, inner) => strip_type_alias(inner.clone()),
        _ => typen,
    }
}

/// True when `typen` is a multi-field record tuple (struct-shaped), after any `Type` alias is
/// stripped.
fn is_multi_field_record_tuple(typen: &CType) -> bool {
    let mut current = typen;
    loop {
        match current {
            CType::Type(_, inner) => current = inner.as_ref(),
            CType::Tuple(fields, _) => {
                return fields.len() > 1 && fields.iter().any(|f| matches!(&**f, CType::Field(..)));
            }
            _ => return false,
        }
    }
}

/// Look up a declared type name for `ty` by scanning scopes from the current position up through
/// parents (nearest scope wins).
pub(super) fn lookup_declared_type_name(ty: Arc<CType>, scope: &Scope) -> Option<String> {
    if let CType::Type(n, _) = ty.as_ref() {
        return Some(n.clone());
    }
    let structural = strip_type_alias(ty.clone().degroup());
    let callable = structural.to_callable_string();
    let mut current = Some(scope);
    while let Some(s) = current {
        for (key, ctype) in &s.types {
            if strip_type_alias(ctype.clone()).to_callable_string() != callable {
                continue;
            }
            if let CType::Type(n, _) = ctype.as_ref() {
                return Some(n.clone());
            }
            if !key.starts_with("Tuple_") && !key.starts_with("Either_") {
                return Some(key.clone());
            }
        }
        current = s.parent;
    }
    None
}

/// Look up a user-facing type name for a structural multi-field record tuple.
fn type_name_for_structural(ty: Arc<CType>, scope: &Scope) -> Option<String> {
    let structural = strip_type_alias(ty.clone().degroup());
    if !is_multi_field_record_tuple(structural.as_ref()) {
        return None;
    }
    lookup_declared_type_name(ty, scope)
}

/// When generic inference would bind a bare structural record tuple, recover the user-facing
/// `Type(name, inner)` so substitution keeps the type name without requiring call sites to pass
/// named wrappers.
fn canonicalize_inferred_generic_type(ty: Arc<CType>, scope: &Scope) -> Arc<CType> {
    if matches!(ty.as_ref(), CType::Type(..)) {
        return ty;
    }
    let structural = strip_type_alias(ty.clone().degroup());
    if !is_multi_field_record_tuple(structural.as_ref()) {
        return ty;
    }
    let Some(name) = type_name_for_structural(ty.clone(), scope) else {
        return ty;
    };
    Arc::new(CType::Type(name, structural))
}

// TODO: This similarly shares a lot of structure with baseassignablelist_to_microstatements, see
// if there is any way to DRY this up, or is it just doomed to be like this?
pub fn typebaselist_to_ctype(
    typebaselist: &[parse::TypeBase],
    scope: &Scope,
) -> Result<Arc<CType>, Box<dyn std::error::Error>> {
    let mut i = 0;
    let mut prior_value = None;
    while i < typebaselist.len() {
        let typebase = &typebaselist[i];
        let nexttypebase = typebaselist.get(i + 1);
        match typebase {
            parse::TypeBase::Constants(c) => {
                // With constants, there are a few situations where they are allowed:
                // 1) When they're used within a `GnCall` as the sole value passed in
                // 2) When they're used as the property of a type, but only in certain scenarios.
                // 2a) If it's an integer indexing into a tuple type or an either type, it returns
                // the type of that element in the tuple or either.
                // 2b) If it's a string indexing into a labeled tuple type (aka a struct), it
                // returns the type of that element in the tuple.
                // 2c) If it's a string that is specifically "input" or "output" indexing on a
                // function type, it returns the input or output type (function types could
                // internally have been a struct-like type with two fields, but they're special for
                // now)
                if let Some(next) = nexttypebase {
                    match next {
                        parse::TypeBase::Variable(_) => {
                            return Err(format!("A constant cannot be directly before a variable without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::TypeBase::GnCall(_) => {
                            return Err(format!("A constant cannot be directly before a generic function call without an operator and type name between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::TypeBase::TypeGroup(_) => {
                            return Err(format!("A constant cannot be directly before a parenthetical grouping without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::TypeBase::Constants(_) => {
                            return Err(format!("A constant cannot be directly before another constant without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                    }
                }
                if prior_value.is_none() {
                    match c {
                        parse::Constants::Bool(b) => {
                            prior_value = Some(Arc::new(CType::Bool(b.as_str() == "true")))
                        }
                        parse::Constants::Strn(s) => {
                            prior_value = Some(Arc::new(CType::TString(if s.starts_with('"') {
                                s.split("\\\"")
                                    .map(|sub| sub.replace("\"", ""))
                                    .collect::<Vec<String>>()
                                    .join("\"")
                            } else {
                                s.split("\\'")
                                    .map(|sub| sub.replace("'", ""))
                                    .collect::<Vec<String>>()
                                    .join("'")
                            })))
                        }
                        parse::Constants::Num(n) => match n {
                            parse::Number::RealNum(r) => {
                                prior_value = Some(Arc::new(CType::Float(
                                    r.replace('_', "").parse::<f64>().unwrap(), // This should never fail if the
                                                                                // parser says it's a float
                                )))
                            }
                            parse::Number::IntNum(i) => {
                                prior_value = Some(Arc::new(CType::Int(
                                    i.replace('_', "").parse::<i128>().unwrap(), // Same deal here
                                )))
                            }
                        },
                    }
                } else {
                    // There are broadly two cases where this can be reasonable: tuple-like access
                    // with integers and struct-like access with strings
                    match c {
                        parse::Constants::Bool(_) => {
                            return Err(format!("A boolean cannot follow another value without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::Constants::Strn(s) => {
                            prior_value = Some(match &*prior_value.unwrap() {
                                CType::Tuple(ts, _) => {
                                    let mut out = None;
                                    for t in ts {
                                        if let CType::Field(f, c) = &**t {
                                            if f.as_str() == s.as_str() {
                                                out = Some(c.clone());
                                            }
                                        }
                                    }
                                    match out {
                                        Some(o) => o,
                                        None => CType::fail(&format!("{ts:?} does not have a property named {s}")),
                                    }
                                }
                                CType::Function(i, o) => match s.as_str() {
                                    "input" => i.clone(),
                                    "output" => o.clone(),
                                    _ => CType::fail("Function types only have \"input\" and \"output\" properties"),
                                }
                                other => CType::fail(&format!("String properties are not allowed on {other:?}")),
                            });
                        }
                        parse::Constants::Num(n) => {
                            match n {
                                parse::Number::RealNum(_) => {
                                    return Err(format!("A floating point number cannot follow another value without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                                }
                                parse::Number::IntNum(i) => {
                                    let idx = match i.parse::<usize>() {
                                    Ok(idx) => idx,
                                    Err(_) => CType::fail("Indexing into a type must be done with positive integers"),
                                };
                                    prior_value = Some(match &*prior_value.unwrap() {
                                        CType::Tuple(ts, _) => match ts.get(idx) {
                                            Some(t) => t.clone(),
                                            None => CType::fail(&format!(
                                                "{idx} is larger than the size of {ts:?}"
                                            )),
                                        },
                                        CType::Either(ts, _) => match ts.get(idx) {
                                            Some(t) => t.clone(),
                                            None => CType::fail(&format!(
                                                "{idx} is larger than the size of {ts:?}"
                                            )),
                                        },
                                        other => CType::fail(&format!(
                                            "{other:?} cannot be indexed by an integer"
                                        )),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            parse::TypeBase::Variable(var) => {
                // Variables can be used to access sub-types in a type, or used as method-style
                // execution of a prior value. For method access, if the function takes only one
                // argument, it should still work even if the follow-on curly braces are not
                // written, so there's a little bit of extra logic added here for that situation,
                // otherwise it's handled by the GnCall path. When it's a property access, it
                // replaces the prior CType with the sub-type of the prior value.
                // For the simpler case when it's *just* a reference to a prior variable, it just
                // becomes a `Type` CType providing an alias for the named type.
                let mut args = Vec::new();
                if let Some(val) = &prior_value {
                    args.push(val.clone())
                };
                prior_value = Some(match scope.resolve_type(var) {
                    Some(t) => {
                        // TODO: Once interfaces are a thing, there needs to be a built-in
                        // interface called `Label` that we can use here to mark the first argument
                        // to `Field` as a `Label` and turn this logic into something regularized
                        // For now, we're just special-casing the `Field` built-in generic type.
                        match &*t {
                            CType::IntrinsicGeneric(p, 2) if p == "Prop" => {
                                match nexttypebase {
                                    None => {},
                                    Some(next) => match next {
                                        parse::TypeBase::GnCall(g) => {
                                            // There should be only two args, the first arg is
                                            // coerced from a variable to a string, the second arg
                                            // is treated like normal
                                            if g.typecalllist.len() != 3 {
                                                CType::fail("The Prop generic type accepts only two parameters");
                                            }
                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[0].clone()], scope)?);
                                            match g.typecalllist[0].to_string().parse::<i128>() {
                                                Ok(i) => args.push(Arc::new(CType::Int(i))),
                                                Err(_) => {
                                                    if let parse::WithTypeOperators::TypeBaseList(tbl) = &g.typecalllist[2] {
                                                        if tbl.len() > 1 {
                                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[2].clone()], scope)?);
                                                        } else {
                                                            let argstr = g.typecalllist[2].to_string();
                                                            match argstr.as_str() {
                                                                "true" => args.push(Arc::new(CType::Bool(true))),
                                                                "false" => args.push(Arc::new(CType::Bool(false))),
                                                                _ => args.push(Arc::new(CType::TString(argstr)))
                                                            }
                                                        }
                                                    } else {
                                                        CType::fail("huh?")
                                                    }
                                                }
                                            }
                                        }
                                        _ => CType::fail("Cannot follow method style syntax without an operator in between"),
                                    }
                                }
                            }
                            CType::IntrinsicGeneric(e, 2) if e == "Exclude" => {
                                match nexttypebase {
                                    None => {},
                                    Some(next) => match next {
                                        parse::TypeBase::GnCall(g) => {
                                            // Same as Prop: the second arg (the index/field to exclude)
                                            // can be a bare identifier treated as a string
                                            if g.typecalllist.len() != 3 {
                                                CType::fail("The Exclude generic type accepts only two parameters");
                                            }
                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[0].clone()], scope)?);
                                            let second_arg_str = g.typecalllist[2].to_string();
                                            match second_arg_str.parse::<i128>() {
                                                Ok(i) => args.push(Arc::new(CType::Int(i))),
                                                Err(_) => {
                                                    if let parse::WithTypeOperators::TypeBaseList(tbl) = &g.typecalllist[2] {
                                                        if tbl.len() > 1 {
                                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[2].clone()], scope)?);
                                                        } else {
                                                            match second_arg_str.as_str() {
                                                                "true" => args.push(Arc::new(CType::Bool(true))),
                                                                "false" => args.push(Arc::new(CType::Bool(false))),
                                                                _ => args.push(Arc::new(CType::TString(second_arg_str)))
                                                            }
                                                        }
                                                    } else {
                                                        CType::fail("huh?")
                                                    }
                                                }
                                            }
                                        }
                                        _ => CType::fail("Cannot follow method style syntax without an operator in between"),
                                    }
                                }
                            }
                            CType::IntrinsicGeneric(f, 2) if f == "Field" => {
                                match nexttypebase {
                                    None => {},
                                    Some(next) => match next {
                                        parse::TypeBase::GnCall(g) => {
                                            // There should be only two args, the first arg is
                                            // coerced from a variable to a string, the second arg
                                            // is treated like normal
                                            if g.typecalllist.len() != 3 {
                                                CType::fail("The Field generic type accepts only two parameters");
                                            }
                                            // Special hack to de-stringify the field label if and
                                            // only if it's trying to cast to a string here
                                            let label = g.typecalllist[0].to_string();
                                            if label.starts_with("String{") {
                                                args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[0].clone()], scope)?);
                                            } else {
                                                args.push(Arc::new(CType::TString(label)));
                                            }
                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[2].clone()], scope)?);
                                        }
                                        _ => CType::fail("Cannot follow method style syntax without an operator in between"),
                                    }
                                }
                            }
                            _ => {
                                match nexttypebase {
                                    None => {},
                                    Some(next) => match next {
                                        parse::TypeBase::GnCall(g) => {
                                            // Unfortunately ambiguous, but commas behave
                                            // differently in here, so we're gonna chunk this,
                                            // split by commas, then iterate on those chunks
                                            let mut temp_args = Vec::new();
                                            for ta in &g.typecalllist {
                                                temp_args.push(ta.clone());
                                            }
                                            let mut arg_block = Vec::new();
                                            for arg in temp_args {
                                                if let parse::WithTypeOperators::Operators(o) = &arg {
                                                    if o.op == "," {
                                                        // Process the arg block that has
                                                        // accumulated
                                                        args.push(withtypeoperatorslist_to_ctype(&arg_block, scope)?);
                                                        arg_block.clear();
                                                        continue;
                                                    }
                                                }
                                                arg_block.push(arg);
                                            }
                                            if !arg_block.is_empty() {
                                                args.push(withtypeoperatorslist_to_ctype(&arg_block, scope)?);
                                            }
                                        }
                                        _ => CType::fail("Cannot follow method style syntax without an operator in between"),
                                    }
                                }
                            }
                        }
                        // Now, we need to validate that the resolved type *is* a generic
                        // type that can be called, and that we have the correct number of
                        // arguments for it, then we can call it and return the resulting
                        // type
                        match &*t {
                            CType::Generic(_name, params, generic_type) => {
                                if params.len() != args.len() {
                                    CType::fail(&format!(
                                        "Generic type {} takes {} arguments but {} given",
                                        var,
                                        params.len(),
                                        args.len()
                                    ))
                                } else {
                                    // We use a temporary scope to resolve the
                                    // arguments to the generic function as the
                                    // specified names
                                    let mut out_type = generic_type.clone();
                                    for i in 0..params.len() {
                                        let generic_arg = Arc::new(CType::Infer(
                                            params[i].clone(),
                                            "Any".to_string(),
                                        ));
                                        out_type =
                                            out_type.swap_subtype(generic_arg, args[i].clone());
                                    }
                                    // Now we return the type we resolve within this
                                    // scope
                                    out_type
                                }
                            }
                            CType::IntrinsicGeneric(name, len) => {
                                if *len != 0 && args.len() != *len {
                                    CType::fail(&format!(
                                        "Generic type {} takes {} arguments but {} given",
                                        var,
                                        len,
                                        args.len()
                                    ))
                                } else {
                                    // TODO: Is there a better way to do this?
                                    match name.as_str() {
                                        "Binds" => CType::binds(args),
                                        "Shared" => Arc::new(CType::Shared(args[0].clone())),
                                        "Promise" => CType::promise(args[0].clone()),
                                        "Int" => CType::intcast(args[0].clone()),
                                        "Float" => CType::floatcast(args[0].clone()),
                                        "Bool" => CType::boolcast(args[0].clone()),
                                        "String" => CType::stringcast(args[0].clone()),
                                        "Group" => Arc::new(CType::Group(args[0].clone())),
                                        "Unwrap" => CType::tunwrap(args[0].clone()),
                                        "Function" => Arc::new(CType::Function(
                                            args[0].clone(),
                                            args[1].clone(),
                                        )),
                                        "Call" => {
                                            Arc::new(CType::Call(args[0].clone(), args[1].clone()))
                                        }
                                        "Infix" => Arc::new(CType::Infix(args[0].clone())),
                                        "Prefix" => Arc::new(CType::Prefix(args[0].clone())),
                                        "Method" => Arc::new(CType::Method(args[0].clone())),
                                        "Property" => Arc::new(CType::Property(args[0].clone())),
                                        "Cast" => Arc::new(CType::Cast(args[0].clone())),
                                        "Own" => Arc::new(CType::Own(args[0].clone())),
                                        "Deref" => Arc::new(CType::Deref(args[0].clone())),
                                        "Mut" => Arc::new(CType::Mut(args[0].clone())),
                                        "Dependency" => Arc::new(CType::Dependency(
                                            args[0].clone(),
                                            args[1].clone(),
                                        )),
                                        "Rust" => Arc::new(CType::Rust(args[0].clone())),
                                        "Nodejs" => Arc::new(CType::Nodejs(args[0].clone())),
                                        "From" => Arc::new(CType::From(args[0].clone())),
                                        "Import" => CType::import(args[0].clone(), args[1].clone()),
                                        "Tuple" => CType::tuple(args.clone()),
                                        "Field" => CType::field(args.clone()),
                                        "Either" => CType::either(args.clone()),
                                        "Prop" => CType::prop(args[0].clone(), args[1].clone()),
                                        "Exclude" => {
                                            let resolved =
                                                CType::exclude(args[0].clone(), args[1].clone());
                                            CType::merge_exclude_parents(resolved, scope)
                                        }
                                        "AnyOf" => CType::anyof(args.clone()),
                                        "Buffer" => CType::buffer(args.clone()),
                                        "Array" => Arc::new(CType::Array(args[0].clone())),
                                        "Fail" => CType::cfail(args[0].clone()),
                                        "Min" => CType::min(args[0].clone(), args[1].clone()),
                                        "Max" => CType::max(args[0].clone(), args[1].clone()),
                                        "Neg" => CType::neg(args[0].clone()),
                                        "Len" => CType::len(args[0].clone()),
                                        "Size" => CType::size(args[0].clone()),
                                        "FileStr" => CType::filestr(args[0].clone()),
                                        "Concat" => CType::concat(args[0].clone(), args[1].clone()),
                                        "Env" => CType::env(args[0].clone()),
                                        "EnvExists" => CType::envexists(args[0].clone()),
                                        "Not" => CType::not(args[0].clone()),
                                        "Add" => CType::add(args[0].clone(), args[1].clone()),
                                        "Sub" => CType::sub(args[0].clone(), args[1].clone()),
                                        "Mul" => CType::mul(args[0].clone(), args[1].clone()),
                                        "Div" => CType::div(args[0].clone(), args[1].clone()),
                                        "Mod" => CType::cmod(args[0].clone(), args[1].clone()),
                                        "Pow" => CType::pow(args[0].clone(), args[1].clone()),
                                        "If" => {
                                            if args.len() == 2 {
                                                CType::tupleif(args[0].clone(), args[1].clone())
                                            } else if args.len() == 3 {
                                                CType::cif(
                                                    args[0].clone(),
                                                    args[1].clone(),
                                                    args[2].clone(),
                                                )
                                            } else {
                                                CType::fail(&format!("Invalid arguments provided to `If{{...}}`: {args:?}"))
                                            }
                                        }
                                        "And" => CType::and(args[0].clone(), args[1].clone()),
                                        "Or" => CType::or(args[0].clone(), args[1].clone()),
                                        "Xor" => CType::xor(args[0].clone(), args[1].clone()),
                                        "Nand" => CType::nand(args[0].clone(), args[1].clone()),
                                        "Nor" => CType::nor(args[0].clone(), args[1].clone()),
                                        "Xnor" => CType::xnor(args[0].clone(), args[1].clone()),
                                        "Eq" => CType::eq(args[0].clone(), args[1].clone()),
                                        "Neq" => CType::neq(args[0].clone(), args[1].clone()),
                                        "Lt" => CType::lt(args[0].clone(), args[1].clone()),
                                        "Lte" => CType::lte(args[0].clone(), args[1].clone()),
                                        "Gt" => CType::gt(args[0].clone(), args[1].clone()),
                                        "Gte" => CType::gte(args[0].clone(), args[1].clone()),
                                        unknown => CType::fail(&format!(
                                            "Unknown ctype {unknown} accessed. How did this happen?"
                                        )),
                                    }
                                }
                            }
                            others => {
                                // If we hit this branch, then the `args` vector needs to have a
                                // length of zero, and then we just bubble up the type as-is
                                if args.is_empty() {
                                    Arc::new(others.clone())
                                } else {
                                    CType::fail(&format!(
                                        "{var} is used as a generic type but is not one: {others:?}, {prior_value:?}",
                                    ))
                                }
                            }
                        }
                    }
                    None => CType::fail(&format!("{var} is not a valid type name")),
                })
            }
            parse::TypeBase::GnCall(_) => { /* We always process GnCall in the Variable path */ }
            parse::TypeBase::TypeGroup(g) => {
                if g.typeassignables.is_empty() {
                    // It's a void type!
                    prior_value = Some(Arc::new(CType::Group(Arc::new(CType::Void))));
                } else {
                    // Simply wrap the returned type in a `CType::Group`
                    prior_value = Some(Arc::new(CType::Group(withtypeoperatorslist_to_ctype(
                        &g.typeassignables,
                        scope,
                    )?)));
                }
            }
        };
        i += 1;
    }
    match prior_value {
        Some(p) => Ok(p),
        None => Err("Somehow did not resolve the type definition into anything".into()),
    }
}
