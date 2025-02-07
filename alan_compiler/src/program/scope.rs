use std::sync::{Arc, OnceLock};

use ordered_hash_map::OrderedHashMap;

use super::ArgKind;
use super::CType;
use super::Const;
use super::Export;
use super::FnKind;
use super::Function;
use super::OperatorMapping;
use super::Program;
use super::TypeOperatorMapping;
use crate::parse;

#[derive(Clone, Debug)]
pub struct Scope<'a> {
    pub path: String,
    pub parent: Option<&'a Scope<'a>>,
    pub types: OrderedHashMap<String, Arc<CType>>,
    pub consts: OrderedHashMap<String, Const>,
    pub functions: OrderedHashMap<String, Vec<Arc<Function>>>,
    pub operatormappings: OrderedHashMap<String, OperatorMapping>,
    pub typeoperatormappings: OrderedHashMap<String, TypeOperatorMapping>,
    pub exports: OrderedHashMap<String, Export>,
    // TODO: Implement these other concepts
    // interfaces: OrderedHashMap<String, Interface>,
    // Should we include something for documentation?
}

impl<'a> Scope<'a> {
    pub fn load_scope(
        mut s: Scope<'a>,
        ast: &parse::Ln,
        is_root: bool,
    ) -> Result<Scope<'a>, Box<dyn std::error::Error>> {
        for (i, element) in ast.body.iter().enumerate() {
            match element {
                parse::RootElements::Types(t) => {
                    let res = CType::from_ast(s, t, false)?;
                    s = res.0;
                }

                parse::RootElements::Functions(f) => s = Function::from_ast(s, f, false)?,
                parse::RootElements::ConstDeclaration(c) => s = Const::from_ast(s, c, false)?,
                parse::RootElements::OperatorMapping(o) => {
                    s = OperatorMapping::from_ast(s, o, false)?
                }
                parse::RootElements::TypeOperatorMapping(o) => {
                    s = TypeOperatorMapping::from_ast(s, o, false)?
                }
                parse::RootElements::Exports(e) => match &e.exportable {
                    parse::Exportable::Functions(f) => s = Function::from_ast(s, f, true)?,
                    parse::Exportable::ConstDeclaration(c) => s = Const::from_ast(s, c, true)?,
                    parse::Exportable::OperatorMapping(o) => {
                        s = OperatorMapping::from_ast(s, o, true)?
                    }
                    parse::Exportable::TypeOperatorMapping(o) => {
                        s = TypeOperatorMapping::from_ast(s, o, true)?
                    }
                    parse::Exportable::Types(t) => {
                        let res = CType::from_ast(s, t, true)?;
                        s = res.0;
                    }
                    e => eprintln!("TODO: Not yet supported export syntax: {:?}\nLast good parsed lines:\n{:?}\n{:?}", e, ast.body[i - 2], ast.body[i - 1]),
                },
                parse::RootElements::Whitespace(_) => { /* Do nothing */ }
                parse::RootElements::CTypes(c) => {
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
                    if !is_root {
                        return Err("ctypes can only be defined in the compiler internals".into());
                    }
                    match c.name.as_str() {
                        "Type" | "Generic" => {
                            /* Do nothing for the 'structural' types */
                        }
                        g @ ("Int" | "Float" | "Bool" | "String" | "Group" | "Unwrap" | "Infix"
                        | "Prefix" | "Method" | "Property" | "Cast" | "Own" | "Deref" | "Mut"
                        | "Rust" | "Nodejs" | "From" | "Array" | "Fail" | "Neg" | "Len" | "Size"
                        | "FileStr" | "Env" | "EnvExists" | "Not") => s = CType::from_generic(s, g, 1),
                        g @ ("Function" | "Call" | "Dependency" | "Import" | "Field"
                        | "Prop" | "Buffer" | "Add" | "Sub" | "Mul" | "Div" | "Mod"
                        | "Pow" | "Min" | "Max" | "Concat" | "And" | "Or" | "Xor" | "Nand"
                        | "Nor" | "Xnor" | "Eq" | "Neq" | "Lt" | "Lte" | "Gt" | "Gte") => s = CType::from_generic(s, g, 2),
                        g @ ("If" | "Binds" | "Tuple" | "Either" | "AnyOf") => {
                            // Not kosher in Rust land, but 0 means "as many as we want"
                            s = CType::from_generic(s, g, 0)
                        }
                        unknown => {
                            panic!("Unknown ctype {} defined in root scope. There's something wrong with the compiler.", unknown);
                        }
                    }
                }
                parse::RootElements::Interfaces(_) => {
                    panic!("Interfaces not yet implemented");
                }
            }
        }
        Ok(s)
    }
    pub fn root() -> &'static Scope<'static> {
        static ROOT_SRC: &str = include_str!("../std/root.ln");
        static ROOT_AST: OnceLock<parse::Ln> = OnceLock::new();
        static ROOT_SCOPE_RS: OnceLock<Scope> = OnceLock::new();
        static ROOT_SCOPE_JS: OnceLock<Scope> = OnceLock::new();

        let ast = ROOT_AST
            .get_or_init(|| parse::get_ast(ROOT_SRC).expect("Invalid root scope source code!"));
        let resolver = || {
            let s = Scope {
                path: "@root".to_string(),
                parent: None,
                types: OrderedHashMap::new(),
                consts: OrderedHashMap::new(),
                functions: OrderedHashMap::new(),
                operatormappings: OrderedHashMap::new(),
                typeoperatormappings: OrderedHashMap::new(),
                exports: OrderedHashMap::new(),
            };
            Scope::load_scope(s, ast, true).expect("Invalid root scope definition")
        };
        if Program::is_target_lang_rs() {
            ROOT_SCOPE_RS.get_or_init(resolver)
        } else {
            ROOT_SCOPE_JS.get_or_init(resolver)
        }
    }
    pub fn from_src(path: &str, src: String) -> Result<(), Box<dyn std::error::Error>> {
        let txt = Box::pin(src);
        let txt_ptr: *const str = &**txt;
        // *How* would this move, anyways? But TODO: See if there's a way to handle this safely
        let ast = unsafe { parse::get_ast(&*txt_ptr)? };
        let mut s = Scope {
            path: path.to_string(),
            parent: Some(Scope::root()),
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            operatormappings: OrderedHashMap::new(),
            typeoperatormappings: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        };
        s = Scope::load_scope(s, &ast, false)?;
        let mut program = Program::get_program();
        program
            .scopes_by_file
            .insert(path.to_string(), (txt, ast, s));
        Program::return_program(program);
        Ok(())
    }

    pub fn child<'b>(&'a self) -> Scope<'b>
    where
        'a: 'b,
    {
        let path = format!("{}/child", self.path);
        Scope {
            path: path.clone(),
            parent: Some(self),
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            operatormappings: OrderedHashMap::new(),
            typeoperatormappings: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        }
    }

    // I hate the borrow checker
    #[allow(clippy::too_many_arguments)]
    pub fn merge(
        &mut self,
        mut types: OrderedHashMap<String, Arc<CType>>,
        mut consts: OrderedHashMap<String, Const>,
        mut functions: OrderedHashMap<String, Vec<Arc<Function>>>,
        mut operatormappings: OrderedHashMap<String, OperatorMapping>,
        mut typeoperatormappings: OrderedHashMap<String, TypeOperatorMapping>,
        mut exports: OrderedHashMap<String, Export>,
    ) {
        for (name, ctype) in types.drain() {
            self.types.insert(name, ctype);
        }
        for (name, constn) in consts.drain() {
            self.consts.insert(name, constn);
        }
        for (name, fs) in functions.drain() {
            if self.functions.contains_key(&name) {
                let func_vec = self.functions.get_mut(&name).unwrap();
                func_vec.splice(0..0, fs);
            } else {
                self.functions.insert(name, fs);
            }
        }
        for (name, opmap) in operatormappings.drain() {
            self.operatormappings.insert(name, opmap);
        }
        for (name, typeopmap) in typeoperatormappings.drain() {
            self.typeoperatormappings.insert(name, typeopmap);
        }
        for (name, export) in exports.drain() {
            self.exports.insert(name, export);
        }
    }

    pub fn resolve_typeoperator(
        &'a self,
        typeoperatorname: &String,
    ) -> Option<&'a TypeOperatorMapping> {
        // Tries to find the specified operator within the portion of the program accessible from the
        // current scope (so first checking the current scope, then all imports, then the root
        // scope) Returns a reference to the type and the scope it came from.
        // TODO: type ambiguity, infix/prefix ambiguity, etc
        match self.typeoperatormappings.get(typeoperatorname) {
            Some(o) => Some(o),
            None => match &self.parent {
                None => None,
                Some(p) => p.resolve_typeoperator(typeoperatorname),
            },
        }
    }

    pub fn resolve_const(&'a self, constname: &String) -> Option<&'a Const> {
        match self.consts.get(constname) {
            Some(c) => Some(c),
            None => match &self.parent {
                None => None,
                Some(p) => p.resolve_const(constname),
            },
        }
    }

    pub fn resolve_type(&'a self, typename: &str) -> Option<Arc<CType>> {
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
        match self.types.get(typename) {
            Some(t) => Some(t.clone()),
            None => match &self.parent {
                None => None,
                Some(p) => p.resolve_type(typename),
            },
        }
    }

    pub fn resolve_operator(&'a self, operatorname: &String) -> Option<&'a OperatorMapping> {
        // Tries to find the specified operator within the portion of the program accessible from the
        // current scope (so first checking the current scope, then all imports, then the root
        // scope) Returns a reference to the type and the scope it came from.
        // TODO: type ambiguity, infix/prefix ambiguity, etc
        match self.operatormappings.get(operatorname) {
            Some(o) => Some(o),
            None => match &self.parent {
                None => None,
                Some(p) => p.resolve_operator(operatorname),
            },
        }
    }

    pub fn resolve_function_types(&'a self, function: &String) -> Arc<CType> {
        // Gets every function visible from the specified scope with the same name and returns the
        // possible types in an array. TODO: Have the Function just have this type on the structure
        // so it doesn't need to be recreated each time.
        let mut scope_to_check: Option<&Scope> = Some(self);
        let mut fs = Vec::new();
        while scope_to_check.is_some() {
            if let Some(s) = scope_to_check {
                if let Some(funcs) = s.functions.get(function) {
                    for f in funcs {
                        fs.push(f.clone()); // TODO: Drop this clone
                    }
                }
                scope_to_check = match &s.parent {
                    Some(p) => Some(*p),
                    None => None,
                };
            }
        }
        let out_types = fs
            .iter()
            .map(|f| {
                let generics = match &f.kind {
                    FnKind::Normal
                    | FnKind::External(_)
                    | FnKind::Bind(_)
                    | FnKind::ExternalBind(_, _)
                    | FnKind::Derived
                    | FnKind::DerivedVariadic
                    | FnKind::Static => None,
                    FnKind::Generic(gs, _)
                    | FnKind::BoundGeneric(gs, _)
                    | FnKind::ExternalGeneric(gs, _, _) => {
                        Some(gs.iter().map(|(g, _)| g.clone()).collect::<Vec<String>>())
                    }
                };
                // TODO: Potentially refactor this
                let input = f
                    .args()
                    .iter()
                    .map(|(_, _, arg)| arg.clone())
                    .collect::<Vec<Arc<CType>>>();
                let output = f.rettype();
                match generics {
                    None => Arc::new(CType::Function(Arc::new(CType::Tuple(input)), output)),
                    Some(gs) => Arc::new(CType::Generic(
                        f.name.clone(),
                        gs,
                        Arc::new(CType::Function(Arc::new(CType::Tuple(input)), output)),
                    )),
                }
            })
            .collect::<Vec<Arc<CType>>>();
        if out_types.is_empty() {
            Arc::new(CType::Void)
        } else if out_types.len() == 1 {
            out_types.into_iter().nth(0).unwrap()
        } else {
            Arc::new(CType::AnyOf(out_types))
        }
    }

    pub fn resolve_function_by_type(
        &'a self,
        function: &String,
        fn_type: Arc<CType>,
    ) -> Option<Arc<Function>> {
        // Iterates through every function with the same name visible from the provided scope and
        // returns the one that matches the provided function type, if any
        let fn_type_str = fn_type.degroup().to_strict_string(false);
        let mut scope_to_check: Option<&Scope> = Some(self);
        while scope_to_check.is_some() {
            if let Some(s) = scope_to_check {
                if let Some(funcs) = s.functions.get(function) {
                    for f in funcs {
                        if f.typen.clone().to_strict_string(false) == fn_type_str {
                            return Some(f.clone());
                        }
                    }
                }
                scope_to_check = match &s.parent {
                    Some(p) => Some(*p),
                    None => None,
                };
            }
        }
        None
    }

    pub fn resolve_generic_function(
        mut self,
        function: &String,
        generic_types: &[Arc<CType>],
        args: &[Arc<CType>],
    ) -> Option<(Scope<'a>, Arc<Function>)> {
        // Tries to find the specified function within the portion of the program accessible from
        // the current scope (so first checking the current scope, then all imports, then the root
        // scope). It checks against the args array to find a match. TODO: Go beyond exact matching
        // Returns a reference to the function and the scope it came from.
        let mut scope_to_check: Option<&Scope> = Some(&self);
        let mut fs = Vec::new();
        while scope_to_check.is_some() {
            if let Some(s) = scope_to_check {
                if let Some(funcs) = s.functions.get(function) {
                    // Why is this okay but cloning funcs and then appending is not?
                    for f in funcs {
                        fs.push(f.clone());
                    }
                }
                scope_to_check = match &s.parent {
                    Some(p) => Some(*p),
                    None => None,
                };
            }
        }
        let mut generic_fs = Vec::new();
        for f in &fs {
            match &f.kind {
                FnKind::Normal
                | FnKind::External(_)
                | FnKind::Bind(_)
                | FnKind::ExternalBind(_, _)
                | FnKind::Derived
                | FnKind::DerivedVariadic
                | FnKind::Static => { /* Do nothing */ }
                FnKind::Generic(g, _)
                | FnKind::BoundGeneric(g, _)
                | FnKind::ExternalGeneric(g, _, _) => {
                    // TODO: Check interface constraints once interfaces exist
                    if g.len() != generic_types.len() {
                        continue;
                    }
                    if args.len() != f.args().len() {
                        continue;
                    }
                    // Passes the preliminary check
                    generic_fs.push(f.clone());
                }
            }
        }
        let mut possible_args_vec = Vec::new();
        for f in &generic_fs {
            match &f.kind {
                FnKind::Normal
                | FnKind::External(_)
                | FnKind::Bind(_)
                | FnKind::ExternalBind(_, _)
                | FnKind::Derived
                | FnKind::DerivedVariadic
                | FnKind::Static => {
                    panic!("This should be impossible. If reached it would generate faulty code");
                }
                FnKind::Generic(gen_args, _)
                | FnKind::BoundGeneric(gen_args, _)
                | FnKind::ExternalGeneric(gen_args, _, _) => {
                    let args = f
                        .args()
                        .iter()
                        .map(|(name, kind, argtype)| {
                            (name.clone(), kind.clone(), {
                                let mut a = argtype.clone();
                                for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                                    a = a.swap_subtype(o.clone(), n.clone());
                                }
                                a
                            })
                        })
                        .collect::<Vec<(String, ArgKind, Arc<CType>)>>();
                    possible_args_vec.push(args);
                }
            }
        }
        let mut match_index = None;
        for (i, possible_args) in possible_args_vec.iter().enumerate() {
            let mut args_match = true;
            for (i, arg) in args.iter().enumerate() {
                // This is pretty cheap, but for now, a "non-strict" string representation
                // of the CTypes is how we'll match the args against each other. TODO: Do
                // this without constructing a string to compare against each other.
                if !possible_args[i].2.clone().accepts(arg.clone()) {
                    args_match = false;
                    break;
                }
            }
            if args_match {
                match_index = Some(i);
                break;
            }
        }
        if let Some(i) = match_index {
            // We've found a match. We now need to return the resolved generic function.
            // If any of the arguments is *itself* a generic function that we have resolved, we
            // *also* need to resolve that as well.
            let generic_f = generic_fs.get(i).unwrap();
            for arg in args {
                match &**arg {
                    CType::Generic(n, _, t) if matches!(&**t, CType::Function(..)) => {
                        if let Some(func) = self.resolve_function_by_type(n, t.clone()) {
                            match Function::from_generic_function(
                                self,
                                &func,
                                generic_types.to_vec(),
                            ) {
                                Ok((s, _)) => {
                                    self = s;
                                }
                                Err(_) => return None,
                            }
                        }
                    }
                    _ => {}
                }
            }
            let temp_scope = self.child();
            match Function::from_generic_function(temp_scope, generic_f, generic_types.to_vec()) {
                Err(_) => return None, // TODO: Should this be a panic?
                Ok((_, realized_f)) => {
                    return Some((self, realized_f));
                }
            }
        }
        None
    }

    pub fn resolve_function(
        self,
        function: &String,
        args: &[Arc<CType>],
    ) -> Option<(Scope<'a>, Arc<Function>)> {
        // We should prefer the "normal" function, if it matches, use it, otherwise try to go with
        // a generic function, if possible.
        // TODO: This boolean *shouldn't* be necessary, but I can't convince the borrow checker
        // otherwise
        let is_normal = self.resolve_normal_function(function, args).is_some();
        if is_normal {
            self.resolve_normal_function(function, args)
                .map(|f| (self, f))
        } else {
            match self.resolve_function_generic_args(function, args) {
                Some(gs) => {
                  self.resolve_generic_function(function, &gs, args)
                }
                None => None,
            }
        }
    }

    pub fn resolve_function_generic_args(
        &'a self,
        function: &String,
        args: &[Arc<CType>],
    ) -> Option<Vec<Arc<CType>>> {
        let mut scope_to_check: Option<&Scope> = Some(self);
        let mut fs = Vec::new();
        while scope_to_check.is_some() {
            if let Some(s) = scope_to_check {
                if let Some(funcs) = s.functions.get(function) {
                    // Why is this okay but cloning funcs and then appending is not?
                    for f in funcs {
                        fs.push(f);
                    }
                }
                // TODO: Types are internally referred to by their structural name, not by the name the
                // user gives them, so a type constructor function needs to have a lookup done by type and
                // then coerce into the constructor function name and then call it. We *should* just be
                // able to use the user's name for the types, but this was undone for generic functions to
                // work correctly. We should try to find a better solution than this function resolution
                // hackery.
                if let Some(t) = s.resolve_type(function) {
                    let constructor_fn_name = t.to_callable_string();
                    match s.functions.get(&constructor_fn_name) {
                        Some(funcs) => {
                            for f in funcs {
                                fs.push(f);
                            }
                        }
                        None => { /* Nothing matched, move on */ }
                    }
                }
                scope_to_check = match &s.parent {
                    Some(p) => Some(*p),
                    None => None,
                };
            }
        }
        for f in &fs {
            // TODO: Handle this more generically, and in a way that allows users to write
            // variadic functions
            match &f.kind {
                FnKind::DerivedVariadic => {
                    // The special path where the length doesn't matter as long as all of the
                    // actual args are the same type as the function's arg.
                    let mut args_match = true;
                    for arg in args.iter() {
                        if !f.args()[0].2.clone().accepts(arg.clone()) {
                            args_match = false;
                            break;
                        }
                    }
                    // If the args match, then we got a hit for a non-generic function first, so we
                    // shouldn't return generic args
                    if args_match {
                        return None;
                    }
                }
                FnKind::Normal
                | FnKind::External(_)
                | FnKind::Bind(_)
                | FnKind::ExternalBind(_, _)
                | FnKind::Derived
                | FnKind::Static => {
                    if args.len() != f.args().len() {
                        continue;
                    }
                    let mut args_match = true;
                    for (i, arg) in args.iter().enumerate() {
                        // This is pretty cheap, but for now, a "non-strict" string representation
                        // of the CTypes is how we'll match the args against each other. TODO: Do
                        // this without constructing a string to compare against each other.
                        if !f.args()[i].2.clone().accepts(arg.clone()) {
                            args_match = false;
                            break;
                        }
                    }
                    // If the args match, then we got a hit for a non-generic function first, so we
                    // shouldn't return generic args
                    if args_match {
                        return None;
                    }
                }
                FnKind::Generic(g, _)
                | FnKind::BoundGeneric(g, _)
                | FnKind::ExternalGeneric(g, _, _) => {
                    if args.len() != f.args().len() {
                        continue;
                    }
                    match CType::infer_generics(self, g, &f.args(), args) {
                        Ok(gs) => return Some(gs),
                        Err(_) => { /* Do nothing */ }
                    };
                }
            }
        }
        None
    }

    pub fn resolve_normal_function(
        &'a self,
        function: &String,
        args: &[Arc<CType>],
    ) -> Option<Arc<Function>> {
        // Tries to find the specified function within the portion of the program accessible from
        // the current scope (so first checking the current scope, then all imports, then the root
        // scope). It checks against the args array to find a match. TODO: Go beyond exact matching
        // Returns a reference to the function and the scope it came from.
        let mut scope_to_check: Option<&Scope> = Some(self);
        let mut fs = Vec::new();
        while scope_to_check.is_some() {
            if let Some(s) = scope_to_check {
                if let Some(funcs) = s.functions.get(function) {
                    // Why is this okay but cloning funcs and then appending is not?
                    for f in funcs {
                        fs.push(f);
                    }
                }
                // TODO: Types are internally referred to by their structural name, not by the name the
                // user gives them, so a type constructor function needs to have a lookup done by type and
                // then coerce into the constructor function name and then call it. We *should* just be
                // able to use the user's name for the types, but this was undone for generic functions to
                // work correctly. We should try to find a better solution than this function resolution
                // hackery.
                if let Some(t) = s.resolve_type(function) {
                    let constructor_fn_name = t.to_callable_string();
                    match s.functions.get(&constructor_fn_name) {
                        Some(funcs) => {
                            for f in funcs {
                                fs.push(f);
                            }
                        }
                        None => { /* Nothing matched, move on */ }
                    }
                }
                scope_to_check = match &s.parent {
                    Some(p) => Some(*p),
                    None => None,
                };
            }
        }
        for f in fs {
            // TODO: Handle this more generically, and in a way that allows users to write
            // variadic functions
            match f.kind {
                FnKind::DerivedVariadic => {
                    // The special path where the length doesn't matter as long as all of the
                    // actual args are the same type as the function's arg.
                    let mut args_match = true;
                    for arg in args.iter() {
                        if !f.args()[0].2.clone().accepts(arg.clone()) {
                            args_match = false;
                            break;
                        }
                    }
                    if args_match {
                        return Some(f.clone());
                    }
                }
                FnKind::Normal
                | FnKind::External(_)
                | FnKind::Bind(_)
                | FnKind::ExternalBind(_, _)
                | FnKind::Derived
                | FnKind::Static => {
                    if args.len() != f.args().len() {
                        continue;
                    }
                    let mut args_match = true;
                    for (i, arg) in args.iter().enumerate() {
                        // This is pretty cheap, but for now, a "non-strict" string representation
                        // of the CTypes is how we'll match the args against each other. TODO: Do
                        // this without constructing a string to compare against each other.
                        if !f.args()[i].2.clone().accepts(arg.clone()) {
                            args_match = false;
                            break;
                        }
                    }
                    if args_match {
                        return Some(f.clone());
                    }
                }
                FnKind::Generic(_, _)
                | FnKind::BoundGeneric(_, _)
                | FnKind::ExternalGeneric(_, _, _) => { /* Do nothing */ }
            }
        }
        None
    }
}

macro_rules! merge {
    ( $parent: expr, $child: expr $(,)?) => {
        let Scope {
            types,
            consts,
            functions,
            operatormappings,
            typeoperatormappings,
            exports,
            ..
        } = $child;
        $parent.merge(
            types,
            consts,
            functions,
            operatormappings,
            typeoperatormappings,
            exports,
        );
    };
}

pub(crate) use merge;
