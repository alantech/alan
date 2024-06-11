use std::fs::read_to_string;
use std::pin::Pin;

use super::CType;
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
        p = p.load("@root".to_string())?;
        // Load the entry file
        p = match p.load(entry_file) {
            Ok(p) => p,
            Err(e) => {
                // Somehow, trying to print this error can crash Rust!? Really not good.
                // Will need to figure out how to make these errors clearer to users.
                return Err(format!("{}", e).into());
            }
        };
        Ok(p)
    }

    pub fn load(self, path: String) -> Result<Program, Box<dyn std::error::Error>> {
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
