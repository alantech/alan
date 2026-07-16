use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use crate::codegen;
use crate::program::CType;

pub fn ctype_to_jtype(
    ctype: Arc<CType>,
    mut deps: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match &*ctype {
        CType::Mut(t) => ctype_to_jtype(t.clone(), deps),
        CType::Void | CType::DerivedVoid(..) => Ok(("".to_string(), deps)),
        CType::Infer(s, _) => Err(format!(
            "Inferred type matching {s} was not realized before code generation"
        )
        .into()),
        CType::Type(n, t) => match &**t {
        CType::Either(ts, _) => {
                if codegen::is_option_or_result_either(ts) {
                    return Ok(("".to_string(), deps));
                }
                for t in ts {
                    match &**t {
                        CType::Field(_, v) => { deps = ctype_to_jtype(v.clone(), deps)?.1; }
                        CType::Type(_, t) => { deps = ctype_to_jtype(t.clone(), deps)?.1; }
                        CType::Group(g) => { deps = ctype_to_jtype(g.clone(), deps)?.1; }
                        CType::Void | CType::DerivedVoid(..) => {}
                        CType::Tuple(ts, _) => { for tt in ts { deps = ctype_to_jtype(tt.clone(), deps)?.1; } }
                        CType::Array(t) => { deps = ctype_to_jtype(t.clone(), deps)?.1; }
                        CType::Binds(..) => { deps = ctype_to_jtype(t.clone(), deps)?.1; }
                        otherwise => { return Err(format!("TODO: What is this? {otherwise:?}").into()); }
                    }
                }
                Ok(("".to_string(), deps))
            }
            CType::Tuple(ts, _) => {
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
                            out.push(format!("arg{i}"));
                        }
                    }
                }
                Ok((
                    format!(
                        "class {} {{\n  constructor({}) {{\n    {}\n  }}\n}}",
                        n.replace("\"", "_"), // TODO: How is this happening?
                        out.join(", "),
                        out.iter()
                            .map(|s| format!("    this.{s} = {s};"))
                            .collect::<Vec<String>>()
                            .join("\n")
                    ),
                    deps,
                ))
            }
            CType::Binds(name, _) => {
                let _ = codegen::render_binds_name(name, |d| super::register_nodejs_dependency(d, &mut deps));
                Ok(("".to_string(), deps))
            }
            _ => Ok(("".to_string(), deps)), // TODO: Is this correct?
        },
        CType::Generic(name, ..) => Ok((name.clone(), deps)),
        CType::Binds(n, args) => {
            for arg in args {
                let res = ctype_to_jtype(arg.clone(), deps)?;
                deps = res.1;
            }
            let base = codegen::render_binds_name(n, |d| super::register_nodejs_dependency(d, &mut deps));
            Ok((base, deps))
        }
        CType::IntrinsicGeneric(..) => Ok(("".to_string(), deps)),
        CType::Int(i) => Ok((i.to_string(), deps)),
        CType::Float(f) => Ok((f.to_string(), deps)),
        CType::Bool(b) => Ok((b.to_string(), deps)),
        CType::TString(s) => Ok((codegen::sanitize_ctype_string(s), deps)),
        CType::Group(g) => {
            let res = ctype_to_jtype(g.clone(), deps)?;
            let s = res.0;
            deps = res.1;
            if !s.is_empty() {
                Ok((format!("({s})"), deps))
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
        CType::Tuple(ts, _) => {
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
        CType::Either(ts, _) => {
            for t in ts {
                let res = ctype_to_jtype(t.clone(), deps)?;
                deps = res.1;
            }
            Ok(("".to_string(), deps))
        }
        // A numeric literal whose `AnyOf` type was never narrowed by context resolves to its FUI
        // default (the last candidate) for codegen. `AnyOf` is otherwise a compile-time-only set.
        CType::AnyOf(_) => ctype_to_jtype(ctype.clone().collapse_anyof_default(), deps),
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
        CType::Shared(t) => {
            let res = ctype_to_jtype(t.clone(), deps)?;
            deps = res.1;
            Ok(("".to_string(), deps))
        }
        CType::Promise(t) => {
            let res = ctype_to_jtype(t.clone(), deps)?;
            deps = res.1;
            Ok(("".to_string(), deps))
        }
        CType::Fail(m) => CType::fail(m),
        otherwise => CType::fail(&format!("Lower stage of the compiler received unresolved algebraic type {}, cannot deal with it here. Please report this error.", Arc::new(otherwise.clone()).to_functional_string())),
    }
}

#[allow(clippy::type_complexity)]
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
