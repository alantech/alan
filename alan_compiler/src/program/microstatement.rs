use std::sync::Arc;

use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::function::{rettypes_match, type_to_args, type_to_rettype};
use super::scope::merge;
use super::ArgKind;
use super::FnKind;
use super::Function;
use super::OperatorMapping;
use super::Program;
use super::Scope;
use crate::parse;
use crate::render::Render;

/// FUI ordering (Floats, Unsigned ints, signed Ints, ascending bit width) of the
/// numeric types an integer literal may resolve to. The *last* surviving entry is
/// the highest-priority default chosen when the literal's `AnyOf` type is never
/// narrowed by context ("pick last in FUI order"), which keeps the historical
/// `i64`/`f64` defaults while allowing e.g. an above-`i64::MAX` literal to land on
/// `u64`.
pub const FUI_INT_TYPES: [&str; 10] = [
    "f32", "f64", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64",
];
pub const FUI_FLOAT_TYPES: [&str; 2] = ["f32", "f64"];

/// Parse an integer literal's source text (honoring `0b`/`0o`/`0x` prefixes, `_`
/// digit separators, and a leading `-`) into an `i128` for range checks. Returns
/// `None` for forms we can't represent in `i128` (e.g. an above-`u64` literal),
/// in which case the caller falls back to the historical `i64` typing.
fn parse_int_literal(s: &str) -> Option<i128> {
    let s = s.replace('_', "");
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r.to_string()),
        None => (false, s.clone()),
    };
    let (radix, digits) = if let Some(r) = rest.strip_prefix("0x") {
        (16, r.to_string())
    } else if let Some(r) = rest.strip_prefix("0b") {
        (2, r.to_string())
    } else if let Some(r) = rest.strip_prefix("0o") {
        (8, r.to_string())
    } else {
        (10, rest.clone())
    };
    let v = i128::from_str_radix(&digits, radix).ok()?;
    Some(if neg { -v } else { v })
}

/// Returns true if the integer value `v` fits in the numeric type named `name`.
/// Float types are always considered capable of holding an integer literal
/// (possibly with precision loss), matching the issue #215 design where a small
/// integer constant keeps `f32`/`f64` as candidates.
fn int_fits_type(v: i128, name: &str) -> bool {
    match name {
        "f32" | "f64" => true,
        "u8" => (0..=u8::MAX as i128).contains(&v),
        "u16" => (0..=u16::MAX as i128).contains(&v),
        "u32" => (0..=u32::MAX as i128).contains(&v),
        "u64" => (0..=u64::MAX as i128).contains(&v),
        "i8" => ((i8::MIN as i128)..=(i8::MAX as i128)).contains(&v),
        "i16" => ((i16::MIN as i128)..=(i16::MAX as i128)).contains(&v),
        "i32" => ((i32::MIN as i128)..=(i32::MAX as i128)).contains(&v),
        "i64" => ((i64::MIN as i128)..=(i64::MAX as i128)).contains(&v),
        _ => false,
    }
}

/// The FUI-ordered list of numeric type names a numeric literal may resolve to,
/// with candidates pruned by the literal's parsed value/sign. Integers that can't
/// be parsed into `i128` fall back to `i64` so behavior is no worse than before.
fn numeric_literal_type_names(repr: &str, is_float: bool) -> Vec<&'static str> {
    if is_float {
        return FUI_FLOAT_TYPES.to_vec();
    }
    match parse_int_literal(repr) {
        Some(v) => FUI_INT_TYPES
            .iter()
            .copied()
            .filter(|n| int_fits_type(v, n))
            .collect(),
        None => vec!["i64"],
    }
}

/// Returns true if the type (after unwrapping `Type`/`Group`) is a bound `f32`/`f64`.
fn ctype_is_float(t: &CType) -> bool {
    match t {
        CType::Binds(inner, _) => {
            matches!(&**inner, CType::TString(s) if s == "f32" || s == "f64")
        }
        CType::Type(_, inner) | CType::Group(inner) => ctype_is_float(inner),
        _ => false,
    }
}

/// Returns true if `s` is the textual form of a numeric literal (rather than a variable name).
/// Variable identifiers can't begin with a digit, `-`, or `.`, so this distinguishes a literal
/// argument (which may be safely retyped) from a reference to a variable (whose storage type is
/// already fixed at its declaration and must not be retyped here).
fn is_numeric_literal_repr(s: &str) -> bool {
    matches!(s.chars().next(), Some(c) if c.is_ascii_digit() || c == '-' || c == '.')
}

/// If `value` is a numeric *literal* whose `AnyOf` candidate set contains the (wrapper-stripped)
/// `target` type, narrow its type to the matching candidate member and return it; otherwise return
/// `value` unchanged. When narrowing an integer-form literal to a floating-point type, the textual
/// representation is given a `.0` suffix, since Rust rejects an integer literal in a float position.
///
/// We narrow to the matching *candidate member* (the concrete numeric type, e.g. `i64`) rather than
/// to `target` itself, because `target` may be a type alias (e.g. `type DupeI64 = i64 | i64`) that
/// dedups to a numeric type, and downstream codegen (notably JS literal boxing) keys off the
/// concrete numeric type, not the alias name.
fn narrow_numeric_literal(value: Microstatement, target: Arc<CType>) -> Microstatement {
    if let Microstatement::Value {
        typen,
        representation,
    } = &value
    {
        if let CType::AnyOf(ts) = &**typen {
            if is_numeric_literal_repr(representation) {
                let target_str = target.strip_value_wrappers().to_strict_string(false);
                let matched = ts
                    .iter()
                    .find(|t| (*t).clone().degroup().to_strict_string(false) == target_str)
                    .cloned();
                if let Some(matched) = matched {
                    let representation = if ctype_is_float(&matched.clone().degroup())
                        && !representation.contains(['.', 'e', 'E'])
                    {
                        format!("{representation}.0")
                    } else {
                        representation.clone()
                    };
                    return Microstatement::Value {
                        typen: matched,
                        representation,
                    };
                }
            }
        }
    }
    value
}

/// Narrow each numeric-literal argument of a resolved call to the concrete parameter type the
/// function expects, recording the choice made during dispatch so codegen emits a correctly-typed
/// constant (rather than the global FUI default). Non-literal arguments (e.g. variable references
/// or nested calls) are left untouched.
fn narrow_call_arg_literals(f: &Arc<Function>, args: Vec<Microstatement>) -> Vec<Microstatement> {
    let fargs = f.args();
    args.into_iter()
        .enumerate()
        .map(|(i, a)| {
            if i < fargs.len() {
                narrow_numeric_literal(a, fargs[i].2.clone())
            } else {
                a
            }
        })
        .collect()
}

/// Microstatements are a reduced syntax that doesn't have operators, methods, or reassigning to
/// the same variable. (We'll rely on LLVM to dedupe variables that are never used again.) This
/// syntax reduction will make generating the final output easier and also simplifies the work
/// needed to determine the actual types of a function's arguments and return type.
#[derive(Clone, Debug)]
pub enum Microstatement {
    Assignment {
        mutable: bool,
        name: String,
        value: Box<Microstatement>,
    },
    Arg {
        name: String,
        kind: ArgKind,
        typen: Arc<CType>,
    },
    FnCall {
        function: Arc<Function>,
        args: Vec<Microstatement>,
    },
    Closure {
        function: Arc<Function>,
    },
    VarCall {
        name: String,
        typen: Arc<CType>,
        args: Vec<Microstatement>,
    },
    Value {
        typen: Arc<CType>,
        representation: String,
    },
    Array {
        typen: Arc<CType>,
        vals: Vec<Microstatement>,
    },
    Return {
        value: Option<Box<Microstatement>>,
    }, // TODO: Conditionals
    /// A call into a native construct of the target language. The `name` (the
    /// bound method/operator/function/cast spelling) and `args` are kept
    /// structurally -- rather than baked into a pre-rendered `Value` string --
    /// so they can be substituted (e.g. by the inliner) and serialized by each
    /// codegen layer. `kind` selects the surface form. Most forms share syntax
    /// across our backends; the only exception is `Cast`, which is Rust-only
    /// (every `Cast{..}` bind is `fn{Rs}`, so it is never constructed when
    /// targeting JavaScript).
    NativeCall {
        typen: Arc<CType>,
        kind: NativeCallKind,
        name: String,
        args: Vec<Microstatement>,
    },
}

/// The kind of native construct a `Microstatement::NativeCall` represents. For
/// the receiver-based forms (`Method`/`Property`) `args[0]` is the receiver.
#[derive(Clone, Debug, PartialEq)]
pub enum NativeCallKind {
    /// `recv.name(rest_args...)`
    Method,
    /// `recv.name` (no call parens; exactly one arg, the receiver)
    Property,
    /// `name(args...)` — a plain function/macro-style native call (e.g.
    /// `format!("{}", arg1)`). There is no receiver; every argument is a call
    /// argument.
    Function,
    /// `(lhs name rhs)` — a native infix operator (e.g. `+`, `==`); exactly two
    /// arguments.
    Infix,
    /// `(name operand)` — a native prefix operator (e.g. `!`); exactly one
    /// argument.
    Prefix,
    /// `(operand as name)` — a native cast to the target type `name`. Rust-only
    /// syntax; never constructed for the JavaScript backend.
    Cast,
}

impl Microstatement {
    pub fn get_type(&self) -> Arc<CType> {
        match self {
            Self::Value { typen, .. } => typen.clone(),
            Self::Array { typen, .. } => typen.clone(),
            Self::Arg { typen, .. } => typen.clone(),
            Self::Assignment { value, .. } => value.get_type(),
            Self::Return { value } => match value {
                Some(v) => v.get_type(),
                None => Arc::new(CType::Void),
            },
            Self::FnCall { function, args: _ } => function.rettype(),
            Self::Closure { function } => function.typen.clone(),
            Self::VarCall { typen, .. } => typen.clone(),
            Self::NativeCall { typen, .. } => typen.clone(),
        }
    }
}

#[derive(Clone, Debug)]
enum BaseChunk<'a> {
    #[allow(clippy::upper_case_acronyms)]
    IIGE(
        Option<&'a Microstatement>,
        &'a parse::Functions,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    GFuncCall(
        Option<&'a Microstatement>,
        &'a String,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    #[allow(clippy::upper_case_acronyms)]
    IIFE(
        Option<&'a Microstatement>,
        &'a parse::Functions,
        Option<&'a parse::FnCall>,
    ),
    FuncCall(
        Option<&'a Microstatement>,
        &'a String,
        Option<&'a parse::FnCall>,
    ),
    TypeCall(
        Option<&'a Microstatement>,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    ConstantAccessor(&'a parse::Constants),
    ArrayAccessor(&'a parse::ArrayBase),
    Function(&'a parse::Functions),
    Group(&'a parse::FnCall),
    Array(&'a parse::ArrayBase),
    Variable(&'a String),
    Constant(&'a parse::Constants),
}

pub fn baseassignablelist_to_microstatements<'a>(
    bal: &[parse::BaseAssignable],
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    let mut i = 0;
    let mut prior_value: Option<Microstatement> = None;
    let l = bal.len();
    while i < l {
        // First find a chunk of the baseassignable list that we can work with and then perform the
        // operation afterwards. Fail with an error message if no valid path forward can be found.
        // I recognize that this could be done with `nom` at a higher level, but I don't think it
        // will buy me much for this little bit of parsing logic, and I am still not satisfied with
        // the lack of metadata tracking with my usage of `nom`.
        let (chunk, inc) = match (
            &prior_value,
            bal.get(i),
            bal.get(i + 1),
            bal.get(i + 2),
            bal.get(i + 3),
        ) {
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
            ) => (BaseChunk::IIGE(Some(p), f, g, Some(h)), 4),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
            ) => (BaseChunk::GFuncCall(Some(p), f, g, Some(h)), 4),
            (
                None,
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
                _,
            ) => (BaseChunk::IIGE(None, f, g, Some(h)), 3),
            (
                None,
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
                _,
            ) => (BaseChunk::GFuncCall(None, f, g, Some(h)), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                _,
            ) => (BaseChunk::IIGE(Some(p), f, g, None), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                _,
            ) => (BaseChunk::GFuncCall(Some(p), f, g, None), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::IIFE(Some(p), f, Some(g)), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::FuncCall(Some(p), f, Some(g)), 3),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::GnCall(t)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::TypeCall(Some(p), t, Some(g)), 3),
            (
                None,
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::IIFE(None, f, Some(g)), 2),
            (
                None,
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::FuncCall(None, f, Some(g)), 2),
            (
                None,
                Some(parse::BaseAssignable::GnCall(t)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::TypeCall(None, t, Some(g)), 2),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                _,
                _,
            ) => (BaseChunk::IIFE(Some(p), f, None), 2),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                _,
                _,
            ) => (BaseChunk::FuncCall(Some(p), f, None), 2),
            (
                Some(p),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::GnCall(t)),
                _,
                _,
            ) => (BaseChunk::TypeCall(Some(p), t, None), 2),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Constants(c)),
                _,
                _,
            ) => (BaseChunk::ConstantAccessor(c), 2),
            (None, Some(parse::BaseAssignable::Functions(f)), _, _, _) => {
                (BaseChunk::Function(f), 1)
            }
            (None, Some(parse::BaseAssignable::FnCall(g)), _, _, _) => (BaseChunk::Group(g), 1),
            (None, Some(parse::BaseAssignable::Array(a)), _, _, _) => (BaseChunk::Array(a), 1),
            (None, Some(parse::BaseAssignable::Variable(v)), _, _, _) => {
                (BaseChunk::Variable(v), 1)
            }
            (None, Some(parse::BaseAssignable::Constants(c)), _, _, _) => {
                (BaseChunk::Constant(c), 1)
            }
            (Some(_), Some(parse::BaseAssignable::Array(a)), _, _, _) => {
                (BaseChunk::ArrayAccessor(a), 1)
            }
            _ => {
                return Err(format!(
                    "Invalid syntax: {}\n Cannot parse after {}, l - i = {}",
                    bal.iter()
                        .map(|ba| ba.to_string())
                        .collect::<Vec<String>>()
                        .join(""),
                    bal[i - 1].to_string(),
                    l - i,
                )
                .into());
            }
        };
        i += inc;
        // Now we just operate on our chunk and create a new prior_value to replace the old one, if
        // any exists. We'll start from the easier ones first and work our way up to the
        // complicated ones
        match chunk {
            // We know it has to be defined because we blew up earlier if not
            BaseChunk::Constant(c) => {
                match c {
                    parse::Constants::Bool(b) => {
                        let booln = scope.resolve_type("bool").unwrap().clone();
                        prior_value = Some(Microstatement::Value {
                            typen: booln,
                            representation: b.clone(),
                        });
                    }
                    parse::Constants::Strn(s) => {
                        let string = scope.resolve_type("string").unwrap().clone();
                        prior_value = Some(Microstatement::Value {
                            typen: string,
                            representation: if s.starts_with('"') {
                                s.clone()
                            } else {
                                // TODO: Is there a cheaper way to do this conversion?
                                s.replace('\"', "\\\"")
                                    .replace("\\'", "\\\\\"")
                                    .replace('\'', "\"")
                                    .replace("\\\\\"", "'")
                            },
                        });
                    }
                    parse::Constants::Num(n) => {
                        // A numeric literal is typed as an `AnyOf` over the numeric types that can
                        // hold its value (in FUI order), or that single type if only one survives.
                        // Context (function args, `let`/return annotations) narrows it later; any
                        // still-ambiguous literal collapses to the last (FUI) candidate before
                        // codegen. See `docs/int-float-constant-selection-plan.md`.
                        let (repr, is_float) = match n {
                            parse::Number::RealNum(r) => (r.clone(), true),
                            parse::Number::IntNum(i) => (i.clone(), false),
                        };
                        let names = numeric_literal_type_names(&repr, is_float);
                        let mut candidates: Vec<Arc<CType>> = Vec::new();
                        for nm in &names {
                            if let Some(t) = scope.resolve_type(nm) {
                                candidates.push(t.clone());
                            }
                        }
                        let typen = match candidates.len() {
                            0 => scope
                                .resolve_type(if is_float { "f64" } else { "i64" })
                                .unwrap()
                                .clone(),
                            1 => candidates.into_iter().next().unwrap(),
                            _ => Arc::new(CType::AnyOf(candidates)),
                        };
                        prior_value = Some(Microstatement::Value {
                            typen,
                            representation: repr,
                        });
                    }
                }
            }
            BaseChunk::Variable(v) => {
                let typen = match microstatements.iter().find(|m| match m {
                    Microstatement::Assignment { name, .. } => v == name,
                    Microstatement::Arg { name, .. } => v == name,
                    _ => false,
                }) {
                    // Reaching the `Some` path requires it to be of type
                    // Microstatment::Assignment, but Rust doesn't seem to know that, so force
                    // it.
                    Some(m) => match m {
                        Microstatement::Assignment { value, .. } => {
                            Ok::<Arc<CType>, Box<dyn std::error::Error>>(value.get_type())
                        }
                        Microstatement::Arg { typen, .. } => Ok(typen.clone()),
                        _ => unreachable!(),
                    },
                    None => {
                        // It could be a function.
                        let mut function_types = scope.resolve_function_types(v);
                        if let Some(pf) = &parent_fn {
                            if pf.origin_scope_path != scope.path {
                                let program = Program::get_program();
                                if let Ok(origin_scope) =
                                    program.scope_by_file(&pf.origin_scope_path)
                                {
                                    let other_function_types =
                                        origin_scope.resolve_function_types(v);
                                    function_types =
                                        match (&*function_types, &*other_function_types) {
                                            (
                                                CType::Void | CType::DerivedVoid(..),
                                                CType::Void | CType::DerivedVoid(..),
                                            ) => Arc::new(CType::Void),
                                            (CType::Void | CType::DerivedVoid(..), _) => {
                                                other_function_types
                                            }
                                            (_, CType::Void | CType::DerivedVoid(..)) => {
                                                function_types
                                            }
                                            (CType::AnyOf(t1), CType::AnyOf(t2)) => {
                                                Arc::new(CType::AnyOf({
                                                    let mut v = Vec::new();
                                                    v.append(&mut t1.clone());
                                                    v.append(&mut t2.clone());
                                                    v
                                                }))
                                            }
                                            (_, CType::AnyOf(t2)) => Arc::new(CType::AnyOf({
                                                let mut v = Vec::new();
                                                v.push(function_types);
                                                v.append(&mut t2.clone());
                                                v
                                            })),
                                            (CType::AnyOf(t1), _) => Arc::new(CType::AnyOf({
                                                let mut v = Vec::new();
                                                v.append(&mut t1.clone());
                                                v.push(other_function_types);
                                                v
                                            })),
                                            (_, _) => Arc::new(CType::AnyOf(vec![
                                                function_types,
                                                other_function_types,
                                            ])),
                                        };
                                }
                                Program::return_program(program);
                            }
                        }
                        match &*function_types {
                            CType::Void | CType::DerivedVoid(..) => {
                                // It could be a constant
                                let maybe_c = scope.resolve_const(v);
                                match maybe_c {
                                    None => Err(format!("Couldn't find variable {v}").into()),
                                    Some(c) => {
                                        // TODO: Confirm the specified typename matches the
                                        // actual typename of the value
                                        let mut temp_scope = scope.child();
                                        let res = withoperatorslist_to_microstatements(
                                            &c.assignables,
                                            parent_fn,
                                            temp_scope,
                                            microstatements,
                                        )?;
                                        temp_scope = res.0;
                                        microstatements = res.1;
                                        let cm = microstatements.pop().unwrap();
                                        let typen = match &cm {
                                            Microstatement::Value { typen, .. } | Microstatement::Array { typen, .. } => Ok(typen.clone()),
                                            Microstatement::FnCall { function: _, args: _ } => Err("TODO: Support global constant function calls"),
                                            _ => Err("This should be impossible, a constant has to be a value, array, or fncall"),
                                        }?;
                                        merge!(scope, temp_scope);
                                        microstatements.push(Microstatement::Assignment {
                                            mutable: false,
                                            name: v.clone(),
                                            value: Box::new(cm),
                                        });
                                        Ok(typen)
                                    }
                                }
                            }
                            _ => Ok(function_types),
                        }
                    }
                }?;
                prior_value = Some(Microstatement::Value {
                    typen,
                    representation: v.to_string(),
                });
            }
            BaseChunk::Array(a) => {
                // We don't allow `[]` syntax, so blow up if the assignablelist is empty
                if a.assignablelist.is_empty() {
                    return Err("Cannot create an empty array with bracket syntax, use `Array{MyType}()` syntax instead".into());
                }
                let mut array_vals = Vec::new();
                for wol in &a.assignablelist {
                    let res = withoperatorslist_to_microstatements(
                        wol,
                        parent_fn,
                        scope,
                        microstatements,
                    )?;
                    scope = res.0;
                    microstatements = res.1;
                    array_vals.push(microstatements.pop().unwrap());
                }
                // TODO: Currently assuming all array values are the same type, should check that
                // better
                // Collapse an `AnyOf` element type (e.g. from numeric literals like `[1, 2, 3]`) to
                // its FUI default so the array's element type is a concrete, name-able type rather
                // than the whole candidate set.
                let inner_type = array_vals[0].get_type().collapse_anyof_default();
                let inner_type_str = inner_type.clone().to_callable_string();
                let array_type_name = format!("Array_{inner_type_str}_");
                let array_type = Arc::new(CType::Array(inner_type));
                let type_str = format!("type {array_type_name} = {inner_type_str}[];");
                let parse_type = parse::types(&type_str);
                let res = CType::from_ast(scope, &parse_type.unwrap().1, false)?;
                scope = res.0;
                prior_value = Some(Microstatement::Array {
                    typen: array_type,
                    vals: array_vals,
                });
            }
            BaseChunk::Group(g) => {
                // TODO: Add support for anonymous tuples with this syntax, for now break if the
                // group's inner length is greater that one record
                if g.assignablelist.len() != 1 {
                    return Err("Anonymous tuple support not yet implemented".into());
                }
                let res = withoperatorslist_to_microstatements(
                    &g.assignablelist[0],
                    parent_fn,
                    scope,
                    microstatements,
                )?;
                scope = res.0;
                microstatements = res.1;
                prior_value = microstatements.pop();
            }
            BaseChunk::Function(f) => {
                // TODO: Move a lot of this into `Function`
                // First, some restrictions on closure function syntax (at least for now)
                if f.opttypegenerics.is_some() {
                    return Err(
                        "Conditional compilation not supported for closure functions".into(),
                    );
                }
                if f.optgenerics.is_some() {
                    return Err("Generics not supported for closure functions".into());
                }
                // If we got here, we know we're making a "normal" function
                let kind = FnKind::Normal;
                let mut inner_scope = scope.child();
                let original_len = microstatements.len();
                let statements = match &f.fullfunctionbody {
                    parse::FullFunctionBody::DecOnly(_) => Vec::new(), // TODO: Explode instead?
                    parse::FullFunctionBody::FunctionBody(body) => body.statements.clone(),
                    parse::FullFunctionBody::AssignFunction(assign) => {
                        vec![parse::Statement::Returns(parse::Returns {
                            returnn: "return".to_string(),
                            a: " ".to_string(),
                            retval: Some(parse::RetVal {
                                assignables: assign.assignables.clone(),
                                a: "".to_string(),
                            }),
                            semicolon: ";".to_string(),
                        })]
                    }
                };
                // TODO: A big blob of crap copied from Function that should really live there
                // *and* needs refactoring
                // TODO: Add code to properly convert the typeassignable vec into a CType tree and use it.
                // For now, just hardwire the parsing as before.
                let mut typen = match &f.opttype {
                    None => {
                        Ok::<Arc<CType>, Box<dyn std::error::Error>>(Arc::new(CType::Function(
                            Arc::new(CType::Void),
                            Arc::new(CType::Infer("unknown".to_string(), "unknown".to_string())),
                        )))
                    }
                    Some(typeassignable) if typeassignable.is_empty() => {
                        Ok(Arc::new(CType::Function(
                            Arc::new(CType::Void),
                            Arc::new(CType::Infer("unknown".to_string(), "unknown".to_string())),
                        )))
                    }
                    Some(typeassignable) => match &kind {
                        FnKind::Generic(gs, _) | FnKind::BoundGeneric(gs, _) => {
                            // This lets us partially resolve the function argument and return types
                            let mut temp_scope = inner_scope.child();
                            for g in gs {
                                temp_scope =
                                    CType::from_ctype(temp_scope, g.0.clone(), g.1.clone());
                            }
                            let ctype =
                                withtypeoperatorslist_to_ctype(typeassignable, &temp_scope)?;
                            // If the `ctype` is a Function type, we have both the input and output defined. If
                            // it's any other type, we presume it's only the input type defined
                            let (input_type, output_type) = match &*ctype {
                                CType::Function(i, o) => (i.clone(), o.clone()),
                                _ => (
                                    ctype,
                                    Arc::new(CType::Infer(
                                        "unknown".to_string(),
                                        "unknown".to_string(),
                                    )),
                                ),
                            };
                            // In case there were any created functions (eg constructor or accessor
                            // functions) in that path, we need to merge the child's functions back up
                            merge!(inner_scope, temp_scope);
                            // The input type will be interpreted in many different ways:
                            // If it's a Group, unwrap it and continue. Ideally after that it's a Tuple
                            // type containing Field types, that's a "conventional" function
                            // definition, where the label becomes an argument name and the type is the
                            // type. If the tuple doesn't have Fields inside of it, we auto-generate
                            // argument names, eg `arg0`, `arg1`, etc. If it is not a Tuple type but is
                            // a Field type, we have a single argument function with a specified
                            // variable name. If it's any other type, we just label it `arg0`
                            let degrouped_input = input_type.degroup();
                            Ok(Arc::new(CType::Function(degrouped_input, output_type)))
                        }
                        _ => {
                            let ctype = withtypeoperatorslist_to_ctype(typeassignable, &scope)?;
                            // If the `ctype` is a Function type, we have both the input and output defined. If
                            // it's any other type, we presume it's only the input type defined
                            let (input_type, output_type) = match &*ctype {
                                CType::Function(i, o) => (i.clone(), o.clone()),
                                _otherwise => (
                                    ctype,
                                    Arc::new(CType::Infer(
                                        "unknown".to_string(),
                                        "unknonw".to_string(),
                                    )),
                                ),
                            };
                            let degrouped_input = input_type.degroup();
                            Ok(Arc::new(CType::Function(degrouped_input, output_type)))
                        }
                    },
                }?;
                for (name, kind, typen) in type_to_args(typen.clone()) {
                    microstatements.push(Microstatement::Arg { name, kind, typen });
                }
                // Route the closure body through the tail-aware driver so that any
                // block-level conditionals inside it are handled (and consume their tails).
                let res = statements_to_microstatements(
                    &statements,
                    parent_fn,
                    inner_scope,
                    microstatements,
                )?;
                inner_scope = res.0;
                microstatements = res.1;
                let ms = microstatements.split_off(original_len);
                match ms.last() {
                    // Don't do anything in this path, this is probably a derived function
                    Some(Microstatement::Arg { .. }) => {}
                    last => {
                        let current_rettype = type_to_rettype(typen.clone());
                        // A trailing `return <expr>` defines the return type. Anything else --
                        // including an empty body (`None`, e.g. the synthesized `fn() {}` else
                        // arm of a void conditional) -- is a void return.
                        let actual_rettype = match last {
                            Some(Microstatement::Return { value: Some(v) }) => v.get_type(),
                            _ => Arc::new(CType::Void),
                        };
                        if let CType::Infer(..) = &*current_rettype {
                            // We're definitely replacing with the inferred type
                            let input_type = match &*typen {
                                CType::Function(i, _) => i.clone(),
                                _ => Arc::new(CType::Void),
                            };
                            typen = Arc::new(CType::Function(input_type, actual_rettype));
                        } else if !rettypes_match(&current_rettype, &actual_rettype) {
                            CType::fail(&format!(
                                "Function {} specified to return {} but actually returns {}",
                                match &f.optname {
                                    Some(name) => name,
                                    None => "closure",
                                },
                                current_rettype.to_strict_string(false),
                                actual_rettype.to_strict_string(false),
                            ));
                        } else {
                            // Do nothing, they're the same
                        }
                    }
                }
                match &*typen {
                    CType::Function(i, o) => {
                        match &**o {
                            CType::Void | CType::DerivedVoid(..) => { /* Do nothing */ }
                            CType::Infer(t, _) if t == "unknown" => {
                                CType::fail(&format!(
                                    "The return type for {}({}) could not be inferred.",
                                    match &f.optname {
                                        Some(name) => name,
                                        None => "closure",
                                    },
                                    i.clone().to_strict_string(false)
                                ));
                            }
                            CType::Infer(..) => { /* Do nothing */ }
                            _otherwise => {
                                let name = o.clone().to_callable_string();
                                if scope.resolve_type(&name).is_none() {
                                    scope = CType::from_ctype(scope, name, o.clone());
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                let function = Arc::new(Function {
                    name: match &f.optname {
                        Some(name) => name.clone(),
                        None => "closure".to_string(),
                    },
                    typen,
                    microstatements: ms,
                    kind,
                    origin_scope_path: scope.path.clone(),
                    lazy_body: None,
                });
                prior_value = Some(Microstatement::Closure { function });
            }
            BaseChunk::ArrayAccessor(a) => {
                if let Some(prior) = &prior_value {
                    let mut temp_scope = scope.child();
                    let mut array_accessor_microstatements = vec![prior.clone()];
                    for wol in &a.assignablelist {
                        let res = withoperatorslist_to_microstatements(
                            wol,
                            parent_fn,
                            temp_scope,
                            microstatements,
                        )?;
                        temp_scope = res.0;
                        microstatements = res.1;
                        array_accessor_microstatements.push(microstatements.pop().unwrap());
                    }
                    let mut arg_types = Vec::new();
                    for m in &array_accessor_microstatements {
                        arg_types.push(m.get_type());
                    }
                    let res = temp_scope.resolve_function(&"get".to_string(), &arg_types);
                    match res {
                        Some((mut temp_scope, f)) => {
                            temp_scope
                                .functions
                                .insert("get".to_string(), vec![f.clone()]);
                            merge!(scope, temp_scope);
                            prior_value = Some(Microstatement::FnCall {
                                args: narrow_call_arg_literals(&f, array_accessor_microstatements),
                                function: f,
                            })
                        }
                        None => {
                            return Err(format!(
                                "A function with the signature get({}) does not exist",
                                arg_types
                                    .iter()
                                    .map(|a| a.clone().to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            )
                            .into());
                        }
                    }
                } else {
                    // This is impossible, but I'm having a hard time convincing Rust of that
                    panic!("Impossible to reach the ArrayAccessor path without a prior value");
                }
            }
            BaseChunk::ConstantAccessor(c) => {
                if let Some(prior) = &prior_value {
                    // If the accessor names a numeric type that is one of a numeric literal's
                    // candidate types (e.g. `5.u64`), narrow the literal to that type directly. This
                    // turns the accessor into a zero-cost type ascription -- the identity conversion,
                    // with no runtime `as` cast -- for in-range literals. A variable, a non-numeric
                    // accessor, or an out-of-range literal (e.g. `300.u8`, where `u8` was pruned from
                    // the candidate set) is left alone and still goes through the normal
                    // conversion/cast function.
                    let prior = match scope.resolve_type(&c.to_string()) {
                        Some(target) => narrow_numeric_literal(prior.clone(), target),
                        None => prior.clone(),
                    };
                    let mut temp_scope = scope.child();
                    let constant_accessor_microstatements = vec![prior];
                    let mut arg_types = Vec::new();
                    for m in &constant_accessor_microstatements {
                        // Collapse an `AnyOf` literal type to its FUI default so the accessor (e.g.
                        // `5.u64`) dispatches against, and registers, a concrete type rather than
                        // the whole candidate set.
                        let t = m.get_type().collapse_anyof_default();
                        arg_types.push(t.clone());
                        // In case the type constructor has not already been created
                        temp_scope =
                            CType::from_ctype(temp_scope, t.clone().to_callable_string(), t);
                    }
                    let res = temp_scope.resolve_function(&c.to_string(), &arg_types);
                    match res {
                        Some((mut temp_scope, f)) => {
                            temp_scope.functions.insert(c.to_string(), vec![f.clone()]);
                            merge!(scope, temp_scope);
                            prior_value = Some(Microstatement::FnCall {
                                args: narrow_call_arg_literals(
                                    &f,
                                    constant_accessor_microstatements,
                                ),
                                function: f,
                            })
                        }
                        None => {
                            return Err(format!(
                                "A function with the signature {}({}) does not exist",
                                c.to_string(),
                                arg_types
                                    .iter()
                                    .map(|a| a.clone().to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            )
                            .into());
                        }
                    }
                } else {
                    // This is impossible, but I'm having a hard time convincing Rust of that
                    panic!("Impossible to reach the ConstantAccessor path without a prior value");
                }
            }
            BaseChunk::TypeCall(prior, g, f) => {
                // Get all of the arguments for the function into an array. If there's a prior
                // value it becomes the first argument.
                let mut arg_microstatements = match prior {
                    Some(p) => vec![p.clone()],
                    None => Vec::new(),
                };
                match f {
                    None => {}
                    Some(fncall) => {
                        for arg in &fncall.assignablelist {
                            let res = withoperatorslist_to_microstatements(
                                arg,
                                parent_fn,
                                scope,
                                microstatements,
                            )?;
                            scope = res.0;
                            microstatements = res.1;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type());
                }
                let ctype = withtypeoperatorslist_to_ctype(&g.typecalllist, &scope)?;
                let callable_name = ctype.clone().to_callable_string();
                // For intrinsic generics wrapping resolved types, construct alias-preserved name
                let alias_name: Option<String> = match &*ctype {
                    CType::Shared(inner) | CType::Array(inner) => {
                        let inner_name = inner.clone().to_callable_string();
                        let outer = match &*ctype {
                            CType::Shared(_) => "Shared",
                            CType::Array(_) => "Array",
                            _ => unreachable!(),
                        };
                        Some(format!("{}{{{}}}", outer, inner_name))
                    }
                    _ => None,
                };
                let name = alias_name.clone().unwrap_or(callable_name.clone());
                scope = CType::from_ctype(scope, name.clone(), ctype.clone());
                let temp_scope = scope.child();
                let res = temp_scope.resolve_function(&name, &arg_types);
                match res {
                    Some((mut temp_scope, f)) => {
                        temp_scope.functions.insert(name.clone(), vec![f.clone()]);
                        merge!(scope, temp_scope);
                        prior_value = Some(Microstatement::FnCall {
                            args: narrow_call_arg_literals(&f, arg_microstatements),
                            function: f.clone(),
                        })
                    }
                    None => {
                        return Err(format!(
                            "A function with the signature {}({}) does not exist",
                            name,
                            arg_types
                                .iter()
                                .map(|a| a.clone().to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                        .into());
                    }
                }
            }
            BaseChunk::FuncCall(prior, f, g) => {
                // Get all of the arguments for the function into an array. If there's a prior
                // value it becomes the first argument.
                let mut arg_microstatements = match prior {
                    Some(p) => vec![p.clone()],
                    None => Vec::new(),
                };
                match g {
                    None => {}
                    Some(fncall) => {
                        for arg in &fncall.assignablelist {
                            let res = withoperatorslist_to_microstatements(
                                arg,
                                parent_fn,
                                scope,
                                microstatements,
                            )?;
                            scope = res.0;
                            microstatements = res.1;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                // If this call names a numeric type (e.g. `5.u64` or `u64(5)`) and its first
                // argument is a numeric literal whose candidate set includes that type, narrow the
                // literal directly so the *identity* conversion is selected -- a zero-cost type
                // ascription rather than a runtime `as` cast. A variable argument or an out-of-range
                // literal (where the type was pruned from the candidate set) is left alone and still
                // goes through the normal conversion/cast function.
                if !arg_microstatements.is_empty() {
                    if let Some(target) = scope.resolve_type(f) {
                        arg_microstatements[0] =
                            narrow_numeric_literal(arg_microstatements[0].clone(), target);
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type());
                }
                // Look for closure functions in the microstatement array first to see if that's
                // what should be called, scanning in reverse order to find the most recent
                // definition that matches, if multiple match
                let mut closure_fn = None;
                let mut var_fn = None;
                for ms in microstatements.iter().rev() {
                    match ms {
                        Microstatement::Closure { function } => {
                            if &function.name == f && function.args().len() == arg_types.len() {
                                let mut works = true;
                                for ((_, _, a), b) in function.args().iter().zip(&arg_types) {
                                    if !a.clone().accepts(b.clone()) {
                                        works = false;
                                    }
                                }
                                if works {
                                    closure_fn = Some(function.clone());
                                    break;
                                }
                            }
                        }
                        Microstatement::Arg {
                            name,
                            kind: _,
                            typen,
                        } => {
                            if name == f {
                                if let CType::Function(i, o) = &**typen {
                                    let mut works = true;
                                    // TODO: Really need to just have the Function use the Function
                                    // CType instead of this stuff
                                    let farg_types = match &**i {
                                        CType::Void | CType::DerivedVoid(..) => Vec::new(),
                                        CType::Tuple(ts, _) => ts.clone(),
                                        _other => vec![i.clone()],
                                    };
                                    for (a, b) in farg_types.iter().zip(&arg_types) {
                                        if !a.clone().accepts(b.clone()) {
                                            works = false;
                                        }
                                    }
                                    if works {
                                        var_fn = Some((name.clone(), o.clone()));
                                        break;
                                    }
                                }
                            }
                        }
                        Microstatement::Assignment { .. } => {
                            // TODO
                        }
                        _ => { /* Do nothing */ }
                    }
                }
                if let Some(func) = closure_fn {
                    prior_value = Some(Microstatement::FnCall {
                        args: narrow_call_arg_literals(&func, arg_microstatements),
                        function: func,
                    });
                } else if let Some((name, typen)) = var_fn {
                    prior_value = Some(Microstatement::VarCall {
                        name,
                        typen,
                        args: arg_microstatements,
                    });
                } else {
                    // Now confirm that there's actually a function with this name that takes these
                    // types
                    let mut temp_scope = scope.child();
                    let maybe_origin_scope;
                    let res = match temp_scope.resolve_function(f, &arg_types) {
                        Some(r) => Some(r),
                        None => match &parent_fn {
                            Some(parent) => {
                                // Perhaps this function is defined in the parent function's origin scope?
                                let program = Program::get_program();
                                let origin_scope = program.scope_by_file(&parent.origin_scope_path);
                                let out = match origin_scope {
                                    Ok(origin) => {
                                        maybe_origin_scope = Some(origin.clone());
                                        temp_scope = maybe_origin_scope.as_ref().unwrap().child();
                                        temp_scope.resolve_function(f, &arg_types)
                                    }
                                    Err(_) => None,
                                };
                                Program::return_program(program);
                                out
                            }
                            None => None,
                        },
                    };
                    match res {
                        Some((mut temp_scope, fun)) => {
                            // Success! Let's emit this
                            // TODO: Do a better job at type rewriting here
                            let funargs = fun.args();
                            #[allow(clippy::needless_range_loop)]
                            for i in 0..funargs.len() {
                                match &arg_microstatements[i] {
                                    Microstatement::Value {
                                        typen,
                                        representation,
                                    } => {
                                        let actual_typen = funargs[i].2.clone();
                                        if typen != &actual_typen {
                                            if matches!(&*actual_typen, CType::Function(..)) {
                                                let temp_scope_2 = temp_scope.child();
                                                match temp_scope_2.resolve_function(
                                                    representation,
                                                    &type_to_args(actual_typen.clone())
                                                        .into_iter()
                                                        .map(|(_, _, t)| t)
                                                        .collect::<Vec<Arc<CType>>>(),
                                                ) {
                                                    None => {
                                                        arg_microstatements[i] =
                                                            Microstatement::Value {
                                                                typen: actual_typen.clone(),
                                                                representation: representation
                                                                    .clone(),
                                                            };
                                                    }
                                                    Some((s, func)) => {
                                                        if temp_scope
                                                            .functions
                                                            .contains_key(&func.name)
                                                        {
                                                            arg_microstatements[i] =
                                                                Microstatement::Value {
                                                                    typen: actual_typen.clone(),
                                                                    representation: func
                                                                        .name
                                                                        .clone(),
                                                                };
                                                        } else {
                                                            arg_microstatements[i] =
                                                                Microstatement::Value {
                                                                    typen: actual_typen.clone(),
                                                                    representation: representation
                                                                        .clone(),
                                                                };
                                                        }
                                                        merge!(temp_scope, s);
                                                    }
                                                }
                                            } else {
                                                // Don't strip Shared wrapper: preserve original type
                                                // so codegen can detect Shared{T} for deep clone, etc.
                                                if !matches!(typen.as_ref(), CType::Shared(..)) {
                                                    arg_microstatements[i] =
                                                        Microstatement::Value {
                                                            typen: actual_typen.clone(),
                                                            representation: representation.clone(),
                                                        };
                                                }
                                            }
                                        }
                                    }
                                    _ => { /* Do nothing */ }
                                }
                            }
                            merge!(scope, temp_scope);

                            prior_value = Some(Microstatement::FnCall {
                                args: narrow_call_arg_literals(&fun, arg_microstatements.clone()),
                                function: fun.clone(), // TODO: Drop the clone
                            });
                        }
                        None => {
                            return Err(format!(
                                "Could not find a function with a call signature of {}({})",
                                f,
                                arg_types
                                    .iter()
                                    .map(|a| a.clone().collapse_anyof_default().to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            )
                            .into());
                        }
                    }
                }
            }
            BaseChunk::IIFE(_prior, _f, _g) => {
                // TODO: This may just be some simple microstatement generation here compared to
                // actual closure creation
                return Err("TODO: Implement IIFE support".into());
            }
            BaseChunk::GFuncCall(prior, f, g, h) => {
                // Get all of the arguments for the function into an array. If there's a prior
                // value it becomes the first argument.
                let mut arg_microstatements = match prior {
                    Some(p) => vec![p.clone()],
                    None => Vec::new(),
                };
                match h {
                    None => {}
                    Some(fncall) => {
                        for arg in &fncall.assignablelist {
                            let res = withoperatorslist_to_microstatements(
                                arg,
                                parent_fn,
                                scope,
                                microstatements,
                            )?;
                            scope = res.0;
                            microstatements = res.1;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type());
                }
                let generic_types = {
                    let mut out = vec![];
                    for tc in g.typecalllist.split(
                        |tc| matches!(tc, parse::WithTypeOperators::Operators(o) if o.op == ","),
                    ) {
                        out.push(withtypeoperatorslist_to_ctype(&tc.to_vec(), &scope)?);
                    }
                    out
                };
                for g in &generic_types {
                    scope = CType::from_ctype(scope, g.clone().to_callable_string(), g.clone());
                }
                let maybe_type = scope.resolve_type(f);
                let temp_scope = scope.child();
                let maybe_generic_function =
                    temp_scope.resolve_generic_function(f, &generic_types, &arg_types);
                match (maybe_type, maybe_generic_function) {
                    (None, None) => {
                        return Err(format!(
                            "Generic type or function {}{}({}) not found",
                            f,
                            g.to_string(),
                            arg_types
                                .iter()
                                .map(|t| t.clone().to_functional_string())
                                .collect::<Vec<String>>()
                                .join(", "),
                        )
                        .into());
                    }
                    (_, Some((temp_scope, func))) => {
                        merge!(scope, temp_scope);
                        prior_value = Some(Microstatement::FnCall {
                            args: narrow_call_arg_literals(&func, arg_microstatements),
                            function: func.clone(), // TODO: Drop the clone
                        });
                    }
                    (Some(_), None) => {
                        // Confirmed that this type exists, we now need to generate a realized
                        // generic type for this specified type and shove it into the non-exported
                        // scope, then we can be sure we can call it.
                        let name = format!(
                            "{}{}",
                            f,
                            g.to_string().replace([' ', ',', ':', '{', '}'], "_")
                        )
                        .replace('|', "_")
                        .replace("()", "void"); // Really bad
                        let parse_type = parse::Types {
                            typen: "type".to_string(),
                            a: "".to_string(),
                            opttypegenerics: None,
                            b: "".to_string(),
                            fulltypename: parse::FullTypename {
                                typename: name.clone(),
                                opttypegenerics: None,
                            },
                            c: "".to_string(),
                            typedef: parse::TypeDef {
                                a: Some("=".to_string()),
                                b: "".to_string(),
                                typeassignables: vec![parse::WithTypeOperators::TypeBaseList(
                                    vec![
                                        parse::TypeBase::Variable(f.to_string()),
                                        parse::TypeBase::GnCall(g.clone()),
                                    ],
                                )],
                            },
                            optsemicolon: ";".to_string(),
                        };
                        let res = CType::from_ast(scope, &parse_type, false)?;
                        scope = res.0;
                        let t = res.1;
                        // Try the type alias name first (e.g. TreeInner_T_), then fall back to
                        // the callable string of the expanded type (e.g. Tuple_Field_valsL_...)
                        let callable_name = Arc::new(t.clone()).to_callable_string();
                        let temp_scope = scope.child();
                        let res = temp_scope.resolve_function(&name, &arg_types);
                        let res = match res {
                            Some((s, f)) => Some((s, f)),
                            None => {
                                let temp_scope2 = scope.child();
                                temp_scope2.resolve_function(&callable_name, &arg_types)
                            }
                        };
                        match res {
                            Some((mut temp_scope, func)) => {
                                temp_scope.functions.insert(f.clone(), vec![func.clone()]);
                                merge!(scope, temp_scope);
                                let res = CType::from_ast(scope, &parse_type, false)?; // TODO: Remove this
                                                                                       // duplicate
                                scope = res.0;
                                prior_value = Some(Microstatement::FnCall {
                                    args: narrow_call_arg_literals(&func, arg_microstatements),
                                    function: func.clone(), // TODO: Drop the clone?
                                })
                            }
                            None => {
                                let arg_str = arg_types
                                    .iter()
                                    .map(|a| a.clone().to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                return Err(format!(
                                    "A function with the signature {}({}) or {}({}) does not exist",
                                    name, arg_str, callable_name, arg_str
                                )
                                .into());
                            }
                        }
                    }
                }
            }
            BaseChunk::IIGE(_prior, _f, _g, _h) => {
                // TODO: This may similarly be just some simple microstatement generation here
                return Err("TODO: Implement IIGE support".into());
            }
        }
    }
    // Push the generated statement that *probably* exists into the microstatements array
    if let Some(prior) = prior_value {
        microstatements.push(prior);
    }
    Ok((scope, microstatements))
}

pub fn withoperatorslist_to_microstatements<'a>(
    withoperatorslist: &Vec<parse::WithOperators>,
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    // To properly linearize the operations here, we need to scan through all of the operators,
    // determine which is the highest precedence, whether it is infix or prefix (or maybe postfix
    // in the future?) and then process them and whichever of the baseassignables surrounding them
    // are associated, then put those results in the same "slot" as before and check again. Because
    // users can define these operators, that makes it theoretically possible for the same operator
    // to be used in both an infix or prefix manner, or with different precedence levels, depending
    // on the types of the data involved, which makes things *really* complicated here.
    let mut queue = withoperatorslist.clone();
    while !queue.is_empty() {
        let mut largest_operator_level: i8 = -1;
        let mut largest_operator_index: i64 = -1;
        let mut op = None;
        for (i, assignable_or_operator) in queue.iter().enumerate() {
            // This can sometimes be ambiguous on the symbol, `-` is both an infix subtract and a
            // prefix negate operation, and they have different precedence levels. If and only if
            // it might have the highest precedence do we check if it could reasonably resolve in
            // that way. (For a prefix, there must either be nothing before it or what's before it
            // needs to be an operator and what's after it must be an assignable, for a postfix
            // there must be nothing after it or what's after it is an operator and what's before
            // it is an assignable, and for an infix there must be an assignable before and after
            // it.) If it doesn't match those criteria we skip over that possibility and move on to
            // others.
            if let parse::WithOperators::Operators(o) = assignable_or_operator {
                let operatorname = o.trim();
                let prefix_op = scope.resolve_operator(&format!("prefix{operatorname}"));
                let infix_op = scope.resolve_operator(&format!("infix{operatorname}"));
                let postfix_op = scope.resolve_operator(&format!("postfix{operatorname}"));
                let mut level = -1;
                let mut operator = None;
                for local_op in [&prefix_op, &infix_op, &postfix_op] {
                    let local_level = match local_op {
                        Some(o) => match o {
                            OperatorMapping::Prefix { level, .. } => {
                                match (queue.get(i.wrapping_sub(1)), queue.get(i.wrapping_add(1))) {
                                    (None, Some(parse::WithOperators::BaseAssignableList(_)))
                                    | (
                                        Some(parse::WithOperators::Operators(_)),
                                        Some(parse::WithOperators::BaseAssignableList(_)),
                                    ) => *level,
                                    _ => -1,
                                }
                            }
                            OperatorMapping::Infix { level, .. } => {
                                match (queue.get(i.wrapping_sub(1)), queue.get(i.wrapping_add(1))) {
                                    (
                                        Some(parse::WithOperators::BaseAssignableList(_)),
                                        Some(parse::WithOperators::BaseAssignableList(_)),
                                    ) => *level,
                                    _ => -1,
                                }
                            }
                            OperatorMapping::Postfix { level, .. } => {
                                match (queue.get(i.wrapping_sub(1)), queue.get(i.wrapping_add(1))) {
                                    (Some(parse::WithOperators::BaseAssignableList(_)), None)
                                    | (
                                        Some(parse::WithOperators::BaseAssignableList(_)),
                                        Some(parse::WithOperators::Operators(_)),
                                    ) => *level,
                                    _ => -1,
                                }
                            }
                        },
                        _ => -1,
                    };
                    if local_level > level {
                        level = local_level;
                        operator = *local_op;
                    }
                }
                if level > largest_operator_level {
                    largest_operator_level = level;
                    largest_operator_index = i as i64;
                    op = operator;
                }
            }
        }
        if largest_operator_index > -1 {
            let operator = op.unwrap(); // Should be guaranteed to exist
            let functionname = match operator {
                OperatorMapping::Prefix { functionname, .. } => functionname.clone(),
                OperatorMapping::Infix { functionname, .. } => functionname.clone(),
                OperatorMapping::Postfix { functionname, .. } => functionname.clone(),
            };
            let is_infix = match operator {
                OperatorMapping::Prefix { .. } => false,
                OperatorMapping::Postfix { .. } => false,
                OperatorMapping::Infix { .. } => true,
            };
            let is_prefix = match operator {
                OperatorMapping::Prefix { .. } => true,
                OperatorMapping::Postfix { .. } => false,
                OperatorMapping::Infix { .. } => false,
            };
            if is_infix {
                // Confirm that we have records before and after the operator and that they are
                // baseassignables.
                let first_arg = match match queue.get(largest_operator_index as usize - 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {} is an infix operator but missing a left-hand side value",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        }
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => {
                        Ok(baseassignablelist)
                    }
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is an infix operator but preceded by another operator {}",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                        o
                    )),
                }?;
                let second_arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!("Operator {} is an infix operator but missing a right-hand side value",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        })),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => Ok(baseassignablelist),
                    parse::WithOperators::Operators(o) => Err(format!("Operator{} is an infix operator but followed by a lower precedence operator {}",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        }, o)),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `a + b` and turn it into `add(a, b)`
                let rewrite = parse::WithOperators::BaseAssignableList(vec![
                    parse::BaseAssignable::Variable(functionname),
                    parse::BaseAssignable::FnCall(parse::FnCall {
                        openparen: "(".to_string(),
                        a: "".to_string(),
                        assignablelist: parse::AssignableList {
                            elements: vec![
                                vec![parse::WithOperators::BaseAssignableList(first_arg.to_vec())],
                                vec![parse::WithOperators::BaseAssignableList(
                                    second_arg.to_vec(),
                                )],
                            ],
                            separators: vec![],
                        },
                        b: "".to_string(),
                        closeparen: ")".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithOperators> = queue
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
                        "Operator {} is a prefix operator but missing a right-hand side value",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => {
                        Ok(baseassignablelist)
                    }
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is an prefix operator but followed by another operator {}",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                        o
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `#array` and turn it into `len(array)`
                let rewrite = parse::WithOperators::BaseAssignableList(vec![
                    parse::BaseAssignable::Variable(functionname),
                    parse::BaseAssignable::FnCall(parse::FnCall {
                        openparen: "(".to_string(),
                        a: "".to_string(),
                        assignablelist: parse::AssignableList {
                            elements: vec![vec![parse::WithOperators::BaseAssignableList(
                                arg.to_vec(),
                            )]],
                            separators: vec![],
                        },
                        b: "".to_string(),
                        closeparen: ")".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithOperators> = queue
                    .splice(
                        (largest_operator_index as usize)..(largest_operator_index as usize + 2),
                        vec![rewrite],
                    )
                    .collect();
            } else {
                let arg = match match queue.get(largest_operator_index as usize - 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {} is a postfix operator but missing a left-hand side value",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(bal) => Ok(bal),
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is a postfix operator but preceded by another operator {}",
                        match operator {
                            OperatorMapping::Prefix { operatorname, .. }
                            | OperatorMapping::Infix { operatorname, .. }
                            | OperatorMapping::Postfix { operatorname, .. } => operatorname,
                        },
                        o
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `var?` and turn it into `Maybe(var)`
                let rewrite = parse::WithOperators::BaseAssignableList(vec![
                    parse::BaseAssignable::Variable(functionname),
                    parse::BaseAssignable::FnCall(parse::FnCall {
                        openparen: "(".to_string(),
                        a: "".to_string(),
                        assignablelist: parse::AssignableList {
                            elements: vec![vec![parse::WithOperators::BaseAssignableList(
                                arg.to_vec(),
                            )]],
                            separators: vec![],
                        },
                        b: "".to_string(),
                        closeparen: ")".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithOperators> = queue
                    .splice(
                        (largest_operator_index as usize - 1)
                            ..(largest_operator_index as usize + 1),
                        vec![rewrite],
                    )
                    .collect();
            }
        } else {
            // We have no more operators, there should only be one reworked baseassignablelist now
            if queue.len() != 1 {
                // No idea how such a wonky thing could occur. TODO: Improve error message
                return Err(format!("Invalid syntax: {withoperatorslist:?}").into());
            }
            let baseassignablelist = match match queue.pop() {
                Some(v) => Ok(v),
                None => Err(format!(
                    "Somehow we collapsed the statement into nothing? {withoperatorslist:?}"
                )),
            }? {
                parse::WithOperators::BaseAssignableList(b) => Ok(b),
                _ => Err(format!(
                    "Somehow we collapse the statement into a solitary operator? {withoperatorslist:?}"
                )),
            }?;
            let res = baseassignablelist_to_microstatements(
                &baseassignablelist,
                parent_fn,
                scope,
                microstatements,
            )?;
            scope = res.0;
            microstatements = res.1;
        }
    }
    Ok((scope, microstatements))
}

pub fn assignablestatement_to_microstatements<'a>(
    assignable: &parse::AssignableStatement,
    parent_fn: Option<&Function>,
    scope: Scope<'a>,
    microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    let res = withoperatorslist_to_microstatements(
        &assignable.assignables,
        parent_fn,
        scope,
        microstatements,
    )?;
    Ok(res)
}

pub fn returns_to_microstatements<'a>(
    returns: &parse::Returns,
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    if let Some(retval) = &returns.retval {
        // We get all of the microstatements involved in the return statement, then we pop
        // off the last one, if any exists, to get the final return value. Then we shove
        // the other microstatements into the array and the new Return microstatement with
        // that last value attached to it.
        let res = withoperatorslist_to_microstatements(
            &retval.assignables,
            parent_fn,
            scope,
            microstatements,
        )?;
        scope = res.0;
        microstatements = res.1;
        let value = microstatements.pop().map(Box::new);
        microstatements.push(Microstatement::Return { value });
    } else {
        microstatements.push(Microstatement::Return { value: None });
    }
    Ok((scope, microstatements))
}

pub fn declarations_to_microstatements<'a>(
    declarations: &parse::Declarations,
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    let (name, assignables, mutable, typedec) = match &declarations {
        parse::Declarations::Const(c) => (c.variable.clone(), &c.assignables, false, &c.typedec),
        parse::Declarations::Let(l) => (l.variable.clone(), &l.assignables, true, &l.typedec),
    };
    // Get all of the assignable microstatements generated
    let res = withoperatorslist_to_microstatements(assignables, parent_fn, scope, microstatements)?;
    scope = res.0;
    microstatements = res.1;
    let mut value = match microstatements.pop() {
        None => Err("An assignment without a value should be impossible."),
        Some(v) => Ok(v),
    }?;
    // If the declaration carries an explicit type annotation (`let x: u64 = ...`) and the value is
    // a numeric literal whose `AnyOf` candidate set includes that type, narrow the literal to it.
    // This makes e.g. `let big: u64 = 18446744073709551615` produce a `u64` constant directly
    // (no `i64` intermediate, no runtime cast) and is what lets above-`i64::MAX` constants compile.
    if let Some(td) = typedec {
        let target_name = td.fulltypename.to_string();
        if let Some(target) = scope.resolve_type(&target_name) {
            value = narrow_numeric_literal(value, target);
        }
    } else if let Microstatement::Value { typen, .. } = &value {
        // No annotation: pin an unconstrained numeric-literal `AnyOf` to its global FUI default now,
        // so the bound variable has a single concrete type. A later reference to this variable must
        // not be re-narrowed (its storage type is fixed here), so we collapse rather than leave the
        // `AnyOf` open. (A direct literal *argument* is handled separately, at the call site.)
        if matches!(&**typen, CType::AnyOf(_)) {
            let collapsed = typen.clone().collapse_anyof_default();
            if !matches!(&*collapsed, CType::AnyOf(_)) {
                value = narrow_numeric_literal(value, collapsed);
            }
        }
    }
    microstatements.push(Microstatement::Assignment {
        name,
        value: Box::new(value),
        mutable,
    });
    Ok((scope, microstatements))
}

pub fn statement_to_microstatements<'a>(
    statement: &parse::Statement,
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    match statement {
        // This is just whitespace, so we do nothing here
        parse::Statement::A(_) => Ok((scope, microstatements)),
        parse::Statement::Declarations(declarations) => Ok(declarations_to_microstatements(
            declarations,
            parent_fn,
            scope,
            microstatements,
        )?),
        parse::Statement::ArrayAssignment(arrayassignment) => {
            let mut args = Vec::new();
            let res = baseassignablelist_to_microstatements(
                std::slice::from_ref(&arrayassignment.name),
                parent_fn,
                scope,
                microstatements,
            )?;
            scope = res.0;
            let mut ms = res.1;
            args.push(ms.pop().unwrap());
            for arg in &arrayassignment.array.assignablelist {
                let res = withoperatorslist_to_microstatements(arg, parent_fn, scope, ms)?;
                scope = res.0;
                ms = res.1;
                args.push(ms.pop().unwrap());
            }
            let res = withoperatorslist_to_microstatements(
                &arrayassignment.assignables,
                parent_fn,
                scope,
                ms,
            )?;
            scope = res.0;
            ms = res.1;
            args.push(ms.pop().unwrap());
            let arg_types = args
                .iter()
                .map(|a| a.get_type())
                .collect::<Vec<Arc<CType>>>();
            let store_fn = {
                // TODO: Do we really need this temp_scope?
                let temp_scope = scope.child();
                match temp_scope.resolve_function(&"store".to_string(), &arg_types) {
                    Some((s, f)) => {
                        merge!(scope, s);
                        Ok(f)
                    }
                    None => Err(format!(
                        "Could not find store function with arguments {}",
                        arg_types
                            .iter()
                            .map(|a| a.clone().to_strict_string(false))
                            .collect::<Vec<String>>()
                            .join(", "),
                    )),
                }?
            };
            ms.push(Microstatement::FnCall {
                args: narrow_call_arg_literals(&store_fn, args),
                function: store_fn,
            });
            Ok((scope, ms))
        }
        parse::Statement::Assignables(assignable) => Ok(assignablestatement_to_microstatements(
            assignable,
            parent_fn,
            scope,
            microstatements,
        )?),
        parse::Statement::Returns(returns) => Ok(returns_to_microstatements(
            returns,
            parent_fn,
            scope,
            microstatements,
        )?),
        // Conditionals are tail-aware and must be driven by `statements_to_microstatements`,
        // which hands them the remaining statements of the enclosing block. Reaching this arm
        // directly means a block loop wasn't routed through the driver.
        parse::Statement::Conditional(_) => {
            Err("Conditional statements must be processed via statements_to_microstatements".into())
        }
    }
}

/// Tail-aware block driver. Processes a slice of statements, and on hitting a `Conditional` hands
/// the conditional the remaining statements (the *tail*) and stops -- the conditional consumes the
/// rest of the block by folding the tail into its branch closures.
pub fn statements_to_microstatements<'a>(
    statements: &[parse::Statement],
    parent_fn: Option<&Function>,
    mut scope: Scope<'a>,
    mut microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    for (i, statement) in statements.iter().enumerate() {
        if let parse::Statement::Conditional(conditional) = statement {
            let tail = &statements[i + 1..];
            return conditional_to_microstatements(
                conditional,
                tail,
                parent_fn,
                scope,
                microstatements,
            );
        }
        let res = statement_to_microstatements(statement, parent_fn, scope, microstatements)?;
        scope = res.0;
        microstatements = res.1;
    }
    Ok((scope, microstatements))
}

/// Extract the statement list from a `Blocklike`. Only the `{ ... }` (FunctionBody) form is
/// supported as a conditional branch body; the bare-function form is not.
fn blocklike_statements(blocklike: &parse::Blocklike) -> Option<Vec<parse::Statement>> {
    match blocklike {
        parse::Blocklike::FunctionBody(body) => Some(body.statements.clone()),
        parse::Blocklike::Functions(_) => None,
    }
}

/// The last statement in a block that isn't pure whitespace.
fn last_meaningful(stmts: &[parse::Statement]) -> Option<&parse::Statement> {
    stmts
        .iter()
        .rev()
        .find(|s| !matches!(s, parse::Statement::A(_)))
}

/// Does every control-flow path through this block end in a `return`? True when the last
/// meaningful statement is a `Returns`, or a `Conditional` with an else where both arms return.
fn block_returns(stmts: &[parse::Statement]) -> bool {
    match last_meaningful(stmts) {
        Some(parse::Statement::Returns(_)) => true,
        Some(parse::Statement::Conditional(c)) => conditional_returns(c, false),
        _ => false,
    }
}

/// Like `block_returns`, but additionally requires the terminal returns to carry a *value* (so the
/// conditional is value-producing, and the synthesized call should be wrapped in `return`).
fn block_returns_value(stmts: &[parse::Statement]) -> bool {
    match last_meaningful(stmts) {
        Some(parse::Statement::Returns(r)) => r.retval.is_some(),
        Some(parse::Statement::Conditional(c)) => conditional_returns(c, true),
        _ => false,
    }
}

/// Whether a conditional returns on all paths: it must have an else, and both the then-arm and the
/// else-arm (recursively, for `else if`) must return. When `value` is set, the returns must also
/// carry values.
fn conditional_returns(c: &parse::Conditional, value: bool) -> bool {
    let then_stmts = match blocklike_statements(&c.blocklike) {
        Some(s) => s,
        None => return false,
    };
    let then_ok = if value {
        block_returns_value(&then_stmts)
    } else {
        block_returns(&then_stmts)
    };
    if !then_ok {
        return false;
    }
    match &c.optelsebranch {
        None => false,
        Some(eb) => match &*eb.condorblock {
            parse::CondOrBlock::Conditional(inner) => conditional_returns(inner, value),
            parse::CondOrBlock::Blocklike(b) => match blocklike_statements(b) {
                Some(s) => {
                    if value {
                        block_returns_value(&s)
                    } else {
                        block_returns(&s)
                    }
                }
                None => false,
            },
        },
    }
}

/// Append a block's tail to a branch body. If the branch already returns on all paths the tail is
/// unreachable from it and the body is returned unchanged; otherwise the tail is concatenated. Any
/// nested trailing conditional in the body is handled naturally when the resulting (re-parsed)
/// branch closure is itself driven through `statements_to_microstatements`, which stops at that
/// conditional and hands it the concatenated tail.
fn append_tail(
    mut block: Vec<parse::Statement>,
    tail: &[parse::Statement],
) -> Vec<parse::Statement> {
    if block_returns(&block) {
        return block;
    }
    block.extend(tail.iter().cloned());
    block
}

/// If `src` is a simple reassignment `<ident> = <rhs>` (where the `=` is the store operator, not a
/// `==`/`=>`/`<=`/`>=`/`!=` comparison -- the leading-identifier rule already rules out the latter
/// three), return the identifier and the right-hand-side source. Used to recognize the pure
/// conditional-assignment shape that can be lowered as a value `if` rather than a tail fold.
fn split_simple_assignment(src: &str) -> Option<(String, String)> {
    let s = src.trim();
    let b = s.as_bytes();
    let mut i = 0;
    if i >= b.len() || !(b[i].is_ascii_alphabetic() || b[i] == b'_') {
        return None;
    }
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    let ident = s[0..i].to_string();
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() || b[i] != b'=' {
        return None;
    }
    if i + 1 < b.len() && (b[i + 1] == b'=' || b[i + 1] == b'>') {
        return None;
    }
    let rhs = s[i + 1..].trim().to_string();
    if rhs.is_empty() {
        return None;
    }
    Some((ident, rhs))
}

/// If every meaningful statement in `stmts` is a simple reassignment `<ident> = <rhs>` and no
/// identifier is assigned more than once, return them in order. Returns `None` for any branch that
/// does something else (a `let`, a `return`, a nested conditional, a bare call, or a repeated
/// assignment to the same variable), so such branches fall back to the tail-folding lowering.
fn collect_branch_assignments(stmts: &[parse::Statement]) -> Option<Vec<(String, String)>> {
    let mut out: Vec<(String, String)> = Vec::new();
    for s in stmts {
        match s {
            parse::Statement::A(_) => continue,
            parse::Statement::Assignables(a) => {
                let (var, rhs) = split_simple_assignment(&a.assignables.render())?;
                if out.iter().any(|(v, _)| v == &var) {
                    // A variable reassigned twice in one branch can't be captured by a single
                    // per-variable phi; leave it to the fold.
                    return None;
                }
                out.push((var, rhs));
            }
            _ => return None,
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Detect a *pure conditional assignment* conditional: one whose then-branch (and, recursively,
/// every `else if`/`else` branch) consists solely of simple reassignments to already-bound
/// variables (each at most once per branch). Returns, for every variable assigned anywhere in the
/// chain (in first-assignment order), a synthesized nested value-`if` expression for its post-
/// conditional value. A branch that doesn't assign a given variable -- including a missing final
/// `else` -- leaves it unchanged (the arm yields the variable's prior value).
///
/// Each variable is then re-bound by `conditional_to_microstatements` with a shadowing
/// `let <var> = <if-expr>`, rather than folding the block tail into both arms. This is what makes
/// natural imperative conditional mutation work in *GPU* code: each `let <var> = if(...)` lowers
/// through the GPU closure-`if` into a real `var <var>; if (c) { <var> = ...; } else { ... }` WGSL
/// block, so the un-folded tail simply reads the new bindings. The fold instead makes each arm
/// return the whole `GPGPU[]` shader (which the GPU `if` cannot combine) and, for plain
/// reassignments, silently drops them (the GPU scalar `store` has no assignable lvalue). On the CPU
/// it lowers to the same native control flow, just without duplicating the tail.
fn conditional_assignments(c: &parse::Conditional) -> Option<Vec<(String, String)>> {
    let then_stmts = blocklike_statements(&c.blocklike)?;
    let then_assigns = collect_branch_assignments(&then_stmts)?;
    let cond_src = c.assignables.render();
    // Each entry is a variable's *already-synthesized* arm expression for the else side: a nested
    // `if(...)` (for `else if`), a plain rhs (for a final `else`), or absent (no `else`).
    let else_assigns: Vec<(String, String)> = match &c.optelsebranch {
        None => Vec::new(),
        Some(eb) => match &*eb.condorblock {
            parse::CondOrBlock::Conditional(inner) => conditional_assignments(inner)?,
            parse::CondOrBlock::Blocklike(b) => {
                let else_stmts = blocklike_statements(b)?;
                collect_branch_assignments(&else_stmts)?
            }
        },
    };
    // Union of all assigned variables, preserving first-assignment order (then-branch first). A
    // variable's arm defaults to its own prior value when an arm doesn't assign it.
    let mut vars: Vec<String> = Vec::new();
    for (v, _) in then_assigns.iter().chain(else_assigns.iter()) {
        if !vars.iter().any(|x| x == v) {
            vars.push(v.clone());
        }
    }
    let arm = |assigns: &[(String, String)], var: &str| -> Option<String> {
        assigns
            .iter()
            .find(|(v, _)| v == var)
            .map(|(_, e)| e.clone())
    };
    let mut result = Vec::new();
    for var in &vars {
        let then_e = clone_if_bare_ident(arm(&then_assigns, var).unwrap_or_else(|| var.clone()));
        let else_e = clone_if_bare_ident(arm(&else_assigns, var).unwrap_or_else(|| var.clone()));
        result.push((
            var.clone(),
            format!("if({cond_src}, fn() {{ return {then_e}; }}, fn() {{ return {else_e}; }})"),
        ));
    }
    Some(result)
}

/// A branch arm that is just a bare variable reference (`<ident>`) -- which happens for a variable
/// an arm leaves unchanged (it yields its prior value) or a plain `a = b` copy -- must `.clone()`
/// the binding: the synthesized arm closure is a reusable `Fn`, so returning the captured variable
/// directly would move out of it. Non-trivial expressions build fresh values and are left alone.
fn clone_if_bare_ident(expr: String) -> String {
    let t = expr.trim();
    let is_bare_ident = !t.is_empty()
        && t.bytes().enumerate().all(|(i, b)| {
            if i == 0 {
                b.is_ascii_alphabetic() || b == b'_'
            } else {
                b.is_ascii_alphanumeric() || b == b'_'
            }
        });
    if is_bare_ident {
        format!("{t}.clone()")
    } else {
        expr
    }
}

/// Transform of a block-level `if`/`else` conditional (plus the enclosing block's tail) into a
/// synthesized `if(<cond>, fn() = {then}, fn() = {else})` call that runs through the normal
/// resolution machinery, landing on the realized `cfn if{T}`.
fn conditional_to_microstatements<'a>(
    conditional: &parse::Conditional,
    tail: &[parse::Statement],
    parent_fn: Option<&Function>,
    scope: Scope<'a>,
    microstatements: Vec<Microstatement>,
) -> Result<(Scope<'a>, Vec<Microstatement>), Box<dyn std::error::Error>> {
    // A pure conditional-assignment conditional (every branch only reassigns already-bound
    // variables) is lowered as one value-producing `let <var> = if(...)` per assigned variable,
    // each shadowing its prior binding, rather than by folding the block tail into both arms. This
    // is what makes natural conditional mutation work in GPU shaders (see `conditional_assignments`);
    // on the CPU it is equivalent to the fold but avoids duplicating the tail.
    if let Some(assignments) = conditional_assignments(conditional) {
        // Re-bind each conditionally-assigned variable to its post-conditional value with a
        // shadowing `let`, rather than folding the block tail into both arms. Each fresh binding's
        // value carries the conditional logic (on GPU, the closure-`if`'s `var x; if (c) { x = ...;
        // }` block), so the un-folded tail simply reads the new bindings. Bindings are emitted in
        // first-assignment order so an arm referencing an earlier-assigned variable sees its
        // updated value. Rust shadows natively; the JS backend renders a re-declaration of an
        // existing name as a plain reassignment (see `from_microstatement`).
        let mut scope = scope;
        let mut microstatements = microstatements;
        for (var_name, if_expr) in assignments {
            let decl_src = format!("let {var_name} = {if_expr};");
            let parsed = match parse::statement(&decl_src) {
                Ok((rem, stmt)) if rem.trim().is_empty() => stmt,
                _ => {
                    return Err(format!(
                        "Failed to synthesize a conditional assignment from:\n{decl_src}"
                    )
                    .into())
                }
            };
            let res = statement_to_microstatements(&parsed, parent_fn, scope, microstatements)?;
            scope = res.0;
            microstatements = res.1;
        }
        return statements_to_microstatements(tail, parent_fn, scope, microstatements);
    }

    let cond_src = conditional.assignables.render();
    let then_stmts = match blocklike_statements(&conditional.blocklike) {
        Some(s) => s,
        None => {
            return Err("Only `{ ... }` block bodies are supported for conditional branches".into())
        }
    };
    // Normalize the else branch: an `else if` becomes a single-statement else block holding the
    // inner conditional. A plain else block contributes its statements. No else stays `None`.
    let else_stmts: Option<Vec<parse::Statement>> = match &conditional.optelsebranch {
        None => None,
        Some(eb) => match &*eb.condorblock {
            parse::CondOrBlock::Conditional(inner) => {
                Some(vec![parse::Statement::Conditional(inner.clone())])
            }
            parse::CondOrBlock::Blocklike(b) => match blocklike_statements(b) {
                Some(s) => Some(s),
                None => {
                    return Err(
                        "Only `{ ... }` block bodies are supported for conditional branches".into(),
                    )
                }
            },
        },
    };

    let has_explicit_else = else_stmts.is_some();
    let then_returns = block_returns(&then_stmts);
    let else_returns = else_stmts.as_ref().is_some_and(|s| block_returns(s));
    let tail_nonempty = last_meaningful(tail).is_some();

    // Build the final then/else branch bodies, folding in the tail.
    let mut final_then = then_stmts;
    let mut final_else = else_stmts;
    if tail_nonempty {
        if has_explicit_else && then_returns && else_returns {
            return Err(
                "Unreachable statements after a conditional in which both branches return".into(),
            );
        }
        if !then_returns {
            final_then = append_tail(final_then, tail);
        }
        match final_else {
            Some(e) => {
                if !else_returns {
                    final_else = Some(append_tail(e, tail));
                } else {
                    final_else = Some(e);
                }
            }
            // No explicit else: synthesize one from the tail so the condition-false path still
            // runs the rest of the block (the accepted duplication).
            None => final_else = Some(tail.to_vec()),
        }
    }

    // The conditional is value-producing (and so wrapped in `return`) when both arms terminally
    // return a value. A void conditional is emitted as a bare statement.
    let value_producing = block_returns_value(&final_then)
        && final_else.as_ref().is_some_and(|s| block_returns_value(s));

    let then_src = final_then.render();
    let else_src = final_else.as_ref().map(|s| s.render()).unwrap_or_default();
    let call_src = format!("if({cond_src}, fn() {{{then_src}}}, fn() {{{else_src}}})");
    let stmt_src = if value_producing {
        format!("return {call_src};")
    } else {
        format!("{call_src};")
    };

    let parsed = match parse::statement(&stmt_src) {
        Ok((rem, stmt)) if rem.trim().is_empty() => stmt,
        _ => {
            return Err(
                format!("Failed to synthesize a valid conditional from:\n{stmt_src}").into(),
            )
        }
    };
    statement_to_microstatements(&parsed, parent_fn, scope, microstatements)
}
