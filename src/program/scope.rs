use std::sync::OnceLock;

use ordered_hash_map::OrderedHashMap;

use super::CType;
use super::Const;
use super::Export;
use super::FnKind;
use super::Function;
use super::Import;
use super::OperatorMapping;
use super::Program;
use super::TypeOperatorMapping;
use crate::parse;

#[derive(Clone, Debug)]
pub struct Scope<'a> {
    pub path: String, // Now necessary since we reference by path name :/
    pub parent: Option<&'a Scope<'a>>,
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

impl<'a> Scope<'a> {
    pub fn root() -> &'static Scope<'static> {
        static ROOT_SRC: &str = include_str!("../std/root.ln");
        static ROOT_AST: OnceLock<parse::Ln> = OnceLock::new();
        static ROOT_SCOPE: OnceLock<Scope> = OnceLock::new();

        let ast = ROOT_AST
            .get_or_init(|| parse::get_ast(ROOT_SRC).expect("Invalid root scope source code!"));
        ROOT_SCOPE.get_or_init(|| {
            let mut s = Scope {
                path: "@root".to_string(),
                parent: None,
                imports: OrderedHashMap::new(),
                types: OrderedHashMap::new(),
                consts: OrderedHashMap::new(),
                functions: OrderedHashMap::new(),
                operatormappings: OrderedHashMap::new(),
                typeoperatormappings: OrderedHashMap::new(),
                exports: OrderedHashMap::new(),
            };
            // The root scope has no imports, so this portion is skipped
            // TODO: Eliminate the duplicate code
            for (i, element) in ast.body.iter().enumerate() {
                match element {
                    parse::RootElements::Types(t) => {
                        CType::from_ast(&mut s, t, false).expect("Invalid root scope type");
                    }

                    parse::RootElements::Functions(f) => Function::from_ast(&mut s, f, false).expect("Invalid root scope function"),
                    parse::RootElements::ConstDeclaration(c) => Const::from_ast(&mut s, c, false).expect("Invalid root scope const declaration"),
                    parse::RootElements::OperatorMapping(o) => {
                        OperatorMapping::from_ast(&mut s, o, false).expect("Invalid root scope operator mapping")
                    }
                    parse::RootElements::TypeOperatorMapping(o) => {
                        TypeOperatorMapping::from_ast(&mut s, o, false).expect("Invalid root scope type operator mapping")
                    }
                    parse::RootElements::Exports(e) => match &e.exportable {
                        parse::Exportable::Functions(f) => Function::from_ast(&mut s, f, true).expect("Invalid root scope exported function"),
                        parse::Exportable::ConstDeclaration(c) => Const::from_ast(&mut s, c, true).expect("Invalid root scope exported const declaration"),
                        parse::Exportable::OperatorMapping(o) => {
                            OperatorMapping::from_ast(&mut s, o, true).expect("Invalid root scope exported operator mapping")
                        }
                        parse::Exportable::TypeOperatorMapping(o) => {
                            TypeOperatorMapping::from_ast(&mut s, o, true).expect("Invalid root scope exported type operator mapping")
                        }
                        parse::Exportable::Types(t) => {
                            CType::from_ast(&mut s, t, true).expect("Invalid root scope exported type");
                        }
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
                            match c.name.as_str() {
                                "Type" | "Generic" | "Bound" | "BoundGeneric" | "Int" | "Float"
                                | "Bool" | "String" => { /* Do nothing for the 'structural' types */ }
                                g @ ("Group" | "Array" | "Fail" | "Neg" | "Len" | "Size" | "FileStr"
                                | "Env" | "EnvExists" | "Not") => CType::from_generic(&mut s, g, 1),
                                g @ ("Function" | "Tuple" | "Field" | "Either" | "AnyOf" | "Buffer" | "Add"
                                | "Sub" | "Mul" | "Div" | "Mod" | "Pow" | "Min" | "Max" | "If" | "And" | "Or"
                                | "Xor" | "Nand" | "Nor" | "Xnor" | "Eq" | "Neq" | "Lt" | "Lte"
                                | "Gt" | "Gte") => CType::from_generic(&mut s, g, 2),
                                // TODO: Also add support for three arg `If` and `Env` with a
                                // default property via overloading types
                                unknown => {
                                    panic!("Unknown ctype {} defined in root scope. There's something wrong with the compiler.", unknown);
                                }
                            }
                        }
                        e => eprintln!("TODO: Not yet supported export syntax: {:?}\nLast good parsed lines:\n{:?}\n{:?}", e, ast.body[i - 2], ast.body[i - 1]),
                    },
                    parse::RootElements::Whitespace(_) => { /* Do nothing */ }
                    parse::RootElements::Interfaces(_) => {
                        panic!("Interfaces not yet implemented");
                    }
                }
            }
            s
        })
    }
    pub fn from_src(
        program: &'a mut Program,
        path: &str,
        src: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let txt = Box::pin(src);
        let txt_ptr: *const str = &**txt;
        // *How* would this move, anyways? But TODO: See if there's a way to handle this safely
        let ast = unsafe { parse::get_ast(&*txt_ptr)? };
        let mut s = Scope {
            path: path.to_string(),
            parent: Some(Scope::root()),
            imports: OrderedHashMap::new(),
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            operatormappings: OrderedHashMap::new(),
            typeoperatormappings: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        };
        for i in ast.imports.iter() {
            Import::from_ast(program, path.to_string(), &mut s, i)?;
        }
        for (i, element) in ast.body.iter().enumerate() {
            match element {
                parse::RootElements::Types(t) => match CType::from_ast(&mut s, t, false) {
                    Err(e) => Err(e),
                    Ok(_) => Ok(()),
                }?, // TODO: Make this match the rest?

                parse::RootElements::Functions(f) => Function::from_ast(&mut s, f, false)?,
                parse::RootElements::ConstDeclaration(c) => Const::from_ast(&mut s, c, false)?,
                parse::RootElements::OperatorMapping(o) => {
                    OperatorMapping::from_ast(&mut s, o, false)?
                }
                parse::RootElements::TypeOperatorMapping(o) => {
                    TypeOperatorMapping::from_ast(&mut s, o, false)?
                }
                parse::RootElements::Exports(e) => match &e.exportable {
                    parse::Exportable::Functions(f) => Function::from_ast(&mut s, f, true)?,
                    parse::Exportable::ConstDeclaration(c) => Const::from_ast(&mut s, c, true)?,
                    parse::Exportable::OperatorMapping(o) => {
                        OperatorMapping::from_ast(&mut s, o, true)?
                    }
                    parse::Exportable::TypeOperatorMapping(o) => {
                        TypeOperatorMapping::from_ast(&mut s, o, true)?
                    }
                    parse::Exportable::Types(t) => match CType::from_ast(&mut s, t, true) {
                        Err(e) => Err(e),
                        Ok(_) => Ok(()),
                    }?, // TODO: Make this match the rest?
                    parse::Exportable::CTypes(_) => {
                        return Err(
                            "ctypes can only be defined in the compiler internals".into()
                        );
                    }
                    e => eprintln!("TODO: Not yet supported export syntax: {:?}\nLast good parsed lines:\n{:?}\n{:?}", e, ast.body[i - 2], ast.body[i - 1]),
                },
                parse::RootElements::Whitespace(_) => { /* Do nothing */ }
                parse::RootElements::Interfaces(_) => {
                    return Err("Interfaces not yet implemented".into());
                }
            }
        }
        program
            .scopes_by_file
            .insert(path.to_string(), (txt, ast, s));
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
            imports: OrderedHashMap::new(),
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            operatormappings: OrderedHashMap::new(),
            typeoperatormappings: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        }
    }

    pub fn merge_functions(&mut self, mut functions: OrderedHashMap<String, Vec<Function>>) {
        for (name, fs) in functions.drain() {
            if self.functions.contains_key(&name) {
                let func_vec = self.functions.get_mut(&name).unwrap();
                for f in fs {
                    func_vec.push(f);
                }
            } else {
                self.functions.insert(name, fs);
            }
        }
    }

    pub fn resolve_typeoperator(
        &'a self,
        typeoperatorname: &String,
    ) -> Option<&TypeOperatorMapping> {
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

    pub fn resolve_const(&'a self, constname: &String) -> Option<&Const> {
        match self.consts.get(constname) {
            Some(c) => Some(c),
            None => match &self.parent {
                None => None,
                Some(p) => p.resolve_const(constname),
            },
        }
    }

    pub fn resolve_type(&'a self, typename: &String) -> Option<&CType> {
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
            Some(t) => Some(t),
            None => match &self.parent {
                None => None,
                Some(p) => p.resolve_type(typename),
            },
        }
    }

    pub fn resolve_operator(&'a self, operatorname: &String) -> Option<&OperatorMapping> {
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

    pub fn resolve_function_types(&'a self, function: &String) -> CType {
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
                    | FnKind::Bind(_)
                    | FnKind::Derived
                    | FnKind::DerivedVariadic
                    | FnKind::Static => None,
                    FnKind::Generic(gs, _) | FnKind::BoundGeneric(gs, _) => {
                        Some(gs.iter().map(|(g, _)| g.clone()).collect::<Vec<String>>())
                    }
                };
                let input = f
                    .args
                    .iter()
                    .map(|(_, arg)| arg.clone())
                    .collect::<Vec<CType>>();
                let output = f.rettype.clone();
                match generics {
                    None => CType::Function(Box::new(CType::Tuple(input)), Box::new(output)),
                    Some(gs) => CType::Generic(
                        f.name.clone(),
                        gs,
                        Box::new(CType::Function(
                            Box::new(CType::Tuple(input)),
                            Box::new(output),
                        )),
                    ),
                }
            })
            .collect::<Vec<CType>>();
        if out_types.is_empty() {
            CType::Void
        } else if out_types.len() == 1 {
            out_types.into_iter().nth(0).unwrap()
        } else {
            CType::AnyOf(out_types)
        }
    }

    pub fn resolve_function_by_type(
        &'a self,
        function: &String,
        fn_type: &CType,
    ) -> Option<&Function> {
        // Iterates through every function with the same name visible from the provided scope and
        // returns the one that matches the provided function type, if any
        let fn_type_str = fn_type.degroup().to_strict_string(false);
        let mut scope_to_check: Option<&Scope> = Some(self);
        while scope_to_check.is_some() {
            if let Some(s) = scope_to_check {
                if let Some(funcs) = s.functions.get(function) {
                    for f in funcs {
                        // TODO: Just have the function type on the Function object
                        let f_type = CType::Function(
                            Box::new(CType::Tuple(
                                f.args
                                    .iter()
                                    .map(|(_, t)| t.clone())
                                    .collect::<Vec<CType>>(),
                            )),
                            Box::new(f.rettype.clone()),
                        );
                        if f_type.degroup().to_strict_string(false) == fn_type_str {
                            return Some(f);
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
        &'a mut self,
        function: &String,
        generic_types: &[CType],
        args: &[CType],
    ) -> Option<&Function> {
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
                | FnKind::Bind(_)
                | FnKind::Derived
                | FnKind::DerivedVariadic
                | FnKind::Static => { /* Do nothing */ }
                FnKind::Generic(g, _) | FnKind::BoundGeneric(g, _) => {
                    // TODO: Check interface constraints once interfaces exist
                    if g.len() != generic_types.len() {
                        continue;
                    }
                    if args.len() != f.args.len() {
                        continue;
                    }
                    // Passes the preliminary check
                    generic_fs.push(f.clone());
                }
            }
        }
        // The following is insanity to deal with the borrow checker. It could have been done
        // without the temporary vector or the child scopes, but using a mutable reference in a
        // loop is a big no-no.
        let mut temp_scopes = Vec::new();
        for _ in &generic_fs {
            temp_scopes.push(self.child());
        }
        let mut realized_fs = Vec::new();
        for (i, temp_scope) in temp_scopes.iter_mut().enumerate() {
            let f = generic_fs.get(i).unwrap();
            match Function::from_generic_function(temp_scope, f, generic_types.to_vec()) {
                Err(_) => { /* Do nothing */ }
                Ok(f) => realized_fs.push(f.clone()),
            }
        }
        let mut funcs = Vec::new();
        for temp_scope in temp_scopes {
            let Scope { functions, .. } = temp_scope;
            funcs.push(functions);
        }
        for functions in funcs {
            self.merge_functions(functions);
        }
        for f in realized_fs {
            let mut args_match = true;
            for (i, arg) in args.iter().enumerate() {
                // This is pretty cheap, but for now, a "non-strict" string representation
                // of the CTypes is how we'll match the args against each other. TODO: Do
                // this without constructing a string to compare against each other.
                if !f.args[i].1.accepts(arg) {
                    args_match = false;
                    break;
                }
                // In case this function generated any new types, let's make sure the
                // constructor and helper functions all exist, though we can assume this is
                // true of the inputs, it's only the return type we need to double-check
                let fun_name = f.rettype.to_callable_string();
                let (_, fs) = f.rettype.to_functions(fun_name.clone());
                self.types.insert(fun_name, f.rettype.clone());
                if !fs.is_empty() {
                    let mut name_fn_pairs = OrderedHashMap::new();
                    for f in fs {
                        if name_fn_pairs.contains_key(&f.name) {
                            let v: &mut Vec<Function> = name_fn_pairs.get_mut(&f.name).unwrap();
                            v.push(f.clone());
                        } else {
                            name_fn_pairs.insert(f.name.clone(), vec![f.clone()]);
                        }
                    }
                    for (name, fns) in name_fn_pairs.drain() {
                        if self.functions.contains_key(&name) {
                            let func_vec = self.functions.get_mut(&name).unwrap();
                            for f in fns {
                                func_vec.push(f);
                            }
                        } else {
                            self.functions.insert(name, fns);
                        }
                    }
                }
            }
            if args_match {
                // We want to keep this one around, so we copy this function to the correct
                // scope
                if self.functions.contains_key(&f.name) {
                    let func_vec = self.functions.get_mut(&f.name).unwrap();
                    func_vec.push(f.clone());
                } else {
                    self.functions.insert(f.name.clone(), vec![f.clone()]);
                }
                return match self.functions.get(&f.name) {
                    None => None,
                    Some(fs) => {
                        // We know it's the last one because we just put it there
                        fs.last()
                    }
                };
            }
        }
        None
    }

    pub fn resolve_function(&'a mut self, function: &String, args: &[CType]) -> Option<&Function> {
        // First we try to get generic arguments for this function, if they exist, we return a
        // generic function realization, otherwise we return a normal function
        match self.resolve_function_generic_args(function, args) {
            Some(gs) => self.resolve_generic_function(function, &gs, args),
            None => self.resolve_normal_function(function, args),
        }
    }

    pub fn resolve_function_generic_args(
        &'a self,
        function: &String,
        args: &[CType],
    ) -> Option<Vec<CType>> {
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
                        if !f.args[0].1.accepts(arg) {
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
                FnKind::Normal | FnKind::Bind(_) | FnKind::Derived | FnKind::Static => {
                    if args.len() != f.args.len() {
                        continue;
                    }
                    let mut args_match = true;
                    for (i, arg) in args.iter().enumerate() {
                        // This is pretty cheap, but for now, a "non-strict" string representation
                        // of the CTypes is how we'll match the args against each other. TODO: Do
                        // this without constructing a string to compare against each other.
                        if !f.args[i].1.accepts(arg) {
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
                FnKind::Generic(g, _) | FnKind::BoundGeneric(g, _) => {
                    if args.len() != f.args.len() {
                        continue;
                    }
                    match CType::infer_generics(self, g, &f.args, args) {
                        Ok(gs) => {
                            return Some(gs);
                        }
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
        args: &[CType],
    ) -> Option<&Function> {
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
        for f in &fs {
            // TODO: Handle this more generically, and in a way that allows users to write
            // variadic functions
            match &f.kind {
                FnKind::DerivedVariadic => {
                    // The special path where the length doesn't matter as long as all of the
                    // actual args are the same type as the function's arg.
                    let mut args_match = true;
                    for arg in args.iter() {
                        if !f.args[0].1.accepts(arg) {
                            args_match = false;
                            break;
                        }
                    }
                    if args_match {
                        return Some(f);
                    }
                }
                FnKind::Normal | FnKind::Bind(_) | FnKind::Derived | FnKind::Static => {
                    if args.len() != f.args.len() {
                        continue;
                    }
                    let mut args_match = true;
                    for (i, arg) in args.iter().enumerate() {
                        // This is pretty cheap, but for now, a "non-strict" string representation
                        // of the CTypes is how we'll match the args against each other. TODO: Do
                        // this without constructing a string to compare against each other.
                        if !f.args[i].1.accepts(arg) {
                            args_match = false;
                            break;
                        }
                    }
                    if args_match {
                        return Some(f);
                    }
                }
                FnKind::Generic(_, _) | FnKind::BoundGeneric(_, _) => { /* Do nothing */ }
            }
        }
        None
    }
}
