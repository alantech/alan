use ordered_hash_map::OrderedHashMap;

use super::CType;
use super::Const;
use super::Export;
use super::Function;
use super::Import;
use super::OperatorMapping;
use super::Program;
use super::TypeOperatorMapping;
use crate::parse;

#[derive(Clone, Debug)]
pub struct Scope {
    pub path: String,           // Now necessary since we reference by path name :/
    pub parent: Option<String>, // TODO: Figure out lifetimes and make this a real reference
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
    pub fn from_src(
        program: &mut Program,
        path: &String,
        src: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let txt = Box::pin(src);
        let txt_ptr: *const str = &**txt;
        // *How* would this move, anyways? But TODO: See if there's a way to handle this safely
        let ast = unsafe { parse::get_ast(&*txt_ptr)? };
        let s = Scope {
            path: path.clone(),
            parent: match program.scopes_by_file.get("@root") {
                None => None,
                Some(..) => Some("@root".to_string()),
            },
            imports: OrderedHashMap::new(),
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            operatormappings: OrderedHashMap::new(),
            typeoperatormappings: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        };
        // TODO: Figure out a better way to do this. The compiler is no longer threadsafe this way
        program.scopes_by_file.insert(path.clone(), (txt, ast, s));
        let (_, ast, s) = program.scopes_by_file.get_mut(path).unwrap();
        let ast_ptr: *const parse::Ln = &*ast;
        let s_ptr: *mut Scope = &mut *s;
        let ast = unsafe { &*ast_ptr };
        let mut s = unsafe { &mut *s_ptr };
        for i in ast.imports.iter() {
            Import::from_ast(program, path.clone(), s, i)?;
        }
        for (i, element) in ast.body.iter().enumerate() {
            match element {
                parse::RootElements::Types(t) => match CType::from_ast(s, program, t, false) {
                    Err(e) => Err(e),
                    Ok(_) => Ok(()),
                }?, // TODO: Make this match the rest?

                parse::RootElements::Functions(f) => Function::from_ast(s, program, f, false)?,
                parse::RootElements::ConstDeclaration(c) => Const::from_ast(s, program, c, false)?,
                parse::RootElements::OperatorMapping(o) => {
                    OperatorMapping::from_ast(s, program, o, false)?
                }
                parse::RootElements::TypeOperatorMapping(o) => {
                    TypeOperatorMapping::from_ast(s, program, o, false)?
                }
                parse::RootElements::Exports(e) => match &e.exportable {
                    parse::Exportable::Functions(f) => Function::from_ast(s, program, f, true)?,
                    parse::Exportable::ConstDeclaration(c) => Const::from_ast(s, program, c, true)?,
                    parse::Exportable::OperatorMapping(o) => {
                        OperatorMapping::from_ast(s, program, o, true)?
                    }
                    parse::Exportable::TypeOperatorMapping(o) => {
                        TypeOperatorMapping::from_ast(s, program, o, true)?
                    }
                    parse::Exportable::Types(t) => match CType::from_ast(s, program, t, true) {
                        Err(e) => Err(e),
                        Ok(_) => Ok(()),
                    }?, // TODO: Make this match the rest?
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
                                g @ ("Group" | "Array" | "Fail" | "Neg" | "Len" | "Size" | "FileStr"
                                | "Env" | "EnvExists" | "Not") => CType::from_generic(&mut s, g, 1),
                                g @ ("Function" | "Tuple" | "Field" | "Either" | "Buffer" | "Add"
                                | "Sub" | "Mul" | "Div" | "Mod" | "Pow" | "Min" | "Max" | "If" | "And" | "Or"
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
        Ok(())
    }

    pub fn temp_child(&self) -> Scope {
        let path = format!("{}/temp_child", self.path);
        Scope {
            path: path,
            parent: Some(self.path.clone()),
            imports: OrderedHashMap::new(),
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            operatormappings: OrderedHashMap::new(),
            typeoperatormappings: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        }
    }

    pub fn child(&self, program: &mut Program) -> Scope {
        let path = format!("{}/child", self.path);
        let s = Scope {
            path: path.clone(),
            parent: Some(self.path.clone()),
            imports: OrderedHashMap::new(),
            types: OrderedHashMap::new(),
            consts: OrderedHashMap::new(),
            functions: OrderedHashMap::new(),
            operatormappings: OrderedHashMap::new(),
            typeoperatormappings: OrderedHashMap::new(),
            exports: OrderedHashMap::new(),
        };
        let txt = Box::pin("".to_string());
        let txt_ptr: *const str = &**txt;
        // *How* would this move, anyways? But TODO: See if there's a way to handle this safely
        let ast = unsafe { parse::get_ast(&*txt_ptr).unwrap() };
        program.scopes_by_file.insert(path, (txt, ast, s.clone()));
        s
    }

    pub fn merge_child_functions(&mut self, child: &mut Scope) {
        for (name, fs) in child.functions.drain() {
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
}
