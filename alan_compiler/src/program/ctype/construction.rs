use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::withtypeoperatorslist_to_ctype;
use super::CType;
use super::Export;
use super::Function;
use super::Program;
use super::Scope;
use crate::parse;

impl CType {
    pub fn from_ast<'a>(
        mut scope: Scope<'a>,
        type_ast: &parse::Types,
        is_export: bool,
    ) -> Result<(Scope<'a>, CType), Box<dyn std::error::Error>> {
        let name = type_ast.fulltypename.typename.clone();
        if let Some(generics) = &type_ast.opttypegenerics {
            // We are going to conditionally compile this type declaration. If the we get true, we
            // continue, if we get false, we don't compile and return a Fail type that isn't added
            // to the scope to cause compilation to crash *if* something tries to use this, and if
            // we don't get a boolean at all or we get multiple inner values in the generic call,
            // we bail out immediately because of a syntax error.
            let generic_call = withtypeoperatorslist_to_ctype(&generics.typecalllist, &scope)?;
            match &*generic_call {
                CType::Bool(b) => match b {
                    false => return Ok((scope, CType::Fail(format!("{name} is not supposed to be compiled because the conditional compilation generic value is false")))),
                    true => { /* Do nothing */ }
                },
                CType::Type(n, c) => match &**c {
                    CType::Bool(b) => match b {
                        false => return Ok((scope, CType::Fail(format!("{name} is not supposed to be compiled because {n} is false")))),
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

        let (t, fs) = match &type_ast.fulltypename.opttypegenerics {
            None => {
                // This is a "normal" type
                // When creating a "normal" type, we also create constructor and optionally
                // accessor functions. This is not done for bound types nor done for
                // generics until the generic type has been constructed. We create a set of
                // `derived` Function objects and add it to the scope that a later stage of
                // the compiler is responsible for actually creating. All of the types get
                // one or more constructor functions, while struct-like Tuples and Either
                // get accessor functions to dig into the sub-types.
                let mut inner_type =
                    withtypeoperatorslist_to_ctype(&type_ast.typedef.typeassignables, &scope)?;
                // Unwrap a Group type, if any exists, we don't want it here.
                while matches!(&*inner_type, CType::Group(_)) {
                    inner_type = match &*inner_type {
                        CType::Group(t) => t.clone(),
                        _t => inner_type,
                    };
                }
                // Let's just avoid the "bare field" type definition and auto-wrap into a tuple
                if let CType::Field(..) = &*inner_type {
                    inner_type = Arc::new(CType::Tuple(vec![inner_type], Vec::new()));
                }
                // Magic hackery to convert a `From` type into an `Import` type if it's the top-level type
                inner_type = match &*inner_type {
                    CType::From(t) => {
                        CType::import(Arc::new(CType::TString(name.clone())), t.clone())
                    }
                    _t => inner_type,
                };
                // If we've got an `Import` type, we need to grab the actual type definition from
                // the other file and pull it in here.
                if let CType::Import(name, dep) = &*inner_type {
                    match &**dep {
                        CType::TString(dep_name) => {
                            let program = Program::get_program_guard();
                            let scope = program.get_ref().scope_by_file(dep_name)?;
                            match &**name {
                                CType::TString(n) => {
                                    inner_type = match scope.types.get(n) {
                                        None => {
                                            CType::fail(&format!("{n} not found in {dep_name}"))
                                        }
                                        Some(t) => match &**t {
                                            CType::Type(_, t) => t.clone(),
                                            _t => t.clone(),
                                        },
                                    }
                                }
                                _ => CType::fail("The name of the import must be a string"),
                            };
                        }
                        _ => CType::fail("TODO: Support imports beyond local directories"),
                    }
                }
                inner_type.to_functions(name.clone(), &scope)
            }
            Some(g) => {
                // This is a "generic" type
                // TODO: Stronger checking on the usage here
                let args = g
                    .typecalllist
                    .iter()
                    .map(|tc| tc.to_string())
                    .collect::<Vec<String>>()
                    .join("")
                    .split(", ")
                    .map(|r| r.trim().to_string())
                    .collect::<Vec<String>>();
                let mut temp_scope = scope.child();
                for arg in &args {
                    temp_scope.types.insert(
                        arg.clone(),
                        Arc::new(CType::Infer(arg.clone(), "Any".to_string())),
                    );
                }
                let generic_call =
                    withtypeoperatorslist_to_ctype(&type_ast.typedef.typeassignables, &temp_scope)?;
                (CType::Generic(name.clone(), args, generic_call), Vec::new())
            }
        };
        if is_export {
            scope.exports.insert(name.clone(), Export::Type);
            if !fs.is_empty() {
                let mut names = HashSet::new();
                for f in &fs {
                    names.insert(f.name.clone());
                }
                for name in names {
                    scope.exports.insert(name.clone(), Export::Function);
                }
            }
        }
        let insert_t = Arc::new(t.clone());
        scope.types.insert(name.clone(), insert_t.clone());
        scope
            .types
            .insert(insert_t.clone().to_callable_string(), insert_t.clone());
        if !fs.is_empty() {
            let mut name_fn_pairs = HashMap::new();
            for f in fs {
                if name_fn_pairs.contains_key(&f.name) {
                    let v: &mut Vec<Arc<Function>> = name_fn_pairs.get_mut(&f.name).unwrap();
                    v.push(f.clone());
                } else {
                    name_fn_pairs.insert(f.name.clone(), vec![f.clone()]);
                }
            }
            for (name, fns) in name_fn_pairs.drain() {
                if scope.functions.contains_key(&name) {
                    let func_vec = scope.functions.get_mut(&name).unwrap();
                    func_vec.splice(0..0, fns);
                } else {
                    scope.functions.insert(name, fns);
                }
            }
        }
        Ok((scope, t))
    }

    pub fn from_ctype(mut scope: Scope, name: String, ctype: Arc<CType>) -> Scope {
        scope.exports.insert(name.clone(), Export::Type);
        let (_, fs) = ctype.clone().to_functions(name.clone(), &scope);
        scope.types.insert(name, ctype.clone());
        scope
            .types
            .insert(ctype.clone().to_callable_string(), ctype);
        if !fs.is_empty() {
            let mut name_fn_pairs = HashMap::new();
            for f in fs {
                // We need to similarly load all of the return types from the functions created by
                // this from_ctype call if they don't already exist
                let mut contains_rettype = false;
                let retstr = f.rettype().to_functional_string();
                for t in scope.types.values() {
                    if retstr == t.clone().to_functional_string() {
                        contains_rettype = true;
                    }
                }
                if !contains_rettype {
                    scope = CType::from_ctype(scope, retstr, f.rettype().clone());
                }
                if name_fn_pairs.contains_key(&f.name) {
                    let v: &mut Vec<Arc<Function>> = name_fn_pairs.get_mut(&f.name).unwrap();
                    v.push(f.clone());
                } else {
                    name_fn_pairs.insert(f.name.clone(), vec![f.clone()]);
                }
            }
            for (name, fns) in name_fn_pairs.drain() {
                if scope.functions.contains_key(&name) {
                    let func_vec = scope.functions.get_mut(&name).unwrap();
                    func_vec.splice(0..0, fns);
                } else {
                    scope.functions.insert(name, fns);
                }
            }
        }
        scope
    }

    pub fn from_generic<'a>(scope: Scope<'a>, name: &str, arglen: usize) -> Scope<'a> {
        CType::from_ctype(
            scope,
            name.to_string(),
            Arc::new(CType::IntrinsicGeneric(name.to_string(), arglen)),
        )
    }
    pub fn swap_subtype(
        self: Arc<CType>,
        old_type: Arc<CType>,
        new_type: Arc<CType>,
    ) -> Arc<CType> {
        // Implemented recursively to be easier to follow. It would be nice to avoid all of the
        // cloning if the old type is not anywhere in the CType tree, but that would be a lot
        // harder to detect ahead of time.
        if self == old_type {
            return new_type;
        }
        match &*self {
            CType::Void
            | CType::DerivedVoid(..)
            | CType::Infer(..)
            | CType::Generic(..)
            | CType::IntrinsicGeneric(..)
            | CType::Int(_)
            | CType::Float(_)
            | CType::Bool(_)
            | CType::TString(_)
            | CType::Fail(_) => self.clone(),
            CType::Type(name, ct) => Arc::new(CType::Type(
                name.clone(),
                ct.clone().swap_subtype(old_type, new_type),
            )),
            CType::Binds(name, gen_type_resolved) => Arc::new(CType::Binds(
                name.clone()
                    .swap_subtype(old_type.clone(), new_type.clone()),
                gen_type_resolved
                    .iter()
                    .map(|gtr| gtr.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Shared(t) => Arc::new(CType::Shared(t.clone().swap_subtype(old_type, new_type))),
            CType::Promise(t) => {
                Arc::new(CType::Promise(t.clone().swap_subtype(old_type, new_type)))
            }
            CType::IntCast(i) => CType::intcast(i.clone().swap_subtype(old_type, new_type)),
            CType::FloatCast(f) => CType::floatcast(f.clone().swap_subtype(old_type, new_type)),
            CType::BoolCast(b) => CType::boolcast(b.clone().swap_subtype(old_type, new_type)),
            CType::StringCast(s) => CType::stringcast(s.clone().swap_subtype(old_type, new_type)),
            CType::Group(g) => g.clone().swap_subtype(old_type, new_type),
            CType::Unwrap(t) => CType::tunwrap(t.clone().swap_subtype(old_type, new_type)),
            CType::Function(i, o) => Arc::new(CType::Function(
                i.clone().swap_subtype(old_type.clone(), new_type.clone()),
                o.clone().swap_subtype(old_type, new_type),
            )),
            CType::Call(n, f) => Arc::new(CType::Call(
                n.clone().swap_subtype(old_type.clone(), new_type.clone()),
                f.clone().swap_subtype(old_type, new_type),
            )),
            CType::Infix(o) => Arc::new(CType::Infix(o.clone().swap_subtype(old_type, new_type))),
            CType::Prefix(o) => Arc::new(CType::Prefix(o.clone().swap_subtype(old_type, new_type))),
            CType::Method(f) => Arc::new(CType::Method(f.clone().swap_subtype(old_type, new_type))),
            CType::Property(p) => {
                Arc::new(CType::Property(p.clone().swap_subtype(old_type, new_type)))
            }
            CType::Cast(t) => Arc::new(CType::Cast(t.clone().swap_subtype(old_type, new_type))),
            CType::Own(t) => Arc::new(CType::Own(t.clone().swap_subtype(old_type, new_type))),
            CType::Deref(t) => Arc::new(CType::Deref(t.clone().swap_subtype(old_type, new_type))),
            CType::Mut(t) => Arc::new(CType::Mut(t.clone().swap_subtype(old_type, new_type))),
            CType::Dependency(n, v) => Arc::new(CType::Dependency(
                n.clone().swap_subtype(old_type.clone(), new_type.clone()),
                v.clone().swap_subtype(old_type, new_type),
            )),
            CType::Rust(d) => Arc::new(CType::Rust(d.clone().swap_subtype(old_type, new_type))),
            CType::Nodejs(d) => Arc::new(CType::Nodejs(d.clone().swap_subtype(old_type, new_type))),
            CType::From(d) => Arc::new(CType::From(d.clone().swap_subtype(old_type, new_type))),
            CType::Import(n, d) => Arc::new(CType::Import(
                n.clone().swap_subtype(old_type.clone(), new_type.clone()),
                d.clone().swap_subtype(old_type, new_type),
            )),
            CType::Tuple(ts, parents) => Arc::new(CType::Tuple(
                ts.iter()
                    .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>(),
                parents
                    .iter()
                    .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Field(name, t) => Arc::new(CType::Field(
                name.clone(),
                t.clone().swap_subtype(old_type, new_type),
            )),
            CType::Either(ts, parents) => {
                let new_ts = ts
                    .iter()
                    .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>();
                let new_parents = parents
                    .iter()
                    .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>();
                // Substitution can produce an `Either` whose variants are no longer distinct -- e.g.
                // `Maybe{()}` becomes `() | ()`, or `Fallible{Error}` becomes `Error | Error`. Such a
                // degenerate `Either` has no meaningful tag to discriminate on and is really just its
                // single underlying type, so we dedup the variants and collapse a singleton back to a
                // plain type (matching `CType::either`'s own normalization).
                //
                // This collapse can cause a generic compound function (e.g. `print{T}(v: T!)`, whose
                // body recurses on the `Error` variant) to collapse into a signature identical to a
                // leaf call like `print(Error)`. That is safe *as long as the leaf has a concrete
                // (non-generic) definition*, since concrete functions are resolved before generics
                // (`Scope::resolve_function`), terminating the recursion. The `print`/`eprint` leaves
                // in `root.ln` are concrete on both backends for exactly this reason.
                let mut seen = std::collections::HashSet::new();
                let deduped: Vec<Arc<CType>> = new_ts
                    .into_iter()
                    .filter(|t| seen.insert(t.clone().degroup().to_callable_string()))
                    .collect();
                if deduped.len() == 1 {
                    deduped.into_iter().next().unwrap()
                } else {
                    Arc::new(CType::Either(deduped, new_parents))
                }
            }
            CType::Prop(t, p) => CType::prop(
                t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                p.clone().swap_subtype(old_type, new_type),
            ),
            CType::AnyOf(ts) => Arc::new(CType::AnyOf(
                ts.iter()
                    .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Buffer(t, size) => Arc::new(CType::Buffer(
                t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                size.clone().swap_subtype(old_type, new_type),
            )),
            CType::Array(t) => Arc::new(CType::Array(t.clone().swap_subtype(old_type, new_type))),
            // For these when we swap, we check to see if we can "condense" them down into simpler
            // types (eg `Add{N, 1}` swapping `N` for `3` should just yield `4`)
            CType::Add(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::add)
                .unwrap(),
            CType::Sub(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::sub)
                .unwrap(),
            CType::Mul(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::mul)
                .unwrap(),
            CType::Div(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::div)
                .unwrap(),
            CType::Mod(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::cmod)
                .unwrap(),
            CType::Pow(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::pow)
                .unwrap(),
            CType::Min(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::min)
                .unwrap(),
            CType::Max(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::max)
                .unwrap(),
            CType::Neg(t) => CType::neg(t.clone().swap_subtype(old_type, new_type)),
            CType::Len(t) => CType::len(t.clone().swap_subtype(old_type, new_type)),
            CType::Exclude(t, p) => CType::exclude(
                t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                p.clone().swap_subtype(old_type, new_type),
            ),
            CType::Size(t) => CType::size(t.clone().swap_subtype(old_type, new_type)),
            CType::FileStr(t) => CType::filestr(t.clone().swap_subtype(old_type, new_type)),
            CType::Concat(a, b) => CType::concat(
                a.clone().swap_subtype(old_type.clone(), new_type.clone()),
                b.clone().swap_subtype(old_type, new_type),
            ),
            CType::Env(ts) => {
                if ts.len() == 1 {
                    CType::env(ts[0].clone().swap_subtype(old_type, new_type))
                } else if ts.len() == 2 {
                    CType::envdefault(
                        ts[0]
                            .clone()
                            .swap_subtype(old_type.clone(), new_type.clone()),
                        ts[1]
                            .clone()
                            .swap_subtype(old_type.clone(), new_type.clone()),
                    )
                } else {
                    CType::fail("Somehow gave Env{..} an incorrect number of args and caught during generic resolution")
                }
            }
            CType::EnvExists(t) => CType::envexists(t.clone().swap_subtype(old_type, new_type)),
            CType::TIf(t, ts) => {
                if ts.len() == 1 {
                    CType::tupleif(
                        t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                        ts[0].clone().swap_subtype(old_type, new_type),
                    )
                } else if ts.len() == 2 {
                    CType::cif(
                        t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                        ts[0]
                            .clone()
                            .swap_subtype(old_type.clone(), new_type.clone()),
                        ts[1].clone().swap_subtype(old_type, new_type),
                    )
                } else {
                    CType::fail("Somehow gave If{..} an incorrect number of args and caught during generic resolution")
                }
            }
            CType::And(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::and)
                .unwrap(),
            CType::Or(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::or)
                .unwrap(),
            CType::Xor(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::xor)
                .unwrap(),
            CType::Not(t) => CType::not(t.clone().swap_subtype(old_type, new_type)),
            CType::Nand(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::nand)
                .unwrap(),
            CType::Nor(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::nor)
                .unwrap(),
            CType::Xnor(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::xnor)
                .unwrap(),
            CType::TEq(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::eq)
                .unwrap(),
            CType::Neq(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::neq)
                .unwrap(),
            CType::Lt(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::lt)
                .unwrap(),
            CType::Lte(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::lte)
                .unwrap(),
            CType::Gt(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::gt)
                .unwrap(),
            CType::Gte(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::gte)
                .unwrap(),
        }
    }

    pub fn binds(args: Vec<Arc<CType>>) -> Arc<CType> {
        if args.is_empty() {
            return Arc::new(CType::Binds(Arc::new(CType::Void), Vec::new()));
        }
        let base_type = args[0].clone();
        if matches!(
            &*base_type,
            CType::TString(_) | CType::Import(..) | CType::From(_)
        ) {
            let mut out_vec = Vec::new();
            #[allow(clippy::needless_range_loop)] // It's not needless
            for i in 1..args.len() {
                out_vec.push(args[i].clone());
            }
            Arc::new(CType::Binds(base_type, out_vec))
        } else {
            CType::fail(
                "Binds{T, ...} must be given a string or an import for the base type to bind",
            );
        }
    }

    pub fn intcast(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::IntCast(arg))
        } else {
            match &*arg {
                CType::Int(_) => arg,
                CType::Float(f) => Arc::new(CType::Int(*f as i128)),
                CType::Bool(b) => Arc::new(if *b { CType::Int(1) } else { CType::Int(0) }),
                CType::TString(s) => match s.parse::<i128>() {
                    Ok(v) => Arc::new(CType::Int(v)),
                    Err(e) => Arc::new(CType::Fail(format!("{e:?}"))),
                },
                _ => Arc::new(CType::Fail("Not implemented".into())),
            }
        }
    }

    pub fn floatcast(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::FloatCast(arg))
        } else {
            match &*arg {
                CType::Float(_) => arg,
                CType::Int(i) => Arc::new(CType::Float(*i as f64)),
                CType::Bool(b) => Arc::new(if *b {
                    CType::Float(1.0)
                } else {
                    CType::Float(0.0)
                }),
                CType::TString(s) => match s.parse::<f64>() {
                    Ok(v) => Arc::new(CType::Float(v)),
                    Err(e) => Arc::new(CType::Fail(format!("{e:?}"))),
                },
                _ => Arc::new(CType::Fail("Not implemented".into())),
            }
        }
    }

    pub fn boolcast(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::BoolCast(arg))
        } else {
            match &*arg {
                CType::Bool(_) => arg,
                CType::Float(f) => Arc::new(CType::Bool(*f != 0.0)),
                CType::Int(i) => Arc::new(CType::Bool(*i != 0)),
                CType::TString(s) => Arc::new(CType::Bool(s == "true")),
                _ => Arc::new(CType::Fail("Not implemented".into())),
            }
        }
    }

    pub fn stringcast(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::StringCast(arg))
        } else {
            Arc::new(CType::TString(CType::to_functional_string(arg)))
        }
    }

    pub fn tunwrap(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::Unwrap(arg))
        } else {
            match &*arg {
                CType::Type(_, t) | CType::Group(t) | CType::Unwrap(t) => t.clone(),
                _ => arg,
            }
        }
    }

    pub fn promise(arg: Arc<CType>) -> Arc<CType> {
        let mut t = arg.clone();
        while matches!(&*t, CType::Type(..) | CType::Group(_)) {
            t = match &*t {
                CType::Type(_, inner) | CType::Group(inner) => inner.clone(),
                _ => unreachable!(),
            };
        }
        if matches!(&*t, CType::Promise(_)) {
            arg
        } else {
            Arc::new(CType::Promise(arg))
        }
    }

    pub fn import(name: Arc<CType>, dep: Arc<CType>) -> Arc<CType> {
        if let CType::Infer(..) = &*name {
            Arc::new(CType::Import(name, dep))
        } else if let CType::Infer(..) = &*dep {
            Arc::new(CType::Import(name, dep))
        } else if !matches!(&*name, CType::TString(_)) {
            CType::fail("The Import{N, D} N parameter must be a string")
        } else {
            match &*dep {
                CType::TString(s) => {
                    // Load the dependency
                    if let Err(e) = Program::load(s.clone()) {
                        CType::fail(&format!("Failed to load dependency {s}: {e:?}"))
                    } else {
                        let program = Program::get_program_guard();
                        let out = match program.get_ref().scope_by_file(s) {
                            Err(e) => CType::fail(&format!("Failed to load dependency {s}: {e:?}")),
                            Ok(dep_scope) => {
                                // Currently can only import types and functions. Constants and
                                // operator mappings don't have a syntax to express this. TODO:
                                // Figure out how to tackle this syntactically, and then update
                                // this logic.
                                if let CType::TString(n) = &*name {
                                    let found = dep_scope.types.contains_key(n)
                                        || dep_scope.functions.contains_key(n);
                                    if !found {
                                        CType::fail(&format!("{n} not found in {s}"))
                                    } else {
                                        // We're good
                                        Arc::new(CType::Import(name, dep))
                                    }
                                } else {
                                    CType::fail("The Import{N, D} N parameter must be a string")
                                }
                            }
                        };
                        out
                    }
                }
                CType::Dependency(..) => CType::fail("TODO: Alan package import support"),
                CType::Nodejs(_) | CType::Rust(_) => Arc::new(CType::Import(name, dep)),
                CType::Type(_, t) if matches!(**t, CType::Nodejs(_) | CType::Rust(_) | CType::Binds(..)) => {
                    Arc::new(CType::Import(name, dep))
                }
                CType::Binds(..) => Arc::new(CType::Import(name, dep)),
                otherwise => CType::fail(&format!(
                    "Invalid import defined {} <- {}",
                    name.clone().to_functional_string(),
                    Arc::new(otherwise.clone()).to_functional_string()
                )),
            }
        }
    }
    // Special implementation for the tuple and either types since they *are* CTypes, but if one of
    // the provided input types *is* the same kind of CType, it should produce a merged version.
    pub fn tuple(args: Vec<Arc<CType>>) -> Arc<CType> {
        let mut out_vec = Vec::new();
        for arg in args {
            match &*arg {
                CType::Tuple(ts, _) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                _other => out_vec.push(arg),
            }
        }
        Arc::new(CType::Tuple(out_vec, Vec::new()))
    }
    pub fn either(args: Vec<Arc<CType>>) -> Arc<CType> {
        let mut out_vec = Vec::new();
        for arg in args {
            match &*arg {
                CType::Either(ts, _) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                _other => out_vec.push(arg),
            }
        }
        // Deduplicate by string representation
        let mut seen = HashSet::new();
        let deduped: Vec<Arc<CType>> = out_vec
            .into_iter()
            .filter(|t| {
                let key = t.clone().degroup().to_callable_string();
                seen.insert(key)
            })
            .collect();
        // If deduplication leaves a single type, unwrap the Either
        if deduped.len() == 1 {
            deduped.into_iter().next().unwrap()
        } else {
            Arc::new(CType::Either(deduped, Vec::new()))
        }
    }
    pub fn prop(t: Arc<CType>, p: Arc<CType>) -> Arc<CType> {
        // Check the arguments first to see if they're to be inferred
        if t.clone().has_infer() || p.clone().has_infer() {
            return Arc::new(CType::Prop(t, p));
        }
        match &*t {
            CType::Infer(..) => unreachable!(),
            CType::Type(_, t) => CType::prop(t.clone(), p),
            CType::Group(t) => CType::prop(t.clone(), p),
            CType::Field(n, f) => match &*p {
                CType::TString(s) => {
                    if n == s {
                        f.clone()
                    } else {
                        Arc::new(CType::Fail(format!(
                            "Property {} not found on type {:?}",
                            s, &t
                        )))
                    }
                }
                CType::Int(i) => match i {
                    0 => Arc::new(CType::TString(n.to_string())),
                    1 => f.clone(),
                    _ => Arc::new(CType::Fail(
                        "Only 0 or 1 are valid integer accesses on a field".into(),
                    )),
                },
                otherwise => Arc::new(CType::Fail(format!(
                    "Properties must be a name or integer location, not {otherwise:?}",
                ))),
            },
            CType::Tuple(ts, _) | CType::Either(ts, _) => match &*p {
                CType::TString(s) => {
                    if let Ok(i) = s.parse::<i128>() {
                        if (0..ts.len()).contains(&(i as usize)) {
                            return ts[i as usize].clone();
                        } else {
                            return Arc::new(CType::Fail(format!(
                                "{i} is out of bounds for type {t:?}"
                            )));
                        }
                    }
                    for inner in ts {
                        if let CType::Field(n, f) = &**inner {
                            if n == s {
                                return f.clone();
                            }
                        }
                    }
                    Arc::new(CType::Fail(format!("Property {s} not found on type {t:?}")))
                }
                CType::Int(i) => {
                    if (0..ts.len()).contains(&(*i as usize)) {
                        ts[*i as usize].clone()
                    } else {
                        Arc::new(CType::Fail(format!("{i} is out of bounds for type {t:?}")))
                    }
                }
                otherwise => Arc::new(CType::Fail(format!(
                    "Properties must be a name or integer location, not {otherwise:?}",
                ))),
            },
            CType::TIf(_, tf) => {
                match &*p {
                    CType::TString(s) => {
                        // TODO: Is this path reachable?
                        if s == "true" {
                            tf[0].clone()
                        } else if s == "false" {
                            tf[1].clone()
                        } else {
                            CType::fail("Only true or false (or 1 or 0) are valid for accessing the types from an If{C, A, B} type")
                        }
                    }
                    CType::Bool(b) => {
                        if *b {
                            tf[0].clone()
                        } else {
                            tf[1].clone()
                        }
                    }
                    CType::Int(i) => {
                        if (0..2).contains(i) {
                            tf[*i as usize].clone()
                        } else {
                            CType::fail("Only true or false (or 1 or 0) are valid for accessing the types from an If{C, A, B} type")
                        }
                    }
                    otherwise => CType::fail(&format!(
                        "Properties must be a name or integer location, not {otherwise:?}",
                    )),
                }
            }
            otherwise => CType::fail(&format!(
                "Properties cannot be accessed from type {otherwise:?}"
            )),
        }
    }
    pub fn anyof(args: Vec<Arc<CType>>) -> Arc<CType> {
        // `AnyOf{...}` is a type-position construct meaning "resolves to exactly one of these,
        // chosen by context" (see `CType::AnyOf`). It must produce the `AnyOf` variant -- not
        // `Either` (a tagged union that physically holds one variant at runtime). Nested `AnyOf`s
        // are flattened into a single set.
        let mut out_vec = Vec::new();
        for arg in args {
            match &*arg {
                CType::AnyOf(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                _other => out_vec.push(arg),
            }
        }
        Arc::new(CType::AnyOf(out_vec))
    }
    /// Strip the value-position wrappers (`Type`/`Group` aliases plus `Deref`/`Mut`/`Own`/`Shared`)
    /// from a type to expose the underlying "core" type. Used when matching a numeric-literal
    /// candidate against a function parameter type (which may be e.g. `Deref{i64}`).
    pub fn strip_value_wrappers(self: Arc<CType>) -> Arc<CType> {
        let t = self.degroup();
        match &*t {
            CType::Deref(inner) | CType::Mut(inner) | CType::Own(inner) | CType::Shared(inner) => {
                inner.clone().strip_value_wrappers()
            }
            _ => t,
        }
    }
    /// Collapse an `AnyOf` to its single default type by picking the *last*
    /// candidate (the highest-priority entry under the FUI ordering used when
    /// typing numeric literals: Floats, Unsigned ints, signed Ints, ascending bit
    /// width). This is the "pick last in FUI order" rule that resolves a numeric
    /// literal whose type was never narrowed by context. Non-`AnyOf` types are
    /// returned unchanged.
    pub fn collapse_anyof_default(self: Arc<CType>) -> Arc<CType> {
        match &*self {
            CType::AnyOf(ts) => {
                // Only collapse "value" `AnyOf`s such as numeric-literal candidate sets. `AnyOf`s
                // whose members are function types come from overload resolution / operator-return
                // merging and must be left intact so higher-order dispatch can narrow them.
                fn is_function_like(t: &Arc<CType>) -> bool {
                    match &*t.clone().degroup() {
                        CType::Function(..) => true,
                        CType::Generic(_, _, inner) => is_function_like(inner),
                        _ => false,
                    }
                }
                if ts.iter().any(is_function_like) {
                    return self;
                }
                match ts.last() {
                    Some(t) => t.clone().collapse_anyof_default(),
                    None => self,
                }
            }
            _ => self,
        }
    }
    pub fn field(mut args: Vec<Arc<CType>>) -> Arc<CType> {
        if args.len() != 2 {
            CType::fail("Field{K, V} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            match (&*arg0, &*arg1) {
                (CType::TString(key), anything) => {
                    Arc::new(CType::Field(key.clone(), Arc::new(anything.clone())))
                }
                _ => CType::fail("The field key must be a quoted string at this time"),
            }
        }
    }
    // Some validation for buffer creation, too
    pub fn buffer(mut args: Vec<Arc<CType>>) -> Arc<CType> {
        if args.len() != 2 {
            CType::fail("Buffer{T, S} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap().degroup();
            let arg0 = args.pop().unwrap().degroup();
            match (&*arg0, &*arg1) {
                (CType::Infer(..), _) => Arc::new(CType::Buffer(arg0.clone(), arg1.clone())),
                (_, CType::Infer(..)) => Arc::new(CType::Buffer(arg0.clone(), arg1.clone())),
                (_, CType::Int(size)) => {
                    if *size < 0 {
                        CType::fail("The buffer size must be a positive integer")
                    } else {
                        Arc::new(CType::Buffer(arg0, Arc::new(CType::Int(*size))))
                    }
                }
                otherwise => CType::fail(&format!(
                    "The buffer size must be a positive integer {otherwise:?}"
                )),
            }
        }
    }
    // Implementation of the ctypes that aren't storage but compute into another CType
    pub fn fail(message: &str) -> ! {
        // TODO: Include more information on where this compiler exit is coming from
        eprintln!("{message}");
        std::process::exit(1);
    }
    pub fn cfail(message: Arc<CType>) -> Arc<CType> {
        match &*message {
            CType::TString(s) => Arc::new(CType::Fail(s.clone())),
            _ => CType::fail("Fail passed a type that does not resolve into a message string"),
        }
    }
}
