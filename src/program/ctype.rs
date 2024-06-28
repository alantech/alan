use std::collections::{HashMap, HashSet};

use super::Export;
use super::FnKind;
use super::Function;
use super::Program;
use super::Scope;
use super::TypeOperatorMapping;
use crate::parse;

#[derive(Clone, Debug, PartialEq)]
pub enum CType {
    Void,
    Infer(String, String), // TODO: Switch to an Interface here once they exist
    Type(String, Box<CType>),
    Generic(String, Vec<String>, Vec<parse::WithTypeOperators>),
    Bound(String, String),
    BoundGeneric(String, Vec<String>, String),
    ResolvedBoundGeneric(String, Vec<String>, Vec<CType>, String),
    IntrinsicGeneric(String, usize),
    Int(i128),
    Float(f64),
    Bool(bool),
    TString(String),
    Group(Box<CType>),
    Function(Box<CType>, Box<CType>),
    Tuple(Vec<CType>),
    Field(String, Box<CType>),
    Either(Vec<CType>),
    Buffer(Box<CType>, usize),
    Array(Box<CType>),
}

impl CType {
    pub fn to_string(&self) -> String {
        self.to_strict_string(true)
    }
    pub fn to_strict_string(&self, strict: bool) -> String {
        match self {
            CType::Void => "()".to_string(),
            CType::Infer(s, _) => s.clone(), // TODO: Replace this
            CType::Type(n, t) => match strict {
                true => format!("{}", n),
                false => t.to_strict_string(strict),
            },
            CType::Generic(n, a, _) => format!("{}{{{}}}", n, a.join(", ")),
            CType::Bound(s, _) => format!("{}", s),
            CType::BoundGeneric(s, a, _) => format!("{}{{{}}}", s, a.join(", ")),
            CType::ResolvedBoundGeneric(s, _, a, _) => format!(
                "{}{{{}}}",
                s,
                a.iter()
                    .map(|b| b.to_strict_string(strict))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::IntrinsicGeneric(s, l) => format!(
                "{}{{{}}}",
                s,
                (0..*l)
                    .map(|b| format!("arg{}", b))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Int(i) => format!("{}", i),
            CType::Float(f) => format!("{}", f),
            CType::Bool(b) => match b {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            CType::TString(s) => s.clone(),
            CType::Group(t) => match strict {
                true => format!("({})", t.to_strict_string(strict)),
                false => t.to_strict_string(strict),
            },
            CType::Function(i, o) => format!(
                "{} -> {}",
                i.to_strict_string(strict),
                o.to_strict_string(strict)
            ),
            CType::Tuple(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(", "),
            CType::Field(l, t) => match strict {
                true => format!("{}: {}", l, t.to_strict_string(strict)),
                false => t.to_strict_string(strict),
            },
            CType::Either(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" | "),
            CType::Buffer(t, s) => format!("{}[{}]", t.to_strict_string(strict), s),
            CType::Array(t) => format!("{}[]", t.to_strict_string(strict)),
        }
    }
    pub fn to_functional_string(&self) -> String {
        match self {
            CType::Void => "void".to_string(),
            CType::Infer(s, _) => s.clone(), // TODO: What to do here?
            CType::Type(_, t) => t.to_functional_string(),
            CType::Generic(n, gs, _) => format!("{}{{{}}}", n, gs.join(", ")),
            CType::Bound(_, b) => b.clone(),
            CType::BoundGeneric(_, _, b) => b.clone(),
            CType::ResolvedBoundGeneric(_, gs, ts, b) => {
                let mut out = b.clone();
                for (g, t) in gs.iter().zip(ts.iter()) {
                    out = out.replace(g, &t.to_functional_string());
                }
                out
            }
            CType::IntrinsicGeneric(s, u) => format!("{}{{{}}}", s, {
                let mut out = Vec::new();
                for i in 0..(*u as u32) {
                    let a = 'a' as u32;
                    let c = char::from_u32(a + i).unwrap();
                    out.push(c.to_string());
                }
                out.join(", ")
            }),
            CType::Int(i) => format!("{}", i),
            CType::Float(f) => format!("{}", f),
            CType::Bool(b) => match b {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            CType::TString(s) => s.clone(),
            CType::Group(t) => t.to_functional_string(),
            CType::Function(i, o) => format!(
                "Function{{{}, {}}}",
                i.to_functional_string(),
                o.to_functional_string()
            ),
            CType::Tuple(ts) => format!(
                "Tuple{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Field(l, t) => format!("Label{{{}, {}}}", l, t.to_functional_string()),
            CType::Either(ts) => format!(
                "Either{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Buffer(t, s) => format!("Buffer{{{}, {}}}", t.to_functional_string(), s),
            CType::Array(t) => format!("Array{{{}}}", t.to_functional_string()),
        }
    }
    pub fn to_callable_string(&self) -> String {
        // TODO: Be more efficient with this later
        self.to_functional_string()
            .replace(" ", "_")
            .replace(",", "_")
            .replace("{", "_")
            .replace("}", "_")
    }
    pub fn degroup(&self) -> CType {
        match self {
            CType::Void => CType::Void,
            CType::Infer(s, i) => CType::Infer(s.clone(), i.clone()),
            CType::Type(n, t) => CType::Type(n.clone(), Box::new((*t).degroup())),
            CType::Generic(n, gs, wtos) => CType::Generic(n.clone(), gs.clone(), wtos.clone()),
            CType::Bound(n, b) => CType::Bound(n.clone(), b.clone()),
            CType::BoundGeneric(n, gs, b) => CType::BoundGeneric(n.clone(), gs.clone(), b.clone()),
            CType::ResolvedBoundGeneric(n, gs, ts, b) => CType::ResolvedBoundGeneric(
                n.clone(),
                gs.clone(),
                ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>(),
                b.clone(),
            ),
            CType::IntrinsicGeneric(n, s) => CType::IntrinsicGeneric(n.clone(), *s),
            CType::Int(i) => CType::Int(*i),
            CType::Float(f) => CType::Float(*f),
            CType::Bool(b) => CType::Bool(*b),
            CType::TString(s) => CType::TString(s.clone()),
            CType::Group(t) => t.degroup(),
            CType::Function(i, o) => {
                CType::Function(Box::new((*i).degroup()), Box::new((*o).degroup()))
            }
            CType::Tuple(ts) => {
                CType::Tuple(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>())
            }
            CType::Field(l, t) => CType::Field(l.clone(), Box::new((*t).degroup())),
            CType::Either(ts) => {
                CType::Either(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>())
            }
            CType::Buffer(t, s) => CType::Buffer(Box::new((*t).degroup()), *s),
            CType::Array(t) => CType::Array(Box::new((*t).degroup())),
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
    pub fn infer_generics(
        scope: &Scope,
        generics: &Vec<(String, CType)>,
        fn_args: &Vec<(String, CType)>,
        call_args: &Vec<CType>,
    ) -> Result<Vec<CType>, Box<dyn std::error::Error>> {
        let mut temp_scope = scope.temp_child();
        for (generic_name, generic_type) in generics {
            temp_scope
                .types
                .insert(generic_name.clone(), generic_type.clone());
        }
        let input_types = fn_args
            .iter()
            .map(|(_, t)| t.clone())
            .collect::<Vec<CType>>();
        let mut generic_types = HashMap::new();
        for (a, i) in call_args.iter().zip(input_types.iter()) {
            let mut arg = vec![a];
            let mut input = vec![i];
            while arg.len() > 0 {
                let a = arg.pop();
                let i = input.pop();
                match (a, i) {
                    (Some(CType::Void), Some(CType::Void)) => { /* Do nothing */ }
                    (Some(CType::Infer(s, _)), _) => {
                        return Err(format!(
                            "While attempting to infer {} but found an inference type for {} somehow",
                            generics.iter().map(|(n, _)| n.as_str()).collect::<Vec<&str>>().join(", "),
                            s
                        )
                        .into());
                    }
                    (Some(CType::Type(_, t1)), Some(CType::Type(_, t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Type(_, t1)), Some(b)) => {
                        arg.push(t1);
                        input.push(b);
                    }
                    (Some(a), Some(CType::Type(_, t2))) => {
                        arg.push(a);
                        input.push(t2);
                    }
                    (Some(CType::Generic(..)), _) => {
                        return Err(format!(
                            "Ran into an unresolved generic in the arguments list: {:?}",
                            arg
                        )
                        .into());
                    }
                    (Some(CType::Bound(n1, b1)), Some(CType::Bound(n2, b2))) => {
                        if !(n1 == n2 && b1 == b2) {
                            return Err(format!(
                                "Mismatched bound types {} -> {} and {} -> {} during inference",
                                n1, b1, n2, b2
                            )
                            .into());
                        }
                    }
                    (
                        Some(CType::BoundGeneric(n1, gs1, b1)),
                        Some(CType::BoundGeneric(n2, gs2, b2)),
                    ) => {
                        if !(n1 == n2 && b1 == b2 && gs1.len() == gs2.len()) {
                            // TODO: Better generic arg matching
                            return Err(format!("Mismatched bound generic types {}{{{}}} -> {} and {}{{{}}} -> {} during inference", n1, gs1.join(", "), b1, n2, gs2.join(", "), b2).into());
                        }
                    }
                    (
                        Some(CType::ResolvedBoundGeneric(n1, gs1, ts1, b1)),
                        Some(CType::ResolvedBoundGeneric(n2, gs2, ts2, b2)),
                    ) => {
                        if !(n1 == n2 && b1 == b2 && gs1.len() == gs2.len()) {
                            // TODO: Better generic arg matching
                            return Err(format!("Mismatched resolved bound generic types {}{{{}}} -> {} and {}{{{}}} -> {} during inference", n1, gs1.join(", "), b1, n2, gs2.join(", "), b2).into());
                        }
                        // Enqueue the bound types for checking purposes
                        for t1 in ts1 {
                            arg.push(&t1);
                        }
                        for t2 in ts2 {
                            input.push(&t2);
                        }
                    }
                    (
                        Some(CType::IntrinsicGeneric(n1, s1)),
                        Some(CType::IntrinsicGeneric(n2, s2)),
                    ) => {
                        if !(n1 == n2 && s1 == s2) {
                            return Err(format!(
                                "Mismatched generics {} and {} during inference",
                                n1, n2
                            )
                            .into());
                        }
                    }
                    (Some(CType::Int(i1)), Some(CType::Int(i2))) => {
                        if i1 != i2 {
                            return Err(format!(
                                "Mismatched integers {} and {} during inference",
                                i1, i2
                            )
                            .into());
                        }
                    }
                    (Some(CType::Float(f1)), Some(CType::Float(f2))) => {
                        if f1 != f2 {
                            return Err(format!(
                                "Mismatched floats {} and {} during inference",
                                f1, f2
                            )
                            .into());
                        }
                    }
                    (Some(CType::Bool(b1)), Some(CType::Bool(b2))) => {
                        if b1 != b2 {
                            return Err(format!("Mismatched booleans during inference").into());
                        }
                    }
                    (Some(CType::TString(s1)), Some(CType::TString(s2))) => {
                        if s1 != s2 {
                            return Err(format!(
                                "Mismatched strings {} and {} during inference",
                                s1, s2
                            )
                            .into());
                        }
                    }
                    (Some(CType::Group(g1)), Some(CType::Group(g2))) => {
                        arg.push(&*g1);
                        input.push(&*g2);
                    }
                    (Some(CType::Group(g1)), Some(b)) => {
                        arg.push(&*g1);
                        input.push(b);
                    }
                    (Some(a), Some(CType::Group(g2))) => {
                        arg.push(a);
                        input.push(&*g2);
                    }
                    (Some(CType::Function(i1, o1)), Some(CType::Function(i2, o2))) => {
                        arg.push(&*i1);
                        arg.push(&*o1);
                        input.push(&*i2);
                        input.push(&*o2);
                    }
                    (Some(CType::Tuple(ts1)), Some(CType::Tuple(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched tuple types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        // TODO: Allow out-of-order listing based on Field labels
                        for t1 in ts1 {
                            arg.push(&*t1);
                        }
                        for t2 in ts2 {
                            input.push(&*t2);
                        }
                    }
                    (Some(CType::Tuple(ts1)), Some(b)) if ts1.len() == 1 => {
                        arg.push(&ts1[0]);
                        input.push(b);
                    }
                    (Some(a), Some(CType::Tuple(ts2))) if ts2.len() == 1 => {
                        arg.push(a);
                        input.push(&ts2[0]);
                    }
                    (Some(CType::Field(l1, t1)), Some(CType::Field(l2, t2))) => {
                        // TODO: Allow out-of-order listing based on Field labels
                        if l1 != l2 {
                            return Err(format!(
                                "Mismatched fields {} and {} during inference",
                                l1, l2
                            )
                            .into());
                        }
                        arg.push(&*t1);
                        input.push(&*t2);
                    }
                    (Some(a), Some(CType::Field(_, t2))) => {
                        arg.push(&a);
                        input.push(&*t2);
                    }
                    (Some(CType::Field(_, t1)), Some(b)) => {
                        arg.push(&*t1);
                        input.push(&b);
                    }
                    (Some(CType::Either(ts1)), Some(CType::Either(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched either types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(&*t1);
                        }
                        for t2 in ts2 {
                            input.push(&*t2);
                        }
                    }
                    (Some(CType::Buffer(t1, s1)), Some(CType::Buffer(t2, s2))) => {
                        if s1 != s2 {
                            return Err(format!(
                                "Mismatched buffer lengths {} and {} found during inference",
                                s1, s2
                            )
                            .into());
                        }
                        arg.push(&*t1);
                        input.push(&*t2);
                    }
                    (Some(CType::Array(t1)), Some(CType::Array(t2))) => {
                        arg.push(&*t1);
                        input.push(&*t2);
                    }
                    (Some(a), Some(CType::Infer(g, _))) => {
                        // Found place to infer!
                        if generic_types.contains_key(g) {
                            // Possible found the same thing, already, let's confirm that we aren't
                            // in an impossible scenario.
                            let other_type: &CType = generic_types.get(g).unwrap();
                            if other_type.degroup().to_callable_string()
                                != a.degroup().to_callable_string()
                            {
                                return Err(format!(
                                    "Generic type {} resolved to both {} and {}",
                                    g,
                                    other_type.to_string(),
                                    a.to_string()
                                )
                                .into());
                            }
                        } else {
                            generic_types.insert(g.clone(), a.clone());
                        }
                    }
                    _ => {
                        return Err(format!("Mismatch between {:?} and {:?}", a, i).into());
                    }
                }
            }
        }
        let mut output_types = Vec::new();
        for (generic_name, _) in generics {
            output_types.push(match generic_types.get(generic_name) {
                Some(t) => Ok(t.clone()),
                None => Err(format!("No inferred type found for {}", generic_name)),
            }?);
        }
        Ok(output_types)
    }
    pub fn from_ast(
        scope: &mut Scope,
        program: &mut Program,
        type_ast: &parse::Types,
        is_export: bool,
    ) -> Result<CType, Box<dyn std::error::Error>> {
        let name = type_ast.fulltypename.typename.clone();
        if let Some(generics) = &type_ast.opttypegenerics {
            // We are going to conditionally compile this type declaration. If the we get true, we
            // continue, if we get false, we don't compile and return a Fail type that isn't added
            // to the scope to cause compilation to crash *if* something tries to use this, and if
            // we don't get a boolean at all or we get multiple inner values in the generic call,
            // we bail out immediately because of a syntax error.
            let generic_call =
                withtypeoperatorslist_to_ctype(&generics.typecalllist, scope, program)?;
            match generic_call {
                CType::Bool(b) => match b {
                    false => return Ok(CType::Void), // TODO: Have a lazy Fail type
                    true => { /* Do nothing */ }
                },
                CType::Type(_, c) => match *c {
                    CType::Bool(b) => match b {
                        false => return Ok(CType::Void),
                        true => { /* Do nothing */ }
                    },
                    _ => {
                        return Err(format!(
                        "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                        name,
                        generics.to_string()
                    )
                        .into())
                    }
                },
                _ => {
                    return Err(format!(
                    "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                    name,
                    generics.to_string()
                )
                    .into())
                }
            }
        }

        let (t, fs) = match &type_ast.fulltypename.opttypegenerics {
            None => {
                // This is a "normal" type
                match &type_ast.typedef {
                    parse::TypeDef::TypeCreate(create) => {
                        // When creating a "normal" type, we also create constructor and optionally
                        // accessor functions. This is not done for bound types nor done for
                        // generics until the generic type has been constructed. We create a set of
                        // `derived` Function objects and add it to the scope that a later stage of
                        // the compiler is responsible for actually creating. All of the types get
                        // one or more constructor functions, while struct-like Tuples and Either
                        // get accessor functions to dig into the sub-types.
                        let mut inner_type = withtypeoperatorslist_to_ctype(
                            &create.typeassignables,
                            &scope,
                            &program,
                        )?;
                        // Unwrap a Group type, if any exists, we don't want it here.
                        while match &inner_type {
                            CType::Group(_) => true,
                            _ => false,
                        } {
                            inner_type = match inner_type {
                                CType::Group(t) => *t,
                                t => t,
                            };
                        }
                        let t = CType::Type(name.clone(), Box::new(inner_type.clone()));
                        let mut fs = Vec::new();
                        let constructor_fn_name = t.to_callable_string();
                        match &inner_type {
                            CType::Type(n, _) => {
                                // This is just an alias
                                fs.push(Function {
                                    name: constructor_fn_name.clone(),
                                    args: vec![(n.clone(), inner_type.clone())],
                                    rettype: t.clone(),
                                    microstatements: Vec::new(),
                                    kind: FnKind::Derived,
                                });
                            }
                            CType::Bound(n, _) => {
                                // Also just an alias
                                fs.push(Function {
                                    name: constructor_fn_name.clone(),
                                    args: vec![(n.clone(), inner_type.clone())],
                                    rettype: t.clone(),
                                    microstatements: Vec::new(),
                                    kind: FnKind::Derived,
                                });
                            }
                            CType::Tuple(ts) => {
                                // The constructor function needs to grab the types from all
                                // arguments to construct the desired product type. For any type
                                // that is marked as a field, we also want to create an accessor
                                // function for it to simulate structs better.
                                let mut args = Vec::new();
                                for i in 0..ts.len() {
                                    let ti = &ts[i];
                                    match ti {
                                        CType::Field(n, f) => {
                                            // Create an accessor function
                                            fs.push(Function {
                                                name: n.clone(),
                                                args: vec![("arg0".to_string(), t.clone())],
                                                rettype: *f.clone(),
                                                microstatements: Vec::new(),
                                                kind: FnKind::Derived,
                                            });
                                            // Add a copy of this arg to the args array with the
                                            // name
                                            args.push((n.clone(), *f.clone()));
                                        }
                                        otherwise => {
                                            // Just copy this arg to the args array with a fake
                                            // name
                                            args.push((format!("arg{}", i), otherwise.clone()));
                                        }
                                    }
                                }
                                // Define the constructor function
                                fs.push(Function {
                                    name: constructor_fn_name.clone(),
                                    args,
                                    rettype: t.clone(),
                                    microstatements: Vec::new(),
                                    kind: FnKind::Derived,
                                });
                            }
                            CType::Either(ts) => {
                                // There are an equal number of constructor functions and accessor
                                // functions, one for each inner type of the sum type.
                                for e in ts {
                                    // Create a constructor fn
                                    fs.push(Function {
                                        name: constructor_fn_name.clone(),
                                        args: vec![("arg0".to_string(), e.clone())],
                                        rettype: t.clone(),
                                        microstatements: Vec::new(),
                                        kind: FnKind::Derived,
                                    });
                                    // Create the accessor function, the name of the function will
                                    // depend on the kind of type this is
                                    match e {
                                        CType::Field(n, i) => fs.push(Function {
                                            name: n.clone(),
                                            args: vec![("arg0".to_string(), t.clone())],
                                            rettype: CType::Either(vec![*i.clone(), CType::Void]),
                                            microstatements: Vec::new(),
                                            kind: FnKind::Derived,
                                        }),
                                        CType::Type(n, _) => fs.push(Function {
                                            name: n.clone(),
                                            args: vec![("arg0".to_string(), t.clone())],
                                            rettype: CType::Either(vec![e.clone(), CType::Void]),
                                            microstatements: Vec::new(),
                                            kind: FnKind::Derived,
                                        }),
                                        CType::Bound(n, _) => fs.push(Function {
                                            name: n.clone(),
                                            args: vec![("arg0".to_string(), t.clone())],
                                            rettype: CType::Either(vec![e.clone(), CType::Void]),
                                            microstatements: Vec::new(),
                                            kind: FnKind::Derived,
                                        }),
                                        CType::ResolvedBoundGeneric(n, ..) => fs.push(Function {
                                            name: n.clone(),
                                            args: vec![("arg0".to_string(), t.clone())],
                                            rettype: CType::Either(vec![e.clone(), CType::Void]),
                                            microstatements: Vec::new(),
                                            kind: FnKind::Derived,
                                        }),
                                        _ => {} // We can't make names for other types
                                    }
                                }
                            }
                            CType::Buffer(b, s) => {
                                // For Buffers we can create up to two types, one that takes a
                                // single value to fill in for all records in the buffer, and one
                                // that takes a distinct value for each possible value in the
                                // buffer. If the buffer size is just one element, we only
                                // implement one of these
                                fs.push(Function {
                                    name: constructor_fn_name.clone(),
                                    args: vec![("arg0".to_string(), *b.clone())],
                                    rettype: t.clone(),
                                    microstatements: Vec::new(),
                                    kind: FnKind::Derived,
                                });
                                if *s > 1 {
                                    fs.push(Function {
                                        name: constructor_fn_name.clone(),
                                        args: {
                                            let mut v = Vec::new();
                                            for i in 0..*s {
                                                v.push((format!("arg{}", i), *b.clone()));
                                            }
                                            v
                                        },
                                        rettype: t.clone(),
                                        microstatements: Vec::new(),
                                        kind: FnKind::Derived,
                                    });
                                }
                            }
                            CType::Array(a) => {
                                // For Arrays we create only one kind of array, one that takes any
                                // number of the input type. Until there's better support in the
                                // language for variadic functions, this is faked with a special
                                // DerivedVariadic function type that repeats the first and only
                                // arg for all input arguments. We also need to create `get` and
                                // `set` functions for this type (TODO: This is probably true for
                                // other types, too.
                                fs.push(Function {
                                    name: constructor_fn_name.clone(),
                                    args: vec![("arg0".to_string(), *a.clone())],
                                    rettype: t.clone(),
                                    microstatements: Vec::new(),
                                    kind: FnKind::DerivedVariadic,
                                });
                                fs.push(Function {
                                    name: "get".to_string(),
                                    args: vec![
                                        ("arg0".to_string(), t.clone()),
                                        (
                                            "arg1".to_string(),
                                            CType::Bound("i64".to_string(), "i64".to_string()),
                                        ),
                                    ],
                                    rettype: CType::Type(
                                        format!("Maybe_{}_", a.to_string()),
                                        Box::new(CType::Either(vec![*a.clone(), CType::Void])),
                                    ),
                                    microstatements: Vec::new(),
                                    kind: FnKind::Derived,
                                });
                                // TODO: Add 'set' function
                            }
                            _ => {} // Don't do anything for other types
                        }
                        (t, fs)
                    }
                    parse::TypeDef::TypeBind(bind) => (
                        CType::Bound(name.clone(), bind.othertype.clone()),
                        Vec::new(),
                    ),
                }
            }
            Some(g) => {
                // This is a "generic" type
                match &type_ast.typedef {
                    parse::TypeDef::TypeCreate(create) => (
                        CType::Generic(
                            name.clone(),
                            // TODO: Stronger checking on the usage here
                            g.typecalllist
                                .iter()
                                .map(|tc| tc.to_string())
                                .collect::<Vec<String>>()
                                .join("")
                                .split(",")
                                .map(|r| r.trim().to_string())
                                .collect::<Vec<String>>(),
                            create.typeassignables.clone(),
                        ),
                        Vec::new(),
                    ),
                    parse::TypeDef::TypeBind(bind) => (
                        CType::BoundGeneric(
                            name.clone(),
                            // TODO: Stronger checking on the usage here
                            g.typecalllist
                                .iter()
                                .map(|tc| tc.to_string())
                                .collect::<Vec<String>>()
                                .join("")
                                .split(",")
                                .map(|r| r.trim().to_string())
                                .collect::<Vec<String>>(),
                            bind.othertype.clone(),
                        ),
                        Vec::new(),
                    ),
                }
            }
        };
        if is_export {
            scope.exports.insert(name.clone(), Export::Type);
            if fs.len() > 0 {
                let mut names = HashSet::new();
                for f in &fs {
                    names.insert(f.name.clone());
                }
                for name in names {
                    scope.exports.insert(name.clone(), Export::Function);
                }
            }
        }
        scope.types.insert(name, t.clone());
        if fs.len() > 0 {
            let mut name_fn_pairs = HashMap::new();
            for f in fs {
                if name_fn_pairs.contains_key(&f.name) {
                    let v: &mut Vec<Function> = name_fn_pairs.get_mut(&f.name).unwrap();
                    v.push(f.clone());
                } else {
                    name_fn_pairs.insert(f.name.clone(), vec![f.clone()]);
                }
            }
            for (name, fns) in name_fn_pairs.drain() {
                if scope.functions.contains_key(&name) {
                    let func_vec = scope.functions.get_mut(&name).unwrap();
                    for f in fns {
                        func_vec.push(f);
                    }
                } else {
                    scope.functions.insert(name, fns);
                }
            }
        }
        Ok(t)
    }

    pub fn from_ctype(scope: &mut Scope, name: String, ctype: CType) {
        scope.exports.insert(name.clone(), Export::Type);
        scope.types.insert(name, ctype);
    }

    pub fn from_generic(scope: &mut Scope, name: &str, arglen: usize) {
        CType::from_ctype(
            scope,
            name.to_string(),
            CType::IntrinsicGeneric(name.to_string(), arglen),
        )
    }
    pub fn swap_subtype(self: &Self, old_type: &CType, new_type: &CType) -> CType {
        // Implemented recursively to be easier to follow. It would be nice to avoid all of the
        // cloning if the old type is not anywhere in the CType tree, but that would be a lot
        // harder to detect ahead of time.
        if self == old_type {
            return new_type.clone();
        }
        match self {
            CType::Void
            | CType::Infer(..)
            | CType::Generic(..)
            | CType::Bound(..)
            | CType::BoundGeneric(..)
            | CType::IntrinsicGeneric(..)
            | CType::Int(_)
            | CType::Float(_)
            | CType::Bool(_)
            | CType::TString(_) => self.clone(),
            CType::Type(name, ct) => {
                CType::Type(name.clone(), Box::new(ct.swap_subtype(old_type, new_type)))
            }
            CType::ResolvedBoundGeneric(name, gen_types, gen_type_resolved, bind_str) => {
                CType::ResolvedBoundGeneric(
                    name.clone(),
                    gen_types.clone(),
                    gen_type_resolved
                        .iter()
                        .map(|gtr| gtr.swap_subtype(old_type, new_type))
                        .collect::<Vec<CType>>(),
                    bind_str.clone(),
                )
            }
            CType::Group(g) => g.swap_subtype(old_type, new_type),
            CType::Function(i, o) => CType::Function(
                Box::new(i.swap_subtype(old_type, new_type)),
                Box::new(o.swap_subtype(old_type, new_type)),
            ),
            CType::Tuple(ts) => CType::Tuple(
                ts.iter()
                    .map(|t| t.swap_subtype(old_type, new_type))
                    .collect::<Vec<CType>>(),
            ),
            CType::Field(name, t) => {
                CType::Field(name.clone(), Box::new(t.swap_subtype(old_type, new_type)))
            }
            CType::Either(ts) => CType::Either(
                ts.iter()
                    .map(|t| t.swap_subtype(old_type, new_type))
                    .collect::<Vec<CType>>(),
            ),
            CType::Buffer(t, size) => {
                CType::Buffer(Box::new(t.swap_subtype(old_type, new_type)), *size)
            }
            CType::Array(t) => CType::Array(Box::new(t.swap_subtype(old_type, new_type))),
        }
    }
    // Special implementation for the tuple and either types since they *are* CTypes, but if one of
    // the provided input types *is* the same kind of CType, it should produce a merged version.
    pub fn tuple(args: Vec<CType>) -> CType {
        let mut out_vec = Vec::new();
        for arg in args {
            match arg {
                CType::Tuple(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                other => out_vec.push(other),
            }
        }
        CType::Tuple(out_vec)
    }
    pub fn either(args: Vec<CType>) -> CType {
        let mut out_vec = Vec::new();
        for arg in args {
            match arg {
                CType::Either(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                other => out_vec.push(other),
            }
        }
        CType::Either(out_vec)
    }
    // Special implementation for the field type, too. Right now for easier parsing the key needs
    // to be quoted. TODO: remove this restriction
    pub fn field(mut args: Vec<CType>) -> CType {
        if args.len() != 2 {
            CType::fail("Field{K, V} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            match (arg0, arg1) {
                (CType::TString(key), anything) => {
                    CType::Field(key.clone(), Box::new(anything.clone()))
                }
                _ => CType::fail("The field key must be a quoted string at this time"),
            }
        }
    }
    // Some validation for buffer creation, too
    pub fn buffer(mut args: Vec<CType>) -> CType {
        if args.len() != 2 {
            CType::fail("Buffer{T, S} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            match (arg0, arg1) {
                (anything, CType::Int(size)) => {
                    CType::Buffer(Box::new(anything.clone()), size as usize)
                }
                (anything, CType::Group(g)) => match *g {
                    CType::Int(size) => CType::Buffer(Box::new(anything.clone()), size as usize),
                    _ => CType::fail("The buffer size must be a positive integer"),
                },
                _ => CType::fail("The buffer size must be a positive integer"),
            }
        }
    }
    // Implementation of the ctypes that aren't storage but compute into another CType
    pub fn fail(message: &str) -> ! {
        // TODO: Include more information on where this compiler exit is coming from
        eprintln!("{}", message);
        std::process::exit(1);
    }
    pub fn cfail(message: &CType) -> ! {
        match message {
            CType::TString(s) => CType::fail(&s),
            _ => CType::fail("Fail passed a type that does not resolve into a message string"),
        }
    }
    pub fn neg(t: &CType) -> CType {
        match t {
            &CType::Int(v) => CType::Int(-v),
            &CType::Float(v) => CType::Float(-v),
            _ => CType::fail(
                "Attempting to add non-integer or non-float types together at compile time",
            ),
        }
    }
    pub fn len(t: &CType) -> CType {
        match t {
            CType::Tuple(tup) => CType::Int(tup.len() as i128),
            CType::Buffer(_, l) => CType::Int(*l as i128),
            CType::Either(eit) => CType::Int(eit.len() as i128),
            CType::Array(_) => {
                CType::fail("Cannot get a compile time length for a variable-length array")
            }
            _ => CType::Int(1),
        }
    }
    pub fn size(_t: &CType) -> CType {
        // TODO: Implementing this might require all types be made C-style structs under the hood,
        // and probably some weird hackery to find out the size including padding on aligned
        // architectures, so I might take it back out before its actually implemented, but I can
        // think of several places where knowing the actual size of the type could be useful,
        // particularly for writing to disk or interfacing with network protocols, etc, so I'd
        // prefer to keep it and have some compile-time guarantees we don't normally see.
        CType::fail("TODO: Implement Size{T}!")
    }
    pub fn filestr(f: &CType) -> CType {
        match f {
            CType::TString(s) => match std::fs::read_to_string(s) {
                Err(e) => CType::fail(&format!("Failed to read {}: {:?}", s, e)),
                Ok(s) => CType::TString(s),
            },
            _ => CType::fail("FileStr{F} must be given a string path to load"),
        }
    }
    pub fn env(k: &CType) -> CType {
        match k {
            CType::TString(s) => match std::env::var(
                s.trim_start_matches(|c| c == '"' || c == '\'')
                    .trim_end_matches(|c| c == '"' || c == '\''),
            ) {
                Err(e) => CType::fail(&format!(
                    "Failed to load environment variable {}: {:?}\nAll current envvars:\n{}",
                    s,
                    e,
                    std::env::vars()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<String>>()
                        .join("\n")
                )),
                // All TStrings are quoted. TODO: Alan supports single-quotes, be less weird here
                Ok(s) => CType::TString(format!("\"{}\"", s.replace("\"", "\\\""))),
            },
            _ => CType::fail("Env{K} must be given a key as a string to load"),
        }
    }
    pub fn envexists(k: &CType) -> CType {
        match k {
            CType::TString(s) => match std::env::var(s) {
                Err(_) => CType::Bool(false),
                Ok(_) => CType::Bool(true),
            },
            _ => CType::fail("EnvExists{K} must be given a key as a string to check"),
        }
    }
    pub fn not(b: &CType) -> CType {
        match b {
            CType::Bool(b) => CType::Bool(!*b),
            _ => CType::fail("Not{B} must be provided a boolean type to invert"),
        }
    }
    pub fn min(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(if a < b { a } else { b }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(if a < b { a } else { b }),
            _ => CType::fail(
                "Attempting to add non-integer or non-float types together at compile time",
            ),
        }
    }
    pub fn max(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(if a > b { a } else { b }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(if a > b { a } else { b }),
            _ => CType::fail(
                "Attempting to add non-integer or non-float types together at compile time",
            ),
        }
    }
    pub fn add(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a + b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a + b),
            _ => CType::fail(
                "Attempting to add non-integer or non-float types together at compile time",
            ),
        }
    }
    pub fn sub(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a - b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a - b),
            _ => CType::fail(
                "Attempting to subtract non-integer or non-float types together at compile time",
            ),
        }
    }
    pub fn mul(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a * b),
            _ => CType::fail(
                "Attempting to multiply non-integer or non-float types together at compile time",
            ),
        }
    }
    pub fn div(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a / b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a / b),
            _ => CType::fail(
                "Attempting to divide non-integer or non-float types together at compile time",
            ),
        }
    }
    pub fn cmod(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            _ => CType::fail("Attempting to modulus non-integer types together at compile time"),
        }
    }
    pub fn pow(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(match a.checked_pow(b as u32) {
                Some(c) => c,
                None => CType::fail("Compile time exponentiation too large"),
            }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a.powf(b)),
            _ => CType::fail(
                "Attempting to divide non-integer or non-float types together at compile time",
            ),
        }
    }
    pub fn cif(c: &CType, a: &CType, b: &CType) -> CType {
        match c {
            CType::Bool(cond) => match cond {
                true => a.clone(),
                false => b.clone(),
            },
            _ => CType::fail("If{C, A, B} must be given a boolean value as the condition"),
        }
    }
    pub fn tupleif(c: &CType, t: &CType) -> CType {
        match c {
            CType::Bool(cond) => {
                match t {
                    CType::Tuple(tup) => {
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
            _ => CType::fail("The first type provided to If{C, T} must be a boolean type"),
        }
    }
    pub fn envdefault(k: &CType, d: &CType) -> CType {
        match (k, d) {
            (CType::TString(s), CType::TString(def)) => match std::env::var(s) {
                Err(_) => CType::TString(def.clone()),
                Ok(v) => CType::TString(v),
            },
            _ => CType::fail("Env{K, D} must be provided a string for each type"),
        }
    }
    pub fn and(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a & *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a && *b),
            _ => CType::fail(
                "And{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    pub fn or(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a | *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a || *b),
            _ => CType::fail(
                "Or{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    pub fn xor(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a ^ *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a ^ *b),
            _ => CType::fail(
                "Or{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    pub fn nand(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a & *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a && *b)),
            _ => CType::fail("Nand{A, B} must be provided two values of the same type, either integer or boolean")
        }
    }
    pub fn nor(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a | *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a || *b)),
            _ => CType::fail(
                "Nor{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    pub fn xnor(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a ^ *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a ^ *b)),
            _ => CType::fail("Xnor{A, B} must be provided two values of the same type, either integer or boolean")
        }
    }
    pub fn eq(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a == *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a == *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a == *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a == *b),
            _ => CType::fail("Eq{A, B} must be provided two values of the same type, one of: integer, float, string, boolean"),
        }
    }
    pub fn neq(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a != *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a != *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a != *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a != *b),
            _ => CType::fail("Neq{A, B} must be provided two values of the same type, one of: integer, float, string, boolean"),
        }
    }
    pub fn lt(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a < *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a < *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a < *b),
            _ => CType::fail("Lt{A, B} must be provided two values of the same type, one of: integer, float, string"),
        }
    }
    pub fn lte(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a <= *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a <= *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a <= *b),
            _ => CType::fail("Lte{A, B} must be provided two values of the same type, one of: integer, float, string"),
        }
    }
    pub fn gt(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a > *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a > *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a > *b),
            _ => CType::fail("Gt{A, B} must be provided two values of the same type, one of: integer, float, string"),
        }
    }
    pub fn gte(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a >= *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a >= *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a >= *b),
            _ => CType::fail("Gte{A, B} must be provided two values of the same type, one of: integer, float, string"),
        }
    }
}

// TODO: I really hoped these two would share more code. Figure out how to DRY this out later, if
// possible
pub fn withtypeoperatorslist_to_ctype(
    withtypeoperatorslist: &Vec<parse::WithTypeOperators>,
    scope: &Scope,
    program: &Program,
) -> Result<CType, Box<dyn std::error::Error>> {
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
    while queue.len() > 0 {
        let mut largest_operator_level: i8 = -1;
        let mut largest_operator_index: i64 = -1;
        for (i, assignable_or_operator) in queue.iter().enumerate() {
            match assignable_or_operator {
                parse::WithTypeOperators::Operators(o) => {
                    let operatorname = o.trim();
                    let operator =
                        match program.resolve_typeoperator(scope, &operatorname.to_string()) {
                            Some(o) => Ok(o),
                            None => Err(format!("Operator {} not found", operatorname)),
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
                _ => {}
            }
        }
        if largest_operator_index > -1 {
            // We have at least one operator, and this is the one to dig into
            let operatorname = match &queue[largest_operator_index as usize] {
                parse::WithTypeOperators::Operators(o) => o.trim(),
                _ => unreachable!(),
            };
            let operator = match program.resolve_typeoperator(scope, &operatorname.to_string()) {
                Some(o) => Ok(o),
                None => Err(format!("Operator {} not found", operatorname)),
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
                        "Operator {} is an infix operator but missing a left-hand side value",
                        operatorname
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is an infix operator but preceded by another operator {}",
                        operatorname, o
                    )),
                }?;
                let second_arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!("Operator {} is an infix operator but missing a right-hand side value", operatorname)),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!("Operator{} is an infix operator but followed by a lower precedence operator {}", operatorname, o)),
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
                            parse::WithTypeOperators::Operators(",".to_string()),
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
                        "Operator {} is a prefix operator but missing a right-hand side value",
                        operatorname
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is an prefix operator but followed by another operator {}",
                        operatorname, o
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
                        "Operator {} is a postfix operator but missing a left-hand side value",
                        operatorname
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is a postfix operator but preceded by another operator {}",
                        operatorname, o
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
                return Err(format!("Invalid syntax: {:?}", withtypeoperatorslist).into());
            }
            let typebaselist = match match queue.pop() {
                Some(v) => Ok(v),
                None => Err(format!(
                    "Somehow we collapsed the statement into nothing? {:?}",
                    withtypeoperatorslist
                )),
            }? {
                parse::WithTypeOperators::TypeBaseList(b) => Ok(b),
                _ => Err(format!(
                    "Somehow we collapse the statement into a solitary operator? {:?}",
                    withtypeoperatorslist
                )),
            }?;
            out_ctype = Some(typebaselist_to_ctype(&typebaselist, scope, program)?);
        }
    }
    match out_ctype {
        Some(ctype) => Ok(ctype),
        None => Err(format!("Never resolved a type from {:?}", withtypeoperatorslist).into()),
    }
}

// TODO: This similarly shares a lot of structure with baseassignablelist_to_microstatements, see
// if there is any way to DRY this up, or is it just doomed to be like this?
pub fn typebaselist_to_ctype(
    typebaselist: &Vec<parse::TypeBase>,
    scope: &Scope,
    program: &Program,
) -> Result<CType, Box<dyn std::error::Error>> {
    let mut i = 0;
    let mut prior_value = None;
    while i < typebaselist.len() {
        let typebase = &typebaselist[i];
        let nexttypebase = typebaselist.get(i + 1);
        match typebase {
            parse::TypeBase::MethodSep(_) => {
                // The `MethodSep` symbol doesn't do anything on its own, it only validates that
                // the syntax before and after it is sane
                if prior_value.is_none() {
                    return Err(format!(
                        "Cannot start a statement with a property access: {}",
                        typebaselist
                            .iter()
                            .map(|tb| tb.to_string())
                            .collect::<Vec<String>>()
                            .join("")
                    )
                    .into());
                }
                match nexttypebase {
                    None => {
                        return Err(format!(
                            "Cannot end a statement with a property access: {}",
                            typebaselist
                                .iter()
                                .map(|tb| tb.to_string())
                                .collect::<Vec<String>>()
                                .join("")
                        )
                        .into());
                    }
                    Some(next) => match next {
                        parse::TypeBase::GnCall(_) => {
                            return Err(format!(
                                "A generic function call is not a property: {}",
                                typebaselist
                                    .iter()
                                    .map(|tb| tb.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                            )
                            .into());
                        }
                        parse::TypeBase::TypeGroup(_) => {
                            return Err(format!(
                                "A parenthetical grouping is not a property: {}",
                                typebaselist
                                    .iter()
                                    .map(|tb| tb.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                            )
                            .into());
                        }
                        parse::TypeBase::MethodSep(_) => {
                            return Err(format!(
                                "Too many `.` symbols for the method access: {}",
                                typebaselist
                                    .iter()
                                    .map(|tb| tb.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                            )
                            .into());
                        }
                        _ => {}
                    },
                }
            }
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
                // Similarly, the only thing that can follow a constant value is a `MethodSep` to
                // be used for a method-syntax function call and all others are errors. The
                // "default" path is for a typebaselist with a length of one containing only the
                // constant.
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
                        parse::TypeBase::MethodSep(_) => {} // The only allowed follow-up
                    }
                }
                if prior_value.is_none() {
                    match c {
                        parse::Constants::Bool(b) => {
                            prior_value = Some(CType::Bool(match b.as_str() {
                                "true" => true,
                                _ => false,
                            }))
                        }
                        parse::Constants::Strn(s) => {
                            prior_value = Some(CType::TString(if s.starts_with('"') {
                                s.clone()
                            } else {
                                // TODO: Is there a cheaper way to do this conversion?
                                s.replace("\"", "\\\"")
                                    .replace("\\'", "\\\\\"")
                                    .replace("'", "\"")
                                    .replace("\\\\\"", "'")
                            }))
                        }
                        parse::Constants::Num(n) => match n {
                            parse::Number::RealNum(r) => {
                                prior_value = Some(CType::Float(
                                    r.replace("_", "").parse::<f64>().unwrap(), // This should never fail if the
                                                                                // parser says it's a float
                                ))
                            }
                            parse::Number::IntNum(i) => {
                                prior_value = Some(CType::Int(
                                    i.replace("_", "").parse::<i128>().unwrap(), // Same deal here
                                ))
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
                            prior_value = Some(match prior_value.unwrap() {
                                CType::Tuple(ts) => {
                                    let mut out = None;
                                    for t in &ts {
                                        match t {
                                            CType::Field(f, c) => {
                                                if f.as_str() == s.as_str() {
                                                    out = Some(*c.clone());
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    match out {
                                        Some(o) => o,
                                        None => CType::fail(&format!("{:?} does not have a property named {}", ts, s)),
                                    }
                                }
                                CType::Function(i, o) => match s.as_str() {
                                    "input" => *i.clone(),
                                    "output" => *o.clone(),
                                    _ => CType::fail("Function types only have \"input\" and \"output\" properties"),
                                }
                                other => CType::fail(&format!("String properties are not allowed on {:?}", other)),
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
                                    prior_value = Some(match prior_value.unwrap() {
                                        CType::Tuple(ts) => match ts.get(idx) {
                                            Some(t) => t.clone(),
                                            None => CType::fail(&format!(
                                                "{} is larger than the size of {:?}",
                                                idx, ts
                                            )),
                                        },
                                        CType::Either(ts) => match ts.get(idx) {
                                            Some(t) => t.clone(),
                                            None => CType::fail(&format!(
                                                "{} is larger than the size of {:?}",
                                                idx, ts
                                            )),
                                        },
                                        other => CType::fail(&format!(
                                            "{:?} cannot be indexed by an integer",
                                            other
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
                match &prior_value {
                    Some(val) => args.push(val.clone()),
                    None => {}
                };
                prior_value = Some(match program.resolve_type(scope, var) {
                    Some(t) => {
                        // TODO: Once interfaces are a thing, there needs to be a built-in
                        // interface called `Label` that we can use here to mark the first argument
                        // to `Field` as a `Label` and turn this logic into something regularized
                        // For now, we're just special-casing the `Field` built-in generic type.
                        match &t {
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
                                            args.push(CType::TString(g.typecalllist[0].to_string()));
                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[2].clone()], scope, program)?);
                                        }
                                        parse::TypeBase::MethodSep(_) => {},
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
                                                if let parse::WithTypeOperators::Operators(a) = &arg {
                                                    if a.trim() == "," {
                                                        // Process the arg block that has
                                                        // accumulated
                                                        args.push(withtypeoperatorslist_to_ctype(&arg_block, scope, program)?);
                                                        arg_block.clear();
                                                        continue;
                                                    }
                                                }
                                                arg_block.push(arg);
                                            }
                                            if arg_block.len() > 0 {
                                                args.push(withtypeoperatorslist_to_ctype(&arg_block, scope, program)?);
                                            }
                                        }
                                        parse::TypeBase::MethodSep(_) => {},
                                        _ => CType::fail("Cannot follow method style syntax without an operator in between"),
                                    }
                                }
                            }
                        }
                        // Now, we need to validate that the resolved type *is* a generic
                        // type that can be called, and that we have the correct number of
                        // arguments for it, then we can call it and return the resulting
                        // type
                        match t {
                            CType::Generic(_name, params, withtypeoperatorslist) => {
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
                                    let mut temp_scope = scope.temp_child();
                                    for i in 0..params.len() {
                                        CType::from_ctype(
                                            &mut temp_scope,
                                            params[i].clone(),
                                            args[i].clone(),
                                        );
                                    }
                                    // Now we return the type we resolve within this
                                    // scope
                                    withtypeoperatorslist_to_ctype(
                                        withtypeoperatorslist,
                                        &temp_scope,
                                        program,
                                    )?
                                }
                            }
                            CType::IntrinsicGeneric(name, len) => {
                                if args.len() != *len {
                                    CType::fail(&format!(
                                        "Generic type {} takes {} arguments but {} given",
                                        var,
                                        len,
                                        args.len()
                                    ))
                                } else {
                                    // TODO: Is there a better way to do this?
                                    match name.as_str() {
                                        "Group" => CType::Group(Box::new(args[0].clone())),
                                        "Function" => CType::Function(
                                            Box::new(args[0].clone()),
                                            Box::new(args[1].clone()),
                                        ),
                                        "Tuple" => CType::tuple(args.clone()),
                                        // TODO: Field should ideally not require string
                                        // quoting
                                        "Field" => CType::field(args.clone()),
                                        "Either" => CType::either(args.clone()),
                                        "Buffer" => CType::buffer(args.clone()),
                                        "Array" => CType::Array(Box::new(args[0].clone())),
                                        "Fail" => CType::cfail(&args[0]),
                                        "Min" => CType::min(&args[0], &args[1]),
                                        "Max" => CType::max(&args[0], &args[1]),
                                        "Neg" => CType::neg(&args[0]),
                                        "Len" => CType::len(&args[0]),
                                        "Size" => CType::size(&args[0]),
                                        "FileStr" => CType::filestr(&args[0]),
                                        "Env" => CType::env(&args[0]),
                                        "EnvExists" => CType::envexists(&args[0]),
                                        "Not" => CType::not(&args[0]),
                                        "Add" => CType::add(&args[0], &args[1]),
                                        "Sub" => CType::sub(&args[0], &args[1]),
                                        "Mul" => CType::mul(&args[0], &args[1]),
                                        "Div" => CType::div(&args[0], &args[1]),
                                        "Mod" => CType::cmod(&args[0], &args[1]),
                                        "Pow" => CType::pow(&args[0], &args[1]),
                                        "If" => CType::tupleif(&args[0], &args[1]),
                                        "And" => CType::and(&args[0], &args[1]),
                                        "Or" => CType::or(&args[0], &args[1]),
                                        "Xor" => CType::xor(&args[0], &args[1]),
                                        "Nand" => CType::nand(&args[0], &args[1]),
                                        "Nor" => CType::nor(&args[0], &args[1]),
                                        "Xnor" => CType::xnor(&args[0], &args[1]),
                                        "Eq" => CType::eq(&args[0], &args[1]),
                                        "Neq" => CType::neq(&args[0], &args[1]),
                                        "Lt" => CType::lt(&args[0], &args[1]),
                                        "Lte" => CType::lte(&args[0], &args[1]),
                                        "Gt" => CType::gt(&args[0], &args[1]),
                                        "Gte" => CType::gte(&args[0], &args[1]),
                                        unknown => CType::fail(&format!(
                                            "Unknown ctype {} accessed. How did this happen?",
                                            unknown
                                        )),
                                    }
                                }
                            }
                            CType::BoundGeneric(name, argstrs, binding) => {
                                // We turn this into a `ResolvedBoundGeneric` for the lower layer
                                // of the compiler to make sense of
                                CType::ResolvedBoundGeneric(
                                    name.clone(),
                                    argstrs.clone(),
                                    args,
                                    binding.clone(),
                                )
                            }
                            others => {
                                // If we hit this branch, then the `args` vector needs to have a
                                // length of zero, and then we just bubble up the type as-is
                                if args.len() == 0 {
                                    others.clone()
                                } else {
                                    CType::fail(&format!(
                                        "{} is used as a generic type but is not one: {:?}, {:?}",
                                        var, others, prior_value,
                                    ))
                                }
                            }
                        }
                    }
                    None => CType::fail(&format!("{} is not a valid generic type name", var)),
                })
            }
            parse::TypeBase::GnCall(_) => { /* We always process GnCall in the Variable path */ }
            parse::TypeBase::TypeGroup(g) => {
                if g.typeassignables.len() == 0 {
                    // It's a void type!
                    prior_value = Some(CType::Group(Box::new(CType::Void)));
                } else {
                    // Simply wrap the returned type in a `CType::Group`
                    prior_value = Some(CType::Group(Box::new(withtypeoperatorslist_to_ctype(
                        &g.typeassignables,
                        scope,
                        program,
                    )?)));
                }
            }
        };
        i = i + 1;
    }
    match prior_value {
        Some(p) => Ok(p),
        None => Err("Somehow did not resolve the type definition into anything".into()),
    }
}
