// TODO: Generics/Interfaces resolution
use ordered_hash_map::OrderedHashMap;

use crate::program::{CType, Program, Scope};

pub fn ctype_to_rtype(
    ctype: &CType,
    scope: &Scope,
    program: &Program,
    in_function_type: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    match ctype {
        CType::Void => Ok("()".to_string()),
        CType::Type(name, t) => {
            match &**t {
                CType::Either(ts) => {
                    let mut enum_type_strs = Vec::new();
                    for t in ts {
                        match t {
                            CType::Field(k, v) => {
                                enum_type_strs.push(format!(
                                    "{}({})",
                                    k,
                                    ctype_to_rtype(&v, scope, program, in_function_type)?
                                ));
                            }
                            CType::Type(n, _) | CType::ResolvedBoundGeneric(n, ..) => {
                                enum_type_strs.push(format!("{}({})", n, n));
                            }
                            CType::Bound(n, r) => {
                                enum_type_strs.push(format!("{}({})", n, r));
                            }
                            otherwise => {
                                return Err(format!("TODO: What is this? {:?}", otherwise).into());
                            }
                        }
                    }
                    Ok(format!("enum {} {{ {} }}", name, enum_type_strs.join(", ")))
                }
                _ => Ok(name.clone()), // TODO: Any others?
            }
        }
        CType::Generic(name, args, _) => Ok(format!("{}<{}>", name, args.join(", "))),
        CType::Bound(_, name) => Ok(name.clone()),
        CType::BoundGeneric(name, args, _) => Ok(format!("{}<{}>", name, args.join(", "))),
        CType::ResolvedBoundGeneric(_name, argstrs, args, binding) => {
            let mut args_rtype = Vec::new();
            for arg in args {
                args_rtype.push(ctype_to_rtype(&arg, scope, program, in_function_type)?);
            }
            // TODO: Get a real Rust type parser and do this better
            let mut out_str = binding.clone();
            for i in 0..argstrs.len() {
                out_str = out_str.replace(&argstrs[i], &args_rtype[i]);
            }
            Ok(out_str.to_string())
        }
        CType::IntrinsicGeneric(name, _) => Ok(name.clone()), // How would this even be reached?
        CType::Int(i) => Ok(i.to_string()),
        CType::Float(f) => Ok(f.to_string()),
        CType::Bool(b) => Ok(b.to_string()),
        CType::TString(s) => Ok(s.clone()),
        CType::Group(g) => Ok(format!(
            "({})",
            ctype_to_rtype(g, scope, program, in_function_type)?
        )),
        CType::Function(i, o) => Ok(format!(
            "fn({}) -> {}",
            ctype_to_rtype(i, scope, program, true)?,
            ctype_to_rtype(o, scope, program, true)?
        )),
        CType::Tuple(ts) => {
            let mut out = Vec::new();
            for t in ts {
                out.push(ctype_to_rtype(t, scope, program, in_function_type)?);
            }
            Ok(format!("({})", out.join(", ")))
        }
        CType::Field(k, v) => {
            if in_function_type {
                Ok(ctype_to_rtype(v, scope, program, in_function_type)?)
            } else {
                Ok(format!(
                    "{}: {}",
                    k,
                    ctype_to_rtype(v, scope, program, in_function_type)?
                ))
            }
        }
        CType::Either(_) => Ok("".to_string()), // What to do in this case?
        CType::Buffer(t, s) => Ok(format!(
            "[{};{}]",
            ctype_to_rtype(t, scope, program, in_function_type)?,
            s
        )),
        CType::Array(t) => Ok(format!(
            "Vec<{}>",
            ctype_to_rtype(t, scope, program, in_function_type)?
        )),
    }
}

pub fn generate(
    typen: &CType,
    scope: &Scope,
    program: &Program,
    mut out: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match &typen {
        // The first value is the identifier, and the second is the generated source. For the
        // `Bind` and `Alias` types, these already exist in the Rust environment (or should exist,
        // assuming no bugs in the standard library) so they do not alter the generated source
        // output, while the `Structlike` type requires a new struct to be created and inserted
        // into the source definition, potentially inserting inner types as needed
        CType::Bound(_name, rtype) => Ok((rtype.clone(), out)),
        CType::Type(name, _) => {
            out.insert(name.clone(), ctype_to_rtype(typen, scope, program, false)?);
            Ok((name.clone(), out))
        }
        CType::Void => Ok(("()".to_string(), out)),
        otherwise => {
            let out_str = ctype_to_rtype(&otherwise, scope, program, false)?;
            Ok((out_str, out)) // TODO: Put something into out?
        }
    }
}
