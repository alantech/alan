use std::fs::read_to_string;

use crate::parse;

use ordered_hash_map::OrderedHashMap;

#[derive(Debug)]
pub struct Program {
    entry_file: String,
    files: OrderedHashMap<String, String>,
    scopes_by_file: OrderedHashMap<String, Scope>,
}

impl Program {
    pub fn new(entry_file: String) -> Result<Program, Box<dyn std::error::Error>> {
        let mut p = Program {
            entry_file,
            files: OrderedHashMap::new(),
            scopes_by_file: OrderedHashMap::new(),
        };
        match p.load(p.entry_file.clone()) {
            Ok(_) => {}
            Err(_) => {
                return Err("Failed to load entry file".into());
            }
        }
        Ok(p)
    }

    pub fn load(&mut self, entry_file: String) -> Result<(), Box<dyn std::error::Error + '_>> {
        let ln_file = read_to_string(&entry_file)?;
        self.files.insert(entry_file.clone(), ln_file);
        let ln_ast = parse::get_ast(&self.files.get(&entry_file).unwrap())?;
        let scope = Scope::from_ast(ln_ast)?;
        self.scopes_by_file.insert(entry_file.clone(), scope);
        Ok(())
    }
}

#[derive(Debug)]
struct Scope {
    types: OrderedHashMap<String, Type>,
    consts: OrderedHashMap<String, Const>,
    functions: OrderedHashMap<String, Function>,
    handlers: OrderedHashMap<String, Handler>,
    // TODO: Implement these other concepts
    // operatormappings: Vec<OperatorMapping>,
    // events: Vec<Event>,
    // interfaces: Vec<Interface>,
    // imported: Vec<Scope>, TODO: Will need a wrapper type to indicate which things are imported,
    // whether the imported scope is given a name wrapping the imports or if fields are imported
    // directly, and if any of the imported fields are renamed
    // exported: Scope,
    // Should we include something for documentation? Maybe testing?
}

impl Scope {
    fn from_ast(ast: parse::Ln) -> Result<Scope, Box<dyn std::error::Error>> {
        // TODO: Implement imports
        let mut s = Scope {
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            handlers: OrderedHashMap::new(),
        };
        for element in ast.body.iter() {
            match element {
                parse::RootElements::Types(t) => Type::from_ast(&mut s, t)?,
                parse::RootElements::Handlers(h) => Handler::from_ast(&mut s, h)?,
                parse::RootElements::Functions(f) => Function::from_ast(&mut s, f)?,
                _ => println!("TODO"),
            }
        }
        Ok(s)
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
    todo: String,
}

#[derive(Debug)]
struct Function {
    name: String,
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
        let function = Function { name };
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
                let function = Function { name: name.clone() }; // TODO: Proper function loading, particularly if not properly named, since that's legal for event functions
                scope.functions.insert(name.clone(), function);
                name
            }
            parse::Handler::FnName(name) => name.clone(),
            parse::Handler::FunctionBody(_) => {
                let name = format!(":::on:::{}", &handler_ast.eventname).to_string();
                let function = Function { name: name.clone() }; // TODO: Proper function loading
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
