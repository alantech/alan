// TODO: Generics/Interfaces resolution
use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use crate::codegen;
use crate::program::CType;

/// Does this type denote the alan `string` (which lowers to Rust `String`)?
/// Used so a *borrowed* string parameter of a closure type is rendered as the
/// idiomatic `&str` rather than `&String`. An inline closure passed to an
/// `alan_std` generic monomorphized to `String` still type-checks: its params
/// are inferred to `&String` and the `&str`-typed body call coerces.
fn is_string_ctype(t: &CType) -> bool {
    match t {
        CType::Type(n, inner) => n == "string" || is_string_ctype(inner),
        CType::Group(inner) => is_string_ctype(inner),
        CType::Binds(n, _) => matches!(&**n, CType::TString(s) if s == "String"),
        _ => false,
    }
}

/// The borrowed-element rendering for a closure parameter: `str` for a string
/// (so the surrounding `&` yields `&str`), otherwise the type's own rendering.
fn closure_param_form(t: &CType, rendered: String) -> String {
    if is_string_ctype(t) {
        "str".to_string()
    } else {
        rendered
    }
}

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
                            CType::Tuple(ts, _) => {
                                let mut out = Vec::new();
                                for t in ts {
                                    let res = ctype_to_rtype(t.clone(), deps)?;
                                    let s = res.0;
                                    deps = res.1;
                                    out.push(match &**t {
                                        CType::Mut(_) => format!("mut {s}"),
                                        _ => closure_param_form(t, s),
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
                                    _ => closure_param_form(otherwise, s),
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
        CType::Void | CType::DerivedVoid(..) => Ok(("void".to_string(), deps)),
        CType::Infer(s, _) => Err(format!(
            "Inferred type matching {s} was not realized before code generation"
        )
        .into()),
        CType::Type(_, t) => match &**t {
            CType::Either(ts, _) => {
                if codegen::is_option_or_result_either(ts) {
                    return Ok(("".to_string(), deps));
                }
                let mut enum_type_strs = Vec::new();
                for t in ts {
                    let rendered = match &**t {
                        CType::Field(_, v) => {
                            let res = ctype_to_rtype(v.clone(), deps)?;
                            deps = res.1;
                            res.0
                        }
                        CType::Type(_, t) => {
                            let res = ctype_to_rtype(t.clone(), deps)?;
                            deps = res.1;
                            res.0
                        }
                        CType::Group(g) => {
                            let res = ctype_to_rtype(g.clone(), deps)?;
                            deps = res.1;
                            res.0
                        }
                        CType::Void | CType::DerivedVoid(..) => "void".to_string(),
                        CType::Tuple(ts, _) => {
                            let mut out = Vec::new();
                            for tt in ts {
                                let res = ctype_to_rtype(tt.clone(), deps)?;
                                deps = res.1;
                                out.push(res.0);
                            }
                            out.join(", ")
                        }
                        _ => {
                            let res = ctype_to_rtype(t.clone(), deps)?;
                            deps = res.1;
                            res.0
                        }
                    };
                    enum_type_strs.push(codegen::either_variant_to_rust_str(t, rendered));
                }
                let name = t.clone().to_callable_string();
                Ok((format!(
                    "#[derive(Clone)]\nenum {} {{ {} }}",
                    name,
                    enum_type_strs.join(", ")
                ), deps))
            }
                CType::Tuple(ts, _) => {
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
                let base = codegen::render_binds_name(name, |d| super::register_rust_dependency(d, &mut deps));
                let mut out_args = Vec::new();
                for arg in args {
                    let res = ctype_to_rtype(arg.clone(), deps)?;
                    let s = res.0;
                    deps = res.1;
                    out_args.push(s);
                }
                if out_args.is_empty() {
                    Ok((base, deps))
                } else {
                    Ok((format!("{}<{}>", base, out_args.join(", ")), deps))
                }
            }
            _otherwise => ctype_to_rtype(t.clone(), deps),
        }
        CType::Generic(name, args, _) => Ok((format!("{}<{}>", name, args.join(", ")), deps)),
        CType::Binds(n, args) => {
            let base = codegen::render_binds_name(n, |d| super::register_rust_dependency(d, &mut deps));
            let mut out_args = Vec::new();
            for arg in args {
                let res = ctype_to_rtype(arg.clone(), deps)?;
                let s = res.0;
                deps = res.1;
                out_args.push(s);
            }
            if out_args.is_empty() {
                Ok((base, deps))
            } else {
                Ok((format!("{}<{}>", base, out_args.join(", ")), deps))
            }
        }
        CType::Shared(t) => {
            let res = ctype_to_rtype(t.clone(), deps)?;
            let s = res.0;
            deps = res.1;
            Ok((format!("std::sync::Arc<std::sync::RwLock<{s}>>"), deps))
        }
        CType::IntrinsicGeneric(name, _) => Ok((name.clone(), deps)), // How would this even be reached?
        CType::Int(i) => Ok((i.to_string(), deps)),
        CType::Float(f) => Ok((f.to_string(), deps)),
        CType::Bool(b) => Ok((b.to_string(), deps)),
        CType::TString(s) => Ok((codegen::sanitize_ctype_string(s), deps)),
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
                        CType::Tuple(ts, _) => {
                            let mut out = Vec::new();
                            for t in ts {
                                let res = ctype_to_rtype(t.clone(), deps)?;
                                let s = res.0;
                                deps = res.1;
                                out.push(closure_param_form(t, s));
                            }
                            out.join(", &")
                        },
                        otherwise => {
                            let res = ctype_to_rtype(i.clone(), deps)?;
                            let s = res.0;
                            deps = res.1;
                            closure_param_form(otherwise, s)
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
                   CType::Tuple(ts, _) => {
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
        CType::Either(ts, _) => {
            // Special handling to convert `Either{T, void}` to `Option<T>` and `Either{T, Error}`
            // to `Result<T, AlanError>`
            if ts.len() == 2 {
                let alan_error = "alan_std::AlanError".to_string();
                match &*ts[1] {
                    CType::Void | CType::DerivedVoid(..) => {
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
                                super::register_rust_dependency(d, &mut deps);
                                Ok((format!("Result<{}, {}>", s, "alan_std::AlanError"), deps))
                            }
                            _ => Ok((Arc::new(CType::Either(ts.clone(), Vec::new())).to_callable_string(), deps)),
                        }
                        _ => Ok((Arc::new(CType::Either(ts.clone(), Vec::new())).to_callable_string(), deps)),
                    }
                    CType::Type(_, t) => match &**t {
                        CType::Binds(rustname, _) => match &**rustname {
                            CType::Import(n, d) => match &**n {
                                CType::TString(e) if e == &alan_error => {
                                    let res = ctype_to_rtype(ts[0].clone(), deps)?;
                                    let s = res.0;
                                    deps = res.1;
                                    super::register_rust_dependency(d, &mut deps);
                                    Ok((format!("Result<{}, {}>", s, "alan_std::AlanError"), deps))
                                }
                                _ => Ok((Arc::new(CType::Either(ts.clone(), Vec::new())).to_callable_string(), deps)),
                            }
                            _ => Ok((Arc::new(CType::Either(ts.clone(), Vec::new())).to_callable_string(), deps)),
                        }
                        _ => Ok((Arc::new(CType::Either(ts.clone(), Vec::new())).to_callable_string(), deps)),
                    }
                    _ => Ok((Arc::new(CType::Either(ts.clone(), Vec::new())).to_callable_string(), deps)),
                }
            } else {
                for t in ts {
                    // Make sure we add all of the deps, if necessary
                    let res = ctype_to_rtype(t.clone(), deps)?;
                    deps = res.1;
                }
                Ok((Arc::new(CType::Either(ts.clone(), Vec::new())).to_callable_string(), deps))
            }
        }
        // A numeric literal whose `AnyOf` type was never narrowed by context resolves to its FUI
        // default (the last candidate) for codegen. `AnyOf` is otherwise a compile-time-only set.
        CType::AnyOf(_) => ctype_to_rtype(ctype.clone().collapse_anyof_default(), deps),
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
            let base = codegen::render_binds_name(n, |d| super::register_rust_dependency(d, &mut deps));
            let mut genargs = Vec::new();
            for t in ts {
                let res = ctype_to_rtype(t.clone(), deps)?;
                let s = res.0;
                deps = res.1;
                genargs.push(s);
            }
            if genargs.is_empty() {
                Ok((base, out, deps))
            } else {
                Ok((format!("{}<{}>", base, genargs.join(", ")), out, deps))
            }
        }
        // TODO: The complexity of this function indicates more fundamental issues in the type
        // generation. This needs a rethink and rewrite.
        CType::Type(_name, t) => match &**t {
            CType::Either(_, _) => {
                let res = generate(t.clone(), out, deps)?;
                out = res.1;
                deps = res.2;
                let res = ctype_to_rtype(typen.clone(), deps)?;
                let s = res.0;
                deps = res.1;
                let enum_key = t.clone().to_callable_string();
                out.insert(enum_key.clone(), s);
                Ok((enum_key, out, deps))
            }
            _ => {
                let res = ctype_to_rtype(t.clone(), deps)?;
                let s = res.0;
                deps = res.1;

                Ok((s, out, deps))
            }
        },
        CType::Tuple(_, _) => {
            let res = ctype_to_rtype(typen, deps)?;
            let s = res.0;
            deps = res.1;
            Ok((s, out, deps))
        }
        CType::Void | CType::DerivedVoid(..) => {
            out.insert("void".to_string(), "type void = ();".to_string());
            Ok(("()".to_string(), out, deps))
        }
        CType::Either(ts, _) => {
            // Make sure every sub-type exists
            for t in ts {
                let res = generate(t.clone(), out, deps)?;
                out = res.1;
                deps = res.2;
            }

            // Check if this is a 2-variant Either that maps to Option/Result
            if let Some(kind) = codegen::enum_variant_kind(ts) {
                let res = ctype_to_rtype(ts[0].clone(), deps)?;
                deps = res.1;
                let wrapper = match kind {
                    codegen::EnumVariantKind::Option => format!("Option<{}>", res.0),
                    codegen::EnumVariantKind::Result => format!("Result<{}, alan_std::AlanError>", res.0),
                };
                return Ok((wrapper, out, deps));
            }

            // Build the enum definition for 3+ variant Either
            let mut enum_type_strs = Vec::new();
            for t in ts {
                let rendered = match &**t {
                    CType::Field(_, v) => {
                        let res = ctype_to_rtype(v.clone(), deps)?;
                        deps = res.1;
                        res.0
                    }
                    CType::Type(_, t) => {
                        let res = ctype_to_rtype(t.clone(), deps)?;
                        deps = res.1;
                        res.0
                    }
                    CType::Void | CType::DerivedVoid(..) => "void".to_string(),
                    _ => {
                        let res = ctype_to_rtype(t.clone(), deps)?;
                        deps = res.1;
                        res.0
                    }
                };
                enum_type_strs.push(codegen::either_variant_to_rust_str(t, rendered));
            }
            let enum_key = typen.to_callable_string();
            let enum_def = format!(
                "#[derive(Clone)]\nenum {} {{ {} }}",
                enum_key,
                enum_type_strs.join(", ")
            );
            out.insert(enum_key.clone(), enum_def);
            Ok((enum_key, out, deps))
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
