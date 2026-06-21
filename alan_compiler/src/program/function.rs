use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::ctype::{withtypeoperatorslist_to_ctype, CType};
use super::microstatement::{statement_to_microstatements, Microstatement};
use super::scope::merge;
use super::ArgKind;

use super::Export;
use super::FnKind;
use super::Scope;
use crate::parse;

thread_local! {
    // Tracks the set of lazy functions (by `Arc` identity) whose bodies are currently being
    // resolved, so `resolve_lazy` can detect and reject unsupported recursive definitions instead
    // of looping forever.
    static RESOLVING: RefCell<HashSet<usize>> = RefCell::new(HashSet::new());

    // A monotonically increasing counter assigning each source-defined function a "definition
    // order" index reflecting where it appears during evaluation of the source. Alan's function
    // dispatch is order-sensitive: a function body only "sees" definitions that appear before it,
    // and the most-recent matching definition wins. We must preserve this under lazy loading.
    static DEF_COUNTER: Cell<usize> = const { Cell::new(0) };
    // Maps a source-defined function (by `Arc` identity) to its definition-order index.
    static FN_DEF_INDEX: RefCell<HashMap<usize, usize>> = RefCell::new(HashMap::new());
    // A stack of "visibility boundaries" -- the definition index of the (non-generic) function
    // whose body is currently being resolved. While a boundary is in effect, function lookups only
    // consider definitions that appear strictly before it, mirroring eager, in-order resolution.
    static VISIBILITY_STACK: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

fn fn_ptr_key(f: &Arc<Function>) -> usize {
    Arc::as_ptr(f) as *const () as usize
}

/// Assigns the next definition-order index to a freshly-created source-defined function and records
/// it, returning the index. Functions created by other means (generic realizations, derived
/// constructors/accessors, etc.) are intentionally *not* recorded -- they have no source position
/// and are treated as always-visible.
fn record_def_index(f: &Arc<Function>) {
    let idx = DEF_COUNTER.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    });
    FN_DEF_INDEX.with(|m| m.borrow_mut().insert(fn_ptr_key(f), idx));
}

/// Looks up a function's definition-order index, if it has one.
pub fn def_index_of(f: &Arc<Function>) -> Option<usize> {
    FN_DEF_INDEX.with(|m| m.borrow().get(&fn_ptr_key(f)).copied())
}

/// Records an explicit definition-order index for a function. Used when a lazily-resolved function
/// is replaced by its fully-resolved equivalent (a new `Arc`) that persists in the scope -- the
/// replacement must inherit the original's index so dispatch ordering stays correct.
pub fn set_def_index(f: &Arc<Function>, idx: usize) {
    FN_DEF_INDEX.with(|m| m.borrow_mut().insert(fn_ptr_key(f), idx));
}

/// Whether the given function is visible under the current visibility boundary (see
/// `VISIBILITY_STACK`). With no boundary in effect (e.g. resolving user code, where the whole root
/// scope is already loaded) everything is visible. Functions without a definition index (generic
/// realizations, derived functions, etc.) are always visible.
pub fn is_visible(f: &Arc<Function>) -> bool {
    VISIBILITY_STACK.with(|s| match s.borrow().last() {
        None => true,
        Some(&boundary) => match def_index_of(f) {
            Some(idx) => idx < boundary,
            None => true,
        },
    })
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub typen: Arc<CType>,
    pub microstatements: Vec<Microstatement>,
    pub kind: FnKind,
    pub origin_scope_path: String,
    // When `Some`, this function's body has not yet been resolved into microstatements. It holds
    // the parsed statements so the body can be resolved on-demand (lazily) the first time the
    // function is actually referenced during codegen-reachable resolution. This lets us skip
    // resolving the bodies of the thousands of standard-library functions that a given program
    // never calls. `None` means the function is fully resolved (the normal, eager state).
    pub lazy_body: Option<Vec<parse::Statement>>,
}

pub fn type_to_args(t: Arc<CType>) -> Vec<(String, ArgKind, Arc<CType>)> {
    match &*t {
        CType::Function(i, _) => {
            let mut args = Vec::new();
            match &**i {
                CType::Tuple(ts, _) => {
                    for (i, t) in ts.iter().enumerate() {
                        args.push(match &**t {
                            CType::Field(argname, t) => match &**t {
                                CType::Own(t) => (argname.clone(), ArgKind::Own, t.clone()),
                                CType::Deref(t) => (argname.clone(), ArgKind::Deref, t.clone()),
                                CType::Mut(t) => (argname.clone(), ArgKind::Mut, t.clone()),
                                _otherwise => (argname.clone(), ArgKind::Ref, t.clone()),
                            },
                            CType::Own(t) => (format!("arg{i}"), ArgKind::Own, t.clone()),
                            CType::Deref(t) => (format!("arg{i}"), ArgKind::Deref, t.clone()),
                            CType::Mut(t) => (format!("arg{i}"), ArgKind::Mut, t.clone()),
                            _otherwise => (format!("arg{i}"), ArgKind::Ref, t.clone()),
                        });
                    }
                }
                CType::Field(argname, t) => match &**t {
                    CType::Own(t) => args.push((argname.clone(), ArgKind::Own, t.clone())),
                    CType::Deref(t) => args.push((argname.clone(), ArgKind::Deref, t.clone())),
                    CType::Mut(t) => args.push((argname.clone(), ArgKind::Mut, t.clone())),
                    _otherwise => args.push((argname.clone(), ArgKind::Ref, t.clone())),
                },
                CType::Void | CType::DerivedVoid(..) => { /* Do nothing */ }
                CType::Own(t) => args.push(("arg0".to_string(), ArgKind::Own, t.clone())),
                CType::Deref(t) => args.push(("arg0".to_string(), ArgKind::Deref, t.clone())),
                CType::Mut(t) => args.push(("arg0".to_string(), ArgKind::Mut, t.clone())),
                _otherwise => args.push(("arg0".to_string(), ArgKind::Ref, i.clone())),
            }
            args
        }
        CType::Tuple(ts, _) => {
            let mut args = Vec::new();
            for (i, t) in ts.iter().enumerate() {
                args.push(match &**t {
                    CType::Field(argname, t) => match &**t {
                        CType::Own(t) => (argname.clone(), ArgKind::Own, t.clone()),
                        CType::Deref(t) => (argname.clone(), ArgKind::Deref, t.clone()),
                        CType::Mut(t) => (argname.clone(), ArgKind::Mut, t.clone()),
                        _otherwise => (argname.clone(), ArgKind::Ref, t.clone()),
                    },
                    CType::Own(t) => (format!("arg{i}"), ArgKind::Own, t.clone()),
                    CType::Deref(t) => (format!("arg{i}"), ArgKind::Deref, t.clone()),
                    CType::Mut(t) => (format!("arg{i}"), ArgKind::Mut, t.clone()),
                    _otherwise => (format!("arg{i}"), ArgKind::Ref, t.clone()),
                });
            }
            args
        }
        CType::Field(argname, t) => match &**t {
            CType::Own(t) => vec![(argname.clone(), ArgKind::Own, t.clone())],
            CType::Deref(t) => vec![(argname.clone(), ArgKind::Deref, t.clone())],
            CType::Mut(t) => vec![(argname.clone(), ArgKind::Mut, t.clone())],
            _otherwise => vec![(argname.clone(), ArgKind::Ref, t.clone())],
        },
        CType::Void | CType::DerivedVoid(..) => Vec::new(),
        CType::Own(t) => vec![("arg0".to_string(), ArgKind::Own, t.clone())],
        CType::Deref(t) => vec![("arg0".to_string(), ArgKind::Deref, t.clone())],
        CType::Mut(t) => vec![("arg0".to_string(), ArgKind::Mut, t.clone())],
        _ => vec![("arg0".to_string(), ArgKind::Ref, t.clone())],
    }
}

pub fn type_to_rettype(t: Arc<CType>) -> Arc<CType> {
    match &*t {
        CType::Function(_, o) => o.clone(),
        _ => Arc::new(CType::Void),
    }
}

fn is_promise_head(t: Arc<CType>) -> bool {
    let mut t = t.degroup();
    while matches!(&*t, CType::Type(..)) {
        t = match &*t {
            CType::Type(_, inner) => inner.clone().degroup(),
            _ => unreachable!(),
        };
    }
    matches!(&*t, CType::Promise(_))
}

fn microstatement_awaits(ms: &Microstatement) -> bool {
    match ms {
        Microstatement::Assignment { value, .. } => microstatement_awaits(value),
        Microstatement::FnCall { function, args } => {
            is_promise_head(function.rettype()) || args.iter().any(microstatement_awaits)
        }
        Microstatement::VarCall { typen, args, .. } => {
            is_promise_head(typen.clone()) || args.iter().any(microstatement_awaits)
        }
        Microstatement::Array { vals, .. } => vals.iter().any(microstatement_awaits),
        Microstatement::Return { value } => value
            .as_deref()
            .is_some_and(microstatement_awaits),
        Microstatement::NativeCall { args, .. } => args.iter().any(microstatement_awaits),
        // Deliberately do not descend into nested closures.
        Microstatement::Closure { .. } | Microstatement::Arg { .. } | Microstatement::Value { .. } => {
            false
        }
    }
}

fn body_awaits(microstatements: &[Microstatement]) -> bool {
    microstatements.iter().any(microstatement_awaits)
}

pub fn args_and_rettype_to_type(
    args: Vec<(String, ArgKind, Arc<CType>)>,
    rettype: Arc<CType>,
) -> Arc<CType> {
    Arc::new(CType::Function(
        Arc::new(if args.is_empty() {
            CType::Void
        } else {
            CType::Tuple(
                args.into_iter()
                    .map(|(n, k, t)| {
                        Arc::new(CType::Field(
                            n,
                            match k {
                                ArgKind::Mut => Arc::new(CType::Mut(t)),
                                ArgKind::Ref => t,
                                ArgKind::Own | ArgKind::Deref => CType::fail(
                                    "Somehow got an Own or Deref for a normal Alan function",
                                ),
                            },
                        ))
                    })
                    .collect::<Vec<Arc<CType>>>(),
                Vec::new(),
            )
        }),
        rettype,
    ))
}

impl Function {
    pub fn args(&self) -> Vec<(String, ArgKind, Arc<CType>)> {
        type_to_args(self.typen.clone())
    }

    pub fn rettype(&self) -> Arc<CType> {
        type_to_rettype(self.typen.clone())
    }

    pub fn from_ast<'a>(
        scope: Scope<'a>,
        function_ast: &parse::Functions,
        is_export: bool,
    ) -> Result<Scope<'a>, Box<dyn std::error::Error>> {
        // In the top-level of a file, all functions *must* be named
        let name = match &function_ast.optname {
            Some(name) => name.clone(),
            None => {
                return Err("Top-level function without a name!".into());
            }
        };
        Function::from_ast_with_name(scope, function_ast, is_export, name)
    }

    pub fn from_ast_with_name<'a>(
        mut scope: Scope<'a>,
        function_ast: &parse::Functions,
        is_export: bool,
        name: String,
    ) -> Result<Scope<'a>, Box<dyn std::error::Error>> {
        if let Some(generics) = &function_ast.opttypegenerics {
            // We are going to conditionally compile this type declaration. If the we get true, we
            // continue, if we get false, we don't compile and return a Fail type that isn't added
            // to the scope to cause compilation to crash *if* something tries to use this, and if
            // we don't get a boolean at all or we get multiple inner values in the generic call,
            // we bail out immediately because of a syntax error.
            let generic_call = withtypeoperatorslist_to_ctype(&generics.typecalllist, &scope)?;
            match &*generic_call {
                CType::Bool(b) => match b {
                    false => return Ok(scope),
                    true => { /* Do nothing */ }
                },
                CType::Type(_, c) => match &**c {
                    CType::Bool(b) => match b {
                        false => return Ok(scope),
                        true => { /* Do nothing */ }
                    },
                    _ => {
                        return Err(format!(
                        "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                        name,
                        generics.to_string()
                    )
                        .into())
                    }
                },
                _ => {
                    return Err(format!(
                    "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                    name,
                    generics.to_string()
                )
                    .into())
                }
            }
        }
        if let parse::FullFunctionBody::DecOnly(_) = &function_ast.fullfunctionbody {
            if let Some(fntype) = &function_ast.opttype {
                if let Some(g) = &function_ast.optgenerics {
                    let mut generics = Vec::new();
                    // TODO: The semantics in here are different, so we may want to make a new parser
                    // type here, but for now, just do some manual parsing and blow up if we encounter
                    // something unexpected
                    let mut i = 0;
                    while i < g.typecalllist.len() {
                        match (
                            g.typecalllist.get(i),
                            g.typecalllist.get(i + 1),
                            g.typecalllist.get(i + 2),
                            g.typecalllist.get(i + 3),
                        ) {
                            (Some(t1), Some(t2), Some(t3), Some(t4))
                                if t2.to_string().trim() == ":" && t4.to_string().trim() == "," =>
                            {
                                // TODO: This should be an interface type, instead
                                generics.push((
                                    t1.to_string().trim().to_string(),
                                    Arc::new(CType::Infer(
                                        t1.to_string().trim().to_string(),
                                        t3.to_string().trim().to_string(),
                                    )),
                                ));
                                i += 4;
                            }
                            (Some(t1), Some(t2), Some(t3), None)
                                if t2.to_string().trim() == ":" =>
                            {
                                // TODO: This should be an interface type, instead
                                generics.push((
                                    t1.to_string().trim().to_string(),
                                    Arc::new(CType::Infer(
                                        t1.to_string().trim().to_string(),
                                        t3.to_string().trim().to_string(),
                                    )),
                                ));
                                i += 3; // This should exit the loop
                            }
                            (Some(t1), Some(t2), _, _) if t2.to_string().trim() == "," => {
                                // TODO: This should be an interface type, instead
                                generics.push((
                                    t1.to_string().trim().to_string(),
                                    Arc::new(CType::Infer(
                                        t1.to_string().trim().to_string(),
                                        "Any".to_string(),
                                    )),
                                ));
                                i += 2;
                            }
                            (Some(t1), None, None, None) => {
                                // TODO: This should be an interface type, instead
                                generics.push((
                                    t1.to_string().trim().to_string(),
                                    Arc::new(CType::Infer(
                                        t1.to_string().trim().to_string(),
                                        "Any".to_string(),
                                    )),
                                ));
                                i += 1;
                            }
                            (a, b, c, d) => {
                                // Any other patterns are invalid
                                return Err(format!("Unexpected generic type definition, failure to parse at {a:?} {b:?} {c:?} {d:?}").into());
                            }
                        }
                    }
                    let mut temp_scope = scope.child();
                    // This lets us partially resolve the function argument and return types
                    for g in &generics {
                        temp_scope = CType::from_ctype(temp_scope, g.0.clone(), g.1.clone());
                    }
                    let ctype = withtypeoperatorslist_to_ctype(fntype, &temp_scope)?;
                    // If the `ctype` is a Function type, we have both the input and output defined.
                    // If the `ctype` is a From type, we re-eval it as an `Import` type with the
                    // function name declaring what's being imported. If it's an `Import` type we
                    // grab the specified function to import. If it's any other type, we presume
                    // it's only the input type defined
                    let (kind, input_type, rettype) = match &*ctype {
                        CType::From(_) => CType::fail(
                            "TODO: Support importing a function from an Alan dependency.",
                        ),
                        CType::Import(..) => CType::fail(
                            "TODO: Support importing a function from an Alan dependency.",
                        ),
                        CType::Call(n, f) => match &**n {
                            CType::TString(s) => {
                                match &**f {
                                    CType::Function(i, o) => (FnKind::BoundGeneric(generics, s.clone()), i.clone(), o.clone()),
                                    _otherwise => (FnKind::BoundGeneric(generics, s.clone()), f.clone(), Arc::new(CType::Infer("unknown".to_string(), "unknown".to_string()))),
                                }
                            }
                            CType::Import(n, d) => {
                                match &**n {
                                    CType::TString(s) => {
                                        match &**f {
                                            CType::Function(i, o) => (FnKind::ExternalGeneric(generics, s.clone(), d.clone()), i.clone(), o.clone()),
                                            _otherwise => (FnKind::ExternalGeneric(generics, s.clone(), d.clone()), f.clone(), Arc::new(CType::Infer("unknown".to_string(), "unknown".to_string()))),
                                        }
                                    }
                                    _ => CType::fail("TODO: Support more than bare function imports for generic function binding"),
                                }
                            }
                            _ => CType::fail("TODO: Support more than bare function calls for generic function binding"),
                        }
                        otherwise => CType::fail(&format!(
                            "A declaration-only function must be a binding Call{{N, F}}: {otherwise:?}"
                        )),
                    };
                    // In case there were any created functions (eg constructor or accessor
                    // functions) in that path, we need to merge the child's functions back up
                    // TODO: Why can't I box this up into a function?
                    merge!(scope, temp_scope);
                    let degrouped_input = input_type.degroup();
                    let function = Arc::new(Function {
                        name,
                        typen: Arc::new(CType::Function(degrouped_input, rettype)),
                        microstatements: Vec::new(),
                        kind,
                        origin_scope_path: scope.path.clone(),
                        lazy_body: None,
                    });
                    record_def_index(&function);
                    if is_export {
                        scope
                            .exports
                            .insert(function.name.clone(), Export::Function);
                    }
                    if scope.functions.contains_key(&function.name) {
                        let func_vec = scope.functions.get_mut(&function.name).unwrap();
                        func_vec.insert(0, function);
                    } else {
                        scope
                            .functions
                            .insert(function.name.clone(), vec![function]);
                    }
                } else {
                    let ctype = withtypeoperatorslist_to_ctype(fntype, &scope)?;
                    // Converts a From type into an Import type so we can pull the correct function
                    // from the specified dependency.
                    let ctype = match &*ctype {
                        CType::From(t) => {
                            CType::import(Arc::new(CType::TString(name.clone())), t.clone())
                        }
                        _ => ctype,
                    };
                    if is_export {
                        scope.exports.insert(name.clone(), Export::Function);
                    }
                    if scope.functions.contains_key(&name) {
                        let fns = ctype.to_functions(name.clone(), &scope).1;
                        scope.functions.get_mut(&name).unwrap().splice(0..0, fns);
                    } else {
                        scope
                            .functions
                            .insert(name.clone(), ctype.to_functions(name.clone(), &scope).1);
                    }
                }
                return Ok(scope);
            } else {
                return Err("Declaration-only functions must have a declared function type".into());
            }
        }
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
            parse::FullFunctionBody::DecOnly(_) => unreachable!(),
        };
        let kind = match (&function_ast.fullfunctionbody, &function_ast.optgenerics) {
            (parse::FullFunctionBody::DecOnly(_), _) => unreachable!(),
            (_, Some(g)) => {
                let mut generics = Vec::new();
                // TODO: The semantics in here are different, so we may want to make a new parser
                // type here, but for now, just do some manual parsing and blow up if we encounter
                // something unexpected
                let mut i = 0;
                while i < g.typecalllist.len() {
                    match (
                        g.typecalllist.get(i),
                        g.typecalllist.get(i + 1),
                        g.typecalllist.get(i + 2),
                        g.typecalllist.get(i + 3),
                    ) {
                        (Some(t1), Some(t2), Some(t3), Some(t4))
                            if t2.to_string().trim() == ":" && t4.to_string().trim() == "," =>
                        {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                Arc::new(CType::Infer(
                                    t1.to_string().trim().to_string(),
                                    t3.to_string().trim().to_string(),
                                )),
                            ));
                            i += 4;
                        }
                        (Some(t1), Some(t2), Some(t3), None) if t2.to_string().trim() == ":" => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                Arc::new(CType::Infer(
                                    t1.to_string().trim().to_string(),
                                    t3.to_string().trim().to_string(),
                                )),
                            ));
                            i += 3; // This should exit the loop
                        }
                        (Some(t1), Some(t2), _, _) if t2.to_string().trim() == "," => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                Arc::new(CType::Infer(
                                    t1.to_string().trim().to_string(),
                                    "Any".to_string(),
                                )),
                            ));
                            i += 2;
                        }
                        (Some(t1), None, None, None) => {
                            // TODO: This should be an interface type, instead
                            generics.push((
                                t1.to_string().trim().to_string(),
                                Arc::new(CType::Infer(
                                    t1.to_string().trim().to_string(),
                                    "Any".to_string(),
                                )),
                            ));
                            i += 1;
                        }
                        (a, b, c, d) => {
                            // Any other patterns are invalid
                            return Err(format!("Unexpected generic type definition, failure to parse at {a:?} {b:?} {c:?} {d:?}").into());
                        }
                    }
                }
                FnKind::Generic(generics, statements.clone())
            }
            _ => FnKind::Normal,
        };
        let mut typen = match &function_ast.opttype {
            None => Ok::<Arc<CType>, Box<dyn std::error::Error>>(Arc::new(CType::Function(
                Arc::new(CType::Void),
                Arc::new(CType::Infer("unknown".to_string(), "unknown".to_string())),
            ))),
            Some(typeassignable) if typeassignable.is_empty() => Ok(Arc::new(CType::Function(
                Arc::new(CType::Void),
                Arc::new(CType::Infer("unknown".to_string(), "unknown".to_string())),
            ))),
            Some(typeassignable) => match &kind {
                FnKind::Generic(gs, _) | FnKind::BoundGeneric(gs, _) => {
                    let mut temp_scope = scope.child();
                    // This lets us partially resolve the function argument and return types
                    for g in gs {
                        temp_scope = CType::from_ctype(temp_scope, g.0.clone(), g.1.clone());
                    }
                    let ctype = withtypeoperatorslist_to_ctype(typeassignable, &temp_scope)?;
                    // If the `ctype` is a Function type, we have both the input and output defined. If
                    // it's any other type, we presume it's only the input type defined
                    let (input_type, output_type) = match &*ctype {
                        CType::Function(i, o) => (i.clone(), o.clone()),
                        _otherwise => (
                            ctype,
                            Arc::new(CType::Infer("unknown".to_string(), "unknown".to_string())),
                        ),
                    };
                    // In case there were any created functions (eg constructor or accessor
                    // functions) in that path, we need to merge the child's functions back up
                    merge!(scope, temp_scope);
                    let degrouped_input = input_type.degroup();
                    Ok(Arc::new(CType::Function(degrouped_input, output_type)))
                }
                _ => {
                    let ctype = withtypeoperatorslist_to_ctype(typeassignable, &scope)?;
                    // If the `ctype` is a Function type, we have both the input and output defined. If
                    // it's any other type, we presume it's only the input type defined

                    let (input_type, output_type) = match &*ctype {
                        CType::Function(i, o) => (i.clone(), o.clone()),
                        _otherwise => (
                            ctype,
                            Arc::new(CType::Infer("unknown".to_string(), "unknown".to_string())),
                        ),
                    };
                    let degrouped_input = input_type.degroup();
                    Ok(Arc::new(CType::Function(degrouped_input, output_type)))
                }
            },
        }?;
        // We defer resolving the bodies of non-generic functions in the *root* scope. The root
        // scope holds thousands of standard-library functions, the vast majority of which any
        // given program never calls. Resolving all of their bodies up-front dominates compile
        // latency, so instead we stash the parsed statements in `lazy_body` and resolve them
        // on-demand the first time the function is actually referenced (see
        // `Scope::resolve_function`). Functions in user scopes (including `main`) are still
        // resolved eagerly so that codegen, which only has immutable access to the scope, always
        // sees fully-resolved functions; their resolution transitively forces resolution of any
        // root functions they reach.
        let defer = function_ast.optgenerics.is_none() && scope.path == "@root";
        let microstatements = {
            let mut ms = Vec::new();
            for (name, kind, typen) in type_to_args(typen.clone()) {
                ms.push(Microstatement::Arg { name, kind, typen });
            }
            // We can't generate the rest of the microstatements while the generic function is
            // still generic, and we skip it entirely for deferred (lazy) root functions.
            if function_ast.optgenerics.is_none() && !defer {
                for statement in &statements {
                    // The construction of microstatements in non-generic functions will never
                    // actually use the provided function for scope resolution, so we just give it
                    // a dummy function to work with.
                    let res = statement_to_microstatements(statement, None, scope, ms)?;
                    scope = res.0;
                    ms = res.1;
                }
            }
            ms
        };
        // Determine the actual return type of the function and check if it matches the specified
        // return type (or update that return type if it's to be inferred. Skipped for deferred
        // functions, whose return type is inferred when their body is later resolved.
        if defer {
            // Nothing to infer yet; the body is resolved lazily.
        } else if let Some(ms) = microstatements.last() {
            if let Microstatement::Arg { .. } = ms {
                // Don't do anything in this path, this is probably a derived function
            } else {
                let current_rettype = type_to_rettype(typen.clone());
                let mut actual_rettype = match ms {
                    Microstatement::Return { value: Some(v) } => v.get_type(),
                    _ => Arc::new(CType::Void),
                };
                if body_awaits(&microstatements) && !is_promise_head(actual_rettype.clone()) {
                    actual_rettype = CType::promise(actual_rettype);
                }
                if let CType::Infer(..) = &*current_rettype {
                    // We're definitely replacing with the inferred type
                    let input_type = match &*typen {
                        CType::Function(i, _) => i.clone(),
                        _ => Arc::new(CType::Void),
                    };
                    typen = Arc::new(CType::Function(input_type, actual_rettype));
                } else if current_rettype.clone().to_strict_string(false)
                    != actual_rettype.clone().to_strict_string(false)
                {
                    CType::fail(&format!(
                        "Function {} specified to return {} but actually returns {}",
                        name,
                        current_rettype.to_strict_string(false),
                        actual_rettype.to_strict_string(false),
                    ));
                } else {
                    // Do nothing, they're the same
                }
            }
        }
        // TODO: This is getting duplicated in a few different places. The CType creation
        // should probably centralize creating these type names and constructor functions
        // for us rather than this hackiness. Only adding the hackery to the output_type
        // because that's all I need, and the input type would be much more convoluted.
        match &*typen {
            CType::Function(i, o) => {
                match &**o {
                    CType::Void | CType::DerivedVoid(..) => { /* Do nothing */ }
                    CType::Infer(t, _)
                        if t == "unknown" && function_ast.optgenerics.is_none() && !defer =>
                    {
                        CType::fail(&format!(
                            "The return type for {}({}) could not be inferred.",
                            name,
                            i.clone().to_strict_string(false)
                        ));
                    }
                    CType::Infer(..) => { /* Do nothing */ }
                    _otherwise => {
                        let name = o.clone().to_callable_string();
                        // Don't recreate the exact same thing. It only causes pain
                        if scope.resolve_type(&name).is_none() {
                            scope = CType::from_ctype(scope, name, o.clone());
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        let function = Arc::new(Function {
            name,
            typen,
            microstatements,
            kind,
            origin_scope_path: scope.path.clone(),
            lazy_body: if defer { Some(statements) } else { None },
        });
        record_def_index(&function);
        if is_export {
            scope
                .exports
                .insert(function.name.clone(), Export::Function);
        }
        if scope.functions.contains_key(&function.name) {
            let func_vec = scope.functions.get_mut(&function.name).unwrap();
            func_vec.insert(0, function);
        } else {
            scope
                .functions
                .insert(function.name.clone(), vec![function]);
        }
        Ok(scope)
    }

    // Resolves the deferred body of a lazily-loaded function (see the `lazy_body` field and
    // `from_ast_with_name`). It mirrors the body-resolution logic of `from_ast_with_name` for
    // non-generic functions: it builds the body microstatements (on top of the already-present
    // `Arg` microstatements), infers/validates the return type, and ensures the return type's
    // constructor exists in the scope. It returns the fully-resolved function (with `lazy_body`
    // set to `None`) along with the updated scope (which may have gained realized generic
    // functions/types reached transitively by the body).
    pub fn resolve_lazy<'a>(
        mut scope: Scope<'a>,
        func: Arc<Function>,
    ) -> Result<(Scope<'a>, Arc<Function>), Box<dyn std::error::Error>> {
        let statements = match &func.lazy_body {
            Some(s) => s,
            None => return Ok((scope, func)),
        };
        // Guard against infinite recursion. Non-generic functions cannot legally recurse by name
        // (the eager path never had the function in scope while building its own body), so if we
        // re-enter resolution for the same function it indicates an unsupported recursive
        // definition rather than a legitimate case. Bail out cleanly so we don't hang the
        // compiler.
        let key = Arc::as_ptr(&func) as *const () as usize;
        if RESOLVING.with(|r| r.borrow().contains(&key)) {
            return Err(format!(
                "Function {} appears to be recursively defined, which is not supported.",
                func.name
            )
            .into());
        }
        RESOLVING.with(|r| {
            r.borrow_mut().insert(key);
        });
        // While resolving this (non-generic) function's body, restrict visibility to definitions
        // that appear before it -- mirroring Alan's order-sensitive dispatch. Generic functions are
        // *not* resolved through this path; they are realized at their call site and so inherit
        // whatever boundary is already in effect there.
        let boundary = def_index_of(&func);
        if let Some(b) = boundary {
            VISIBILITY_STACK.with(|s| s.borrow_mut().push(b));
        }
        let result = (|| {
            let mut typen = func.typen.clone();
            let mut ms = func.microstatements.clone();
            for statement in statements {
                let res = statement_to_microstatements(statement, None, scope, ms)?;
                scope = res.0;
                ms = res.1;
            }
            // Determine the actual return type of the function and check it against any declared
            // return type (or infer it if it was left to be inferred).
            if let Some(last) = ms.last() {
                if let Microstatement::Arg { .. } = last {
                    // Don't do anything in this path, this is probably a derived function
                } else {
                    let current_rettype = type_to_rettype(typen.clone());
                    let mut actual_rettype = match last {
                        Microstatement::Return { value: Some(v) } => v.get_type(),
                        _ => Arc::new(CType::Void),
                    };
                    if body_awaits(&ms) && !is_promise_head(actual_rettype.clone()) {
                        actual_rettype = CType::promise(actual_rettype);
                    }
                    if let CType::Infer(..) = &*current_rettype {
                        let input_type = match &*typen {
                            CType::Function(i, _) => i.clone(),
                            _ => Arc::new(CType::Void),
                        };
                        typen = Arc::new(CType::Function(input_type, actual_rettype));
                    } else if current_rettype.clone().to_strict_string(false)
                        != actual_rettype.clone().to_strict_string(false)
                    {
                        CType::fail(&format!(
                            "Function {} specified to return {} but actually returns {}",
                            func.name,
                            current_rettype.to_strict_string(false),
                            actual_rettype.to_strict_string(false),
                        ));
                    }
                }
            }
            // Ensure the return type's constructor exists in the scope.
            match &*typen {
                CType::Function(i, o) => match &**o {
                    CType::Void | CType::DerivedVoid(..) => { /* Do nothing */ }
                    CType::Infer(t, _) if t == "unknown" => {
                        CType::fail(&format!(
                            "The return type for {}({}) could not be inferred.",
                            func.name,
                            i.clone().to_strict_string(false)
                        ));
                    }
                    CType::Infer(..) => { /* Do nothing */ }
                    _otherwise => {
                        let n = o.clone().to_callable_string();
                        if scope.resolve_type(&n).is_none() {
                            scope = CType::from_ctype(scope, n, o.clone());
                        }
                    }
                },
                _ => unreachable!(),
            }
            let resolved = Arc::new(Function {
                name: func.name.clone(),
                typen,
                microstatements: ms,
                kind: func.kind.clone(),
                origin_scope_path: func.origin_scope_path.clone(),
                lazy_body: None,
            });
            Ok((scope, resolved))
        })();
        if boundary.is_some() {
            VISIBILITY_STACK.with(|s| {
                s.borrow_mut().pop();
            });
        }
        RESOLVING.with(|r| {
            r.borrow_mut().remove(&key);
        });
        result
    }

    pub fn from_generic_function<'a>(
        mut scope: Scope<'a>,
        generic_function: &Function,
        generic_types: Vec<Arc<CType>>,
    ) -> Result<(Scope<'a>, Arc<Function>), Box<dyn std::error::Error>> {
        match &generic_function.kind {
            FnKind::Normal
            | FnKind::External(_)
            | FnKind::Bind(_)
            | FnKind::ExternalBind(_, _)
            | FnKind::Derived
            | FnKind::DerivedVariadic
            | FnKind::Static
            | FnKind::CfnRealized(_) => {
                Err("Should be impossible. Attempted to realize a non-generic function".into())
            }
            FnKind::BoundGeneric(gen_args, generic_fn_string)
            | FnKind::ExternalGeneric(gen_args, generic_fn_string, _) => {
                let arg_strs = generic_types
                    .iter()
                    .map(|a| a.clone().to_string())
                    .collect::<Vec<String>>();
                let mut bind_str = generic_fn_string.clone();
                for (i, arg_str) in arg_strs.iter().enumerate() {
                    let gen_str = &gen_args[i].0;
                    bind_str = bind_str.replace(gen_str, arg_str);
                }
                let kind = match &generic_function.kind {
                    FnKind::BoundGeneric(..) => FnKind::Bind(bind_str),
                    FnKind::ExternalGeneric(_, _, d) => FnKind::ExternalBind(bind_str, d.clone()),
                    _ => unreachable!(),
                };
                let args = generic_function
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
                // Make sure all argument types exist within the generic function call scope
                for (_, _, arg) in &args {
                    scope = CType::from_ctype(scope, arg.clone().to_callable_string(), arg.clone());
                }
                let mut rettype = {
                    let mut a = generic_function.rettype().clone();
                    for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                        a = a.swap_subtype(o.clone(), n.clone());
                    }
                    a
                };
                let microstatements = {
                    let mut ms = Vec::new();
                    for (name, kind, typen) in &args {
                        ms.push(Microstatement::Arg {
                            name: name.clone(),
                            kind: kind.clone(),
                            typen: typen.clone(),
                        });
                    }
                    ms
                };
                // Determine the actual return type of the function and check if it matches the specified
                // return type (or update that return type if it's to be inferred
                if let Some(ms) = microstatements.last() {
                    if let Microstatement::Arg { .. } = ms {
                        // Don't do anything in this path, this is probably a derived function
                    } else {
                        let mut actual_rettype = match ms {
                            Microstatement::Return { value: Some(v) } => v.get_type(),
                            _ => Arc::new(CType::Void),
                        };
                        if body_awaits(&microstatements) && !is_promise_head(actual_rettype.clone()) {
                            actual_rettype = CType::promise(actual_rettype);
                        }
                        if let CType::Infer(..) = &*rettype {
                            rettype = actual_rettype;
                        } else if rettype.clone().to_strict_string(false)
                            != actual_rettype.clone().to_strict_string(false)
                        {
                            CType::fail(&format!(
                                "Function {} specified to return {} but actually returns {}",
                                generic_function.name,
                                rettype.to_strict_string(false),
                                actual_rettype.to_strict_string(false),
                            ));
                        } else {
                            // Do nothing, they're the same
                        }
                    }
                }
                let name = format!(
                    "{}_{}",
                    generic_function.name,
                    generic_types
                        .iter()
                        .map(|t| t.clone().to_callable_string())
                        .collect::<Vec<String>>()
                        .join("_")
                ); // Really bad
                let f = Arc::new(Function {
                    name,
                    // TODO: Can I eliminate this indirection?
                    typen: args_and_rettype_to_type(args, rettype),
                    microstatements,
                    kind,
                    origin_scope_path: scope.path.clone(),
                    lazy_body: None,
                });
                if scope.functions.contains_key(&f.name) {
                    let func_vec = scope.functions.get_mut(&f.name).unwrap();
                    func_vec.insert(0, f.clone());
                } else {
                    scope.functions.insert(f.name.clone(), vec![f.clone()]);
                }
                let res = match scope.functions.get(&f.name) {
                    None => Err("This should be impossible. Cannot get the function we just added to the scope"),
                    Some(fs) => Ok(fs.first().unwrap().clone()), // We know it's the first one
                                                                // because we just put it there
                }?;
                Ok((scope, res))
            }
            FnKind::Generic(gen_args, statements) => {
                // Empty-body generic functions that return a type (constructors) should
                // use FnKind::Derived so constructor code generation kicks in.
                // A constructor is: name matches return type name AND return type is not void.
                let ret_degrouped = generic_function.rettype().degroup();
                let is_constructor = match &*ret_degrouped {
                    CType::Void | CType::DerivedVoid(..) => false,
                    CType::Type(name, _) => name == &generic_function.name && name != "void",
                    _ => false,
                };
                let kind = if statements.is_empty() && is_constructor {
                    FnKind::Derived
                } else {
                    FnKind::Normal
                };
                let args = generic_function
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
                // Make sure all argument types exist within the generic function call scope
                for (_, _, arg) in &args {
                    scope = CType::from_ctype(scope, arg.clone().to_callable_string(), arg.clone());
                }
                let mut rettype = {
                    let mut a = generic_function.rettype().clone();
                    for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                        a = a.swap_subtype(o.clone(), n.clone());
                    }
                    a
                };
                // Make the generic names aliases to these types during statement-to-microstatement
                // generation
                for (i, (n, _)) in gen_args.iter().enumerate() {
                    scope.types.insert(n.clone(), generic_types[i].clone());
                }
                let realized_name = format!(
                    "{}_{}",
                    generic_function.name,
                    generic_types
                        .iter()
                        .map(|t| t.clone().to_callable_string())
                        .collect::<Vec<_>>()
                        .join("_")
                );
                let microstatements = {
                    let mut ms = Vec::new();
                    for (name, kind, typen) in &args {
                        ms.push(Microstatement::Arg {
                            name: name.clone(),
                            kind: kind.clone(),
                            typen: typen.clone(),
                        });
                    }
                    for statement in statements {
                        let res = statement_to_microstatements(
                            statement,
                            Some(generic_function),
                            scope,
                            ms,
                        )?;
                        scope = res.0;
                        ms = res.1;
                    }
                    ms
                };
                // Determine the actual return type of the function and check if it matches the specified
                // return type (or update that return type if it's to be inferred
                if let Some(ms) = microstatements.last() {
                    if let Microstatement::Arg { .. } = ms {
                        // Don't do anything in this path, this is probably a derived function
                    } else {
                        let mut actual_rettype = match ms {
                            Microstatement::Return { value: Some(v) } => v.get_type(),
                            _ => Arc::new(CType::Void),
                        };
                        if body_awaits(&microstatements) && !is_promise_head(actual_rettype.clone()) {
                            actual_rettype = CType::promise(actual_rettype);
                        }
                        if let CType::Infer(..) = &*rettype {
                            rettype = actual_rettype;
                        } else if rettype.clone().to_strict_string(false)
                            != actual_rettype.clone().to_strict_string(false)
                        {
                            CType::fail(&format!(
                                "Function {} specified to return {} but actually returns {}",
                                generic_function.name,
                                rettype.to_strict_string(false),
                                actual_rettype.to_strict_string(false),
                            ));
                        } else {
                            // Do nothing, they're the same
                        }
                    }
                }
                // Create the actual realized function
                let f = Arc::new(Function {
                    name: realized_name.clone(),
                    typen: args_and_rettype_to_type(args, rettype),
                    microstatements,
                    kind,
                    origin_scope_path: scope.path.clone(),
                    lazy_body: None,
                });
                // Insert under suffixed name, deduplicating by both name and signature
                if scope.functions.contains_key(&f.name) {
                    let func_vec = scope.functions.get_mut(&f.name).unwrap();
                    if !func_vec
                        .iter()
                        .any(|fn_| fn_.name == f.name && fn_.typen == f.typen)
                    {
                        func_vec.insert(0, f.clone());
                    }
                } else {
                    scope.functions.insert(f.name.clone(), vec![f.clone()]);
                }
                let res = match scope.functions.get(&f.name) {
                    None => Err("This should be impossible. Cannot get the function we just added to the scope"),
                    Some(fs) => Ok(fs.first().unwrap().clone()),
                }?;
                Ok((scope, res))
            }
            FnKind::Cfn(cfn_kind, gen_args) => {
                // Realize a compiler-provided generic function. The CfnKind survives
                // realization so codegen can match on it directly (no name-based matching).
                let args = generic_function
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
                for (_, _, arg) in &args {
                    scope = CType::from_ctype(scope, arg.clone().to_callable_string(), arg.clone());
                }
                let rettype = {
                    let mut a = generic_function.rettype().clone();
                    for ((_, o), n) in gen_args.iter().zip(generic_types.iter()) {
                        a = a.swap_subtype(o.clone(), n.clone());
                    }
                    a
                };
                let mut microstatements = Vec::new();
                for (name, kind, typen) in &args {
                    microstatements.push(Microstatement::Arg {
                        name: name.clone(),
                        kind: kind.clone(),
                        typen: typen.clone(),
                    });
                }
                let name = format!(
                    "{}_{}",
                    generic_function.name,
                    generic_types
                        .iter()
                        .map(|t| t.clone().to_callable_string())
                        .collect::<Vec<String>>()
                        .join("_")
                );
                let f = Arc::new(Function {
                    name,
                    typen: args_and_rettype_to_type(args, rettype),
                    microstatements,
                    kind: FnKind::CfnRealized(cfn_kind.clone()),
                    origin_scope_path: scope.path.clone(),
                    lazy_body: None,
                });
                if scope.functions.contains_key(&f.name) {
                    let func_vec = scope.functions.get_mut(&f.name).unwrap();
                    func_vec.insert(0, f.clone());
                } else {
                    scope.functions.insert(f.name.clone(), vec![f.clone()]);
                }
                let res = match scope.functions.get(&f.name) {
                    None => Err("This should be impossible. Cannot get the function we just added to the scope"),
                    Some(fs) => Ok(fs.first().unwrap().clone()),
                }?;
                Ok((scope, res))
            }
        }
    }
}
