use std::collections::HashMap;
use std::sync::Arc;

use super::canonicalize_inferred_generic_type;
use super::withtypeoperatorslist_to_ctype;
use super::CType;
use super::Scope;
use crate::parse;
use crate::program::ArgKind;

impl CType {
    pub fn has_infer(self: Arc<CType>) -> bool {
        match &*self {
            CType::Void
            | CType::DerivedVoid(..)
            | CType::IntrinsicGeneric(..)
            | CType::Int(_)
            | CType::Float(_)
            | CType::Bool(_)
            | CType::TString(_)
            | CType::Fail(_) => false,
            CType::Infer(..) => true,
            CType::Type(_, t)
            | CType::Generic(_, _, t)
            | CType::IntCast(t)
            | CType::FloatCast(t)
            | CType::BoolCast(t)
            | CType::StringCast(t)
            | CType::Group(t)
            | CType::Unwrap(t)
            | CType::Infix(t)
            | CType::Prefix(t)
            | CType::Method(t)
            | CType::Property(t)
            | CType::Cast(t)
            | CType::Own(t)
            | CType::Deref(t)
            | CType::Mut(t)
            | CType::Rust(t)
            | CType::Nodejs(t)
            | CType::From(t)
            | CType::Array(t)
            | CType::Field(_, t)
            | CType::Shared(t)
            | CType::Promise(t)
            | CType::Neg(t)
            | CType::Len(t)
            | CType::Size(t)
            | CType::FileStr(t)
            | CType::EnvExists(t)
            | CType::Not(t) => t.clone().has_infer(),
            CType::Binds(t, ts) | CType::TIf(t, ts) => {
                t.clone().has_infer() || ts.iter().any(|t| t.clone().has_infer())
            }
            CType::Function(a, b)
            | CType::Call(a, b)
            | CType::Dependency(a, b)
            | CType::Import(a, b)
            | CType::Prop(a, b)
            | CType::Exclude(a, b)
            | CType::Buffer(a, b)
            | CType::Concat(a, b) => a.clone().has_infer() || b.clone().has_infer(),
            CType::Tuple(ts, _)
            | CType::Either(ts, _)
            | CType::AnyOf(ts)
            | CType::Add(ts)
            | CType::Sub(ts)
            | CType::Mul(ts)
            | CType::Div(ts)
            | CType::Mod(ts)
            | CType::Pow(ts)
            | CType::Min(ts)
            | CType::Max(ts)
            | CType::Env(ts)
            | CType::And(ts)
            | CType::Or(ts)
            | CType::Xor(ts)
            | CType::Nand(ts)
            | CType::Nor(ts)
            | CType::Xnor(ts)
            | CType::TEq(ts)
            | CType::Neq(ts)
            | CType::Lt(ts)
            | CType::Lte(ts)
            | CType::Gt(ts)
            | CType::Gte(ts) => ts.iter().any(|t| t.clone().has_infer()),
        }
    }

    pub fn degroup(self: Arc<CType>) -> Arc<CType> {
        match &*self {
            CType::Void | CType::DerivedVoid(..) => self,
            CType::Infer(..) => self,
            CType::Type(n, t) => Arc::new(CType::Type(n.clone(), t.clone().degroup())),
            CType::Generic(..) => self,
            CType::Binds(n, ts) => Arc::new(CType::Binds(
                n.clone().degroup(),
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Shared(t) => Arc::new(CType::Shared(t.clone().degroup())),
            CType::Promise(t) => Arc::new(CType::Promise(t.clone().degroup())),
            CType::IntrinsicGeneric(..) => self,
            CType::IntCast(t) => Arc::new(CType::IntCast(t.clone().degroup())),
            CType::Int(_) => self,
            CType::FloatCast(t) => Arc::new(CType::FloatCast(t.clone().degroup())),
            CType::Float(_) => self,
            CType::BoolCast(t) => Arc::new(CType::BoolCast(t.clone().degroup())),
            CType::Bool(_) => self,
            CType::StringCast(t) => Arc::new(CType::StringCast(t.clone().degroup())),
            CType::TString(_) => self,
            CType::Group(t) => t.clone().degroup(),
            CType::Unwrap(t) => Arc::new(CType::Unwrap(t.clone().degroup())),
            CType::Function(i, o) => {
                Arc::new(CType::Function(i.clone().degroup(), o.clone().degroup()))
            }
            CType::Call(n, f) => Arc::new(CType::Call(n.clone().degroup(), f.clone().degroup())),
            CType::Infix(o) => Arc::new(CType::Infix(o.clone().degroup())),
            CType::Prefix(o) => Arc::new(CType::Prefix(o.clone().degroup())),
            CType::Method(f) => Arc::new(CType::Method(f.clone().degroup())),
            CType::Property(p) => Arc::new(CType::Property(p.clone().degroup())),
            CType::Cast(t) => Arc::new(CType::Cast(t.clone().degroup())),
            CType::Own(t) => Arc::new(CType::Own(t.clone().degroup())),
            CType::Deref(t) => Arc::new(CType::Deref(t.clone().degroup())),
            CType::Mut(t) => Arc::new(CType::Mut(t.clone().degroup())),
            CType::Dependency(n, v) => {
                Arc::new(CType::Dependency(n.clone().degroup(), v.clone().degroup()))
            }
            CType::Rust(d) => Arc::new(CType::Rust(d.clone().degroup())),
            CType::Nodejs(d) => Arc::new(CType::Nodejs(d.clone().degroup())),
            CType::From(d) => Arc::new(CType::From(d.clone().degroup())),
            CType::Import(n, d) => {
                Arc::new(CType::Import(n.clone().degroup(), d.clone().degroup()))
            }
            CType::Tuple(ts, parents) => Arc::new(CType::Tuple(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
                parents
                    .iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Field(l, t) => Arc::new(CType::Field(l.clone(), t.clone().degroup())),
            CType::Either(ts, parents) => Arc::new(CType::Either(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
                parents
                    .iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Prop(t, p) => Arc::new(CType::Prop(t.clone().degroup(), p.clone().degroup())),
            CType::Exclude(t, p) => {
                Arc::new(CType::Exclude(t.clone().degroup(), p.clone().degroup()))
            }
            CType::AnyOf(ts) => Arc::new(CType::AnyOf(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Buffer(t, s) => {
                Arc::new(CType::Buffer(t.clone().degroup(), s.clone().degroup()))
            }
            CType::Array(t) => Arc::new(CType::Array(t.clone().degroup())),
            CType::Fail(_) => self,
            CType::Add(ts) => Arc::new(CType::Add(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Sub(ts) => Arc::new(CType::Sub(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Mul(ts) => Arc::new(CType::Mul(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Div(ts) => Arc::new(CType::Div(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Mod(ts) => Arc::new(CType::Mod(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Pow(ts) => Arc::new(CType::Pow(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Min(ts) => Arc::new(CType::Min(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Max(ts) => Arc::new(CType::Max(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Neg(t) => Arc::new(CType::Neg(t.clone().degroup())),
            CType::Len(t) => Arc::new(CType::Len(t.clone().degroup())),
            CType::Size(t) => Arc::new(CType::Size(t.clone().degroup())),
            CType::FileStr(t) => Arc::new(CType::FileStr(t.clone().degroup())),
            CType::Concat(a, b) => {
                Arc::new(CType::Concat(a.clone().degroup(), b.clone().degroup()))
            }
            CType::Env(ts) => Arc::new(CType::Env(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::EnvExists(t) => Arc::new(CType::EnvExists(t.clone().degroup())),
            CType::TIf(t, ts) => Arc::new(CType::TIf(
                t.clone().degroup(),
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::And(ts) => Arc::new(CType::And(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Or(ts) => Arc::new(CType::Or(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Xor(ts) => Arc::new(CType::Xor(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Not(t) => Arc::new(CType::Not(t.clone().degroup())),
            CType::Nand(ts) => Arc::new(CType::Nand(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Nor(ts) => Arc::new(CType::Nor(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Xnor(ts) => Arc::new(CType::Xnor(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::TEq(ts) => Arc::new(CType::TEq(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Neq(ts) => Arc::new(CType::Neq(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Lt(ts) => Arc::new(CType::Lt(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Lte(ts) => Arc::new(CType::Lte(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Gt(ts) => Arc::new(CType::Gt(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Gte(ts) => Arc::new(CType::Gte(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
        }
    }
    // Given a list of generic type names, a list of argument types provided, and the original type
    // ast of the function, we can infer the generic type mappings by creating a temporary child
    // scope, creating `Infer` CTypes for each generic name and parsing the type ast inside of that
    // scope, then for each input record for the function traverse the tree of the input argument
    // type *and* the created CType tree of the same argument index and we can either error out if
    // they don't match, or reach the `Infer` type on the new tree and assign the corresponding
    // sub-tree of the provided type to the generic in question. If we get a sub-tree for all
    // generic type names, we succeed, otherwise we have to fail on being unable to resolve
    // specific generics.
    pub fn infer_generics_inner_loop(
        scope: &Scope,
        generic_types: &mut HashMap<String, Arc<CType>>,
        arg_type_vec: Vec<(Arc<CType>, Arc<CType>)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (a, i) in arg_type_vec {
            let mut arg = vec![a];
            let mut input = vec![i];
            while let (Some(a), Some(i)) = (arg.pop(), input.pop()) {
                match (&*a, &*i) {
                    (CType::Void, CType::Void) => { /* Do nothing */ }
                    (CType::Infer(s1, _), CType::Infer(s2, _)) if s1 == s2 => {
                        // This is not an error, but we can't garner any useful information here
                    }
                    (CType::Infer(s, _), _) => {
                        return Err(format!(
                            "While attempting to infer generics found an inference type {s} as an input somehow"
                        )
                        .into());
                    }
                    (CType::Type(_, t1), CType::Type(_, t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Type(_, t1), _) if !matches!(&**t1, CType::Binds(..)) => {
                        arg.push(t1.clone());
                        input.push(i.clone());
                    }
                    (_, CType::Type(_, t2)) if !matches!(&**t2, CType::Binds(..)) => {
                        arg.push(a.clone());
                        input.push(t2.clone());
                    }
                    (CType::Shared(a2), CType::Shared(b2)) => {
                        arg.push(a2.clone());
                        input.push(b2.clone());
                    }
                    // Shared{T} can match Mut{T} since it provides mutable access
                    (CType::Shared(a2), CType::Mut(b2)) => {
                        arg.push(a2.clone());
                        input.push(b2.clone());
                    }
                    // Transparent Shared: unwrap when only one side is Shared (but not Mut)
                    (CType::Shared(a2), other) if !matches!(other, CType::Mut(..)) => {
                        arg.push(a2.clone());
                        input.push(Arc::new(other.clone()));
                    }
                    (other, CType::Shared(b2)) if !matches!(other, CType::Mut(..)) => {
                        arg.push(Arc::new(other.clone()));
                        input.push(b2.clone());
                    }
                    (CType::Promise(a2), CType::Promise(b2)) => {
                        arg.push(a2.clone());
                        input.push(b2.clone());
                    }
                    // Transparent Promise: unwrap when only one side is Promise
                    (CType::Promise(a2), other) => {
                        arg.push(a2.clone());
                        input.push(Arc::new(other.clone()));
                    }
                    (other, CType::Promise(b2)) => {
                        arg.push(Arc::new(other.clone()));
                        input.push(b2.clone());
                    }
                    (CType::Array(a2), CType::Array(b2)) => {
                        arg.push(a2.clone());
                        input.push(b2.clone());
                    }
                    (CType::Generic(_, _, t), CType::Function(..))
                        if matches!(&**t, CType::Function(..)) =>
                    {
                        // TODO: How to get the generic args to compare correctly
                        arg.push(t.clone());
                        input.push(i.clone());
                    }
                    (CType::Generic(..), _) => {
                        return Err(format!(
                            "Ran into an unresolved generic in the arguments list: {arg:?}"
                        )
                        .into());
                    }
                    (CType::Binds(n1, ts1), CType::Binds(n2, ts2)) => {
                        if ts1.len() != ts2.len() {
                            // TODO: Better generic arg matching
                            return Err(format!(
                                "Mismatched resolved bound generic types {}{{{}}} and {}{{{}}} during inference",
                                n1.clone().to_strict_string(false),
                                ts1
                                    .iter()
                                    .map(|t| t.clone().to_strict_string(false))
                                    .collect::<Vec<String>>()
                                    .join(", "),
                                n2.clone().to_strict_string(false),
                                ts2
                                    .iter()
                                    .map(|t| t.clone().to_strict_string(false))
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            ).into());
                        }
                        arg.push(n1.clone());
                        input.push(n2.clone());
                        // Enqueue the bound types for checking purposes
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::IntrinsicGeneric(n1, s1), CType::IntrinsicGeneric(n2, s2)) => {
                        if !(n1 == n2 && s1 == s2) {
                            return Err(format!(
                                "Mismatched generics {n1} and {n2} during inference"
                            )
                            .into());
                        }
                    }
                    (CType::Int(i1), CType::Int(i2)) => {
                        if i1 != i2 {
                            return Err(format!(
                                "Mismatched integers {i1} and {i2} during inference"
                            )
                            .into());
                        }
                    }
                    (_, CType::IntCast(_)) => {
                        // Should only be reachable if there's an `infer` in here
                        // Unfortunately may not infer correctly in this scenario as casting to an
                        // integer is lossy.
                        return Err("Cannot infer an integer cast".into());
                    }
                    (CType::Float(f1), CType::Float(f2)) => {
                        if f1 != f2 {
                            return Err(format!(
                                "Mismatched floats {f1} and {f2} during inference"
                            )
                            .into());
                        }
                    }
                    (_, CType::FloatCast(_)) => {
                        // Should only be reachable if there's an `infer` in here
                        // Unfortunately may not infer correctly in this scenario as casting to a
                        // float is lossy.
                        return Err("Cannot infer a float cast".into());
                    }
                    (CType::Bool(b1), CType::Bool(b2)) => {
                        if b1 != b2 {
                            return Err("Mismatched booleans during inference".into());
                        }
                    }
                    (_, CType::BoolCast(_)) => {
                        // Should only be reachable if there's an `infer` in here
                        // Unfortunately may not infer correctly in this scenario as casting to a
                        // boolean is lossy.
                        return Err("Cannot infer a bool cast".into());
                    }
                    (CType::TString(s1), CType::TString(s2)) => {
                        if s1 != s2 {
                            return Err(format!(
                                "Mismatched strings {s1} and {s2} during inference"
                            )
                            .into());
                        }
                    }
                    (CType::TString(s), CType::StringCast(sc)) => {
                        // Should only be reachable if there's an `infer` in here. Fortunately, we
                        // *can* infer the original type from a string cast by re-parsing the
                        // string as a type declaration
                        match &**sc {
                            CType::Infer(..) => {
                                // We need to parse the string back into a type and then pass that
                                // along
                                let wtol = parse::typeassignables(s).expect("should be impossible");
                                let t = withtypeoperatorslist_to_ctype(&wtol.1, scope)?;
                                arg.push(t.clone());
                                input.push(sc.clone());
                            }
                            _ => {
                                return Err(format!(
                                    "Mismatched string {s} and string cast {sc:?} during inference"
                                )
                                .into());
                            }
                        }
                    }
                    (CType::Group(g1), CType::Group(g2)) => {
                        arg.push(g1.clone());
                        input.push(g2.clone());
                    }
                    (CType::Group(g1), _) => {
                        arg.push(g1.clone());
                        input.push(i.clone());
                    }
                    (_, CType::Group(g2)) => {
                        arg.push(a.clone());
                        input.push(g2.clone());
                    }
                    (CType::Function(i1, o1), CType::Function(i2, o2)) => {
                        match &**i1 {
                            CType::Tuple(ts1, _) if ts1.len() == 1 => {
                                arg.push(ts1[0].clone());
                            }
                            _otherwise => arg.push(i1.clone()),
                        }
                        arg.push(o1.clone());
                        match &**i2 {
                            CType::Tuple(ts2, _) if ts2.len() == 1 => {
                                input.push(ts2[0].clone());
                            }
                            _otherwise => input.push(i2.clone()),
                        }
                        input.push(o2.clone());
                    }
                    (CType::Call(n1, f1), CType::Call(n2, f2)) => {
                        arg.push(n1.clone());
                        arg.push(f1.clone());
                        input.push(n2.clone());
                        input.push(f2.clone());
                    }
                    (CType::Infix(o1), CType::Infix(o2)) => {
                        arg.push(o1.clone());
                        input.push(o2.clone());
                    }
                    (CType::Prefix(o1), CType::Prefix(o2)) => {
                        arg.push(o1.clone());
                        input.push(o2.clone());
                    }
                    (CType::Method(f1), CType::Method(f2)) => {
                        arg.push(f1.clone());
                        input.push(f2.clone());
                    }
                    (CType::Property(p1), CType::Property(p2)) => {
                        arg.push(p1.clone());
                        input.push(p2.clone());
                    }
                    (CType::Cast(t1), CType::Cast(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Own(t1), CType::Own(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Deref(t1), CType::Deref(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Mut(t1), CType::Mut(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Dependency(n1, v1), CType::Dependency(n2, v2)) => {
                        arg.push(n1.clone());
                        arg.push(v1.clone());
                        input.push(n2.clone());
                        input.push(v2.clone());
                    }
                    (CType::Rust(d1), CType::Rust(d2)) => {
                        arg.push(d1.clone());
                        input.push(d2.clone());
                    }
                    (CType::Nodejs(d1), CType::Nodejs(d2)) => {
                        arg.push(d1.clone());
                        input.push(d2.clone());
                    }
                    (CType::From(d1), CType::From(d2)) => {
                        arg.push(d1.clone());
                        input.push(d2.clone());
                    }
                    (CType::Import(n1, d1), CType::Import(n2, d2)) => {
                        arg.push(n1.clone());
                        arg.push(d1.clone());
                        input.push(n2.clone());
                        input.push(d2.clone());
                    }
                    (CType::Tuple(ts1, _), CType::Tuple(ts2, _)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched tuple types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        // TODO: Allow out-of-order listing based on Field labels
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Prop(t1, p1), CType::Prop(t2, p2)) => {
                        arg.push(t1.clone());
                        arg.push(p1.clone());
                        input.push(t2.clone());
                        input.push(p2.clone());
                    }
                    (CType::Exclude(t1, p1), CType::Exclude(t2, p2)) => {
                        arg.push(t1.clone());
                        arg.push(p1.clone());
                        input.push(t2.clone());
                        input.push(p2.clone());
                    }
                    (a, CType::Prop(t, p)) => {
                        // TODO: There's probably a generalized way to handle things like this, but
                        // for now, just hardwire this particular generic resolution used for the
                        // GPGPU `map` function
                        // In this case, the type to infer is the key that gets the
                        // value from the tuple type so we iterate through the values
                        // of the Prop tuple for a value that "accepts" the 'a' value
                        match &**p {
                            CType::StringCast(sc) => match &**sc {
                                CType::Infer(..) => match &**t {
                                    CType::Type(_, it) => match &**it {
                                        CType::Tuple(tp, _) => {
                                            let mut found = false;
                                            for r in tp {
                                                if let CType::Field(l, v) = &**r {
                                                    if Arc::new(a.clone()).accepts(v.clone()) {
                                                        // We found a match, parse the label back to a
                                                        // type
                                                        let wtol = parse::typeassignables(l)
                                                            .expect("should be impossible");
                                                        let t = withtypeoperatorslist_to_ctype(
                                                            &wtol.1, scope,
                                                        )?;
                                                        arg.push(t.clone());
                                                        input.push(sc.clone());
                                                        found = true;
                                                    }
                                                }
                                            }
                                            if !found {
                                                return Err(
                                                    "Unable to find property during inference"
                                                        .into(),
                                                );
                                            }
                                        }
                                        _ => {
                                            return Err("Property extraction inference only possible on a tuple type".into());
                                        }
                                    },
                                    CType::Tuple(tp, _) => {
                                        let mut found = false;
                                        for r in tp {
                                            if let CType::Field(l, v) = &**r {
                                                if Arc::new(a.clone()).accepts(v.clone()) {
                                                    // We found a match, parse the label back to a
                                                    // type
                                                    let wtol = parse::typeassignables(l)
                                                        .expect("should be impossible");
                                                    let t = withtypeoperatorslist_to_ctype(
                                                        &wtol.1, scope,
                                                    )?;
                                                    arg.push(t.clone());
                                                    input.push(sc.clone());
                                                    found = true;
                                                }
                                            }
                                        }
                                        if !found {
                                            return Err(
                                                "Unable to find property during inference".into()
                                            );
                                        }
                                    }
                                    _ => {
                                        return Err("Property extraction inference only possible on a tuple type".into());
                                    }
                                },
                                _ => unreachable!(),
                            },
                            // The other path that is being hardwired right now is the version used
                            // for GBufferTagged. TODO: Figure out a general way to handle this
                            // kind of type inference and replace the hacks with it.
                            CType::TString(s) if s == "typeName" => {
                                match &**t {
                                    CType::Prop(t, p) => {
                                        match &**p {
                                            CType::StringCast(sc) => {
                                                match &**sc {
                                                    CType::Infer(..) => match &**t {
                                                        CType::Type(_, it) => match &**it {
                                                            CType::Tuple(tp, _) => {
                                                                let mut found = false;
                                                                for r in tp {
                                                                    if let CType::Field(l, v) = &**r
                                                                    {
                                                                        if Arc::new(a.clone())
                                                                            .accepts(v.clone())
                                                                        {
                                                                            // We found a match, parse the label back to a
                                                                            // type
                                                                            let wtol = parse::typeassignables(l).expect("should be impossible");
                                                                            let t = withtypeoperatorslist_to_ctype(&wtol.1, scope)?;
                                                                            arg.push(t.clone());
                                                                            input.push(sc.clone());
                                                                            found = true;
                                                                        }
                                                                    }
                                                                }
                                                                if !found {
                                                                    return Err("Unable to find property during inference".into());
                                                                }
                                                            }
                                                            _ => {
                                                                return Err("Property extraction inference only possible on a tuple type".into());
                                                            }
                                                        },
                                                        CType::Tuple(tp, _) => {
                                                            let mut found = false;
                                                            for r in tp {
                                                                if let CType::Field(l, v) = &**r {
                                                                    if Arc::new(a.clone())
                                                                        .accepts(v.clone())
                                                                    {
                                                                        // We found a match, parse the label back to a
                                                                        // type
                                                                        let wtol = parse::typeassignables(l).expect("should be impossible");
                                                                        let t = withtypeoperatorslist_to_ctype(&wtol.1, scope)?;
                                                                        arg.push(t.clone());
                                                                        input.push(sc.clone());
                                                                        found = true;
                                                                    }
                                                                }
                                                            }
                                                            if !found {
                                                                return Err("Unable to find property during inference".into());
                                                            }
                                                        }
                                                        _ => {
                                                            return Err("Property extraction inference only possible on a tuple type".into());
                                                        }
                                                    },
                                                    _ => unreachable!(),
                                                }
                                            }
                                            _ => {
                                                return Err(format!(
                                                    "Mismatch between {} and {} during inference",
                                                    CType::to_error_string(
                                                        Arc::new(a.clone()),
                                                        scope
                                                    ),
                                                    CType::to_error_string(i.clone(), scope)
                                                )
                                                .into());
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(format!(
                                            "Mismatch between {} and {} during inference",
                                            CType::to_error_string(Arc::new(a.clone()), scope),
                                            CType::to_error_string(i.clone(), scope)
                                        )
                                        .into());
                                    }
                                }
                            }
                            _ => {
                                return Err(format!(
                                    "Mismatch between {} and {} during inference",
                                    CType::to_error_string(Arc::new(a.clone()), scope),
                                    CType::to_error_string(i.clone(), scope)
                                )
                                .into());
                            }
                        }
                    }
                    (CType::Field(l1, t1), CType::Field(l2, t2)) => {
                        // TODO: Allow out-of-order listing based on Field labels
                        if l1 != l2 {
                            return Err(format!(
                                "Mismatched fields {l1} and {l2} during inference"
                            )
                            .into());
                        }
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (_, CType::Field(_, t2)) => {
                        arg.push(a.clone());
                        input.push(t2.clone());
                    }
                    (CType::Field(_, t1), _) => {
                        arg.push(t1.clone());
                        input.push(i.clone());
                    }
                    (CType::Either(ts1, _), CType::Either(ts2, _)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched either types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Buffer(t1, s1), CType::Buffer(t2, s2)) => {
                        arg.push(t1.clone());
                        arg.push(s1.clone());
                        input.push(t2.clone());
                        input.push(s2.clone());
                    }
                    (CType::AnyOf(ts), CType::Infer(g, _)) => {
                        // Found an interesting inference situation where more than one answer may
                        // be right. We need to check the existing possible matches (if any) and
                        // intersect it with this AnyOf set, then store the set. If there is only
                        // one match in the set, then we store that match directly, instead.
                        if generic_types.contains_key(g) {
                            let other_type: &CType = generic_types.get(g).unwrap();
                            let mut matches = Vec::new();
                            match other_type {
                                CType::AnyOf(t2s) => {
                                    for t1 in ts {
                                        for t2 in t2s {
                                            if t1.clone().degroup().to_callable_string()
                                                == t2.clone().degroup().to_callable_string()
                                            {
                                                matches.push(t1.clone());
                                            }
                                        }
                                    }
                                }
                                otherwise => {
                                    for t1 in ts {
                                        if t1.clone().degroup().to_callable_string()
                                            == Arc::new(otherwise.clone())
                                                .degroup()
                                                .to_callable_string()
                                        {
                                            matches.push(t1.clone());
                                        }
                                    }
                                }
                            }
                            if matches.is_empty() {
                                // Do nothing
                            } else if matches.len() == 1 {
                                generic_types
                                    .insert(g.clone(), matches.into_iter().nth(0).unwrap().clone());
                            } else {
                                generic_types.insert(g.clone(), Arc::new(CType::AnyOf(matches)));
                            }
                        } else {
                            generic_types.insert(g.clone(), Arc::new(CType::AnyOf(ts.clone())));
                        }
                    }
                    (CType::Fail(m1), CType::Fail(m2)) => {
                        if m1 != m2 {
                            return Err(
                                "The two types want to fail in different ways. How bizarre!".into(),
                            );
                        }
                    }
                    (CType::Add(ts1), CType::Add(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched add types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (
                        CType::Int(_) | CType::Float(_),
                        CType::Add(_)
                        | CType::Sub(_)
                        | CType::Mul(_)
                        | CType::Div(_)
                        | CType::Mod(_)
                        | CType::Pow(_)
                        | CType::Min(_)
                        | CType::Max(_)
                        | CType::Neg(_)
                        | CType::Len(_)
                        | CType::Size(_),
                    ) => {
                        // TODO: This should allow us to constrain which generic values are
                        // possible for each generic to infer on the right-hand-side, but for now
                        // we're just going to ignore this path and require the components are
                        // inferred separately in the type system
                    }
                    (
                        CType::Int(_) | CType::Bool(_),
                        CType::And(_)
                        | CType::Or(_)
                        | CType::Xor(_)
                        | CType::Not(_)
                        | CType::Nand(_)
                        | CType::Nor(_)
                        | CType::Xnor(_),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (
                        CType::Int(_) | CType::Float(_) | CType::TString(_) | CType::Bool(_),
                        CType::TEq(_) | CType::Neq(_),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (
                        CType::Int(_) | CType::Float(_) | CType::TString(_),
                        CType::Lt(_) | CType::Lte(_) | CType::Gt(_) | CType::Gte(_),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (CType::Sub(ts1), CType::Sub(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched sub types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Mul(ts1), CType::Mul(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched mul types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Div(ts1), CType::Div(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched div types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Mod(ts1), CType::Mod(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched div types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Pow(ts1), CType::Pow(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched pow types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Min(ts1), CType::Min(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched min types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Max(ts1), CType::Max(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched max types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Neg(t1), CType::Neg(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Len(t1), CType::Len(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Size(t1), CType::Size(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::FileStr(t1), CType::FileStr(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Env(ts1), CType::Env(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched env types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::EnvExists(t1), CType::EnvExists(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::TIf(t1, ts1), CType::TIf(t2, ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched env types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        arg.push(t1.clone());
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        input.push(t2.clone());
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::And(ts1), CType::And(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched and types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Or(ts1), CType::Or(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched or types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Xor(ts1), CType::Xor(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched xor types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Not(t1), CType::Not(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Nand(ts1), CType::Nand(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched nand types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Nor(ts1), CType::Nor(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched nor types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Xnor(ts1), CType::Xnor(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched xnor types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::TEq(ts1), CType::TEq(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched eq types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Neq(ts1), CType::Neq(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched neq types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Lt(ts1), CType::Lt(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched lt types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Lte(ts1), CType::Lte(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched lte types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Gt(ts1), CType::Gt(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched gt types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Gte(ts1), CType::Gte(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched gte types {} and {} found during inference",
                                CType::to_error_string(a.clone(), scope),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (_, CType::Infer(g, _)) => {
                        // Found the normal path to infer. If there's already a match, check if the
                        // existing match is an AnyOf and intersect the set, otherwise a simple
                        // comparison
                        if generic_types.contains_key(g) {
                            // Possible found the same thing, already, let's confirm that we aren't
                            // in an impossible scenario.
                            let other_type: &Arc<CType> = generic_types.get(g).unwrap();
                            let mut matched = false;
                            match &**other_type {
                                CType::AnyOf(ts) => {
                                    for t1 in ts {
                                        if CType::tunwrap(t1.clone().degroup())
                                            .to_functional_string()
                                            == CType::tunwrap(a.clone().degroup())
                                                .to_functional_string()
                                        {
                                            matched = true;
                                        }
                                    }
                                }
                                otherwise => {
                                    if CType::tunwrap(Arc::new(otherwise.clone()).degroup())
                                        .to_functional_string()
                                        == CType::tunwrap(a.clone().degroup())
                                            .to_functional_string()
                                    {
                                        matched = true;
                                    }
                                }
                            }
                            if matched {
                                generic_types.insert(
                                    g.clone(),
                                    canonicalize_inferred_generic_type(a.clone(), scope),
                                );
                            } else {
                                return Err(format!(
                                    "Generic {} matched both {} and {}",
                                    g,
                                    CType::to_error_string(other_type.clone(), scope),
                                    CType::to_error_string(a.clone(), scope)
                                )
                                .into());
                            }
                        } else {
                            generic_types.insert(
                                g.clone(),
                                canonicalize_inferred_generic_type(a.clone(), scope),
                            );
                        }
                    }
                    (CType::AnyOf(ts), _) => {
                        // Multiple of these `AnyOf` types may be viable. Accept all that are, and
                        // later on something should hopefully work as a tiebreaker.
                        let mut success = false;
                        let inner_results = ts
                            .iter()
                            .map(|t| {
                                let mut generic_types_inner = generic_types.clone();
                                if CType::infer_generics_inner_loop(
                                    scope,
                                    &mut generic_types_inner,
                                    vec![(t.clone(), i.clone())],
                                )
                                .is_ok()
                                {
                                    success = true;
                                } else {
                                    // Reset it on failure, just in case
                                    generic_types_inner = generic_types.clone();
                                }
                                generic_types_inner
                            })
                            .collect::<Vec<HashMap<String, Arc<CType>>>>();
                        if !success {
                            return Err(format!(
                                "None of {} matches {}",
                                ts.iter()
                                    .map(|t| CType::to_error_string(t.clone(), scope))
                                    .collect::<Vec<String>>()
                                    .join(" & "),
                                CType::to_error_string(i.clone(), scope)
                            )
                            .into());
                        }
                        // Merge the results into a singular set to check. If there are multiple
                        // values for the same key, merge them as an `AnyOf`.
                        let mut combined_types = HashMap::new();
                        for gti in inner_results {
                            for (k, v) in &gti {
                                match combined_types.get(k) {
                                    None => {
                                        combined_types.insert(k.clone(), v.clone());
                                    }
                                    Some(other_v) => match (&**other_v, &**v) {
                                        (CType::AnyOf(ots), nt) => {
                                            let mut preexists = false;
                                            for t in ots {
                                                if t.clone().to_functional_string()
                                                    == Arc::new(nt.clone()).to_functional_string()
                                                {
                                                    preexists = true;
                                                }
                                            }
                                            if !preexists {
                                                let mut nts = ots.clone();
                                                nts.push(Arc::new(nt.clone()));
                                                combined_types
                                                    .insert(k.clone(), Arc::new(CType::AnyOf(nts)));
                                            }
                                        }
                                        (_, _) => {
                                            combined_types.insert(
                                                k.clone(),
                                                Arc::new(CType::AnyOf(vec![
                                                    other_v.clone(),
                                                    v.clone(),
                                                ])),
                                            );
                                        }
                                    },
                                }
                            }
                        }
                        // Now comparing the combined resolved types with what was in the original
                        // set, anything new gets included, but we attempt to *narrow* the `AnyOf`
                        // to as few as possible, when possible
                        for (k, v) in &combined_types {
                            match generic_types.get(k) {
                                None => {
                                    generic_types.insert(k.clone(), v.clone());
                                }
                                Some(old_v) => match (&**old_v, &**v) {
                                    (CType::AnyOf(oldts), CType::AnyOf(newts)) => {
                                        let mut outts = Vec::new();
                                        for ot in oldts {
                                            for nt in newts {
                                                if ot.clone().to_functional_string()
                                                    == nt.clone().to_functional_string()
                                                {
                                                    outts.push(nt.clone());
                                                }
                                            }
                                        }
                                        generic_types
                                            .insert(k.clone(), Arc::new(CType::AnyOf(outts)));
                                    }
                                    (_, CType::AnyOf(newts)) => {
                                        let mut success = false;
                                        for nt in newts {
                                            if old_v.clone().to_functional_string()
                                                == nt.clone().to_functional_string()
                                            {
                                                success = true;
                                                break;
                                            }
                                        }
                                        if !success {
                                            return Err(format!(
                                                "None of {} matches {}",
                                                newts
                                                    .iter()
                                                    .map(|t| CType::to_error_string(
                                                        t.clone(),
                                                        scope
                                                    ))
                                                    .collect::<Vec<String>>()
                                                    .join(" & "),
                                                CType::to_error_string(old_v.clone(), scope)
                                            )
                                            .into());
                                        }
                                    }
                                    (CType::AnyOf(oldts), _) => {
                                        let mut success = false;
                                        for ot in oldts {
                                            if ot.clone().to_functional_string()
                                                == v.clone().to_functional_string()
                                            {
                                                success = true;
                                                break;
                                            }
                                        }
                                        if !success {
                                            return Err(format!(
                                                "None of {} matches {}",
                                                oldts
                                                    .iter()
                                                    .map(|t| CType::to_error_string(
                                                        t.clone(),
                                                        scope
                                                    ))
                                                    .collect::<Vec<String>>()
                                                    .join(" & "),
                                                CType::to_error_string(v.clone(), scope)
                                            )
                                            .into());
                                        }
                                        generic_types.insert(k.clone(), v.clone());
                                    }
                                    (_, _) => {
                                        if old_v.clone().to_functional_string()
                                            != v.clone().to_functional_string()
                                        {
                                            return Err(format!(
                                                "{} does not match {}",
                                                CType::to_error_string(old_v.clone(), scope),
                                                CType::to_error_string(v.clone(), scope),
                                            )
                                            .into());
                                        }
                                    }
                                },
                            }
                        }
                    }
                    _ => {
                        return Err(format!(
                            "Mismatch between {} and {}",
                            CType::to_error_string(a.clone(), scope),
                            CType::to_error_string(i.clone(), scope)
                        )
                        .into());
                    }
                }
            }
        }
        Ok(())
    }
    pub fn infer_generics(
        scope: &Scope,
        generics: &[(String, Arc<CType>)],
        fn_args: &[(String, ArgKind, Arc<CType>)],
        call_args: &[Arc<CType>],
    ) -> Result<Vec<Arc<CType>>, Box<dyn std::error::Error>> {
        let mut temp_scope = scope.child();
        for (generic_name, generic_type) in generics {
            temp_scope
                .types
                .insert(generic_name.clone(), generic_type.clone());
        }
        let input_types = fn_args
            .iter()
            .map(|(_, _, t)| t.clone())
            .collect::<Vec<Arc<CType>>>();
        let mut generic_types: HashMap<String, Arc<CType>> = HashMap::new();
        CType::infer_generics_inner_loop(
            &temp_scope,
            &mut generic_types,
            call_args
                .iter()
                .zip(input_types.iter())
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect::<Vec<(Arc<CType>, Arc<CType>)>>(),
        )?;
        let mut output_types = Vec::new();
        for (generic_name, _) in generics {
            output_types.push(match generic_types.get(generic_name) {
                Some(t) => Ok(canonicalize_inferred_generic_type(t.clone(), scope)),
                None => Err(format!("No inferred type found for {generic_name}")),
            }?);
        }
        Ok(output_types)
    }
}
