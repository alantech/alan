use std::sync::Arc;

use super::CType;
use super::Program;
use super::Scope;

impl CType {
    #[allow(clippy::should_implement_trait)]
    pub fn neg(t: Arc<CType>) -> Arc<CType> {
        match &*t {
            CType::Int(v) => Arc::new(CType::Int(-v)),
            CType::Float(v) => Arc::new(CType::Float(-v)),
            CType::Infer(..) => Arc::new(CType::Neg(t)),
            _ => CType::fail("Attempting to negate non-integer or non-float types at compile time"),
        }
    }
    pub fn len(t: Arc<CType>) -> Arc<CType> {
        match &*t {
            CType::Tuple(tup, _) => Arc::new(CType::Int(tup.len() as i128)),
            CType::Buffer(_, l) => match **l {
                CType::Int(l) => Arc::new(CType::Int(l)),
                _ => {
                    CType::fail("Cannot get a compile time length for an invalid Buffer definition")
                }
            },
            CType::Either(eit, _) => Arc::new(CType::Int(eit.len() as i128)),
            CType::Array(_) => {
                CType::fail("Cannot get a compile time length for a variable-length array")
            }
            CType::Infer(..) => Arc::new(CType::Len(t)),
            CType::Void | CType::DerivedVoid(_) => Arc::new(CType::Int(0)),
            _ => Arc::new(CType::Int(1)),
        }
    }
    pub fn exclude(t: Arc<CType>, n: Arc<CType>) -> Arc<CType> {
        if t.clone().has_infer() || n.clone().has_infer() {
            return Arc::new(CType::Exclude(t, n));
        }
        // Collect parent types from the original type, including transitive parents
        let mut parents = Vec::new();
        // Add the original type as the direct parent
        parents.push(t.clone());
        // Collect transitive parents from the unwrapped type
        let unwrapped = t.clone().degroup();
        match &*unwrapped {
            CType::Type(_, inner) => {
                match &**inner {
                    CType::Tuple(_, inner_parents) => parents.extend(inner_parents.clone()),
                    CType::Either(_, inner_parents) => parents.extend(inner_parents.clone()),
                    _ => {}
                }
                return CType::exclude(inner.clone(), n);
            }
            CType::Group(inner) => {
                return CType::exclude(inner.clone(), n);
            }
            CType::Tuple(_, inner_parents) => parents.extend(inner_parents.clone()),
            CType::Either(_, inner_parents) => parents.extend(inner_parents.clone()),
            _ => {}
        }
        match &*unwrapped {
            CType::Infer(..) => unreachable!(),
            CType::Either(ts, _) => {
                let result = match &*n {
                    CType::TString(s) => {
                        let filtered: Vec<Arc<CType>> = ts.iter().filter(|t| {
                            match &***t {
                                CType::Field(name, _) => name != s,
                                _ => (**t).clone().to_callable_string() != *s,
                            }
                        }).cloned().collect();
                        if filtered.len() == ts.len() {
                            return Arc::new(CType::Fail(format!("Variant with name {s} not found in Either type")));
                        }
                        filtered
                    }
                    CType::Int(i) => {
                        let idx = *i as usize;
                        if idx >= ts.len() {
                            return Arc::new(CType::Fail(format!("Index {idx} out of bounds for Either type with {} variants", ts.len())));
                        }
                        let filtered: Vec<Arc<CType>> = ts.iter().enumerate().filter(|(idx_in_either, _)| {
                            *idx_in_either != idx
                         }).map(|(_, t)| t.clone()).collect();
                         if filtered.is_empty() {
                             return Arc::new(CType::DerivedVoid(parents));
                         }
                         filtered
                    }
                    otherwise => {
                        CType::fail(&format!(
                            "Exclude index must be a name or integer, not {otherwise:?}"
                        ))
                    }
                };
                if result.is_empty() {
                    return Arc::new(CType::DerivedVoid(parents));
                }
                if result.len() == 1 {
                    return Arc::new(CType::Either(result, parents));
                }
                Arc::new(CType::Either(result, parents))
            }
            CType::Tuple(ts, _) => {
                let result = match &*n {
                    CType::TString(s) => {
                        let filtered: Vec<Arc<CType>> = ts.iter().filter(|t| {
                            match &***t {
                                CType::Field(name, _) => name != s,
                                _ => (**t).clone().to_callable_string() != *s,
                            }
                        }).cloned().collect();
                        if filtered.len() == ts.len() {
                            return Arc::new(CType::Fail(format!("Element with name {s} not found in Tuple type")));
                        }
                        filtered
                    }
                    CType::Int(i) => {
                        let idx = *i as usize;
                        if idx >= ts.len() {
                            return Arc::new(CType::Fail(format!("Index {idx} out of bounds for Tuple type with {} elements", ts.len())));
                        }
                        let filtered: Vec<Arc<CType>> = ts.iter().enumerate().filter(|(idx_in_tup, _)| {
                            *idx_in_tup != idx
                        }).map(|(_, t)| t.clone()).collect();
                        if filtered.is_empty() {
                            return Arc::new(CType::DerivedVoid(parents));
                        }
                        if filtered.len() == 1 {
                            return Arc::new(CType::Tuple(filtered, parents));
                        }
                        return Arc::new(CType::Tuple(filtered, parents));
                    }
                    otherwise => {
                        CType::fail(&format!(
                            "Exclude index must be a name or integer, not {otherwise:?}"
                        ))
                    }
                };
                if result.is_empty() {
                    return Arc::new(CType::DerivedVoid(parents));
                }
                Arc::new(CType::Tuple(result, parents))
            }
            CType::Field(name, _) => {
                match &*n {
                    CType::TString(s) => {
                        if name == s {
                            Arc::new(CType::Void)
                        } else {
                            Arc::new(CType::Fail(format!("Field name {s} does not match {name}")))
                        }
                    }
                    CType::Int(i) => {
                        if *i == 0 {
                            Arc::new(CType::Void)
                        } else {
                            Arc::new(CType::Fail("Cannot exclude value (index 1) from Field type, only label (index 0)".to_string()))
                        }
                    }
                    otherwise => {
                        CType::fail(&format!(
                            "Exclude index must be a name or integer, not {otherwise:?}"
                        ))
                    }
                }
            }
            otherwise => CType::fail(&format!(
                "Cannot exclude from type {otherwise:?}. Only Either, Tuple, and Field types are supported."
            )),
         }
    }
    // After exclude produces a Tuple/Either with parents, check the scope for an existing type
    // with matching field structure. If found, merge the parent lists and return the updated type.
    pub fn merge_exclude_parents(result: Arc<CType>, scope: &Scope) -> Arc<CType> {
        let (fields, parents) = match &*result {
            CType::Tuple(fields, parents) => (fields.clone(), parents.clone()),
            CType::Either(fields, parents) => (fields.clone(), parents.clone()),
            _ => return result,
        };
        if parents.is_empty() {
            return result;
        }
        let result_fields_keys: Vec<String> = fields
            .iter()
            .map(|f| f.clone().degroup().to_callable_string())
            .collect();
        let result_key = result.clone().to_callable_string();
        // Search scope for a type with matching structure but potentially different parents
        if let Some(existing) = scope.types.get(&result_key) {
            let existing_fields = match &**existing {
                CType::Tuple(f, _) => f.clone(),
                CType::Either(f, _) => f.clone(),
                _ => return result,
            };
            // Compare field structure by callable string
            let existing_fields_keys: Vec<String> = existing_fields
                .iter()
                .map(|f| f.clone().degroup().to_callable_string())
                .collect();
            if existing_fields_keys == result_fields_keys {
                // Same field structure, merge parents
                match &**existing {
                    CType::Tuple(f, ep) => {
                        let mut merged = ep.clone();
                        for p in &parents {
                            let pkey = p.clone().degroup().to_callable_string();
                            if !merged
                                .iter()
                                .any(|mp| mp.clone().degroup().to_callable_string() == pkey)
                            {
                                merged.push(p.clone());
                            }
                        }
                        return Arc::new(CType::Tuple(f.clone(), merged));
                    }
                    CType::Either(f, ep) => {
                        let mut merged = ep.clone();
                        for p in &parents {
                            let pkey = p.clone().degroup().to_callable_string();
                            if !merged
                                .iter()
                                .any(|mp| mp.clone().degroup().to_callable_string() == pkey)
                            {
                                merged.push(p.clone());
                            }
                        }
                        return Arc::new(CType::Either(f.clone(), merged));
                    }
                    _ => {}
                }
            }
        }
        result
    }
    pub fn size(t: Arc<CType>) -> Arc<CType> {
        // TODO: Implementing this might require all types be made C-style structs under the hood,
        // and probably some weird hackery to find out the size including padding on aligned
        // architectures, so I might take it back out before its actually implemented, but I can
        // think of several places where knowing the actual size of the type could be useful,
        // particularly for writing to disk or interfacing with network protocols, etc, so I'd
        // prefer to keep it and have some compile-time guarantees we don't normally see.
        match &*t {
            CType::Void | CType::DerivedVoid(..) => Arc::new(CType::Int(0)),
            CType::Infer(..) => Arc::new(CType::Size(t.clone())),
            CType::Type(_, t) => CType::size(t.clone()),
            CType::Generic(..) => CType::fail("Cannot determine the size of an unbound generic"),
            CType::Binds(t, ts) => {
                if !ts.is_empty() {
                    CType::fail("Cannot determine the size of an unbound generic")
                } else {
                    Arc::new(match &**t {
                        CType::TString(n) if n == "i8" => CType::Int(1),
                        CType::TString(n) if n == "u8" => CType::Int(1),
                        CType::TString(n) if n == "i16" => CType::Int(2),
                        CType::TString(n) if n == "u16" => CType::Int(2),
                        CType::TString(n) if n == "i32" => CType::Int(4),
                        CType::TString(n) if n == "u32" => CType::Int(4),
                        CType::TString(n) if n == "f32" => CType::Int(4),
                        CType::TString(n) if n == "i64" => CType::Int(8),
                        CType::TString(n) if n == "u64" => CType::Int(8),
                        CType::TString(n) if n == "f64" => CType::Int(8),
                        CType::TString(n) => {
                            CType::fail(&format!("Cannot determine the size of {n}"))
                        }
                        _ => CType::fail(&format!(
                            "Cannot determine the size of {}",
                            t.clone().to_functional_string()
                        )),
                    })
                }
            }
            CType::IntrinsicGeneric(..) => {
                CType::fail("Cannot determine the size of an unbound generic")
            }
            CType::Int(_) | CType::Float(_) => Arc::new(CType::Int(8)),
            CType::Bool(_) => Arc::new(CType::Int(1)),
            CType::TString(s) => Arc::new(CType::Int(s.capacity() as i128)),
            CType::Group(t) | CType::Field(_, t) => CType::size(t.clone()),
            CType::Tuple(ts, _) => {
                let sizes = ts
                    .clone()
                    .into_iter()
                    .map(CType::size)
                    .collect::<Vec<Arc<CType>>>();
                let mut out_size = 0;
                for t in sizes {
                    match &*t {
                        CType::Int(s) => out_size += s,
                        _ => unreachable!(),
                    }
                }
                Arc::new(CType::Int(out_size))
            }
            CType::Either(ts, _) => {
                let sizes = ts
                    .clone()
                    .into_iter()
                    .map(CType::size)
                    .collect::<Vec<Arc<CType>>>();
                let mut out_size = 0;
                for t in sizes {
                    match &*t {
                        CType::Int(s) => out_size = i128::max(out_size, *s),
                        _ => unreachable!(),
                    }
                }
                Arc::new(CType::Int(out_size))
            }
            CType::Buffer(b, s) => {
                let base_size = CType::size(b.clone());
                match (&*base_size, &**s) {
                    (CType::Int(a), CType::Int(b)) => Arc::new(CType::Int(a + b)),
                    (CType::Infer(..), _) | (_, CType::Infer(..)) => {
                        Arc::new(CType::Size(b.clone()))
                    }
                    _ => unreachable!(),
                }
            }
            CType::Array(_) => {
                CType::fail("Cannot determine the size of an array, it's length is not static")
            }
            CType::Function(..)
            | CType::Call(..)
            | CType::Infix(_)
            | CType::Prefix(_)
            | CType::Method(_)
            | CType::Property(_) => CType::fail("Cannot determine the size of a function"),
            _ => CType::fail(&format!(
                "Getting the size of {} doesn't make any sense",
                t.to_functional_string()
            )),
        }
    }
    pub fn filestr(f: Arc<CType>) -> Arc<CType> {
        match &*f {
            CType::TString(s) => match std::fs::read_to_string(s) {
                Err(e) => CType::fail(&format!("Failed to read {s}: {e:?}")),
                Ok(s) => Arc::new(CType::TString(s)),
            },
            CType::Infer(..) => f,
            _ => CType::fail("FileStr{F} must be given a string path to load"),
        }
    }
    pub fn concat(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        match (&*a, &*b) {
            (CType::Infer(..), _) | (_, CType::Infer(..)) => Arc::new(CType::Concat(a, b)),
            (CType::TString(a), CType::TString(b)) => Arc::new(CType::TString(format!("{a}{b}"))),
            _ => CType::fail("Concat{A, B} must be given strings to concatenate"),
        }
    }
    pub fn env(k: Arc<CType>) -> Arc<CType> {
        let out = match &*k {
            CType::TString(s) => match Program::compile_env_get(s) {
                None => CType::fail(&format!("Failed to load environment variable {s}",)),
                Some(s) => CType::TString(s),
            },
            CType::Infer(..) => CType::Env(vec![k.clone()]),
            _ => CType::fail("Env{K} must be given a key as a string to load"),
        };
        Arc::new(out)
    }
    pub fn envexists(k: Arc<CType>) -> Arc<CType> {
        let out = match &*k {
            CType::TString(s) => CType::Bool(Program::compile_env_contains(s)),
            CType::Infer(..) => CType::EnvExists(k),
            _ => CType::fail("EnvExists{K} must be given a key as a string to check"),
        };
        Arc::new(out)
    }
    #[allow(clippy::should_implement_trait)]
    pub fn not(b: Arc<CType>) -> Arc<CType> {
        Arc::new(match &*b {
            CType::Bool(b) => CType::Bool(!*b),
            CType::Int(b) => CType::Int(!*b),
            CType::Infer(..) => CType::Not(b),
            _ => CType::fail("Not{B} must be provided a boolean or integer type to invert"),
        })
    }
    pub fn min(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(if a < b { a } else { b }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(if a < b { a } else { b }),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Min(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Min(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to min non-integer or non-float types together at compile time",
            ),
        })
    }
    pub fn max(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(if a > b { a } else { b }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(if a > b { a } else { b }),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Max(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Max(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to max non-integer or non-float types together at compile time",
            ),
        })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn add(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a + b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a + b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Add(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Add(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to add non-integer or non-float types together at compile time",
            ),
        })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn sub(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a - b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a - b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Sub(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Sub(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to subtract non-integer or non-float types together at compile time",
            ),
        })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn mul(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a * b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Mul(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Mul(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to multiply non-integer or non-float types together at compile time",
            ),
        })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn div(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a / b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a / b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Div(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Div(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to divide non-integer or non-float types together at compile time",
            ),
        })
    }
    pub fn cmod(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_)) => {
                CType::Mod(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_), &CType::Infer(..)) => CType::Mod(vec![a.clone(), b.clone()]),
            _ => CType::fail("Attempting to modulus non-integer types together at compile time"),
        })
    }
    pub fn pow(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(match a.checked_pow(b as u32) {
                Some(c) => c,
                None => CType::fail("Compile time exponentiation too large"),
            }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a.powf(b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Pow(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Pow(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to divide non-integer or non-float types together at compile time",
            ),
        })
    }
    pub fn cif(c: Arc<CType>, a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        match &*CType::tunwrap(c.clone().degroup()) {
            CType::Bool(cond) => match cond {
                true => a.clone(),
                false => b.clone(),
            },
            CType::Infer(..) => Arc::new(CType::TIf(c.clone(), vec![a.clone(), b.clone()])),
            _ => CType::fail("If{C, A, B} must be given a boolean value as the condition"),
        }
    }
    pub fn tupleif(c: Arc<CType>, t: Arc<CType>) -> Arc<CType> {
        match &*c {
            CType::Bool(cond) => {
                match &*t {
                    CType::Tuple(tup, _) => {
                        if tup.len() == 2 {
                            match cond {
                                true => tup[0].clone(),
                                false => tup[1].clone(),
                            }
                        } else {
                            CType::fail("The tuple type provided to If{C, T} must have exactly two elements")
                        }
                    }
                    _ => CType::fail(
                        "The second type provided to If{C, T} must be a tuple of two types",
                    ),
                }
            }
            CType::Infer(..) => Arc::new(CType::TIf(c.clone(), vec![t.clone()])),
            _ => CType::fail("The first type provided to If{C, T} must be a boolean type"),
        }
    }
    pub fn envdefault(k: Arc<CType>, d: Arc<CType>) -> Arc<CType> {
        let out = match (&*k, &*d) {
            (CType::TString(s), CType::TString(def)) => match Program::compile_env_get(s) {
                None => CType::TString(def.clone()),
                Some(v) => CType::TString(v),
            },
            (CType::Infer(..), CType::TString(_))
            | (CType::TString(_), CType::Infer(..))
            | (CType::Infer(..), CType::Infer(..)) => CType::Env(vec![k.clone(), d.clone()]),
            _ => CType::fail("Env{K, D} must be provided a string for each type"),
        };
        Arc::new(out)
    }
    pub fn and(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::and(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::and(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a & *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a && *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::And(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::And(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "And{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        })
    }
    pub fn or(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::or(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::or(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a | *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a || *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Or(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Or(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Or{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        })
    }
    pub fn xor(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::xor(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::xor(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a ^ *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a ^ *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Xor(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Xor(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Xor{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        })
    }
    pub fn nand(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::nand(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::nand(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a & *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a && *b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Nand(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Nand(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Nand{A, B} must be provided two values of the same type, either integer or boolean")
        })
    }
    pub fn nor(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::nor(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::nor(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a | *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a || *b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Nor(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Nor(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Nor{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        })
    }
    pub fn xnor(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::xnor(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::xnor(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a ^ *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a ^ *b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Xnor(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Xnor(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Xnor{A, B} must be provided two values of the same type, either integer or boolean")
        })
    }
    pub fn eq(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::eq(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::eq(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a == *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a == *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a == *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a == *b),
            (
                &CType::Infer(..),
                &CType::Infer(..)
                | &CType::Int(_)
                | &CType::Float(_)
                | &CType::TString(_)
                | &CType::Bool(_),
            ) => CType::TEq(vec![a.clone(), b.clone()]),
            (
                &CType::Int(_) | &CType::Float(_) | &CType::TString(_) | &CType::Bool(_),
                &CType::Infer(..),
            ) => CType::TEq(vec![a.clone(), b.clone()]),
            (a, b) => CType::Bool(
                Arc::new(a.clone()).to_callable_string()
                    == Arc::new(b.clone()).to_callable_string(),
            ),
        })
    }
    pub fn neq(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::neq(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::neq(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a != *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a != *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a != *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a != *b),
            (
                &CType::Infer(..),
                &CType::Infer(..)
                | &CType::Int(_)
                | &CType::Float(_)
                | &CType::TString(_)
                | &CType::Bool(_),
            ) => CType::Neq(vec![a.clone(), b.clone()]),
            (
                &CType::Int(_) | &CType::Float(_) | &CType::TString(_) | &CType::Bool(_),
                &CType::Infer(..),
            ) => CType::Neq(vec![a.clone(), b.clone()]),
            (a, b) => CType::Bool(
                Arc::new(a.clone()).to_callable_string()
                    != Arc::new(b.clone()).to_callable_string(),
            ),
        })
    }
    pub fn lt(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::lt(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::lt(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a < *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a < *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a < *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_)) => {
                CType::Lt(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_), &CType::Infer(..)) => {
                CType::Lt(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Lt{A, B} must be provided two values of the same type, one of: integer, float, string"),
        })
    }
    pub fn lte(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::lte(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::lte(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a <= *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a <= *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a <= *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_)) => {
                CType::Lte(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_), &CType::Infer(..)) => {
                CType::Lte(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Lte{A, B} must be provided two values of the same type, one of: integer, float, string"),
        })
    }
    pub fn gt(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::gt(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::gt(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a > *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a > *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a > *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_)) => {
                CType::Gt(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_), &CType::Infer(..)) => {
                CType::Gt(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Gt{A, B} must be provided two values of the same type, one of: integer, float, string"),
        })
    }
    pub fn gte(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::gte(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::gte(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a >= *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a >= *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a >= *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_)) => {
                CType::Gte(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_), &CType::Infer(..)) => {
                CType::Gte(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Gte{A, B} must be provided two values of the same type, one of: integer, float, string"),
        })
    }
}
