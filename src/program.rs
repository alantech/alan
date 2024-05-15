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

    pub fn resolve_type<'a>(
        self: &'a Self,
        scope: &'a Scope,
        typename: &String,
    ) -> Option<(&Type, &Scope)> {
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
                let (_, _, root_scope) = self.scopes_by_file.get("@root").unwrap();
                match &root_scope.types.get(typename) {
                    Some(t) => Some((&t, &root_scope)),
                    None => None,
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
                let (_, _, root_scope) = self.scopes_by_file.get("@root").unwrap();
                match &root_scope.operatormappings.get(operatorname) {
                    Some(o) => Some((&o, &root_scope)),
                    None => None,
                }
            }
        }
    }

    pub fn resolve_function<'a>(
        self: &'a Self,
        scope: &'a Scope,
        function: &String,
        args: &Vec<String>,
    ) -> Option<(&Function, &Scope)> {
        // Tries to find the specified function within the portion of the program accessible from
        // the current scope (so first checking the current scope, then all imports, then the root
        // scope). It checks against the args array to find a match. TODO: Go beyond exact matching
        // Returns a reference to the function and the scope it came from.
        match scope.functions.get(function) {
            Some(fs) => {
                for f in fs {
                    if args.len() != f.args.len() {
                        continue;
                    }
                    let mut args_match = true;
                    for (i, arg) in args.iter().enumerate() {
                        if &f.args[i].1 != arg {
                            args_match = false;
                            break;
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
                let (_, _, root_scope) = self.scopes_by_file.get("@root").unwrap();
                match root_scope.functions.get(function) {
                    Some(fs) => {
                        for f in fs {
                            if args.len() != f.args.len() {
                                continue;
                            }
                            let mut args_match = true;
                            for (i, arg) in args.iter().enumerate() {
                                if &f.args[i].1 != arg {
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
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    pub imports: OrderedHashMap<String, Import>,
    pub types: OrderedHashMap<String, Type>,
    pub consts: OrderedHashMap<String, Const>,
    pub functions: OrderedHashMap<String, Vec<Function>>,
    pub operatormappings: OrderedHashMap<String, OperatorMapping>,
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
            exports: OrderedHashMap::new(),
        };
        let mut p = program;
        for i in ast.imports.iter() {
            p = Import::from_ast(p, path.clone(), &mut s, i)?;
        }
        for element in ast.body.iter() {
            match element {
                parse::RootElements::Types(t) => Type::from_ast(&mut s, t, false)?,
                parse::RootElements::Functions(f) => Function::from_ast(&mut s, &p, f, false)?,
                parse::RootElements::ConstDeclaration(c) => Const::from_ast(&mut s, c, false)?,
                parse::RootElements::OperatorMapping(o) => {
                    OperatorMapping::from_ast(&mut s, o, false)?
                }
                parse::RootElements::Exports(e) => match &e.exportable {
                    parse::Exportable::Functions(f) => Function::from_ast(&mut s, &p, f, true)?,
                    parse::Exportable::ConstDeclaration(c) => Const::from_ast(&mut s, c, true)?,
                    parse::Exportable::OperatorMapping(o) => {
                        OperatorMapping::from_ast(&mut s, o, true)?
                    }
                    parse::Exportable::Types(t) => Type::from_ast(&mut s, t, true)?,
                    e => println!("TODO: Not yet supported export syntax: {:?}", e),
                },
                parse::RootElements::Whitespace(_) => { /* Do nothing */ }
                parse::RootElements::Interfaces(_) => {
                    return Err("Interfaces not yet implemented".into());
                }
                parse::RootElements::CTypes(c) => {
                    // For now this is just declaring in the Alan source code the compile-time
                    // types that can be used, and is simply a special kind of documentation.
                    // *Only* the root scope is allowed to use this syntax, and I cannot imagine
                    // any other way, since the compiler needs to exactly match what is declared.
                    // So we return an error if they're encountered outside of the root scope and
                    // simply verify that each `ctype` we encounter is one of a set the compiler
                    // expects to exist. Later when `cfn` is implemented these will be loaded up
                    // for verification of the meta-typing of the compile-time functions.
                    if path == "@root" {
                        match c.name.as_str() {
                            "Type" | "Int" | "Float" | "Bool" | "String" => { /* Do nothing */ }
                            unknown => {
                                println!("What!? {}", unknown);
                                return Err(format!("Unknown ctype {} defined in root scope. There's something wrong with the compiler.", unknown).into());
                            }
                        }
                    } else {
                        return Err("ctypes can only be defined in the compiler internals".into());
                    }
                }
            }
        }
        Ok((p, (txt, ast, s)))
    }
}

#[derive(Debug)]
pub enum ImportType {
    // For both of these, the first string is the original name, and the second is the rename.
    // To simplify later logic, there's always a rename even if the user didn't rename anything, it
    // will just make a copy of the module or field name in those cases
    Standard((String, String)),
    Fields(Vec<(String, String)>),
}

#[derive(Debug)]
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
                    import_type: ImportType::Standard((ln_file.clone(), import_name)),
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

#[derive(Debug)]
pub enum TypeType {
    Structlike(parse::TypeBody),
    Alias(parse::FullTypename),
    Bind(String),
}

#[derive(Debug)]
pub struct Type {
    pub typename: parse::FullTypename,
    pub typetype: TypeType,
}

impl Type {
    fn from_ast(
        scope: &mut Scope,
        type_ast: &parse::Types,
        is_export: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = type_ast.fulltypename.to_string();
        let t = Type {
            typename: type_ast.fulltypename.clone(),
            typetype: match &type_ast.typedef {
                parse::TypeDef::TypeBody(body) => TypeType::Structlike(body.clone()),
                parse::TypeDef::TypeAlias(alias) => TypeType::Alias(alias.fulltypename.clone()),
                parse::TypeDef::TypeBind(bind) => TypeType::Bind(
                    format!(
                        "{}{}",
                        bind.rustpath.join("::"),
                        match &bind.opttypegenerics {
                            None => "".to_string(),
                            Some(generics) => generics.to_string(),
                        }
                    )
                    .to_string(),
                ),
            },
        };
        if is_export {
            scope.exports.insert(name.clone(), Export::Type);
        }
        scope.types.insert(name, t);
        Ok(())
    }
}

#[derive(Debug)]
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
        typen: String,
    },
    FnCall {
        function: String, // TODO: It would be nice to make this a vector of pointers to function objects so we can narrow down the exact implementation sooner
        args: Vec<Microstatement>,
    },
    Value {
        typen: String,          // TODO: Do better on this, too.
        representation: String, // TODO: Can we do better here?
    },
    Type {
        typen: String, // TODO: Do better on this
        keyvals: OrderedHashMap<String, Microstatement>,
    },
    Array {
        typen: String, // TODO: Do better on this
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
    ) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Self::Value { typen, .. } => Ok(typen.clone()),
            Self::Array { typen, .. } => Ok(typen.clone()),
            Self::Type { typen, .. } => Ok(typen.clone()),
            Self::Arg { typen, .. } => Ok(typen.clone()),
            Self::Assignment { value, .. } => value.get_type(scope, program),
            Self::Return { value } => match value {
                Some(v) => v.get_type(scope, program),
                None => Ok("void".to_string()),
            },
            Self::FnCall { function, args } => {
                let mut arg_types = Vec::new();
                for arg in args {
                    let arg_type = arg.get_type(scope, program)?;
                    arg_types.push(arg_type);
                }
                match program.resolve_function(scope, function, &arg_types) {
                    Some((function_object, _s)) => match &function_object.rettype {
                        // TODO: Handle implied return types better
                        None => Ok("void".to_string()),
                        Some(v) => Ok(v.clone()),
                    },
                    None => Err(format!("Could not find function {}", function).into()),
                }
            }
        }
    }
}

fn baseassignablelist_to_microstatements(
    baseassignablelist: &Vec<parse::BaseAssignable>,
    scope: &Scope,
    program: &Program,
    mut microstatements: Vec<Microstatement>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    let mut i = 0;
    let mut prior_value = None;
    while i < baseassignablelist.len() {
        let baseassignable = &baseassignablelist[i];
        let nextassignable = baseassignablelist.get(i + 1);
        match baseassignable {
            parse::BaseAssignable::MethodSep(_) => {
                // The `MethodSep` symbol doesn't do anything on its own, it only validates that
                // the syntax before and after it is sane
                if prior_value.is_none() {
                    return Err(format!(
                        "Cannot start a statement with a property access: {}",
                        baseassignablelist
                            .iter()
                            .map(|ba| ba.to_string())
                            .collect::<Vec<String>>()
                            .join("")
                    )
                    .into());
                }
                match nextassignable {
                    None => {
                        return Err(format!(
                            "Cannot end a statement with a property access: {}",
                            baseassignablelist
                                .iter()
                                .map(|ba| ba.to_string())
                                .collect::<Vec<String>>()
                                .join("")
                        )
                        .into());
                    }
                    Some(next) => {
                        match next {
                            parse::BaseAssignable::FnCall(_) => {
                                return Err(format!("A function call or parenthetical grouping is not a property: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                            }
                            parse::BaseAssignable::ObjectLiterals(_) => {
                                return Err(format!(
                                    "An object creation is not a property: {}",
                                    baseassignablelist
                                        .iter()
                                        .map(|ba| ba.to_string())
                                        .collect::<Vec<String>>()
                                        .join("")
                                )
                                .into());
                            }
                            parse::BaseAssignable::Functions(_) => {
                                return Err(format!(
                                    "A function definition is not a property: {}",
                                    baseassignablelist
                                        .iter()
                                        .map(|ba| ba.to_string())
                                        .collect::<Vec<String>>()
                                        .join("")
                                )
                                .into());
                            }
                            parse::BaseAssignable::MethodSep(_) => {
                                return Err(format!(
                                    "Too many `.` symbols for the method access: {}",
                                    baseassignablelist
                                        .iter()
                                        .map(|ba| ba.to_string())
                                        .collect::<Vec<String>>()
                                        .join("")
                                )
                                .into());
                            }
                            _ => {}
                        }
                    }
                }
            }
            parse::BaseAssignable::Constants(c) => {
                // With constants, it's relatively simple. *Only* if the constant in question is an
                // integer is it legitimate to have a constant follow a dot notation, for
                // tuple/fixed-array accesses. It becomes syntactic sugar for `get(prior_value, 0)`
                // (or whatever the number is). In all other cases, a prior value is an error.
                // Similarly, the only thing that can follow a constant value is a `MethodSep` to
                // be used for a method-syntax function call and all others are errors. The
                // "default" path is for a baseassignablelist with a length of one containing only
                // the constant.
                if let Some(next) = nextassignable {
                    match next {
                        parse::BaseAssignable::Variable(_) => {
                            return Err(format!("A constant cannot be directly before a variable without an operator between them: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::BaseAssignable::FnCall(_) => {
                            return Err(format!("A constant cannot be directly before a parenthetical grouping without an operator between them: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::BaseAssignable::ObjectLiterals(_) => {
                            return Err(format!(
                                "A constant cannot be directly before an object definition: {}",
                                baseassignablelist
                                    .iter()
                                    .map(|ba| ba.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                            )
                            .into());
                        }
                        parse::BaseAssignable::Functions(_) => {
                            return Err(format!(
                                "A constant cannot be directly before a function definition: {}",
                                baseassignablelist
                                    .iter()
                                    .map(|ba| ba.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                            )
                            .into());
                        }
                        parse::BaseAssignable::Constants(_) => {
                            return Err(format!("A constant cannot be directly before another constant without an operator between them: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::BaseAssignable::MethodSep(_) => {} // The only allowed follow-up
                    }
                }
                if prior_value.is_none() {
                    match c {
                        parse::Constants::Bool(b) => {
                            prior_value = Some(Microstatement::Value {
                                typen: "bool".to_string(),
                                representation: b.clone(),
                            })
                        }
                        parse::Constants::Strn(s) => {
                            prior_value = Some(Microstatement::Value {
                                typen: "string".to_string(),
                                representation: if s.starts_with('"') {
                                    s.clone()
                                } else {
                                    // TODO: Is there a cheaper way to do this conversion?
                                    s.replace("\"", "\\\"")
                                        .replace("\\'", "\\\\\"")
                                        .replace("'", "\"")
                                        .replace("\\\\\"", "'")
                                },
                            })
                        }
                        parse::Constants::Num(n) => match n {
                            parse::Number::RealNum(r) => {
                                prior_value = Some(Microstatement::Value {
                                    typen: "f64".to_string(), // TODO: Something more intelligent here?
                                    representation: r.clone(),
                                })
                            }
                            parse::Number::IntNum(i) => {
                                prior_value = Some(Microstatement::Value {
                                    typen: "i64".to_string(), // TODO: Same here. This feels dumb
                                    representation: i.clone(),
                                })
                            }
                        },
                    }
                } else {
                    // There's only one case where a prior value is allowed, when the constant
                    // is an integer
                    match c {
                        parse::Constants::Bool(_) => {
                            return Err(format!("A boolean cannot follow another value without an operator between them: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::Constants::Strn(_) => {
                            return Err(format!("A string cannot follow another value without an operator between them: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::Constants::Num(n) => match n {
                            parse::Number::RealNum(_) => {
                                return Err(format!("A floating point number cannot follow another value without an operator between them: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                            }
                            parse::Number::IntNum(i) => {
                                // Convert the prior value and the new integer into `get(prior, i)`
                                let prior = prior_value.unwrap();
                                prior_value = Some(Microstatement::FnCall {
                                    function: "get".to_string(),
                                    args: vec![
                                        prior,
                                        Microstatement::Value {
                                            typen: "i64".to_string(),
                                            representation: i.clone(),
                                        },
                                    ],
                                });
                            }
                        },
                    }
                }
            }
            parse::BaseAssignable::Functions(_f) => {
                // Function declaration is very simple. If there's any prior value, it's an error.
                // It can only be followed by nothing or by a function call for an IIFE, which is
                // generally not useful, but could be used similar to Rust's block syntax to create
                // a value, mutate it, but then finally assign it as an immutable constant outside
                // of that scope.
                if let Some(_) = prior_value {
                    return Err(format!(
                        "A function definition cannot follow another value: {}",
                        baseassignablelist
                            .iter()
                            .map(|ba| ba.to_string())
                            .collect::<Vec<String>>()
                            .join("")
                    )
                    .into());
                }
                if let Some(next) = nextassignable {
                    match next {
                        parse::BaseAssignable::Variable(_) => {
                            return Err(format!(
                                "A variable cannot follow a closure function definition: {}",
                                baseassignablelist
                                    .iter()
                                    .map(|ba| ba.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                            )
                            .into());
                        }
                        parse::BaseAssignable::ObjectLiterals(_) => {
                            return Err(format!("An object definition cannot follow a closure function definition: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::BaseAssignable::Functions(_) => {
                            return Err(format!("A closure function definition cannot immediately follow another closure function definition: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::BaseAssignable::Constants(_) => {
                            return Err(format!("A constant cannot immediately follow a closure function definition: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::BaseAssignable::MethodSep(_) => {
                            // TODO: Potentially reconsider for method syntax for higher-order
                            // functions?
                            return Err(format!("A closure function definition does not have any properties to access: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::BaseAssignable::FnCall(_) => {} // Allowed for IIFEs
                    }
                }
                // Now we can create the closure function, but first we need to update the
                // Microstatement type to define closure functions as a thing
                return Err("TODO: Implement closure functions in Microstatement syntax".into());
            }
            parse::BaseAssignable::ObjectLiterals(ol) => {
                // When an object literal has nothing before it, it constructs the object in
                // question. When there is a value before it, it can only be an array literal
                // syntax and is instead operating as the array accessor syntax, which is syntactic
                // sugar for `get(prior_value, i, ...)` with the contents of the array syntax
                // becoming the args passed to the `get` function.
                if let Some(prior) = prior_value {
                    match ol {
                        parse::ObjectLiterals::ArrayLiteral(a) => {
                            match a {
                                parse::ArrayLiteral::ArrayBase(b) => {
                                    let mut array_accessor_microstatements = vec![prior];
                                    for withoperators in &b.assignablelist {
                                        microstatements = withoperatorslist_to_microstatements(
                                            withoperators,
                                            scope,
                                            program,
                                            microstatements,
                                        )?;
                                        array_accessor_microstatements
                                            .push(microstatements.pop().unwrap());
                                    }
                                    prior_value = Some(Microstatement::FnCall {
                                        function: "get".to_string(),
                                        args: array_accessor_microstatements,
                                    });
                                }
                                parse::ArrayLiteral::FullArrayLiteral(_) => {
                                    return Err(format!("An array definition cannot directly follow another value: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                                }
                            }
                        }
                        parse::ObjectLiterals::TypeLiteral(_) => {
                            return Err(format!(
                                "An object definition cannot directly follow another value: {}",
                                baseassignablelist
                                    .iter()
                                    .map(|ba| ba.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                            )
                            .into());
                        }
                    }
                } else {
                    match ol {
                        parse::ObjectLiterals::ArrayLiteral(a) => match a {
                            parse::ArrayLiteral::ArrayBase(b) => {
                                let mut array_microstatements = Vec::new();
                                for withoperators in &b.assignablelist {
                                    microstatements = withoperatorslist_to_microstatements(
                                        withoperators,
                                        scope,
                                        program,
                                        microstatements,
                                    )?;
                                    array_microstatements.push(microstatements.pop().unwrap());
                                }
                                // TODO: Currently assuming the type of the first element of the array
                                // matches all of them. Should confirm that in the future
                                prior_value = Some(Microstatement::Array {
                                    typen: format!(
                                        "Array<{}>",
                                        array_microstatements[0].get_type(scope, program)?
                                    )
                                    .to_string(),
                                    vals: array_microstatements,
                                });
                            }
                            parse::ArrayLiteral::FullArrayLiteral(f) => {
                                let mut array_microstatements = Vec::new();
                                for withoperators in &f.arraybase.assignablelist {
                                    microstatements = withoperatorslist_to_microstatements(
                                        withoperators,
                                        scope,
                                        program,
                                        microstatements,
                                    )?;
                                    array_microstatements.push(microstatements.pop().unwrap());
                                }
                                // TODO: Currently assuming the type of the array *is* the type specified.
                                // Should confirm that in the future
                                prior_value = Some(Microstatement::Array {
                                    typen: f.literaldec.fulltypename.to_string(),
                                    vals: array_microstatements,
                                });
                            }
                        },
                        parse::ObjectLiterals::TypeLiteral(t) => {
                            let mut struct_microstatements = OrderedHashMap::new();
                            for typeassign in &t.typebase.typeassignlist {
                                microstatements = withoperatorslist_to_microstatements(
                                    &typeassign.assignables,
                                    scope,
                                    program,
                                    microstatements,
                                )?;
                                struct_microstatements.insert(
                                    typeassign.variable.clone(),
                                    microstatements.pop().unwrap(),
                                );
                            }
                            // TODO: Currently assuming that the specified type matches the properties
                            // provided. Should confirm that in the future
                            prior_value = Some(Microstatement::Type {
                                typen: t.literaldec.fulltypename.to_string(),
                                keyvals: struct_microstatements,
                            });
                        }
                    }
                }
            }
            parse::BaseAssignable::Variable(var) => {
                // Variables can be used as property or method accesses on the prior value. For
                // method access, if the function takes only one argument, it should still work
                // even if the follow-on parenthesis are not written, so there's a little bit of
                // extra logic added here for that situation, otherwise it's handled by the FnCall
                // path. When it's a property access, it replaces the prior Microstatement with a
                // `Value` statement operating on the prior value. Sometimes this requires pushing
                // it into the `microstatements` array, sometimes modifying it, sometimes needing
                // to put it into an `Assignment` before pushing it to the `microstatements` array
                // to reference it.
                // For the simpler case when it's *just* a reference to a prior variable, it just
                // becomes a `Value` microstatement.
                let typen = match microstatements.iter().find(|m| match m {
                    Microstatement::Assignment { name, .. } => var == name,
                    Microstatement::Arg { name, .. } => var == name,
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
                        match scope.functions.get(var) {
                            Some(_) => Ok("function".to_string()), // TODO: Do better
                            None => {
                                // Check the root scope, too
                                match program.scopes_by_file.get("@root") {
                                    Some((_, _, s)) => match s.functions.get(var) {
                                        Some(_) => Ok("function".to_string()), // TODO: Do better
                                        None => {
                                            Err(format!("Couldn't find variable {}", var).into())
                                        }
                                    },
                                    None => Err(format!("Couldn't find variable {}", var).into()),
                                }
                            }
                        }
                    }
                }?;
                if let Some(prior) = prior_value {
                    if typen == "function" {
                        // TODO: Do better on function dispatch
                        // If this is a method call we need to pull in the rest of the arguments
                        // (if there are any) from the next assignable and then skip over it.
                        // Otherwise either generate the function call as-is or return an error if
                        // the follow-on assignable can't follow this one.
                        if let Some(next) = nextassignable {
                            match next {
                                parse::BaseAssignable::Variable(v) => {
                                    return Err(format!("A variable ({}) cannot directly follow another value without an operator: {}", v, baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                                }
                                parse::BaseAssignable::ObjectLiterals(_) => {
                                    // TODO: Support array access after a method call
                                    return Err(
                                        "TODO: Support array access after a method call".into()
                                    );
                                }
                                parse::BaseAssignable::Functions(_) => {
                                    return Err(format!("A closure function cannot directly follow another value: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                                }
                                parse::BaseAssignable::Constants(c) => {
                                    return Err(format!("A constant ({}) cannot directly follow another value without an operator: {}", c.to_string(), baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                                }
                                parse::BaseAssignable::MethodSep(_) => {
                                    // Call the function with a singular value
                                    match &prior {
                                        Microstatement::Value { .. } => {
                                            prior_value = Some(Microstatement::FnCall {
                                                function: var.clone(),
                                                args: vec![prior.clone()],
                                            });
                                        }
                                        Microstatement::Arg { .. } => {
                                            prior_value = Some(Microstatement::FnCall {
                                                function: var.clone(),
                                                args: vec![prior.clone()],
                                            });
                                        }
                                        Microstatement::Assignment { name, .. } => {
                                            return Err(format!("{} is not a function so it cannot be called as one: {}", name, baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                                        }
                                        Microstatement::FnCall { .. } => {
                                            // TODO: Support function calls on functions that return functions
                                            return Err("Function calls on functions that return functions not yet implemented".into());
                                        }
                                        Microstatement::Type { typen, .. } => {
                                            return Err(format!("{} is not a function so it cannot be called as one: {}", typen, baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                                        }
                                        Microstatement::Array { typen, .. } => {
                                            return Err(format!("{} is not a function so it cannot be called as one: {}", typen, baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                                        }
                                        Microstatement::Return { .. } => {
                                            return Err(format!("You can't call a return statement as a function. How did you even do this, anyway?: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).into());
                                        }
                                    }
                                }
                                parse::BaseAssignable::FnCall(fc) => {
                                    // In this path, we grab the arguments from the function call
                                    // and combine it with the value we're keying off of, then we
                                    // skip the next step by incrementing `i` an extra time
                                    // TODO: Support closure functions
                                    let mut args = vec![prior.clone()];
                                    for arg in &fc.assignablelist {
                                        microstatements = withoperatorslist_to_microstatements(
                                            arg,
                                            scope,
                                            program,
                                            microstatements,
                                        )?;
                                        let lastmicrostatement = microstatements.pop().unwrap();
                                        match lastmicrostatement {
                                            Microstatement::Assignment {
                                                ref name,
                                                ref value,
                                                ..
                                            } => {
                                                // If the last microstatement is an assignment, we need to
                                                // reference it as a value and push it back onto the array
                                                args.push(Microstatement::Value {
                                                    typen: value.get_type(scope, program)?,
                                                    representation: name.clone(),
                                                });
                                                microstatements.push(lastmicrostatement);
                                            }
                                            _ => {
                                                // For everything else, we can just put the statement inside of
                                                // the function call as one of its args directly
                                                args.push(lastmicrostatement);
                                            }
                                        }
                                    }
                                    prior_value = Some(Microstatement::FnCall {
                                        function: var.clone(),
                                        args,
                                    });
                                    i = i + 1;
                                }
                            }
                        } else {
                            // This path is a simple function call with a single argument
                            prior_value = Some(Microstatement::FnCall {
                                function: var.clone(),
                                args: vec![prior.clone()],
                            });
                        }
                    } else {
                        // TODO: Properly support property access eventually
                        return Err(format!(
                            "TODO: Implement property access: {}",
                            baseassignablelist
                                .iter()
                                .map(|ba| ba.to_string())
                                .collect::<Vec<String>>()
                                .join("")
                        )
                        .into());
                    }
                } else {
                    prior_value = Some(Microstatement::Value {
                        typen,
                        representation: var.clone(),
                    });
                }
            }
            parse::BaseAssignable::FnCall(fc) => {
                // FnCall syntax works in a few ways: if it's the first base assignable or
                // completely by itself, it's actually a parenthetical grouping of the values
                // within it and is evaluated that way. If its following something, it is a
                // function call, but it can only follow a variable that represents a function, an
                // actual closure function definition, or an array accessor that returned a closure
                // function (so for both of those paths, it needs to be a closure function
                // microstatement, which doesn't exist yet).
                // However, the method syntax has already been handled by the `Variable` path, so
                // we don't deal with it.
                if let Some(prior) = prior_value {
                    let fn_name = match prior {
                        Microstatement::Assignment { .. } => Err(format!("You can't call an assignment statement as a function: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join(""))),
                        Microstatement::Arg { name, typen } => {
                            if typen == "function" { // TODO: Better typing for functions
                                Ok(name)
                            } else {
                                Err(format!("{} is not a function: {}", name, baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")))
                            }
                        },
                        Microstatement::FnCall { .. } => Err(format!("TODO: Support calling a function immediately from a function: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join(""))),
                        Microstatement::Value { representation, typen } => {
                            if typen == "function" { // TODO: Better typing for functions
                                Ok(representation)
                            } else {
                                Err(format!("{} is not a function: {}", representation, baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")))
                            }
                        },
                        Microstatement::Type { .. } => Err(format!("You can't call an object as a function: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join(""))),
                        Microstatement::Array { .. } => Err(format!("You can't call an array as a function: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join(""))),
                        Microstatement::Return { .. } => Err(format!("You can't call a return statement as a function. How did you even do that?: {}", baseassignablelist.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join(""))),
                    }?;
                    let mut args = Vec::new();
                    for arg in &fc.assignablelist {
                        microstatements = withoperatorslist_to_microstatements(
                            arg,
                            scope,
                            program,
                            microstatements,
                        )?;
                        let lastmicrostatement = microstatements.pop().unwrap();
                        match lastmicrostatement {
                            Microstatement::Assignment {
                                ref name,
                                ref value,
                                ..
                            } => {
                                // If the last microstatement is an assignment, we need to
                                // reference it as a value and push it back onto the array
                                args.push(Microstatement::Value {
                                    typen: value.get_type(scope, program)?,
                                    representation: name.clone(),
                                });
                                microstatements.push(lastmicrostatement);
                            }
                            _ => {
                                // For everything else, we can just put the statement inside of
                                // the function call as one of its args directly
                                args.push(lastmicrostatement);
                            }
                        }
                    }
                    prior_value = Some(Microstatement::FnCall {
                        function: fn_name.clone(),
                        args,
                    });
                } else {
                    // TODO: Allow tuple syntax eventually. For now just keeping the old check
                    // around
                    if fc.assignablelist.len() != 1 {
                        return Err(format!(
                            "Multiple statements found in {}. Perhaps you should remove that comma?",
                            baseassignable.to_string()
                        )
                        .into());
                    }
                    // Happy path, let's get the microstatements from this assignable list
                    microstatements = withoperatorslist_to_microstatements(
                        &fc.assignablelist[0],
                        scope,
                        program,
                        microstatements,
                    )?;
                    prior_value = microstatements.pop();
                }
            }
        }
        i = i + 1;
    }
    // Push the generated statement that *probably* exists into the microstatements array
    if let Some(prior) = prior_value {
        microstatements.push(prior);
    }
    Ok(microstatements)
}

fn withoperatorslist_to_microstatements(
    withoperatorslist: &Vec<parse::WithOperators>,
    scope: &Scope,
    program: &Program,
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
    scope: &Scope,
    program: &Program,
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
    scope: &Scope,
    program: &Program,
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
    scope: &Scope,
    program: &Program,
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
    scope: &Scope,
    program: &Program,
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

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, String)>, // Making everything Stringly-typed kinda sucks, but no good way to give an error message in the parser for unknown types otherwise
    pub rettype: Option<String>,
    pub statements: Vec<parse::Statement>,
    pub microstatements: Vec<Microstatement>,
    pub bind: Option<String>,
}

impl Function {
    fn from_ast(
        scope: &mut Scope,
        program: &Program,
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
        program: &Program,
        function_ast: &parse::Functions,
        is_export: bool,
        name: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let args = match &function_ast.optargs {
            None => Vec::new(),
            Some(arglist) => {
                // TODO: Make the arg types optional
                arglist
                    .arglist
                    .iter()
                    .map(|arg| (arg.variable.clone(), arg.fulltypename.to_string()))
                    .collect()
            }
        };
        let rettype = match &function_ast.optreturntype {
            None => None,
            Some(returntype) => Some(returntype.fulltypename.to_string()),
        };
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
        let bind = match &function_ast.fullfunctionbody {
            parse::FullFunctionBody::BindFunction(b) => Some(b.rustfunc.clone()),
            _ => None,
        };
        let function = Function {
            name,
            args,
            rettype,
            statements,
            microstatements,
            bind,
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

#[derive(Debug)]
pub enum Export {
    // TODO: Add other export types over time
    Function,
    Const,
    Type,
    OpMap,
}

#[derive(Debug)]
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
