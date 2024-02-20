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
                },
            }
        } else {
            read_to_string(&path)?
        };
        let (mut p, tuple) = Scope::from_src(self, &path, ln_src)?;
        p.scopes_by_file.insert(path, tuple);
        Ok(p)
    }
}

#[derive(Debug)]
pub struct Scope {
    pub imports: OrderedHashMap<String, Import>,
    pub types: OrderedHashMap<String, Type>,
    pub consts: OrderedHashMap<String, Const>,
    pub functions: OrderedHashMap<String, Function>,
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
        //program.scopes_by_file.insert(path, (txt, ast, s));
        Ok((p, (txt, ast, s)))
        //Ok(())
    }
}

// import ./foo
// import ./foo as bar
// from ./foo import bar
// from ./foo import bar as baz

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

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, String)>, // Making everything Stringly-typed kinda sucks, but no good way to give an error message in the parser for unknown types otherwise
    pub rettype: Option<String>,
    pub statements: Vec<parse::Statement>, // TODO: Do we need to wrap this, or is the AST fine here?
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
        let bind = match &function_ast.fullfunctionbody {
            parse::FullFunctionBody::BindFunction(b) => Some(b.rustfunc.clone()),
            _ => None,
        };
        let function = Function {
            name,
            args,
            rettype,
            statements,
            bind,
        };
        if is_export {
            scope
                .exports
                .insert(function.name.clone(), Export::Function);
        }
        scope.functions.insert(function.name.clone(), function);
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
                    bind: None,
                };
                scope.functions.insert(name.clone(), function);
                name
            }
            // TODO: Should you be allowed to bind a Rust function as a handler directly?
        };
        let h = Handler {
            eventname: handler_ast.eventname.clone(),
            functionname,
        };
        scope.handlers.insert(h.eventname.clone(), h);
        Ok(())
    }
}
