// TODO: Generics/Interfaces resolution
use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use crate::program::CType;

pub fn ctype_to_rtype(
    ctype: Arc<CType>,
    mut deps: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match &*ctype {
        CType::Mut(t) if matches!(&**t, CType::Function(..)) => {
            // Special path to mark a closure as allowed to mutate its originating scope
            if let CType::Function(i, o) = &**t {
                if let CType::Void = **i {
                    let res = ctype_to_rtype(o.clone(), deps)?;
                    let s = res.0;
                    deps = res.1;
                    Ok((format!("impl FnMut() -> {s}"), deps))
                } else {
                    Ok((format!(
                        "impl FnMut(&{}) -> {}",
                        match &**i {
                            CType::Tuple(ts) => {
                                let mut out = Vec::new();
                                for t in ts {
                                    let res = ctype_to_rtype(t.clone(), deps)?;
                                    let s = res.0;
                                    deps = res.1;
                                    out.push(match &**t {
                                        CType::Mut(_) => format!("mut {s}"),
                                        _ => s,
                                    });
                                }
                                out.join(", &")
                            },
                            otherwise => {
                                let res = ctype_to_rtype(i.clone(), deps)?;
                                let s = res.0;
                                deps = res.1;
                                match &otherwise {
                                    CType::Mut(_) => format!("mut {s}"),
                                    _ => s,
                                }
                            }
                        }, {
                            let res = ctype_to_rtype(o.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            s
                        }
                    ), deps))
                }
            } else {
                unreachable!();
            }
        }
        CType::Mut(t) => {
            ctype_to_rtype(t.clone(), deps)
        }
        CType::Void => Ok(("void".to_string(), deps)),
        CType::Infer(s, _) => Err(format!(
            "Inferred type matching {s} was not realized before code generation"
        )
        .into()),
        CType::Type(_, t) => match &**t {
            CType::Either(ts) => {
                if ts.len() == 2 && (matches!(*ts[1], CType::Void) || matches!(&*ts[1], CType::Type(n, _) if n == "Error")) {
                    return Ok(("".to_string(), deps));
                }
                let mut enum_type_strs = Vec::new();
                for t in ts {
                    match &**t {
                        CType::Field(k, v) => {
                            let res = ctype_to_rtype(v.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            enum_type_strs.push(format!("{k}({s})"));
                        }
                        CType::Type(n, t) => {
                            let res = ctype_to_rtype(t.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            enum_type_strs.push(format!("{n}({s})"));
                        }
                        CType::Group(g) => {
                            let res = ctype_to_rtype(g.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            enum_type_strs.push(s);
                        }
                        CType::Void => enum_type_strs.push("void".to_string()),
                        CType::Tuple(ts) => {
                            let mut out = Vec::new();
                            for t in ts {
                                let res = ctype_to_rtype(t.clone(), deps)?;
                                let s = res.0;
                                deps = res.1;
                                out.push(s);
                            }
                            let name = t.clone().to_callable_string();
                            enum_type_strs.push(format!("{}({})", name, out.join(", ")));
                        }
                        _otherwise => {
                            let res = ctype_to_rtype(t.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            let name = t.clone().to_callable_string();
                            enum_type_strs.push(format!("{name}({s})"));
                        }
                    }
                }
                let name = t.clone().to_callable_string();
                Ok((format!(
                    "#[derive(Clone)]\nenum {} {{ {} }}",
                    name,
                    enum_type_strs.join(", ")
                ), deps))
            }
            CType::Tuple(ts) => {
                let mut out = Vec::new();
                for t in ts {
                    match &**t {
                        CType::Field(_, t2) => {
                            if !matches!(&**t2, CType::Int(_) | CType::Float(_) | CType::Bool(_) | CType::TString(_)) {
                                let res = ctype_to_rtype(t.clone(), deps)?;
                                let s = res.0;
                                deps = res.1;
                                out.push(s);
                            }
                        }
                        _otherwise => {
                            let res = ctype_to_rtype(t.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            out.push(s);
                        }
                    }
                }
                Ok((format!("({})", out.join(", ")), deps))
            }
            CType::Binds(name, args) => {
                let mut out_args = Vec::new();
                for arg in args {
                    let res = ctype_to_rtype(arg.clone(), deps)?;
                    let s = res.0;
                    deps = res.1;
                    out_args.push(s);
                }
                match &**name {
                    CType::TString(s) => {
                        if out_args.is_empty() {
                            Ok((s.clone(), deps))
                        } else {
                            Ok((
                                format!("{}<{}>", s, out_args.join(", ")),
                                deps,
                            ))
                        }
                    }
                    CType::Import(n, d) => {
                        match &**d {
                            CType::Type(_, t) => match &**t {
                                CType::Rust(d) => match &**d {
                                    CType::Dependency(n, v) => {
                                        let name = match &**n {
                                            CType::TString(s) => s.clone(),
                                            _ => CType::fail("Dependency names must be strings"),
                                        };
                                        let version = match &**v {
                                            CType::TString(s) => s.clone(),
                                            _ => CType::fail("Dependency versions must be strings"),
                                        };
                                        deps.insert(name, version);
                                    }
                                    _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                }
                                otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                            }
                            CType::Rust(d) => match &**d {
                                CType::Dependency(n, v) => {
                                    let name = match &**n {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency names must be strings"),
                                    };
                                    let version = match &**v {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency versions must be strings"),
                                    };
                                    deps.insert(name, version);
                                }
                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                        }
                        let native_type = match &**n {
                            CType::TString(s) => s.clone(),
                            _ => CType::fail("Native import names must be strings"),
                        };
                        if out_args.is_empty() {
                            Ok((native_type, deps))
                        } else {
                            Ok((
                                format!("{}<{}>", native_type, out_args.join(", ")),
                                deps,
                            ))
                        }
                    }
                    _ => CType::fail("Bound types must be strings or rust imports"),
                }
            }
            _otherwise => ctype_to_rtype(t.clone(), deps),
        }
        CType::Generic(name, args, _) => Ok((format!("{}<{}>", name, args.join(", ")), deps)),
        CType::Binds(n, args) => {
            let mut out_args = Vec::new();
            for arg in args {
                let res = ctype_to_rtype(arg.clone(), deps)?;
                let s = res.0;
                deps = res.1;
                out_args.push(s);
            }
            match &**n {
                CType::TString(s) => {
                    if out_args.is_empty() {
                        Ok((s.clone(), deps))
                    } else {
                        Ok((
                            format!("{}<{}>", s, out_args.join(", ")),
                            deps,
                        ))
                    }
                }
                CType::Import(n, d) => {
                    match &**d {
                        CType::Type(_, t) => match &**t {
                            CType::Rust(d) => match &**d {
                                CType::Dependency(n, v) => {
                                    let name = match &**n {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency names must be strings"),
                                    };
                                    let version = match &**v {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency versions must be strings"),
                                    };
                                    deps.insert(name, version);
                                }
                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                        }
                        CType::Rust(d) => match &**d {
                            CType::Dependency(n, v) => {
                                let name = match &**n {
                                    CType::TString(s) => s.clone(),
                                    _ => CType::fail("Dependency names must be strings"),
                                };
                                let version = match &**v {
                                    CType::TString(s) => s.clone(),
                                    _ => CType::fail("Dependency versions must be strings"),
                                };
                                deps.insert(name, version);
                            }
                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                        }
                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                    }
                    let native_type = match &**n {
                        CType::TString(s) => s.clone(),
                        _ => CType::fail("Native import names must be strings"),
                    };
                    if out_args.is_empty() {
                        Ok((native_type, deps))
                    } else {
                        Ok((
                            format!("{}<{}>", native_type, out_args.join(", ")),
                            deps,
                        ))
                    }
                }
                _ => CType::fail("Bound types must be strings or rust imports"),
            }
        }
        CType::IntrinsicGeneric(name, _) => Ok((name.clone(), deps)), // How would this even be reached?
        CType::Int(i) => Ok((i.to_string(), deps)),
        CType::Float(f) => Ok((f.to_string(), deps)),
        CType::Bool(b) => Ok((b.to_string(), deps)),
        CType::TString(s) => Ok((s.chars().map(|c| match c {
            '0'..='9' => c,
            'a'..='z' => c,
            'A'..='Z' => c,
            _ => '_',
        }).collect::<String>(), deps)),
        CType::Group(g) => {
            let res = ctype_to_rtype(g.clone(), deps)?;
            let s = res.0;
            deps = res.1;
            Ok((format!("({s})"), deps))
        }
        CType::Function(i, o) => {
            if let CType::Void = **i {
                let res = ctype_to_rtype(o.clone(), deps)?;
                let s = res.0;
                deps = res.1;
                Ok((format!("impl Fn() -> {s}"), deps))
            } else {
                Ok((format!(
                    "impl Fn(&{}) -> {}",
                    match &**i {
                        CType::Tuple(ts) => {
                            let mut out = Vec::new();
                            for t in ts {
                                let res = ctype_to_rtype(t.clone(), deps)?;
                                let s = res.0;
                                deps = res.1;
                                out.push(s);
                            }
                            out.join(", &")
                        },
                        _otherwise => {
                            let res = ctype_to_rtype(i.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            s
                        }
                    }, {
                        let res = ctype_to_rtype(o.clone(), deps)?;
                        let s = res.0;
                        deps = res.1;
                        s
                    }
                ), deps))
            }
        },
        CType::Tuple(ts) => {
            let mut out = Vec::new();
            for t in ts {
                match &**t {
                    CType::Field(_, t2) => {
                        if !matches!(&**t2, CType::Int(_) | CType::Float(_) | CType::Bool(_) | CType::TString(_)) {
                            let res = ctype_to_rtype(t.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            out.push(s);
                        }
                    }
                    _otherwise => {
                        let res = ctype_to_rtype(t.clone(), deps)?;
                        let s = res.0;
                        deps = res.1;
                        out.push(s);
                    }
                }
            }
            if out.len() == 1 {
                Ok((format!("({},)", out[0]), deps))
            } else {
                Ok((format!("({})", out.join(", ")), deps))
            }
        }
        CType::Field(k, v) => {
            let res = ctype_to_rtype(v.clone(), deps)?;
            let s = res.0;
            deps = res.1;
            Ok((format!("/* {k} */ {s}"), deps))
        }
        CType::Either(ts) => {
            // Special handling to convert `Either{T, void}` to `Option<T>` and `Either{T, Error}`
            // to `Result<T, AlanError>`
            if ts.len() == 2 {
                let alan_error = "alan_std::AlanError".to_string();
                match &*ts[1] {
                    CType::Void => {
                        let res = ctype_to_rtype(ts[0].clone(), deps)?;
                        let s = res.0;
                        deps = res.1;
                        Ok((format!("Option<{s}>"), deps))
                    }
                    CType::Binds(rustname, _) => match &**rustname {
                        CType::Import(n, d) => match &**n {
                            CType::TString(e) if e == &alan_error => {
                                let res = ctype_to_rtype(ts[0].clone(), deps)?;
                                let s = res.0;
                                deps = res.1;
                                match &**d {
                                    CType::Type(_, t) => match &**t {
                                        CType::Rust(d) => match &**d {
                                            CType::Dependency(n, v) => {
                                                let name = match &**n {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency names must be strings"),
                                                };
                                                let version = match &**v {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency versions must be strings"),
                                                };
                                                deps.insert(name, version);
                                            }
                                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                        }
                                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                                    }
                                    CType::Rust(d) => match &**d {
                                        CType::Dependency(n, v) => {
                                            let name = match &**n {
                                                CType::TString(s) => s.clone(),
                                                _ => CType::fail("Dependency names must be strings"),
                                            };
                                            let version = match &**v {
                                                CType::TString(s) => s.clone(),
                                                _ => CType::fail("Dependency versions must be strings"),
                                            };
                                            deps.insert(name, version);
                                        }
                                        _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                    }
                                    otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                                }
                                Ok((format!("Result<{}, {}>", s, "alan_std::AlanError"), deps))
                            }
                            _ => Ok((Arc::new(CType::Either(ts.clone())).to_callable_string(), deps)),
                        }
                        _ => Ok((Arc::new(CType::Either(ts.clone())).to_callable_string(), deps)),
                    }
                    CType::Type(_, t) => match &**t {
                        CType::Binds(rustname, _) => match &**rustname {
                            CType::Import(n, d) => match &**n {
                                CType::TString(e) if e == &alan_error => {
                                    let res = ctype_to_rtype(ts[0].clone(), deps)?;
                                    let s = res.0;
                                    deps = res.1;
                                    match &**d {
                                        CType::Type(_, t) => match &**t {
                                            CType::Rust(d) => match &**d {
                                                CType::Dependency(n, v) => {
                                                    let name = match &**n {
                                                        CType::TString(s) => s.clone(),
                                                        _ => CType::fail("Dependency names must be strings"),
                                                    };
                                                    let version = match &**v {
                                                        CType::TString(s) => s.clone(),
                                                        _ => CType::fail("Dependency versions must be strings"),
                                                    };
                                                    deps.insert(name, version);
                                                }
                                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                            }
                                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                                        }
                                        CType::Rust(d) => match &**d {
                                            CType::Dependency(n, v) => {
                                                let name = match &**n {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency names must be strings"),
                                                };
                                                let version = match &**v {
                                                    CType::TString(s) => s.clone(),
                                                    _ => CType::fail("Dependency versions must be strings"),
                                                };
                                                deps.insert(name, version);
                                            }
                                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                                        }
                                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                                    }
                                    Ok((format!("Result<{}, {}>", s, "alan_std::AlanError"), deps))
                                }
                                _ => Ok((Arc::new(CType::Either(ts.clone())).to_callable_string(), deps)),
                            }
                            _ => Ok((Arc::new(CType::Either(ts.clone())).to_callable_string(), deps)),
                        }
                        _ => Ok((Arc::new(CType::Either(ts.clone())).to_callable_string(), deps)),
                    }
                    _ => Ok((Arc::new(CType::Either(ts.clone())).to_callable_string(), deps)),
                }
            } else {
                for t in ts {
                    // Make sure we add all of the deps, if necessary
                    let res = ctype_to_rtype(t.clone(), deps)?;
                    deps = res.1;
                }
                Ok((Arc::new(CType::Either(ts.clone())).to_callable_string(), deps))
            }
        }
        CType::AnyOf(_) => Ok(("".to_string(), deps)), // Does this make any sense in Rust?
        CType::Buffer(t, s) => {
            let res = ctype_to_rtype(t.clone(), deps)?;
            let t = res.0;
            deps = res.1;
            Ok((format!(
                "[{};{}]",
                t,
                match **s {
                    CType::Int(size) => Ok(size as usize),
                    _ =>
                        Err("Somehow received a buffer definition with a non-integer size".to_string()),
                }?
            ), deps))
        }
        CType::Array(t) => {
            let res = ctype_to_rtype(t.clone(), deps)?;
            let s = res.0;
            deps = res.1;
            Ok((format!("Vec<{s}>"), deps))
        }
        CType::Fail(m) => CType::fail(m),
        _otherwise => CType::fail(&format!("Lower stage of the compiler received unresolved algebraic type {}, cannot deal with it here. Please report this error.", ctype.clone().to_functional_string())),
    }
}

#[allow(clippy::type_complexity)]
pub fn generate(
    typen: Arc<CType>,
    mut out: OrderedHashMap<String, String>,
    mut deps: OrderedHashMap<String, String>,
) -> Result<
    (
        String,
        OrderedHashMap<String, String>,
        OrderedHashMap<String, String>,
    ),
    Box<dyn std::error::Error>,
> {
    match &*typen {
        // The first value is the identifier, and the second is the generated source. For the
        // `Bind` and `Alias` types, these already exist in the Rust environment (or should exist,
        // assuming no bugs in the standard library) so they do not alter the generated source
        // output, while the `Structlike` type requires a new struct to be created and inserted
        // into the source definition, potentially inserting inner types as needed
        CType::Binds(n, ts) => {
            let mut genargs = Vec::new();
            for t in ts {
                let res = ctype_to_rtype(t.clone(), deps)?;
                let s = res.0;
                deps = res.1;
                genargs.push(s);
            }
            match &**n {
                CType::TString(s) => {
                    if genargs.is_empty() {
                        Ok((s.clone(), out, deps))
                    } else {
                        Ok((format!("{}<{}>", s, genargs.join(", ")), out, deps))
                    }
                }
                CType::Import(n, d) => {
                    match &**d {
                        CType::Type(_, t) => match &**t {
                            CType::Rust(d) => match &**d {
                                CType::Dependency(n, v) => {
                                    let name = match &**n {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency names must be strings"),
                                    };
                                    let version = match &**v {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency versions must be strings"),
                                    };
                                    deps.insert(name, version);
                                }
                                _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                        }
                        CType::Rust(d) => match &**d {
                            CType::Dependency(n, v) => {
                                let name = match &**n {
                                    CType::TString(s) => s.clone(),
                                    _ => CType::fail("Dependency names must be strings"),
                                };
                                let version = match &**v {
                                    CType::TString(s) => s.clone(),
                                    _ => CType::fail("Dependency versions must be strings"),
                                };
                                deps.insert(name, version);
                            }
                            _ => CType::fail("Rust dependencies must be declared with the dependency syntax"),
                        }
                        otherwise => CType::fail(&format!("Native imports compiled to Rust *must* be declared Rust{{D}} dependencies: {otherwise:?}"))
                    }
                    let native_type = match &**n {
                        CType::TString(s) => s.clone(),
                        _ => CType::fail("Native import names must be strings"),
                    };
                    if genargs.is_empty() {
                        Ok((native_type, out, deps))
                    } else {
                        Ok((
                            format!("{}<{}>", native_type, genargs.join(", ")),
                            out,
                            deps,
                        ))
                    }
                }
                _ => CType::fail("Bound types must be strings or rust imports"),
            }
        }
        // TODO: The complexity of this function indicates more fundamental issues in the type
        // generation. This needs a rethink and rewrite.
        CType::Type(name, t) => match &**t {
            CType::Either(_) => {
                let res = generate(t.clone(), out, deps)?;
                out = res.1;
                deps = res.2;
                let res = ctype_to_rtype(typen.clone(), deps)?;
                let s = res.0;
                deps = res.1;
                out.insert(t.clone().to_callable_string(), s);
                Ok((name.clone(), out, deps))
            }
            _ => {
                let res = ctype_to_rtype(t.clone(), deps)?;
                let s = res.0;
                deps = res.1;

                Ok((s, out, deps))
            }
        },
        CType::Tuple(_) => {
            let res = ctype_to_rtype(typen, deps)?;
            let s = res.0;
            deps = res.1;
            Ok((s, out, deps))
        }
        CType::Void => {
            out.insert("void".to_string(), "type void = ();".to_string());
            Ok(("()".to_string(), out, deps))
        }
        CType::Either(ts) => {
            // Make sure every sub-type exists
            for t in ts {
                let res = generate(t.clone(), out, deps)?;
                out = res.1;
                deps = res.2;
            }

            let res = ctype_to_rtype(typen, deps)?;
            let out_str = res.0;
            deps = res.1;
            Ok((out_str, out, deps)) // TODO: Put something into out?
        }
        CType::Group(g) => {
            let res = generate(g.clone(), out, deps)?;
            out = res.1;
            deps = res.2;
            Ok(("".to_string(), out, deps))
        }
        _otherwise => {
            let res = ctype_to_rtype(typen, deps)?;
            let out_str = res.0;
            deps = res.1;
            Ok((out_str, out, deps)) // TODO: Put something into out?
        }
    }
}
