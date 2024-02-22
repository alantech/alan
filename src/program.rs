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
            Err(e) => {
                println!("{:?}", e);
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
    pub handlers: OrderedHashMap<String, Handler>,
    pub exports: OrderedHashMap<String, Export>,
    // TODO: Implement these other concepts
    // operatormappings: OrderedHashMap<String, OperatorMapping>,
    // events: OrderedHashMap<String, Event>,
    // interfaces: OrderedHashMap<String, Interface>,
    // Should we include something for documentation? Maybe testing?
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
            handlers: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        };
        let mut p = program;
        for i in ast.imports.iter() {
            p = Import::from_ast(p, path.clone(), &mut s, i)?;
        }
        for element in ast.body.iter() {
            match element {
                parse::RootElements::Handlers(h) => Handler::from_ast(&mut s, h)?,
                parse::RootElements::Types(t) => Type::from_ast(&mut s, t, false)?,
                parse::RootElements::Functions(f) => Function::from_ast(&mut s, f, false)?,
                parse::RootElements::ConstDeclaration(c) => Const::from_ast(&mut s, c, false)?,
                parse::RootElements::Exports(e) => match &e.exportable {
                    parse::Exportable::Functions(f) => Function::from_ast(&mut s, f, true)?,
                    parse::Exportable::ConstDeclaration(c) => Const::from_ast(&mut s, c, true)?,
                    parse::Exportable::Types(t) => Type::from_ast(&mut s, t, true)?,
                    _ => println!("TODO"),
                },
                parse::RootElements::Whitespace(_) => { /* Do nothing */ }
                _ => println!("TODO"),
            }
        }
        Ok((p, (txt, ast, s)))
    }
}

#[derive(Debug)]
pub enum ImportType {
    Standard(String),
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
                let i = Import {
                    source_scope_name: ln_file.clone(),
                    import_type: ImportType::Standard(ln_file.clone()),
                };
                scope.imports.insert(ln_file, i);
                Ok(p)
            }
            parse::ImportStatement::From(_f) => {
                // TODO
                Ok(program)
            }
        }
    }
}

#[derive(Debug)]
pub enum TypeType {
    Structlike(parse::TypeBody),
    Alias(parse::FullTypename),
    Bind(parse::FullTypename),
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
                parse::TypeDef::TypeBind(bind) => TypeType::Bind(bind.rusttypename.clone()),
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
#[derive(Debug)]
pub enum Microstatement {
    Assignment {
        name: String,
        value: Box<Microstatement>,
    },
    FnCall {
        function: String, // TODO: It would be nice to make this a vector of pointers to function objects so we can narrow down the exact implementation sooner
        args: Vec<Microstatement>,
    },
    Value {
        typen: String,          // TODO: Do better on this, too.
        representation: String, // TODO: Can we do better here?
    }, // TODO: Conditionals, Emits, and Returns
}

fn baseassignablelist_to_microstatements(
    baseassignablelist: &Vec<parse::BaseAssignable>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    let mut microstatements = Vec::new();
    for (i, baseassignable) in baseassignablelist.iter().enumerate() {
        match baseassignable {
            parse::BaseAssignable::Variable(var) => {
                // The behavior of a variable depends on if there's
                // anything following after it. Many things following are
                // invalid syntax, but `FnCall` and `MethodSep` are valid
                let next = baseassignablelist.get(i + 1);
                if let Some(otherbase) = next {
                    match otherbase {
                        parse::BaseAssignable::FnCall(call) => {
                            // First generate the microstatements to compute the values to pass to the
                            // function that will be called, and populate an array of arg
                            // microstatements for the eventual function call
                            let mut args = Vec::new();
                            for arg in &call.assignablelist {
                                let mut argmicrostatements =
                                    withoperatorslist_to_microstatement(arg)?;
                                let lastmicrostatement = argmicrostatements.pop().unwrap();
                                match lastmicrostatement {
                                    Microstatement::Assignment { ref name, .. } => {
                                        // If the last microstatement is an assignment, we need to
                                        // reference it as a value and push it back onto the array
                                        args.push(Microstatement::Value {
                                            typen: "".to_string(), // TODO: Populate this
                                            representation: name.clone(),
                                        });
                                        argmicrostatements.push(lastmicrostatement);
                                    }
                                    _ => {
                                        // For everything else, we can just put the statement inside of
                                        // the function call as one of its args directly
                                        args.push(lastmicrostatement);
                                    }
                                }
                                // Append the arg microstatements to the microstatements array now, so
                                // they are evaluated before the function is called
                                microstatements.append(&mut argmicrostatements);
                            }
                            microstatements.push(Microstatement::FnCall {
                                function: var.to_string(),
                                args,
                            });
                        }
                        _ => {
                            return Err(format!("Invalid syntax after {}", var).into());
                        }
                    }
                } else {
                    microstatements.push(Microstatement::Value {
                        typen: "".to_string(), // TODO: Figure out how to look up the type data
                        representation: var.to_string(),
                    });
                }
            }
            parse::BaseAssignable::FnCall(_) => {
                // This path doesn't do anything, as the `variable` path should have handled it.
            }
            parse::BaseAssignable::Constants(c) => {
                microstatements.push(Microstatement::Value {
                    typen: "".to_string(), // TODO: Figure out how to determine the kind of number we're dealing with
                    representation: match c {
                        parse::Constants::Bool(b) => b.clone(),
                        parse::Constants::Strn(s) => s.clone(),
                        parse::Constants::Num(n) => match n {
                            parse::Number::RealNum(r) => r.clone(),
                            parse::Number::IntNum(i) => i.clone(),
                        },
                    },
                });
            }
            _ => {
                return Err("Unsupported assignable type".into());
            }
        }
    }
    Ok(microstatements)
}

fn withoperatorslist_to_microstatement(
    withoperatorslist: &Vec<parse::WithOperators>,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    let mut microstatements = Vec::new();
    for assignable_or_operator in withoperatorslist.iter() {
        match assignable_or_operator {
            parse::WithOperators::BaseAssignableList(baseassignablelist) => microstatements.append(
                &mut baseassignablelist_to_microstatements(baseassignablelist)?,
            ),
            _ => {
                return Err("Operators currently unsupported".into());
            }
        }
    }
    Ok(microstatements)
}

fn assignablestatement_to_microstatements(
    assignable: &parse::AssignableStatement,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    let mut microstatements = Vec::new();
    microstatements.append(&mut withoperatorslist_to_microstatement(
        &assignable.assignables,
    )?);
    Ok(microstatements)
}

fn statement_to_microstatements(
    statement: &parse::Statement,
) -> Result<Vec<Microstatement>, Box<dyn std::error::Error>> {
    let mut microstatements = Vec::new();
    match statement {
        parse::Statement::A(_) => {}
        parse::Statement::Assignables(assignable) => {
            microstatements.append(&mut assignablestatement_to_microstatements(assignable)?)
        }
        _ => {}
    }
    Ok(microstatements)
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
        Function::from_ast_with_name(scope, function_ast, is_export, name)
    }

    fn from_ast_with_name(
        scope: &mut Scope,
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
                vec![parse::Statement::Assignables(parse::AssignableStatement {
                    assignables: assign.assignables.clone(),
                    semicolon: ";".to_string(),
                })]
            }
            parse::FullFunctionBody::BindFunction(_) => Vec::new(),
        };
        let microstatements = {
            let mut ms = Vec::new();
            for statement in &statements {
                ms.append(&mut statement_to_microstatements(statement)?);
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
}

#[derive(Debug)]
pub struct Handler {
    pub eventname: String,
    pub functionname: String,
}

impl Handler {
    fn from_ast(
        scope: &mut Scope,
        handler_ast: &parse::Handlers,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let functionname = match &handler_ast.handler {
            parse::Handler::Functions(function) => {
                // Inline defined function possibly with a name, grab the name and shove it into the
                // function list for this scope, otherwise
                let name = match &function.optname {
                    Some(name) => name.clone(),
                    None => format!(":::on:::{}", &handler_ast.eventname).to_string(), // Impossible for users to write, so no collisions ever
                };
                let _ = Function::from_ast_with_name(scope, function, false, name.clone());
                name
            }
            parse::Handler::FnName(name) => name.clone(),
            // This is the *only* place where a function body can just be passed in without the
            // "proper" `fn` declaration prior (at least right now), so just keeping the weird
            // Function object initialization in here instead of as a new method on the Function
            // type.
            parse::Handler::FunctionBody(body) => {
                let name = format!(":::on:::{}", &handler_ast.eventname).to_string();
                let function = Function {
                    name: name.clone(),
                    args: Vec::new(),
                    rettype: None,
                    statements: body.statements.clone(),
                    microstatements: {
                        let mut ms = Vec::new();
                        for statement in &body.statements {
                            ms.append(&mut statement_to_microstatements(statement)?);
                        }
                        ms
                    },
                    bind: None,
                };
                if scope.functions.contains_key(&name) {
                    let func_vec = scope.functions.get_mut(&name).unwrap();
                    func_vec.push(function);
                } else {
                    scope.functions.insert(name.clone(), vec![function]);
                }
                name
            } // TODO: Should you be allowed to bind a Rust function as a handler directly?
        };
        let h = Handler {
            eventname: handler_ast.eventname.clone(),
            functionname,
        };
        scope.handlers.insert(h.eventname.clone(), h);
        Ok(())
    }
}
