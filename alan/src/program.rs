use std::fs::read_to_string;
use std::pin::Pin;

use crate::parse;

use ordered_hash_map::OrderedHashMap;

#[derive(Debug)]
pub struct Program {
    entry_file: String,
    scopes_by_file: OrderedHashMap<String, (Pin<Box<String>>, parse::Ln, Scope)>,
}

impl Program {
    pub fn new(entry_file: String) -> Result<Program, Box<dyn std::error::Error>> {
        let mut p = Program {
            entry_file: entry_file.clone(),
            scopes_by_file: OrderedHashMap::new(),
        };
        p = match p.load(entry_file) {
            Ok(p) => p,
            Err(_) => {
                return Err("Failed to load entry file".into());
            }
        };
        Ok(p)
    }

    pub fn load(self, path: String) -> Result<Program, Box<dyn std::error::Error>> {
        let ln_src = read_to_string(&path)?;
        let (mut p, tuple) = Scope::from_src(self, &path, ln_src)?;
        p.scopes_by_file.insert(path, tuple);
        Ok(p)
    }
}

#[derive(Debug)]
struct Scope {
    imports: OrderedHashMap<String, Import>,
    types: OrderedHashMap<String, Type>,
    consts: OrderedHashMap<String, Const>,
    functions: OrderedHashMap<String, Function>,
    handlers: OrderedHashMap<String, Handler>,
    // TODO: Implement these other concepts
    // operatormappings: OrderedHashMap<String, OperatorMapping>,
    // events: OrderedHashMap<String, Event>,
    // interfaces: OrderedHashMap<String, Interface>,
    // exports: Scope,
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
        };
        let mut p = program;
        for i in ast.imports.iter() {
            p = Import::from_ast(p, path.clone(), &mut s, i)?;
        }
        for element in ast.body.iter() {
            match element {
                parse::RootElements::Types(t) => Type::from_ast(&mut s, t)?,
                parse::RootElements::Handlers(h) => Handler::from_ast(&mut s, h)?,
                parse::RootElements::Functions(f) => Function::from_ast(&mut s, f)?,
                parse::RootElements::ConstDeclaration(c) => Const::from_ast(&mut s, c)?,
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
enum ImportType {
    Standard(String),
    Fields(Vec<(String, String)>),
}

#[derive(Debug)]
struct Import {
    source_scope_name: String,
    import_type: ImportType,
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
            parse::ImportStatement::From(f) => {
                // TODO
                Ok(program)
            }
        }
    }
}

#[derive(Debug)]
enum TypeType {
    Structlike(parse::TypeBody),
    Alias(parse::FullTypename),
}

#[derive(Debug)]
struct Type {
    typename: parse::FullTypename,
    typetype: TypeType,
}

impl Type {
    fn from_ast(
        scope: &mut Scope,
        type_ast: &parse::Types,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let t = Type {
            typename: type_ast.fulltypename.clone(),
            typetype: match &type_ast.typedef {
                parse::TypeDef::TypeBody(body) => TypeType::Structlike(body.clone()),
                parse::TypeDef::TypeAlias(alias) => TypeType::Alias(alias.fulltypename.clone()),
            },
        };
        scope.types.insert(t.typename.to_string(), t);
        Ok(())
    }
}

#[derive(Debug)]
struct Const {
    name: String,
    typename: Option<String>,
    assignables: Vec<parse::WithOperators>,
}

impl Const {
    fn from_ast(
        scope: &mut Scope,
        const_ast: &parse::ConstDeclaration,
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
        scope.consts.insert(c.name.clone(), c);
        Ok(())
    }
}

#[derive(Debug)]
struct Function {
    name: String,
    args: Vec<(String, String)>, // Making everything Stringly-typed kinda sucks, but no good way to give an error message in the parser for unknown types otherwise
    rettype: Option<String>,
    statements: Vec<parse::Statement>, // TODO: Do we need to wrap this, or is the AST fine here?
}

impl Function {
    fn from_ast(
        scope: &mut Scope,
        function_ast: &parse::Functions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // In the top-level of a file, all functions *must* be named
        let name = match &function_ast.optname {
            Some(name) => name.clone(),
            None => {
                return Err("Top-level function without a name!".into());
            }
        };
        Function::from_ast_with_name(scope, function_ast, name)
    }

    fn from_ast_with_name(
        scope: &mut Scope,
        function_ast: &parse::Functions,
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
        };
        let function = Function {
            name,
            args,
            rettype,
            statements,
        };
        scope.functions.insert(function.name.clone(), function);
        Ok(())
    }
}

#[derive(Debug)]
struct Handler {
    eventname: String,
    functionname: String,
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
                let _ = Function::from_ast_with_name(scope, function, name.clone());
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
                };
                scope.functions.insert(name.clone(), function);
                name
            }
        };
        let h = Handler {
            eventname: handler_ast.eventname.clone(),
            functionname,
        };
        scope.handlers.insert(h.eventname.clone(), h);
        Ok(())
    }
}
