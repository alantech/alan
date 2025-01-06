use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use crate::program::CType;

pub fn ctype_to_jtype(
    ctype: Arc<CType>,
    mut deps: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match &*ctype {
        CType::Mut(t) => ctype_to_jtype(t.clone(), deps),
        CType::Void => Ok(("".to_string(), deps)),
        CType::Infer(s, _) => Err(format!(
            "Inferred type matching {} was not realized before code generation",
            s
        )
        .into()),
        CType::Type(n, t) => match &**t {
            CType::Either(ts) => {
                for t in ts {
                    match &**t {
                        CType::Field(_, v) => {
                            let res = ctype_to_jtype(v.clone(), deps)?;
                            deps = res.1;
                        }
                        CType::Type(_, t) => {
                            let res = ctype_to_jtype(t.clone(), deps)?;
                            deps = res.1;
                        }
                        CType::Group(g) => {
                            let res = ctype_to_jtype(g.clone(), deps)?;
                            deps = res.1;
                        }
                        CType::Void => { /* Do nothing */ }
                        CType::Tuple(ts) => {
                            for t in ts {
                                let res = ctype_to_jtype(t.clone(), deps)?;
                                deps = res.1;
                            }
                        }
                        CType::Array(t) => {
                            let res = ctype_to_jtype(t.clone(), deps)?;
                            deps = res.1;
                        }
                        otherwise => {
                            return Err(format!("TODO: What is this? {:?}", otherwise).into());
                        }
                    }
                }
                Ok(("".to_string(), deps))
            }
            CType::Tuple(ts) => {
                let mut out = Vec::new();
                for (i, t) in ts.iter().enumerate() {
                    match &**t {
                        CType::Field(n, t2) => {
                            if !matches!(
                                &**t2,
                                CType::Int(_)
                                    | CType::Float(_)
                                    | CType::Bool(_)
                                    | CType::TString(_)
                            ) {
                                let res = ctype_to_jtype(t.clone(), deps)?;
                                deps = res.1;
                                out.push(n.clone());
                            }
                        }
                        _otherwise => {
                            let res = ctype_to_jtype(t.clone(), deps)?;
                            deps = res.1;
                            out.push(format!("arg{}", i));
                        }
                    }
                }
                Ok((
                    format!(
                        "class {} {{\n  constructor({}) {{\n    {}\n  }}\n}}",
                        n,
                        out.join(", "),
                        out.iter()
                            .map(|s| format!("    this.{} = {};", s, s))
                            .collect::<Vec<String>>()
                            .join("\n")
                    ),
                    deps,
                ))
            }
            CType::Binds(name, _) => match &**name {
                CType::TString(_) => Ok(("".to_string(), deps)),
                CType::Import(n, d) => {
                    match &**d {
                            CType::Type(_, t) => match &**t {
                                CType::Node(d) => match &**d {
                                    CType::Dependency(n, v) => {
                                        let name = match &**n {
                                            CType::TString(s) => s.clone(),
                                            _ => CType::fail("Dependency name must be a string"),
                                        };
                                        let version = match &**v {
                                            CType::TString(s) => s.clone(),
                                            _ => CType::fail("Dependency version must be a string"),
                                        };
                                        deps.insert(name, version);
                                    }
                                    _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                                }
                                otherwise => CType::fail(&format!("Native imports compiled to Javascript *must* be declared Node{{D}} dependencies: {:?}", otherwise))
                            }
                            CType::Node(d) => match &**d {
                                CType::Dependency(n, v) => {
                                    let name = match &**n {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency name must be a string"),
                                    };
                                    let version = match &**v {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency version must be a string"),
                                    };
                                    deps.insert(name, version);
                                }
                                _ => CType::fail("Node.js dependencies must be declared with dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Javascript *must* be declared Node{{D}} dependencies: {:?}", otherwise))
                        }
                    match &**n {
                        CType::TString(s) => s.clone(),
                        _ => CType::fail("Native import names must be strings"),
                    };
                    Ok(("".to_string(), deps))
                }
                otherwise => CType::fail(&format!(
                    "Bound types must be strings or node.js imports: {:?}",
                    otherwise
                )),
            },
            _ => Ok(("".to_string(), deps)), // TODO: Is this correct?
        },
        CType::Generic(name, ..) => Ok((name.clone(), deps)),
        CType::Binds(n, args) => {
            for arg in args {
                let res = ctype_to_jtype(arg.clone(), deps)?;
                deps = res.1;
            }
            match &**n {
                CType::TString(s) => Ok((s.clone(), deps)),
                CType::Import(n, d) => {
                    match &**d {
                        CType::Type(_, t) => match &**t {
                            CType::Node(d) => match &**d {
                                CType::Dependency(n, v) => {
                                    let name = match &**n {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency name must be a string"),
                                    };
                                    let version = match &**v {
                                        CType::TString(s) => s.clone(),
                                        _ => CType::fail("Dependency version must be a string"),
                                    };
                                    deps.insert(name, version);
                                }
                                _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                            }
                            otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                        }
                        CType::Node(d) => match &**d {
                            CType::Dependency(n, v) => {
                                let name = match &**n {
                                    CType::TString(s) => s.clone(),
                                    _ => CType::fail("Dependency name must be a string"),
                                };
                                let version = match &**v {
                                    CType::TString(s) => s.clone(),
                                    _ => CType::fail("Dependency version must be a string"),
                                };
                                deps.insert(name, version);
                            }
                            _ => CType::fail("Node.js dependencies must be declared with the dependency syntax"),
                        }
                        otherwise => CType::fail(&format!("Native imports compiled to Javascript must be declared Node{{D}} dependencies: {:?}", otherwise))
                    }
                    let native_type = match &**n {
                        CType::TString(s) => s.clone(),
                        _ => CType::fail("Native import names must be strings"),
                    };
                    Ok((native_type, deps))
                }
                otherwise => CType::fail(&format!(
                    "Bound types must be strings or node.js imports: {:?}",
                    otherwise
                )),
            }
        }
        CType::IntrinsicGeneric(..) => Ok(("".to_string(), deps)),
        CType::Int(i) => Ok((i.to_string(), deps)),
        CType::Float(f) => Ok((f.to_string(), deps)),
        CType::Bool(b) => Ok((b.to_string(), deps)),
        CType::TString(s) => Ok((
            s.chars()
                .map(|c| match c {
                    '0'..='9' => c,
                    'a'..='z' => c,
                    'A'..='Z' => c,
                    _ => '_',
                })
                .collect::<String>(),
            deps,
        )),
        CType::Group(g) => {
            let res = ctype_to_jtype(g.clone(), deps)?;
            let s = res.0;
            deps = res.1;
            if !s.is_empty() {
                Ok((format!("({})", s), deps))
            } else {
                Ok(("".to_string(), deps))
            }
        }
        CType::Function(i, o) => {
            let res = ctype_to_jtype(i.clone(), deps)?;
            deps = res.1;
            let res = ctype_to_jtype(o.clone(), deps)?;
            deps = res.1;
            Ok(("".to_string(), deps)) // TODO: What do we even do with this?
        }
        CType::Tuple(ts) => {
            for t in ts {
                let res = ctype_to_jtype(t.clone(), deps)?;
                deps = res.1;
            }
            Ok(("".to_string(), deps))
        }
        CType::Field(_, v) => {
            let res = ctype_to_jtype(v.clone(), deps)?;
            deps = res.1;
            Ok(("".to_string(), deps))
        }
        CType::Either(ts) => {
            for t in ts {
                let res = ctype_to_jtype(t.clone(), deps)?;
                deps = res.1;
            }
            Ok(("".to_string(), deps))
        }
        CType::AnyOf(_) => Ok(("".to_string(), deps)), // Does this make any sense?
        CType::Buffer(t, _) => {
            let res = ctype_to_jtype(t.clone(), deps)?;
            deps = res.1;
            Ok(("".to_string(), deps))
        }
        CType::Array(t) => {
            let res = ctype_to_jtype(t.clone(), deps)?;
            deps = res.1;
            Ok(("".to_string(), deps))
        }
        CType::Fail(m) => CType::fail(m),
        otherwise => CType::fail(&format!("Lower stage of the compiler received unresolved algebraic type {}, cannot deal with it here. Please report this error.", Arc::new(otherwise.clone()).to_functional_string())),
    }
}

pub fn generate(
    typen: Arc<CType>,
    mut out: OrderedHashMap<String, String>,
    deps: OrderedHashMap<String, String>,
) -> Result<
    (
        String,
        OrderedHashMap<String, String>,
        OrderedHashMap<String, String>,
    ),
    Box<dyn std::error::Error>,
> {
    let res = ctype_to_jtype(typen.clone(), deps)?;
    if !res.0.is_empty() {
        out.insert(typen.clone().to_callable_string(), res.0.clone());
    }
    Ok((res.0, out, res.1))
}
