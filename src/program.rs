use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::pin::Pin;

use crate::parse;

use ordered_hash_map::OrderedHashMap;

// This data structure should allow file-level reloading, which we can probably use as a rough
// approximation for iterative recompliation and language server support, and since Rust is fast,
// this might just be "good enough" assuming non-insane source file sizes.
#[derive(Debug)]
pub struct Program {
    pub entry_file: String,
    pub scopes_by_file: OrderedHashMap<String, (Pin<Box<String>>, parse::Ln, Scope)>,
}

impl Program {
    pub fn new(entry_file: String) -> Result<Program, Box<dyn std::error::Error>> {
        let mut p = Program {
            entry_file: entry_file.clone(),
            scopes_by_file: OrderedHashMap::new(),
        };
        // Add the root scope that will always be checked as if part of the current scope before
        // failure to resolve
        p = p.load("@root".to_string())?;
        // Load the entry file
        p = match p.load(entry_file) {
            Ok(p) => p,
            Err(_) => {
                // Somehow, trying to print this error can crash Rust!? Really not good.
                // Will need to figure out how to make these errors clearer to users.
                return Err("Failed to load entry file".into());
            }
        };
        Ok(p)
    }

    pub fn load(self, path: String) -> Result<Program, Box<dyn std::error::Error>> {
        let ln_src = if path.starts_with("@") {
            match path.as_str() {
                "@root" => include_str!("./std/root.ln").to_string(),
                "@std/app" => include_str!("./std/app.ln").to_string(),
                _ => {
                    return Err(format!("Unknown standard library named {}", &path).into());
                }
            }
        } else {
            read_to_string(&path)?
        };
        let (mut p, tuple) = Scope::from_src(self, &path, ln_src)?;
        p.scopes_by_file.insert(path, tuple);
        Ok(p)
    }

    pub fn resolve_typeoperator<'a>(
        self: &'a Self,
        scope: &'a Scope,
        typeoperatorname: &String,
    ) -> Option<(&TypeOperatorMapping, &Scope)> {
        // Tries to find the specified operator within the portion of the program accessible from the
        // current scope (so first checking the current scope, then all imports, then the root
        // scope) Returns a reference to the type and the scope it came from.
        // TODO: type ambiguity, infix/prefix ambiguity, etc
        match scope.typeoperatormappings.get(typeoperatorname) {
            Some(o) => Some((o, scope)),
            None => {
                // TODO: Loop over imports looking for the type
                match self.scopes_by_file.get("@root") {
                    None => None,
                    Some((_, _, root_scope)) => {
                        match &root_scope.typeoperatormappings.get(typeoperatorname) {
                            Some(o) => Some((&o, &root_scope)),
                            None => None,
                        }
                    }
                }
            }
        }
    }

    pub fn resolve_type<'a>(
        self: &'a Self,
        scope: &'a Scope,
        typename: &String,
    ) -> Option<(&CType, &Scope)> {
        // Tries to find the specified type within the portion of the program accessible from the
        // current scope (so first checking the current scope, then all imports, then the root
        // scope) Returns a reference to the type and the scope it came from.
        // TODO: Generics and Interfaces complicates this. If given a name that is a concrete
        // version of a generic, it should try to create said generic in the calling scope and then
        // return that if it can't find it already created. This means we need mutable access,
        // which complicated this function's call signature. Further, if the name provided is an
        // interface, we should instead return an array of types that could potentially fit the
        // bill. If the provided typename is a generic type with one of the type parameters being
        // an interface, we may need to provide all possible realized types for all types that
        // match the interface?
        match scope.types.get(typename) {
            Some(t) => Some((t, scope)),
            None => {
                // TODO: Loop over imports looking for the type
                match self.scopes_by_file.get("@root") {
                    None => None,
                    Some((_, _, root_scope)) => match &root_scope.types.get(typename) {
                        Some(t) => Some((&t, &root_scope)),
                        None => None,
                    },
                }
            }
        }
    }

    pub fn resolve_operator<'a>(
        self: &'a Self,
        scope: &'a Scope,
        operatorname: &String,
    ) -> Option<(&OperatorMapping, &Scope)> {
        // Tries to find the specified operator within the portion of the program accessible from the
        // current scope (so first checking the current scope, then all imports, then the root
        // scope) Returns a reference to the type and the scope it came from.
        // TODO: type ambiguity, infix/prefix ambiguity, etc
        match scope.operatormappings.get(operatorname) {
            Some(o) => Some((o, scope)),
            None => {
                // TODO: Loop over imports looking for the type
                match self.scopes_by_file.get("@root") {
                    None => None,
                    Some((_, _, root_scope)) => {
                        match &root_scope.operatormappings.get(operatorname) {
                            Some(o) => Some((&o, &root_scope)),
                            None => None,
                        }
                    }
                }
            }
        }
    }

    pub fn resolve_function<'a>(
        self: &'a Self,
        scope: &'a Scope,
        function: &String,
        args: &Vec<CType>,
    ) -> Option<(&Function, &Scope)> {
        // Tries to find the specified function within the portion of the program accessible from
        // the current scope (so first checking the current scope, then all imports, then the root
        // scope). It checks against the args array to find a match. TODO: Go beyond exact matching
        // Returns a reference to the function and the scope it came from.
        match scope.functions.get(function) {
            Some(fs) => {
                for f in fs {
                    // TODO: Handle this more generically, and in a way that allows users to write
                    // variadic functions
                    let mut args_match = true;
                    if let FnKind::DerivedVariadic = f.kind {
                        // The special path where the length doesn't matter as long as all of the
                        // actual args are the same type as the function's arg.
                        for arg in args.iter() {
                            if f.args[0].1.to_strict_string(false) != arg.to_strict_string(false) {
                                args_match = false;
                                break;
                            }
                        }
                    } else {
                        if args.len() != f.args.len() {
                            continue;
                        }
                        for (i, arg) in args.iter().enumerate() {
                            // This is pretty cheap, but for now, a "non-strict" string representation
                            // of the CTypes is how we'll match the args against each other. TODO: Do
                            // this without constructing a string to compare against each other.
                            if f.args[i].1.to_strict_string(false) != arg.to_strict_string(false) {
                                args_match = false;
                                break;
                            }
                        }
                    }
                    if args_match {
                        return Some((f, scope));
                    }
                }
                None
            }
            None => {
                // TODO: Loop over imports looking for the function
                match self.scopes_by_file.get("@root") {
                    None => None,
                    Some((_, _, root_scope)) => match root_scope.functions.get(function) {
                        Some(fs) => {
                            for f in fs {
                                if args.len() != f.args.len() {
                                    continue;
                                }
                                let mut args_match = true;
                                for (i, arg) in args.iter().enumerate() {
                                    // This is pretty cheap, but for now, a "non-strict" string representation
                                    // of the CTypes is how we'll match the args against each other. TODO: Do
                                    // this without constructing a string to compare against each other.
                                    if f.args[i].1.to_strict_string(false)
                                        != arg.to_strict_string(false)
                                    {
                                        args_match = false;
                                        break;
                                    }
                                }
                                if args_match {
                                    return Some((f, &root_scope));
                                }
                            }
                            None
                        }
                        None => None,
                    },
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub imports: OrderedHashMap<String, Import>,
    pub types: OrderedHashMap<String, CType>,
    pub consts: OrderedHashMap<String, Const>,
    pub functions: OrderedHashMap<String, Vec<Function>>,
    pub operatormappings: OrderedHashMap<String, OperatorMapping>,
    pub typeoperatormappings: OrderedHashMap<String, TypeOperatorMapping>,
    pub exports: OrderedHashMap<String, Export>,
    // TODO: Implement these other concepts
    // interfaces: OrderedHashMap<String, Interface>,
    // Should we include something for documentation?
}

impl Scope {
    fn from_src(
        program: Program,
        path: &String,
        src: String,
    ) -> Result<(Program, (Pin<Box<String>>, parse::Ln, Scope)), Box<dyn std::error::Error>> {
        let txt = Box::pin(src);
        let txt_ptr: *const str = &**txt;
        // *How* would this move, anyways? But TODO: See if there's a way to handle this safely
        let ast = unsafe { parse::get_ast(&*txt_ptr)? };
        let mut s = Scope {
            imports: OrderedHashMap::new(),
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            operatormappings: OrderedHashMap::new(),
            typeoperatormappings: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        };
        let mut p = program;
        for i in ast.imports.iter() {
            p = Import::from_ast(p, path.clone(), &mut s, i)?;
        }
        for (i, element) in ast.body.iter().enumerate() {
            match element {
                parse::RootElements::Types(t) => CType::from_ast(&mut s, &mut p, t, false)?,
                parse::RootElements::Functions(f) => Function::from_ast(&mut s, &mut p, f, false)?,
                parse::RootElements::ConstDeclaration(c) => Const::from_ast(&mut s, c, false)?,
                parse::RootElements::OperatorMapping(o) => {
                    OperatorMapping::from_ast(&mut s, o, false)?
                }
                parse::RootElements::TypeOperatorMapping(o) => {
                    TypeOperatorMapping::from_ast(&mut s, o, false)?
                }
                parse::RootElements::Exports(e) => match &e.exportable {
                    parse::Exportable::Functions(f) => Function::from_ast(&mut s, &mut p, f, true)?,
                    parse::Exportable::ConstDeclaration(c) => Const::from_ast(&mut s, c, true)?,
                    parse::Exportable::OperatorMapping(o) => {
                        OperatorMapping::from_ast(&mut s, o, true)?
                    }
                    parse::Exportable::TypeOperatorMapping(o) => {
                        TypeOperatorMapping::from_ast(&mut s, o, true)?
                    }
                    parse::Exportable::Types(t) => CType::from_ast(&mut s, &mut p, t, true)?,
                    parse::Exportable::CTypes(c) => {
                        // For now this is just declaring in the Alan source code the compile-time
                        // types that can be used, and is simply a special kind of documentation.
                        // *Only* the root scope is allowed to use this syntax, and I cannot imagine
                        // any other way, since the compiler needs to exactly match what is declared.
                        // So we return an error if they're encountered outside of the root scope and
                        // simply verify that each `ctype` we encounter is one of a set the compiler
                        // expects to exist. Later when `cfn` is implemented these will be loaded up
                        // for verification of the meta-typing of the compile-time functions.
                        // This is also an exception in that it is *only* allowed to be exported
                        // (from the root scope) and can't be hidden, as all code will need these
                        // to construct their own types.
                        if path == "@root" {
                            match c.name.as_str() {
                                "Type" | "Generic" | "Bound" | "BoundGeneric" | "Int" | "Float"
                                | "Bool" | "String" => { /* Do nothing for the 'structural' types */ }
                                g @ ("Group" | "Array" | "Fail" | "Len" | "Size" | "FileStr"
                                | "Env" | "EnvExists" | "Not") => CType::from_generic(&mut s, g, 1),
                                g @ ("Function" | "Tuple" | "Field" | "Either" | "Buffer" | "Add"
                                | "Sub" | "Mul" | "Div" | "Mod" | "Pow" | "If" | "And" | "Or"
                                | "Xor" | "Nand" | "Nor" | "Xnor" | "Eq" | "Neq" | "Lt" | "Lte"
                                | "Gt" | "Gte") => CType::from_generic(&mut s, g, 2),
                                // TODO: Also add support for three arg `If` and `Env` with a
                                // default property via overloading types
                                unknown => {
                                    return Err(format!("Unknown ctype {} defined in root scope. There's something wrong with the compiler.", unknown).into());
                                }
                            }
                        } else {
                            return Err(
                                "ctypes can only be defined in the compiler internals".into()
                            );
                        }
                    }
                    e => eprintln!("TODO: Not yet supported export syntax: {:?}\nLast good parsed lines:\n{:?}\n{:?}", e, ast.body[i - 2], ast.body[i - 1]),
                },
                parse::RootElements::Whitespace(_) => { /* Do nothing */ }
                parse::RootElements::Interfaces(_) => {
                    return Err("Interfaces not yet implemented".into());
                }
            }
        }
        Ok((p, (txt, ast, s)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CType {
    Void,
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
                    .map(|b| b.to_string())
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
    fn from_ast(
        scope: &mut Scope,
        program: &mut Program,
        type_ast: &parse::Types,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = type_ast.fulltypename.typename.clone();
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
                        let constructor_fn_name = type_ast.fulltypename.typename.clone();
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
        scope.types.insert(name, t);
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
        Ok(())
    }

    fn from_ctype(scope: &mut Scope, name: String, ctype: CType) {
        scope.exports.insert(name.clone(), Export::Type);
        scope.types.insert(name, ctype);
    }

    fn from_generic(scope: &mut Scope, name: &str, arglen: usize) {
        CType::from_ctype(
            scope,
            name.to_string(),
            CType::IntrinsicGeneric(name.to_string(), arglen),
        )
    }
    // Special implementation for the tuple and either types since they *are* CTypes, but if one of
    // the provided input types *is* the same kind of CType, it should produce a merged version.
    fn tuple(args: Vec<CType>) -> CType {
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
    fn either(args: Vec<CType>) -> CType {
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
    fn field(mut args: Vec<CType>) -> CType {
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
    fn buffer(mut args: Vec<CType>) -> CType {
        if args.len() != 2 {
            CType::fail("Buffer{T, S} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            match (arg0, arg1) {
                (anything, CType::Int(size)) => {
                    CType::Buffer(Box::new(anything.clone()), size as usize)
                }
                _ => CType::fail("The buffer size must be a positive integer"),
            }
        }
    }
    // Implementation of the ctypes that aren't storage but compute into another CType
    fn fail(message: &str) -> ! {
        // TODO: Include more information on where this compiler exit is coming from
        eprintln!("{}", message);
        std::process::exit(1);
    }
    fn cfail(message: &CType) -> ! {
        match message {
            CType::TString(s) => CType::fail(&s),
            _ => CType::fail("Fail passed a type that does not resolve into a message string"),
        }
    }
    fn len(t: &CType) -> CType {
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
    fn size(_t: &CType) -> CType {
        // TODO: Implementing this might require all types be made C-style structs under the hood,
        // and probably some weird hackery to find out the size including padding on aligned
        // architectures, so I might take it back out before its actually implemented, but I can
        // think of several places where knowing the actual size of the type could be useful,
        // particularly for writing to disk or interfacing with network protocols, etc, so I'd
        // prefer to keep it and have some compile-time guarantees we don't normally see.
        CType::fail("TODO: Implement Size{T}!")
    }
    fn filestr(f: &CType) -> CType {
        match f {
            CType::TString(s) => match std::fs::read_to_string(s) {
                Err(e) => CType::fail(&format!("Failed to read {}: {:?}", s, e)),
                Ok(s) => CType::TString(s),
            },
            _ => CType::fail("FileStr{F} must be given a string path to load"),
        }
    }
    fn env(k: &CType) -> CType {
        match k {
            CType::TString(s) => match std::env::var(s) {
                Err(e) => CType::fail(&format!(
                    "Failed to load environment variable {}: {:?}",
                    s, e
                )),
                Ok(s) => CType::TString(s),
            },
            _ => CType::fail("Env{K} must be given a key as a string to load"),
        }
    }
    fn envexists(k: &CType) -> CType {
        match k {
            CType::TString(s) => match std::env::var(s) {
                Err(_) => CType::Bool(false),
                Ok(_) => CType::Bool(true),
            },
            _ => CType::fail("EnvExists{K} must be given a key as a string to check"),
        }
    }
    fn not(b: &CType) -> CType {
        match b {
            CType::Bool(b) => CType::Bool(!*b),
            _ => CType::fail("Not{B} must be provided a boolean type to invert"),
        }
    }
    fn add(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a + b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a + b),
            _ => CType::fail(
                "Attempting to add non-integer or non-float types together at compile time",
            ),
        }
    }
    fn sub(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a - b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a - b),
            _ => CType::fail(
                "Attempting to subtract non-integer or non-float types together at compile time",
            ),
        }
    }
    fn mul(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a * b),
            _ => CType::fail(
                "Attempting to multiply non-integer or non-float types together at compile time",
            ),
        }
    }
    fn div(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a / b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a / b),
            _ => CType::fail(
                "Attempting to divide non-integer or non-float types together at compile time",
            ),
        }
    }
    fn cmod(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            _ => CType::fail("Attempting to modulus non-integer types together at compile time"),
        }
    }
    fn pow(a: &CType, b: &CType) -> CType {
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
    fn cif(c: &CType, a: &CType, b: &CType) -> CType {
        match c {
            CType::Bool(cond) => match cond {
                true => a.clone(),
                false => b.clone(),
            },
            _ => CType::fail("If{C, A, B} must be given a boolean value as the condition"),
        }
    }
    fn tupleif(c: &CType, t: &CType) -> CType {
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
    fn envdefault(k: &CType, d: &CType) -> CType {
        match (k, d) {
            (CType::TString(s), CType::TString(def)) => match std::env::var(s) {
                Err(_) => CType::TString(def.clone()),
                Ok(v) => CType::TString(v),
            },
            _ => CType::fail("Env{K, D} must be provided a string for each type"),
        }
    }
    fn and(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a & *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a && *b),
            _ => CType::fail(
                "And{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    fn or(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a | *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a || *b),
            _ => CType::fail(
                "Or{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    fn xor(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a ^ *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a ^ *b),
            _ => CType::fail(
                "Or{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    fn nand(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a & *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a && *b)),
            _ => CType::fail("Nand{A, B} must be provided two values of the same type, either integer or boolean")
        }
    }
    fn nor(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a | *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a || *b)),
            _ => CType::fail(
                "Nor{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    fn xnor(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a ^ *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a ^ *b)),
            _ => CType::fail("Xnor{A, B} must be provided two values of the same type, either integer or boolean")
        }
    }
    fn eq(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a == *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a == *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a == *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a == *b),
            _ => CType::fail("Eq{A, B} must be provided two values of the same type, one of: integer, float, string, boolean"),
        }
    }
    fn neq(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a != *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a != *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a != *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a != *b),
            _ => CType::fail("Neq{A, B} must be provided two values of the same type, one of: integer, float, string, boolean"),
        }
    }
    fn lt(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a < *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a < *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a < *b),
            _ => CType::fail("Lt{A, B} must be provided two values of the same type, one of: integer, float, string"),
        }
    }
    fn lte(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a <= *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a <= *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a <= *b),
            _ => CType::fail("Lte{A, B} must be provided two values of the same type, one of: integer, float, string"),
        }
    }
    fn gt(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a > *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a > *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a > *b),
            _ => CType::fail("Gt{A, B} must be provided two values of the same type, one of: integer, float, string"),
        }
    }
    fn gte(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a >= *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a >= *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a >= *b),
            _ => CType::fail("Gte{A, B} must be provided two values of the same type, one of: integer, float, string"),
        }
    }
}

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
    fn from_ast(
        program: Program,
        path: String,
        scope: &mut Scope,
        import_ast: &parse::ImportStatement,
    ) -> Result<Program, Box<dyn std::error::Error>> {
        match &import_ast {
            parse::ImportStatement::Standard(s) => {
                // First, get the path for the code
                let ln_file = s.dependency.resolve(path)?;
                let exists = match &program.scopes_by_file.get(&ln_file) {
                    Some(_) => true,
                    None => false,
                };
                let mut p = program;
                if !exists {
                    // Need to load this file into the program first
                    p = p.load(ln_file.clone())?;
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
                Ok(p)
            }
            parse::ImportStatement::From(f) => {
                let ln_file = f.dependency.resolve(path)?;
                let exists = match &program.scopes_by_file.get(&ln_file) {
                    Some(_) => true,
                    None => false,
                };
                let mut p = program;
                if !exists {
                    // Need to load this file into the program first
                    p = p.load(ln_file.clone())?;
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
                Ok(p)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Const {
    pub name: String,
    pub typename: Option<String>,
    pub assignables: Vec<parse::WithOperators>,
}

impl Const {
    fn from_ast(
        scope: &mut Scope,
        const_ast: &parse::ConstDeclaration,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = const_ast.variable.clone();
        let typename = match &const_ast.typedec {
            Some(typedec) => Some(typedec.fulltypename.to_string()),
            None => None,
        };
        let assignables = const_ast.assignables.clone();
        let c = Const {
            name,
            typename,
            assignables,
        };
        if is_export {
            scope.exports.insert(c.name.clone(), Export::Const);
        }
        scope.consts.insert(c.name.clone(), c);
        Ok(())
    }
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
        typen: CType,
    },
    FnCall {
        function: String, // TODO: It would be nice to make this a vector of pointers to function objects so we can narrow down the exact implementation sooner
        args: Vec<Microstatement>,
    },
    Value {
        typen: CType,
        representation: String, // TODO: Can we do better here?
    },
    Array {
        typen: CType,
        vals: Vec<Microstatement>,
    },
    Return {
        value: Option<Box<Microstatement>>,
    }, // TODO: Conditionals
}

impl Microstatement {
    pub fn get_type(
        &self,
        scope: &Scope,
        program: &Program,
    ) -> Result<CType, Box<dyn std::error::Error>> {
        match self {
            Self::Value { typen, .. } => Ok(typen.clone()),
            Self::Array { typen, .. } => Ok(typen.clone()),
            Self::Arg { typen, .. } => Ok(typen.clone()),
            Self::Assignment { value, .. } => value.get_type(scope, program),
            Self::Return { value } => match value {
                Some(v) => v.get_type(scope, program),
                None => Ok(CType::Void),
            },
            Self::FnCall { function, args } => {
                let mut arg_types = Vec::new();
                for arg in args {
                    let arg_type = arg.get_type(scope, program)?;
                    arg_types.push(arg_type);
                }
                match program.resolve_function(scope, function, &arg_types) {
                    Some((function_object, _s)) => Ok(function_object.rettype.clone()),
                    None => Err(format!("Could not find function {}", function).into()),
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum BaseChunk<'a> {
    IIGE(
        &'a Option<Microstatement>,
        &'a parse::Functions,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    GFuncCall(
        &'a Option<Microstatement>,
        &'a String,
        &'a parse::GnCall,
        Option<&'a parse::FnCall>,
    ),
    IIFE(
        &'a Option<Microstatement>,
        &'a parse::Functions,
        Option<&'a parse::FnCall>,
    ),
    FuncCall(
        &'a Option<Microstatement>,
        &'a String,
        Option<&'a parse::FnCall>,
    ),
    TypeCall(
        &'a Option<Microstatement>,
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

fn baseassignablelist_to_microstatements(
    bal: &Vec<parse::BaseAssignable>,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
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
            prior_value.clone(),
            bal.get(i),
            bal.get(i + 1),
            bal.get(i + 2),
            bal.get(i + 3),
        ) {
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
            ) => (BaseChunk::IIGE(&prior_value, f, g, Some(h)), 4),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
            ) => (BaseChunk::GFuncCall(&prior_value, f, g, Some(h)), 4),
            (
                None,
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
                _,
            ) => (BaseChunk::IIGE(&prior_value, f, g, Some(h)), 3),
            (
                None,
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                Some(parse::BaseAssignable::FnCall(h)),
                _,
            ) => (BaseChunk::GFuncCall(&prior_value, f, g, Some(h)), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                _,
            ) => (BaseChunk::IIGE(&prior_value, f, g, None), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::GnCall(g)),
                _,
            ) => (BaseChunk::GFuncCall(&prior_value, f, g, None), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::IIFE(&prior_value, f, Some(g)), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::FuncCall(&prior_value, f, Some(g)), 3),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::GnCall(t)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
            ) => (BaseChunk::TypeCall(&prior_value, t, Some(g)), 3),
            (
                None,
                Some(parse::BaseAssignable::Functions(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::IIFE(&prior_value, f, Some(g)), 2),
            (
                None,
                Some(parse::BaseAssignable::Variable(f)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::FuncCall(&prior_value, f, Some(g)), 2),
            (
                None,
                Some(parse::BaseAssignable::GnCall(t)),
                Some(parse::BaseAssignable::FnCall(g)),
                _,
                _,
            ) => (BaseChunk::TypeCall(&prior_value, t, Some(g)), 2),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Functions(f)),
                _,
                _,
            ) => (BaseChunk::IIFE(&prior_value, f, None), 2),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::Variable(f)),
                _,
                _,
            ) => (BaseChunk::FuncCall(&prior_value, f, None), 2),
            (
                Some(_),
                Some(parse::BaseAssignable::MethodSep(_)),
                Some(parse::BaseAssignable::GnCall(t)),
                _,
                _,
            ) => (BaseChunk::TypeCall(&prior_value, t, None), 2),
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
        i = i + inc;
        // Now we just operate on our chunk and create a new prior_value to replace the old one, if
        // any exists. We'll start from the easier ones first and work our way up to the
        // complicated ones
        match chunk {
            // We know it has to be defined because we blew up earlier if not
            BaseChunk::Constant(c) => {
                match c {
                    parse::Constants::Bool(b) => {
                        prior_value = Some(Microstatement::Value {
                            typen: CType::Bound("bool".to_string(), "bool".to_string()),
                            representation: b.clone(),
                        });
                    }
                    parse::Constants::Strn(s) => {
                        prior_value = Some(Microstatement::Value {
                            typen: CType::Bound("string".to_string(), "String".to_string()),
                            representation: if s.starts_with('"') {
                                s.clone()
                            } else {
                                // TODO: Is there a cheaper way to do this conversion?
                                s.replace("\"", "\\\"")
                                    .replace("\\'", "\\\\\"")
                                    .replace("'", "\"")
                                    .replace("\\\\\"", "'")
                            },
                        });
                    }
                    parse::Constants::Num(n) => match n {
                        parse::Number::RealNum(r) => {
                            prior_value = Some(Microstatement::Value {
                                // TODO: Replace this with the `CType::Float` and have built-ins
                                // that accept them
                                typen: CType::Bound("f64".to_string(), "f64".to_string()),
                                representation: r.clone(),
                            });
                        }
                        parse::Number::IntNum(i) => {
                            prior_value = Some(Microstatement::Value {
                                // TODO: Replace this with `CType::Int` and have built-ins that
                                // accept them
                                typen: CType::Bound("i64".to_string(), "i64".to_string()),
                                representation: i.clone(),
                            });
                        }
                    },
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
                        Microstatement::Assignment { value, .. } => value.get_type(scope, program),
                        Microstatement::Arg { typen, .. } => Ok(typen.clone()),
                        _ => unreachable!(),
                    },
                    None => {
                        // It could be a function. TODO: Use `program.resolve_function` for better
                        // type safety. This requires getting the types of all of the involved
                        // arguments
                        match scope.functions.get(v) {
                            // TODO: the specific function chosen may be wrong, need to use
                            // `program.resolve_function` here
                            Some(f) => Ok(CType::Function(
                                Box::new(if f[0].args.len() == 0 {
                                    CType::Void
                                } else {
                                    CType::Tuple(
                                        f[0].args
                                            .iter()
                                            .map(|(l, t)| {
                                                CType::Field(l.clone(), Box::new(t.clone()))
                                            })
                                            .collect::<Vec<CType>>(),
                                    )
                                }),
                                Box::new(f[0].rettype.clone()),
                            )), // TODO: Convert the Function class to use a
                            // function type directly
                            None => {
                                // Check the root scope, too
                                match program.scopes_by_file.get("@root") {
                                    Some((_, _, s)) => match s.functions.get(v) {
                                        Some(f) => Ok(CType::Function(
                                            Box::new(if f[0].args.len() == 0 {
                                                CType::Void
                                            } else {
                                                CType::Tuple(
                                                    f[0].args
                                                        .iter()
                                                        .map(|(l, t)| {
                                                            CType::Field(
                                                                l.clone(),
                                                                Box::new(t.clone()),
                                                            )
                                                        })
                                                        .collect::<Vec<CType>>(),
                                                )
                                            }),
                                            Box::new(f[0].rettype.clone()),
                                        )), // TODO: Convert the Function class to use a
                                        // function type directly
                                        None => Err(format!("Couldn't find variable {}", v).into()),
                                    },
                                    None => Err(format!("Couldn't find variable {}", v).into()),
                                }
                            }
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
                if a.assignablelist.len() == 0 {
                    return Err("Cannot create an empty array with bracket syntax, use `Array{MyType}()` syntax instead".into());
                }
                let mut array_vals = Vec::new();
                for wol in &a.assignablelist {
                    microstatements =
                        withoperatorslist_to_microstatements(wol, scope, program, microstatements)?;
                    array_vals.push(microstatements.pop().unwrap());
                }
                // TODO: Currently assuming all array values are the same type, should check that
                // better
                let inner_type = array_vals[0].get_type(scope, program)?;
                let array_type = CType::Array(Box::new(inner_type));
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
                microstatements = withoperatorslist_to_microstatements(
                    &g.assignablelist[0],
                    scope,
                    program,
                    microstatements,
                )?;
                prior_value = microstatements.pop();
            }
            BaseChunk::Function(_f) => {
                return Err("TODO: Implement closure functions in Microstatement syntax".into());
            }
            BaseChunk::ArrayAccessor(a) => {
                if let Some(prior) = &prior_value {
                    let mut array_accessor_microstatements = vec![prior.clone()];
                    for wol in &a.assignablelist {
                        microstatements = withoperatorslist_to_microstatements(
                            wol,
                            scope,
                            program,
                            microstatements,
                        )?;
                        array_accessor_microstatements.push(microstatements.pop().unwrap());
                    }
                    // TODO: Check that this function actually exists with the arguments being provided
                    prior_value = Some(Microstatement::FnCall {
                        function: "get".to_string(),
                        args: array_accessor_microstatements,
                    });
                } else {
                    // This is impossible, but I'm having a hard time convincing Rust of that
                    panic!("Impossible to reach the ArrayAccessor path without a prior value");
                }
            }
            BaseChunk::ConstantAccessor(c) => {
                if let Some(prior) = &prior_value {
                    let mut constant_accessor_microstatements = vec![prior.clone()];
                    microstatements = baseassignablelist_to_microstatements(
                        &vec![parse::BaseAssignable::Constants(c.clone())],
                        scope,
                        program,
                        microstatements,
                    )?;
                    constant_accessor_microstatements.push(microstatements.pop().unwrap());
                    // TODO: Check that this function actually exists with the argument being provided
                    prior_value = Some(Microstatement::FnCall {
                        function: "get".to_string(),
                        args: constant_accessor_microstatements,
                    });
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
                            microstatements = withoperatorslist_to_microstatements(
                                &arg,
                                scope,
                                program,
                                microstatements,
                            )?;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type(scope, program)?);
                }
                // We create a type on-the-fly from the contents the GnCall block. It's given a
                // name based on the CType tree with all non-`a-zA-Z0-9_` chars replaced with `-`
                // TODO: Eliminate the duplication of CType generation logic by abstracting out the
                // automatic function creation into a reusable component
                let ctype = withtypeoperatorslist_to_ctype(&g.typecalllist, scope, program)?;
                // TODO: Be more complete here
                let name = ctype
                    .to_strict_string(false)
                    .replace(" ", "_")
                    .replace(",", "_")
                    .replace(":", "_")
                    .replace("{", "_")
                    .replace("}", "_")
                    .replace("|", "_")
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
                    typedef: parse::TypeDef::TypeCreate(parse::TypeCreate {
                        a: "=".to_string(),
                        b: "".to_string(),
                        typeassignables: g.typecalllist.clone(),
                    }),
                    optsemicolon: ";".to_string(),
                };
                CType::from_ast(scope, program, &parse_type, false)?;
                // Now we are sure the type and function exist, and we know the name for the
                // function. It would be best if we could just pass it to ourselves and run the
                // `FuncCall` logic below, but it's easier at the moment to duplicate :( TODO
                prior_value = Some(Microstatement::FnCall {
                    function: name,
                    args: arg_microstatements,
                });
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
                            microstatements = withoperatorslist_to_microstatements(
                                &arg,
                                scope,
                                program,
                                microstatements,
                            )?;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type(scope, program)?);
                }
                // Now confirm that there's actually a function with this name that takes these
                // types
                let func = program.resolve_function(scope, f, &arg_types);
                match func {
                    Some(..) => {
                        // Success! Let's emit this
                        prior_value = Some(Microstatement::FnCall {
                            function: f.to_string(),
                            args: arg_microstatements,
                        });
                    }
                    None => {
                        return Err(format!(
                            "Could not find a function {} that takes args {}",
                            f,
                            arg_types
                                .iter()
                                .map(|a| a.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                        .into());
                    }
                }
            }
            BaseChunk::IIFE(_prior, _f, _g) => {
                // TODO: This may just be some simple microstatement generation here compared to
                // actual closure creation
                return Err("TODO: Implement IIFE support".into());
            }
            BaseChunk::GFuncCall(prior, f, g, h) => {
                // TODO: Actually implement generic functions, for now this is just another way to
                // do a `TypeCall`
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
                            microstatements = withoperatorslist_to_microstatements(
                                &arg,
                                scope,
                                program,
                                microstatements,
                            )?;
                            arg_microstatements.push(microstatements.pop().unwrap());
                        }
                    }
                }
                let mut arg_types = Vec::new();
                for arg in &arg_microstatements {
                    arg_types.push(arg.get_type(scope, program)?);
                }
                match program.resolve_type(scope, f) {
                    None => {
                        return Err("Generic functions not yet implemented".into());
                    }
                    Some(_) => {
                        // Confirmed that this type exists, we now need to generate a realized
                        // generic type for this specified type and shove it into the non-exported
                        // scope, then we can be sure we can call it.
                        let name = format!(
                            "{}{}",
                            f,
                            g.to_string()
                                .replace(" ", "_")
                                .replace(",", "_")
                                .replace(":", "_")
                                .replace("{", "_")
                                .replace("}", "_")
                        )
                        .replace("|", "_")
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
                            typedef: parse::TypeDef::TypeCreate(parse::TypeCreate {
                                a: "=".to_string(),
                                b: "".to_string(),
                                typeassignables: vec![parse::WithTypeOperators::TypeBaseList(
                                    vec![
                                        parse::TypeBase::Variable(f.to_string()),
                                        parse::TypeBase::GnCall(g.clone()),
                                    ],
                                )],
                            }),
                            optsemicolon: ";".to_string(),
                        };
                        CType::from_ast(scope, program, &parse_type, false)?;
                        // Now we are sure the type and function exist, and we know the name for the
                        // function. It would be best if we could just pass it to ourselves and run the
                        // `FuncCall` logic below, but it's easier at the moment to duplicate :( TODO
                        prior_value = Some(Microstatement::FnCall {
                            function: name,
                            args: arg_microstatements,
                        });
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
    Ok(microstatements)
}

// TODO: I really hoped these two would share more code. Figure out how to DRY this out later, if
// possible
fn withtypeoperatorslist_to_ctype(
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
                    let (operator, _) =
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
            let (operator, _) = match program.resolve_typeoperator(scope, &operatorname.to_string())
            {
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
                    None => Err(format!("Operator {} is a postfix operator but missing a left-hand side value", operatorname)),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!("Operator {} is a postfix operator but preceded by another operator {}", operatorname, o)),
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
                        (largest_operator_index as usize - 1)..(largest_operator_index as usize + 1),
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
fn typebaselist_to_ctype(
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
                                    r.parse::<f64>().unwrap(), // This should never fail if the
                                                               // parser says it's a float
                                ))
                            }
                            parse::Number::IntNum(i) => {
                                prior_value = Some(CType::Int(
                                    i.parse::<i128>().unwrap(), // Same deal here
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
                    Some((t, _)) => {
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
                                            if g.typecalllist.len() != 2 {
                                                CType::fail("The Field generic type accepts only two parameters");
                                            }
                                            args.push(CType::TString(g.typecalllist[0].to_string()));
                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[1].clone()], scope, program)?);
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
                                                /*match ta {
                                                    parse::WithTypeOperators::Operators(s) if s.trim() == "," => {
                                                        temp_args.push(arg.clone());
                                                        arg.clear();
                                                    }
                                                    _ => {
                                                      arg.push(ta.clone());
                                                    }
                                                }*/
                                            }
                                            for arg in temp_args {
                                                if let parse::WithTypeOperators::Operators(a) = &arg {
                                                    if a.trim() == "," {
                                                        continue;
                                                    }
                                                }
                                                args.push(withtypeoperatorslist_to_ctype(&vec![arg], scope, program)?);
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
                                    let mut temp_scope = scope.clone();
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

fn withoperatorslist_to_microstatements(
    withoperatorslist: &Vec<parse::WithOperators>,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    // To properly linearize the operations here, we need to scan through all of the operators,
    // determine which is the highest precedence, whether it is infix or prefix (or maybe postfix
    // in the future?) and then process them and whichever of the baseassignables surrounding them
    // are associated, then put those results in the same "slot" as before and check again. Because
    // users can define these operators, that makes it theoretically possible for the same operator
    // to be used in both an infix or prefix manner, or with different precedence levels, depending
    // on the types of the data involved, which makes things *really* complicated here. TODO:
    // Actually implement that complexity, for now, just pretend operators have only one binding.
    let mut queue = withoperatorslist.clone();
    while queue.len() > 0 {
        let mut largest_operator_level: i8 = -1;
        let mut largest_operator_index: i64 = -1;
        for (i, assignable_or_operator) in queue.iter().enumerate() {
            match assignable_or_operator {
                parse::WithOperators::Operators(o) => {
                    let operatorname = o.trim();
                    let (operator, _) =
                        match program.resolve_operator(scope, &operatorname.to_string()) {
                            Some(o) => Ok(o),
                            None => Err(format!("Operator {} not found", operatorname)),
                        }?;
                    let level = match &operator {
                        OperatorMapping::Prefix { level, .. } => level,
                        OperatorMapping::Infix { level, .. } => level,
                        OperatorMapping::Postfix { level, .. } => level,
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
                parse::WithOperators::Operators(o) => o.trim(),
                _ => unreachable!(),
            };
            let (operator, _) = match program.resolve_operator(scope, &operatorname.to_string()) {
                Some(o) => Ok(o),
                None => Err(format!("Operator {} not found", operatorname)),
            }?;
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
                        operatorname
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => {
                        Ok(baseassignablelist)
                    }
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is an infix operator but preceded by another operator {}",
                        operatorname, o
                    )),
                }?;
                let second_arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!("Operator {} is an infix operator but missing a right-hand side value", operatorname)),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => Ok(baseassignablelist),
                    parse::WithOperators::Operators(o) => Err(format!("Operator{} is an infix operator but followed by a lower precedence operator {}", operatorname, o)),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `a + b` and turn it into `add(a, b)`
                let rewrite = parse::WithOperators::BaseAssignableList(vec![
                    parse::BaseAssignable::Variable(functionname),
                    parse::BaseAssignable::FnCall(parse::FnCall {
                        openparen: "(".to_string(),
                        a: "".to_string(),
                        assignablelist: vec![
                            vec![parse::WithOperators::BaseAssignableList(first_arg.to_vec())],
                            vec![parse::WithOperators::BaseAssignableList(
                                second_arg.to_vec(),
                            )],
                        ],
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
                        operatorname
                    )),
                }? {
                    parse::WithOperators::BaseAssignableList(baseassignablelist) => {
                        Ok(baseassignablelist)
                    }
                    parse::WithOperators::Operators(o) => Err(format!(
                        "Operator {} is an prefix operator but followed by another operator {}",
                        operatorname, o
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `#array` and turn it into `len(array)`
                let rewrite = parse::WithOperators::BaseAssignableList(vec![
                    parse::BaseAssignable::Variable(functionname),
                    parse::BaseAssignable::FnCall(parse::FnCall {
                        openparen: "(".to_string(),
                        a: "".to_string(),
                        assignablelist: vec![vec![parse::WithOperators::BaseAssignableList(
                            arg.to_vec(),
                        )]],
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
                // TODO: Add postfix operator support here
            }
        } else {
            // We have no more operators, there should only be one reworked baseassignablelist now
            if queue.len() != 1 {
                // No idea how such a wonky thing could occur. TODO: Improve error message
                return Err(format!("Invalid syntax: {:?}", withoperatorslist).into());
            }
            let baseassignablelist = match match queue.pop() {
                Some(v) => Ok(v),
                None => Err(format!(
                    "Somehow we collapsed the statement into nothing? {:?}",
                    withoperatorslist
                )),
            }? {
                parse::WithOperators::BaseAssignableList(b) => Ok(b),
                _ => Err(format!(
                    "Somehow we collapse the statement into a solitary operator? {:?}",
                    withoperatorslist
                )),
            }?;
            microstatements = baseassignablelist_to_microstatements(
                &baseassignablelist,
                scope,
                program,
                microstatements,
            )?;
        }
    }
    Ok(microstatements)
}

fn assignablestatement_to_microstatements(
    assignable: &parse::AssignableStatement,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    microstatements = withoperatorslist_to_microstatements(
        &assignable.assignables,
        scope,
        program,
        microstatements,
    )?;
    Ok(microstatements)
}

fn returns_to_microstatements(
    returns: &parse::Returns,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    if let Some(retval) = &returns.retval {
        // We get all of the microstatements involved in the return statement, then we pop
        // off the last one, if any exists, to get the final return value. Then we shove
        // the other microstatements into the array and the new Return microstatement with
        // that last value attached to it.
        microstatements = withoperatorslist_to_microstatements(
            &retval.assignables,
            scope,
            program,
            microstatements,
        )?;
        let value = match microstatements.pop() {
            None => None,
            Some(v) => Some(Box::new(v)),
        };
        microstatements.push(Microstatement::Return { value });
    } else {
        microstatements.push(Microstatement::Return { value: None });
    }
    Ok(microstatements)
}

fn declarations_to_microstatements(
    declarations: &parse::Declarations,
    scope: &mut Scope,
    program: &mut Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    let (name, assignables, mutable) = match &declarations {
        parse::Declarations::Const(c) => (c.variable.clone(), &c.assignables, false),
        parse::Declarations::Let(l) => (l.variable.clone(), &l.assignables, true),
    };
    // Get all of the assignable microstatements generated
    microstatements =
        withoperatorslist_to_microstatements(assignables, scope, program, microstatements)?;
    let value = match microstatements.pop() {
        None => Err("An assignment without a value should be impossible."),
        Some(v) => Ok(Box::new(v)),
    }?;
    microstatements.push(Microstatement::Assignment {
        name,
        value,
        mutable,
    });
    Ok(microstatements)
}

fn statement_to_microstatements(
    statement: &parse::Statement,
    scope: &mut Scope,
    program: &mut Program,
    microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    match statement {
        // This is just whitespace, so we do nothing here
        parse::Statement::A(_) => Ok(microstatements),
        parse::Statement::Assignables(assignable) => Ok(assignablestatement_to_microstatements(
            assignable,
            scope,
            program,
            microstatements,
        )?),
        parse::Statement::Returns(returns) => Ok(returns_to_microstatements(
            returns,
            scope,
            program,
            microstatements,
        )?),
        parse::Statement::Declarations(declarations) => Ok(declarations_to_microstatements(
            declarations,
            scope,
            program,
            microstatements,
        )?),
        parse::Statement::Assignments(_assignments) => Err("Implement me".into()),
        parse::Statement::Conditional(_condtitional) => Err("Implement me".into()),
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, CType)>,
    pub rettype: CType,
    pub microstatements: Vec<Microstatement>,
    pub kind: FnKind,
}

#[derive(Clone, Debug)]
pub enum FnKind {
    Normal(Vec<parse::Statement>),
    Bind(String),
    Derived,
    DerivedVariadic,
}

impl Function {
    fn from_ast(
        scope: &mut Scope,
        program: &mut Program,
        function_ast: &parse::Functions,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // In the top-level of a file, all functions *must* be named
        let name = match &function_ast.optname {
            Some(name) => name.clone(),
            None => {
                return Err("Top-level function without a name!".into());
            }
        };
        Function::from_ast_with_name(scope, program, function_ast, is_export, name)
    }

    fn from_ast_with_name(
        scope: &mut Scope,
        program: &mut Program,
        function_ast: &parse::Functions,
        is_export: bool,
        name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Add code to properly convert the typeassignable vec into a CType tree and use it.
        // For now, just hardwire the parsing as before.
        let (args, rettype) = match &function_ast.opttype {
            None => Ok::<(Vec<(String, CType)>, CType), Box<dyn std::error::Error>>((
                Vec::new(),
                CType::Void,
            )), // TODO: Does this path *ever* trigger?
            Some(typeassignable) if typeassignable.len() == 0 => Ok((Vec::new(), CType::Void)),
            Some(typeassignable) => {
                let ctype = withtypeoperatorslist_to_ctype(&typeassignable, scope, program)?;
                // If the `ctype` is a Function type, we have both the input and output defined. If
                // it's any other type, we presume it's only the input type defined
                let (input_type, output_type) = match ctype {
                    CType::Function(i, o) => (*i.clone(), *o.clone()),
                    otherwise => (otherwise.clone(), CType::Void), // TODO: Type inference signaling?
                };
                // The input type will be interpreted in many different ways:
                // If it's a Group, unwrap it and continue. Ideally after that it's a Tuple
                // type containing Field types, that's a "conventional" function
                // definition, where the label becomes an argument name and the type is the
                // type. If the tuple doesn't have Fields inside of it, we auto-generate
                // argument names, eg `arg0`, `arg1`, etc. If it is not a Tuple type but is
                // a Field type, we have a single argument function with a specified
                // variable name. If it's any other type, we just label it `arg0`
                let degrouped_input = match input_type {
                    CType::Group(c) => *c.clone(),
                    otherwise => otherwise.clone(),
                };
                let mut out_args = Vec::new();
                match degrouped_input {
                    CType::Tuple(ts) => {
                        for i in 0..ts.len() {
                            out_args.push(match &ts[i] {
                                CType::Field(argname, t) => (argname.clone(), *t.clone()),
                                otherwise => (format!("arg{}", i), otherwise.clone()),
                            });
                        }
                    }
                    CType::Field(argname, t) => out_args.push((argname.clone(), *t.clone())),
                    CType::Void => {} // Do nothing so an empty set is properly
                    otherwise => out_args.push(("arg0".to_string(), otherwise.clone())),
                }
                Ok((out_args, output_type.clone()))
            }
        }?;
        let statements = match &function_ast.fullfunctionbody {
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
            parse::FullFunctionBody::BindFunction(_) => Vec::new(),
        };
        let microstatements = {
            let mut ms = Vec::new();
            for (name, typen) in &args {
                ms.push(Microstatement::Arg {
                    name: name.clone(),
                    typen: typen.clone(),
                });
            }
            for statement in &statements {
                ms = statement_to_microstatements(statement, scope, program, ms)?;
            }
            ms
        };
        let kind = match &function_ast.fullfunctionbody {
            parse::FullFunctionBody::BindFunction(b) => FnKind::Bind(b.rustfunc.clone()),
            _ => FnKind::Normal(statements),
        };
        let function = Function {
            name,
            args,
            rettype,
            microstatements,
            kind,
        };
        if is_export {
            scope
                .exports
                .insert(function.name.clone(), Export::Function);
        }
        if scope.functions.contains_key(&function.name) {
            let func_vec = scope.functions.get_mut(&function.name).unwrap();
            func_vec.push(function);
        } else {
            scope
                .functions
                .insert(function.name.clone(), vec![function]);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Export {
    // TODO: Add other export types over time
    Function,
    Const,
    Type,
    OpMap,
    TypeOpMap,
}

#[derive(Clone, Debug)]
pub enum OperatorMapping {
    Prefix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
    Infix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
    Postfix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
}

impl OperatorMapping {
    fn from_ast(
        scope: &mut Scope,
        operatormapping_ast: &parse::OperatorMapping,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let opmap = match operatormapping_ast.fix {
            parse::Fix::Prefix(_) => OperatorMapping::Prefix {
                level: operatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: operatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: operatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
            parse::Fix::Infix(_) => OperatorMapping::Infix {
                level: operatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: operatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: operatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
            parse::Fix::Postfix(_) => OperatorMapping::Postfix {
                level: operatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: operatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: operatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
        };
        let name = match &opmap {
            OperatorMapping::Prefix { operatorname, .. } => operatorname.clone(),
            OperatorMapping::Infix { operatorname, .. } => operatorname.clone(),
            OperatorMapping::Postfix { operatorname, .. } => operatorname.clone(),
        };
        if is_export {
            scope.exports.insert(name.clone(), Export::OpMap);
        }
        scope.operatormappings.insert(name, opmap);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum TypeOperatorMapping {
    Prefix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
    Infix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
    Postfix {
        level: i8,
        functionname: String,
        operatorname: String,
    },
}

impl TypeOperatorMapping {
    fn from_ast(
        scope: &mut Scope,
        typeoperatormapping_ast: &parse::TypeOperatorMapping,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let opmap = match typeoperatormapping_ast.fix {
            parse::Fix::Prefix(_) => TypeOperatorMapping::Prefix {
                level: typeoperatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: typeoperatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: typeoperatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
            parse::Fix::Infix(_) => TypeOperatorMapping::Infix {
                level: typeoperatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: typeoperatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: typeoperatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
            parse::Fix::Postfix(_) => TypeOperatorMapping::Postfix {
                level: typeoperatormapping_ast
                    .opmap
                    .get_opprecedence()
                    .num
                    .parse::<i8>()?,
                functionname: typeoperatormapping_ast.opmap.get_fntoop().fnname.clone(),
                operatorname: typeoperatormapping_ast.opmap.get_fntoop().operator.clone(),
            },
        };
        let name = match &opmap {
            TypeOperatorMapping::Prefix { operatorname, .. } => operatorname.clone(),
            TypeOperatorMapping::Infix { operatorname, .. } => operatorname.clone(),
            TypeOperatorMapping::Postfix { operatorname, .. } => operatorname.clone(),
        };
        if is_export {
            scope.exports.insert(name.clone(), Export::TypeOpMap);
        }
        scope.typeoperatormappings.insert(name, opmap);
        Ok(())
    }
}
