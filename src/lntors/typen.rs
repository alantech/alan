// TODO: Generics/Interfaces resolution
use ordered_hash_map::OrderedHashMap;

use crate::program::CType;

pub fn ctype_to_rtype(
    ctype: &CType,
    in_function_type: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    match ctype {
        CType::Void => Ok("void".to_string()),
        CType::Infer(s, _) => Err(format!(
            "Inferred type matching {} was not realized before code generation",
            s
        )
        .into()),
        CType::Type(_, t) => {
            match &**t {
                CType::Either(ts) => {
                    let mut enum_type_strs = Vec::new();
                    for t in ts {
                        match t {
                            CType::Field(k, v) => {
                                enum_type_strs.push(format!(
                                    "{}({})",
                                    k,
                                    ctype_to_rtype(v, in_function_type)?
                                ));
                            }
                            CType::Type(n, t) => {
                                enum_type_strs.push(format!("{}({})", n, ctype_to_rtype(t, in_function_type)?));
                            }
                            CType::Group(g) => {
                                enum_type_strs.push(ctype_to_rtype(g, in_function_type)?);
                            }
                            CType::Void => enum_type_strs.push("void".to_string()),
                            CType::Tuple(ts) => {
                                let mut out = Vec::new();
                                for t in ts {
                                    out.push(ctype_to_rtype(t, in_function_type)?);
                                }
                                enum_type_strs.push(format!("({}", out.join(", ")));
                            }
                            otherwise => {
                                return Err(format!("TODO: What is this? {:?}", otherwise).into());
                            }
                        }
                    }
                    let name = t.to_callable_string();
                    Ok(format!(
                        "#[derive(Clone)]\nenum {} {{ {} }}",
                        name,
                        enum_type_strs.join(", ")
                    ))
                }
                CType::Tuple(ts) => {
                    let mut out = Vec::new();
                    for t in ts {
                        match t {
                            CType::Field(_, t2) => {
                                if !matches!(&**t2, CType::Int(_) | CType::Float(_) | CType::Bool(_) | CType::TString(_)) {
                                    out.push(ctype_to_rtype(t, in_function_type)?);
                                }
                            }
                            t => out.push(ctype_to_rtype(t, in_function_type)?),
                        }
                    }
                    Ok(format!("({})", out.join(", ")))
                }
                CType::Binds(name, args) => {
                    let mut out_args = Vec::new();
                    for arg in args {
                        out_args.push(ctype_to_rtype(arg, in_function_type)?);
                    }
                    if out_args.is_empty() {
                        Ok(name.clone())
                    } else {
                        Ok(format!("{}<{}>", name, out_args.join(", ")))
                    }
                }
                _ => Ok("".to_string()), // TODO: Is this correct?
            }
        }
        CType::Generic(name, args, _) => Ok(format!("{}<{}>", name, args.join(", "))),
        CType::Binds(name, args) => {
            let mut out_args = Vec::new();
            for arg in args {
                out_args.push(ctype_to_rtype(arg, in_function_type)?);
            }
            if out_args.is_empty() {
                Ok(name.clone())
            } else {
                Ok(format!("{}<{}>", name, out_args.join(", ")))
            }
        }
        CType::IntrinsicGeneric(name, _) => Ok(name.clone()), // How would this even be reached?
        CType::Int(i) => Ok(i.to_string()),
        CType::Float(f) => Ok(f.to_string()),
        CType::Bool(b) => Ok(b.to_string()),
        CType::TString(s) => Ok(s.chars().map(|c| match c {
            '0'..='9' => c,
            'a'..='z' => c,
            'A'..='Z' => c,
            _ => '_',
        }).collect::<String>()),
        CType::Group(g) => Ok(format!("({})", ctype_to_rtype(g, in_function_type)?)),
        CType::Function(i, o) => {
            if let CType::Void = **i {
                Ok(format!("impl Fn() -> {}", ctype_to_rtype(o, true)?))
            } else {
                Ok(format!(
                    "impl Fn(&{}) -> {}",
                    match &**i {
                        CType::Tuple(ts) => {
                            let mut out = Vec::new();
                            for t in ts {
                                out.push(ctype_to_rtype(t, true)?);
                            }
                            out.join(", &")
                        },
                        otherwise => ctype_to_rtype(otherwise, true)?,
                    },
                    ctype_to_rtype(o, true)?
                ))
            }
        },
        CType::Tuple(ts) => {
            let mut out = Vec::new();
            for t in ts {
                match t {
                    CType::Field(_, t2) => {
                        if !matches!(&**t2, CType::Int(_) | CType::Float(_) | CType::Bool(_) | CType::TString(_)) {
                            out.push(ctype_to_rtype(t, in_function_type)?);
                        }
                    }
                    t => out.push(ctype_to_rtype(t, in_function_type)?),
                }
            }
            if out.len() == 1 {
                Ok(format!("({},)", out[0]))
            } else {
                Ok(format!("({})", out.join(", ")))
            }
        }
        CType::Field(k, v) => {
            Ok(format!("/* {} */ {}", k, ctype_to_rtype(v, in_function_type)?))
        }
        CType::Either(ts) => {
            // Special handling to convert `Either{T, void}` to `Option<T>` and `Either{T, Error}`
            // to `Result<T, AlanError>`
            if ts.len() == 2 {
                match &ts[1] {
                    CType::Void => Ok(format!("Option<{}>", ctype_to_rtype(&ts[0], in_function_type)?)),
                    CType::Binds(rustname, _) if rustname == "AlanError" => Ok(format!("Result<{}, {}>", ctype_to_rtype(&ts[0], in_function_type)?, rustname)),
                    CType::Type(_, t) if matches!(&**t, CType::Binds(rustname, _) if rustname == "AlanError") => Ok(format!("Result<{}, {}>", ctype_to_rtype(&ts[0], in_function_type)?, "AlanError")),
                    _ => Ok(CType::Either(ts.clone()).to_callable_string()),
                }
            } else {
                Ok(CType::Either(ts.clone()).to_callable_string())
            }
        }
        CType::AnyOf(_) => Ok("".to_string()), // Does this make any sense in Rust?
        CType::Buffer(t, s) => Ok(format!(
            "[{};{}]",
            ctype_to_rtype(t, in_function_type)?,
            match **s {
                CType::Int(size) => Ok(size as usize),
                _ =>
                    Err("Somehow received a buffer definition with a non-integer size".to_string()),
            }?
        )),
        CType::Array(t) => Ok(format!("Vec<{}>", ctype_to_rtype(t, in_function_type)?)),
        CType::Fail(m) => CType::fail(m),
        otherwise => CType::fail(&format!("Lower stage of the compiler received unresolved algebraic type {}, cannot deal with it here. Please report this error.", otherwise.to_strict_string(false))),
    }
}

pub fn generate(
    typen: &CType,
    mut out: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match &typen {
        // The first value is the identifier, and the second is the generated source. For the
        // `Bind` and `Alias` types, these already exist in the Rust environment (or should exist,
        // assuming no bugs in the standard library) so they do not alter the generated source
        // output, while the `Structlike` type requires a new struct to be created and inserted
        // into the source definition, potentially inserting inner types as needed
        CType::Binds(n, ts) => {
            let mut genargs = Vec::new();
            for t in ts {
                genargs.push(ctype_to_rtype(t, false)?);
            }
            if genargs.is_empty() {
                Ok((n.clone(), out))
            } else {
                Ok((format!("{}<{}>", n, genargs.join(", ")), out))
            }
        }
        // TODO: The complexity of this function indicates more fundamental issues in the type
        // generation. This needs a rethink and rewrite.
        CType::Type(name, t) => match &**t {
            CType::Either(_) => {
                let res = generate(t, out)?;
                out = res.1;
                out.insert(t.to_callable_string(), ctype_to_rtype(typen, false)?);
                Ok((name.clone(), out))
            }
            _ => Ok((ctype_to_rtype(t, true)?, out)),
        },
        CType::Tuple(_) => Ok((ctype_to_rtype(typen, true)?, out)),
        CType::Void => {
            out.insert("void".to_string(), "type void = ();".to_string());
            Ok(("()".to_string(), out))
        }
        CType::Either(ts) => {
            // Make sure every sub-type exists
            for t in ts {
                let res = generate(t, out)?;
                out = res.1;
            }

            let out_str = ctype_to_rtype(typen, false)?;
            Ok((out_str, out)) // TODO: Put something into out?
        }
        CType::Group(g) => {
            let res = generate(g, out)?;
            out = res.1;
            Ok(("".to_string(), out))
        }
        otherwise => {
            let out_str = ctype_to_rtype(otherwise, false)?;
            Ok((out_str, out)) // TODO: Put something into out?
        }
    }
}
