use std::fs::read_to_string;
use std::pin::Pin;

use super::CType;
use super::Const;
use super::FnKind;
use super::Function;
use super::OperatorMapping;
use super::Scope;
use super::TypeOperatorMapping;
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
        p.load("@root".to_string())?;
        // Load the entry file
        match p.load(entry_file) {
            Ok(p) => p,
            Err(e) => {
                // Somehow, trying to print this error can crash Rust!? Really not good.
                // Will need to figure out how to make these errors clearer to users.
                return Err(format!("{}", e).into());
            }
        };
        Ok(p)
    }

    pub fn load(self: &mut Self, path: String) -> Result<(), Box<dyn std::error::Error>> {
        let ln_src = if path.starts_with("@") {
            match path.as_str() {
                "@root" => include_str!("../std/root.ln").to_string(),
                "@std/app" => include_str!("../std/app.ln").to_string(),
                _ => {
                    return Err(format!("Unknown standard library named {}", &path).into());
                }
            }
        } else {
            read_to_string(&path)?
        };
        Ok(Scope::from_src(self, &path, ln_src)?)
    }

    pub fn resolve_typeoperator<'a>(
        self: &'a Self,
        scope: &'a Scope,
        typeoperatorname: &String,
    ) -> Option<&TypeOperatorMapping> {
        // Tries to find the specified operator within the portion of the program accessible from the
        // current scope (so first checking the current scope, then all imports, then the root
        // scope) Returns a reference to the type and the scope it came from.
        // TODO: type ambiguity, infix/prefix ambiguity, etc
        match scope.typeoperatormappings.get(typeoperatorname) {
            Some(o) => Some(o),
            None => match &scope.parent {
                None => None,
                Some(p) => match self.scopes_by_file.get(p) {
                    None => None,
                    Some((_, _, s)) => self.resolve_typeoperator(s, typeoperatorname),
                },
            },
        }
    }

    pub fn resolve_const<'a>(
        self: &'a Self,
        scope: &'a Scope,
        constname: &String,
    ) -> Option<&Const> {
        match scope.consts.get(constname) {
            Some(c) => Some(c),
            None => match &scope.parent {
                None => None,
                Some(p) => match self.scopes_by_file.get(p) {
                    None => None,
                    Some((_, _, s)) => self.resolve_const(s, constname),
                },
            },
        }
    }

    pub fn resolve_type<'a>(self: &'a Self, scope: &'a Scope, typename: &String) -> Option<&CType> {
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
            Some(t) => Some(t),
            None => match &scope.parent {
                None => None,
                Some(p) => match self.scopes_by_file.get(p) {
                    None => None,
                    Some((_, _, s)) => self.resolve_type(s, typename),
                },
            },
        }
    }

    pub fn resolve_operator<'a>(
        self: &'a Self,
        scope: &'a Scope,
        operatorname: &String,
    ) -> Option<&OperatorMapping> {
        // Tries to find the specified operator within the portion of the program accessible from the
        // current scope (so first checking the current scope, then all imports, then the root
        // scope) Returns a reference to the type and the scope it came from.
        // TODO: type ambiguity, infix/prefix ambiguity, etc
        match scope.operatormappings.get(operatorname) {
            Some(o) => Some(o),
            None => match &scope.parent {
                None => None,
                Some(p) => match self.scopes_by_file.get(p) {
                    None => None,
                    Some((_, _, s)) => self.resolve_operator(s, operatorname),
                },
            },
        }
    }

    pub fn resolve_generic_function<'a>(
        self: &'a mut Self,
        scope: &'a mut Scope,
        function: &String,
        generic_types: &Vec<CType>,
        args: &Vec<CType>,
    ) -> Option<&Function> {
        // Tries to find the specified function within the portion of the program accessible from
        // the current scope (so first checking the current scope, then all imports, then the root
        // scope). It checks against the args array to find a match. TODO: Go beyond exact matching
        // Returns a reference to the function and the scope it came from.
        let mut scope_to_check: Option<&Scope> = Some(scope);
        let mut fs = Vec::new();
        while scope_to_check.is_some() {
            match scope_to_check {
                Some(s) => {
                    match s.functions.get(function) {
                        Some(funcs) => {
                            // Why is this okay but cloning funcs and then appending is not?
                            for f in funcs {
                                fs.push(f.clone());
                            }
                        }
                        None => {}
                    }
                    scope_to_check = match &s.parent {
                        Some(p) => match self.scopes_by_file.get(p) {
                            Some((_, _, s)) => Some(s),
                            None => None,
                        },
                        None => None,
                    };
                }
                None => {}
            }
        }
        let mut generic_fs = Vec::new();
        for f in &fs {
            match &f.kind {
                FnKind::Normal | FnKind::Bind(_) | FnKind::Derived | FnKind::DerivedVariadic => {
                    /* Do nothing */
                }
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
            temp_scopes.push(scope.child(self));
        }
        let mut realized_fs = Vec::new();
        for (i, temp_scope) in temp_scopes.iter_mut().enumerate() {
            let f = generic_fs.get(i).unwrap();
            match Function::from_generic_function(temp_scope, self, f, generic_types.clone()) {
                Err(_) => { /* Do nothing */ }
                Ok(f) => realized_fs.push(f.clone()),
            }
        }
        loop {
            let temp_scope = temp_scopes.pop();
            match temp_scope {
                Some(mut temp_scope) => {
                    scope.merge_child_functions(&mut temp_scope);
                }
                None => break,
            }
        }
        for f in realized_fs {
            let mut args_match = true;
            for (i, arg) in args.iter().enumerate() {
                // This is pretty cheap, but for now, a "non-strict" string representation
                // of the CTypes is how we'll match the args against each other. TODO: Do
                // this without constructing a string to compare against each other.
                if f.args[i].1.degroup().to_strict_string(false)
                    != arg.degroup().to_strict_string(false)
                {
                    args_match = false;
                    break;
                }
            }
            if args_match {
                // We want to keep this one around, so we copy this function to the correct
                // scope
                if scope.functions.contains_key(&f.name) {
                    let func_vec = scope.functions.get_mut(&f.name).unwrap();
                    func_vec.push(f.clone());
                } else {
                    scope.functions.insert(f.name.clone(), vec![f.clone()]);
                }
                return match scope.functions.get(&f.name) {
                    None => None,
                    Some(fs) => fs.last(), // We know it's the last one because we just put
                                           // it there
                };
            }
        }
        None
    }

    pub fn resolve_function<'a>(
        self: &'a mut Self,
        scope: &'a mut Scope,
        function: &String,
        args: &Vec<CType>,
    ) -> Option<&Function> {
        // First we try to get generic arguments for this function, if they exist, we return a
        // generic function realization, otherwise we return a normal function
        match self.resolve_function_generic_args(scope, function, args) {
            Some(gs) => self.resolve_generic_function(scope, function, &gs, args),
            None => self.resolve_normal_function(scope, function, args),
        }
    }

    pub fn resolve_function_generic_args<'a>(
        self: &'a Self,
        scope: &'a Scope,
        function: &String,
        args: &Vec<CType>,
    ) -> Option<Vec<CType>> {
        let mut scope_to_check: Option<&Scope> = Some(scope);
        let mut fs = Vec::new();
        while scope_to_check.is_some() {
            match scope_to_check {
                Some(s) => {
                    match s.functions.get(function) {
                        Some(funcs) => {
                            // Why is this okay but cloning funcs and then appending is not?
                            for f in funcs {
                                fs.push(f);
                            }
                        }
                        None => {}
                    }
                    // TODO: Types are internally referred to by their structural name, not by the name the
                    // user gives them, so a type constructor function needs to have a lookup done by type and
                    // then coerce into the constructor function name and then call it. We *should* just be
                    // able to use the user's name for the types, but this was undone for generic functions to
                    // work correctly. We should try to find a better solution than this function resolution
                    // hackery.
                    match self.resolve_type(s, function) {
                        Some(t) => {
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
                        None => {}
                    }
                    scope_to_check = match &s.parent {
                        Some(p) => match self.scopes_by_file.get(p) {
                            Some((_, _, s)) => Some(s),
                            None => None,
                        },
                        None => None,
                    };
                }
                None => {}
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
                        if f.args[0].1.degroup().to_strict_string(false)
                            != arg.degroup().to_strict_string(false)
                        {
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
                FnKind::Normal | FnKind::Bind(_) | FnKind::Derived => {
                    if args.len() != f.args.len() {
                        continue;
                    }
                    let mut args_match = true;
                    for (i, arg) in args.iter().enumerate() {
                        // This is pretty cheap, but for now, a "non-strict" string representation
                        // of the CTypes is how we'll match the args against each other. TODO: Do
                        // this without constructing a string to compare against each other.
                        if f.args[i].1.degroup().to_strict_string(false)
                            != arg.degroup().to_strict_string(false)
                        {
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
                    match CType::infer_generics(scope, g, &f.args, args) {
                        Ok(gs) => {
                            return Some(gs);
                        }
                        Err(_) => {
                            continue;
                        }
                    };
                }
            }
        }
        None
    }

    pub fn resolve_normal_function<'a>(
        self: &'a Self,
        scope: &'a Scope,
        function: &String,
        args: &Vec<CType>,
    ) -> Option<&Function> {
        // Tries to find the specified function within the portion of the program accessible from
        // the current scope (so first checking the current scope, then all imports, then the root
        // scope). It checks against the args array to find a match. TODO: Go beyond exact matching
        // Returns a reference to the function and the scope it came from.
        let mut scope_to_check: Option<&Scope> = Some(scope);
        let mut fs = Vec::new();
        while scope_to_check.is_some() {
            match scope_to_check {
                Some(s) => {
                    match s.functions.get(function) {
                        Some(funcs) => {
                            // Why is this okay but cloning funcs and then appending is not?
                            for f in funcs {
                                fs.push(f);
                            }
                        }
                        None => {}
                    }
                    // TODO: Types are internally referred to by their structural name, not by the name the
                    // user gives them, so a type constructor function needs to have a lookup done by type and
                    // then coerce into the constructor function name and then call it. We *should* just be
                    // able to use the user's name for the types, but this was undone for generic functions to
                    // work correctly. We should try to find a better solution than this function resolution
                    // hackery.
                    match self.resolve_type(s, function) {
                        Some(t) => {
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
                        None => {}
                    }
                    scope_to_check = match &s.parent {
                        Some(p) => match self.scopes_by_file.get(p) {
                            Some((_, _, s)) => Some(s),
                            None => None,
                        },
                        None => None,
                    };
                }
                None => {}
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
                        if f.args[0].1.degroup().to_strict_string(false)
                            != arg.degroup().to_strict_string(false)
                        {
                            args_match = false;
                            break;
                        }
                    }
                    if args_match {
                        return Some(f);
                    }
                }
                FnKind::Normal | FnKind::Bind(_) | FnKind::Derived => {
                    if args.len() != f.args.len() {
                        continue;
                    }
                    let mut args_match = true;
                    for (i, arg) in args.iter().enumerate() {
                        // This is pretty cheap, but for now, a "non-strict" string representation
                        // of the CTypes is how we'll match the args against each other. TODO: Do
                        // this without constructing a string to compare against each other.
                        if f.args[i].1.degroup().to_strict_string(false)
                            != arg.degroup().to_strict_string(false)
                        {
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
