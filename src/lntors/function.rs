// Builds a function and everything it needs, recursively. Given a read-only handle on it's own
// scope and the program in case it needs to generate required text from somewhere else.
use ordered_hash_map::OrderedHashMap;

use crate::lntors::typen;
use crate::program::{CType, FnKind, Function, Microstatement, Scope};

pub fn from_microstatement(
    microstatement: &Microstatement,
    scope: &Scope,
    mut out: OrderedHashMap<String, String>,
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn std::error::Error>> {
    match microstatement {
        Microstatement::Arg { name, typen } => {
            // TODO: Update the serialization logic to understand values vs references so we can
            // eliminate this useless (and harmful for mutable references) clone
            if let CType::Function { .. } = typen {
                Ok(("".to_string(), out))
            } else {
                Ok((
                    format!("let mut {} = {}.clone()", name, name), // TODO: not always mutable
                    out,
                ))
            }
        }
        Microstatement::Assignment {
            name,
            value,
            mutable: _,
        } => {
            let (val, o) = from_microstatement(value, scope, out)?;
            // I wish I didn't have to write the following line because you can't re-assign a
            // variable in a let destructuring, afaict
            out = o;
            Ok((
                format!(
                    "let {}{} = {}",
                    // TODO: Shouldn't always be mut
                    "mut ",
                    name,
                    val,
                )
                .to_string(),
                out,
            ))
        }
        Microstatement::Closure { function } => {
            let arg_names = function
                .args
                .iter()
                .map(|(n, _)| n.clone())
                .collect::<Vec<String>>();
            let mut inner_statements = Vec::new();
            for ms in &function.microstatements {
                let (val, o) = from_microstatement(ms, scope, out)?;
                out = o;
                inner_statements.push(val);
            }
            Ok((
                format!(
                    "move |{}| {{\n        {};\n    }}",
                    arg_names.join(", "),
                    inner_statements.join(";\n        ")
                ),
                out,
            ))
        }
        Microstatement::Value {
            typen,
            representation,
        } => match &typen {
            CType::Bound(a, _) if a == "string" => {
                Ok((format!("{}.to_string()", representation).to_string(), out))
            }
            CType::Function(..) => {
                // We need to make sure this function we're referencing exists
                let f = scope.resolve_function_by_type(representation, typen);
                match f {
                    None => Err(format!(
                        "Somehow can't find a definition for function {}",
                        representation
                    )
                    .into()),
                    Some(fun) => {
                        match &fun.kind {
                            FnKind::Normal
                            | FnKind::Generic(..)
                            | FnKind::Derived
                            | FnKind::DerivedVariadic => {
                                let mut arg_strs = Vec::new();
                                for arg in &fun.args {
                                    match typen::ctype_to_rtype(&arg.1, false) {
                                        Err(e) => Err(e),
                                        Ok(s) => {
                                            arg_strs.push(
                                                s.replace(
                                                    ['<', '>', ',', '[', ']', ';', '-', '(', ')'],
                                                    "_",
                                                )
                                                .replace(' ', ""),
                                            );
                                            /* TODO: Handle generic types better, also type inference */
                                            Ok(())
                                        }
                                    }?;
                                }
                                // Come up with a function name that is unique so Rust doesn't choke on
                                // duplicate function names that are allowed in Alan
                                let rustname =
                                    format!("{}_{}", fun.name, arg_strs.join("_")).to_string();
                                // Make the function we need, but with the name we're
                                out = generate(rustname.clone(), fun, scope, out)?;
                                Ok((rustname, out))
                            }
                            FnKind::Bind(rustname) | FnKind::BoundGeneric(_, rustname) => {
                                Ok((rustname.clone(), out))
                            }
                        }
                    }
                }
            }
            _ => Ok((representation.clone(), out)),
        },
        Microstatement::Array { vals, .. } => {
            let mut val_representations = Vec::new();
            for val in vals {
                let (rep, o) = from_microstatement(val, scope, out)?;
                val_representations.push(rep);
                out = o;
            }
            Ok((
                format!("vec![{}]", val_representations.join(", ")).to_string(),
                out,
            ))
        }
        Microstatement::FnCall { function, args } => {
            let mut arg_types = Vec::new();
            let mut arg_type_strs = Vec::new();
            for arg in args {
                let arg_type = arg.get_type();
                let (_, o) = typen::generate(&arg_type, out)?;
                out = o;
                arg_types.push(arg_type.clone());
                match typen::ctype_to_rtype(&arg_type, false) {
                    Err(e) => Err(e),
                    Ok(s) => {
                        arg_type_strs.push(s);
                        Ok(())
                    }
                }?
            }
            match &function.kind {
                FnKind::Generic(..) | FnKind::BoundGeneric(..) => {
                    Err("Generic functions should have been resolved before reaching here".into())
                }
                FnKind::Normal => {
                    let (_, o) = typen::generate(&function.rettype, out)?;
                    out = o;
                    let mut arg_strs = Vec::new();
                    for arg in &function.args {
                        match typen::ctype_to_rtype(&arg.1, false) {
                            Err(e) => Err(e),
                            Ok(s) => {
                                arg_strs.push(
                                    s.replace(['<', '>', ',', '[', ']', ';', '-', '(', ')'], "_")
                                        .replace(' ', ""),
                                );
                                /* TODO: Handle generic types better, also type inference */
                                Ok(())
                            }
                        }?;
                    }
                    // Come up with a function name that is unique so Rust doesn't choke on
                    // duplicate function names that are allowed in Alan
                    let rustname = format!("{}_{}", function.name, arg_strs.join("_")).to_string();
                    // Make the function we need, but with the name we're
                    out = generate(rustname.clone(), function, scope, out)?;
                    // Now call this function
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o) = from_microstatement(arg, scope, out)?;
                        out = o;
                        // If the argument is itself a function, this is the only place in Rust
                        // where you can't pass by reference, so we check the type and change
                        // the argument output accordingly.
                        let arg_type = arg.get_type();
                        match arg_type {
                            CType::Function(..) => argstrs.push(a.to_string()),
                            _ => argstrs.push(format!("&mut {}", a)),
                        }
                    }
                    Ok((
                        format!("{}({})", rustname, argstrs.join(", ")).to_string(),
                        out,
                    ))
                }
                FnKind::Bind(rustname) => {
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o) = from_microstatement(arg, scope, out)?;
                        out = o;
                        // If the argument is itself a function, this is the only place in Rust
                        // where you can't pass by reference, so we check the type and change
                        // the argument output accordingly.
                        let arg_type = arg.get_type();
                        match arg_type {
                            CType::Function(..) => argstrs.push(a.to_string()),
                            _ => argstrs.push(format!("&mut {}", a)),
                        }
                    }
                    Ok((
                        format!("{}({})", rustname, argstrs.join(", ")).to_string(),
                        out,
                    ))
                }
                FnKind::Derived | FnKind::DerivedVariadic => {
                    // The initial work to get the values to construct the type is the same as
                    // with bound functions, though.
                    let (_, o) = typen::generate(&function.rettype, out)?;
                    out = o;
                    let mut argstrs = Vec::new();
                    for arg in args {
                        let (a, o) = from_microstatement(arg, scope, out)?;
                        out = o;
                        // If the argument is itself a function, this is the only place in Rust
                        // where you can't pass by reference, so we check the type and change
                        // the argument output accordingly.
                        let arg_type = arg.get_type();
                        match arg_type {
                            CType::Function(..) => argstrs.push(a.to_string()),
                            _ => argstrs.push(format!("&mut {}", a)),
                        }
                    }
                    // The behavior of the generated code depends on the structure of the
                    // return type and the input types. We also do some logic based on the name
                    // of the function.
                    // 1) If the name of the function matches the name of return type, it's a
                    //    constructor function, and will interpret the arguments in different
                    //    ways:
                    //    a) If the return type is a Buffer, the arg count must be either the
                    //       size of the buffer with all args having the same type *or* it must
                    //       be exactly 1, with the arg matching the buffer's primary type that
                    //       the buffer will be filled with. In case someone creates a
                    //       one-element buffer, well, those two definitions are the same so it
                    //       will use the first implementation (as it will be faster).
                    //    b) If the return type is an Array, any number of values can be
                    //       provided and it will pre-populate the array with those values.
                    //    c) If the return type is an Either, it will expect only *one*
                    //       argument, and fail otherwise. The argument needs to be one of the
                    //       possibilities, which it will then put into the correct enum. An
                    //       earlier stage of the compiler should have generated function
                    //       definitions for each type in the Either.
                    //    d) If the return type is a tuple type, each argument of the function
                    //       needs to match, in the same order, the tuple's types. It doesn't
                    //       matter if the type itself has fields with names, those are ignored
                    //       and they're all turned into tuples.
                    //    e) If the return type is a group type or "type" type, it's unwrapped
                    //       and checked if it is one of the types above.
                    //    f) If it's any other type, it's a compiler error. There's no way to
                    //       derive an implementation for them that would be sensical.
                    // 2) If the input type is a tuple and the name of the function matches the
                    //    name of a field in the tuple, it's an accessor function.
                    // 3) If the input type is an either and the name of the function matches
                    //    the name of a sub-type, it returns a Maybe{T} for the type in
                    //    question. (This conflicts with (1) so it's checked first.)
                    if function.args.len() == 1 {
                        // This is a wacky unwrapping logic...
                        let mut input_type = &function.args[0].1;
                        while matches!(input_type, CType::Type(..) | CType::Group(_)) {
                            input_type = match input_type {
                                CType::Type(_, t) => t,
                                CType::Group(t) => t,
                                t => t,
                            };
                        }
                        match input_type {
                            CType::Tuple(ts) => {
                                // Short-circuit for direct `<N>` function calls (which can only be
                                // generated by the internals of the compiler)
                                if let Ok(i) = function.name.parse::<i64>() {
                                    return Ok((format!("{}.{}", argstrs[0], i), out));
                                }
                                let accessor_field = ts.iter().enumerate().find(|(_, t)| match t {
                                    CType::Field(n, _) => *n == function.name,
                                    _ => false,
                                });
                                if let Some((i, _)) = accessor_field {
                                    return Ok((format!("{}.{}", argstrs[0], i), out));
                                }
                            }
                            CType::Either(ts) => {
                                // The kinds of types allowed here are `Type`, `Bound`, and
                                // `ResolvedBoundGeneric`, and `Field`. Other types don't have
                                // a string name we can match against the function name
                                let accessor_field = ts.iter().find(|t| match t {
                                    CType::Field(n, _) => *n == function.name,
                                    CType::Type(n, _) => *n == function.name,
                                    CType::Bound(n, _) => *n == function.name,
                                    CType::ResolvedBoundGeneric(n, ..) => *n == function.name,
                                    _ => false,
                                });
                                // We're assuming the enum sub-type naming scheme also follows
                                // the convention of matching the type name or field name,
                                // which works because we're generating all of the code that
                                // defines the enums. We also need the name of the enum for
                                // this to work, so we're assuming we got it from the first
                                // function argument. We blow up here if the first argument is
                                // *not* a Type we can get an enum name from (it *shouldn't* be
                                // possible, but..)
                                let enum_type = function.args[0].1.degroup();
                                let enum_name = enum_type.to_callable_string();
                                // We pass through to the main path if we can't find a matching
                                // name
                                if accessor_field.is_some() {
                                    // Special-casing for Option and Result mapping. TODO:
                                    // Make this more centralized
                                    if ts.len() == 2 {
                                        if let CType::Void = &ts[1] {
                                            return Ok((argstrs[0].clone(), out));
                                        } else if let CType::Bound(name, _) = &ts[1] {
                                            if name == "Error" {
                                                if function.name == "Error" {
                                                    return Ok((format!("(match {} {{ Err(e) => Some(e.clone()), _ => None }})", argstrs[0]), out));
                                                } else {
                                                    return Ok((format!("(match {} {{ Ok(v) => Some(v.clone()), _ => None }})", argstrs[0]), out));
                                                }
                                            }
                                        }
                                    }
                                    return Ok((
                                        format!(
                                            "(match {} {{ {}::{}(v) => Some(v.clone()), _ => None }})",
                                            argstrs[0], enum_name, function.name
                                        ),
                                        out,
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                    let ret_type = &function.rettype.degroup();
                    let ret_name = ret_type.to_callable_string();
                    if function.name == ret_name {
                        let inner_ret_type = match ret_type {
                            CType::Field(_, t) => *t.clone(),
                            CType::Type(_, t) => *t.clone(),
                            t => t.clone(),
                        };
                        match inner_ret_type {
                            CType::Buffer(_, s) => {
                                let size = match *s {
                                    CType::Int(s) => Ok(s as usize),
                                    _ => Err("Somehow received a buffer with a non-integer size"
                                        .to_string()),
                                }?;
                                if argstrs.len() == size {
                                    return Ok((
                                        format!(
                                            "[{}]",
                                            argstrs
                                                .iter()
                                                .map(|a| match a.strip_prefix("&mut ") {
                                                    Some(v) => v,
                                                    None => a,
                                                })
                                                .collect::<Vec<&str>>()
                                                .join(", ")
                                        ),
                                        out,
                                    ));
                                } else if argstrs.len() == 1 {
                                    return Ok((
                                        format!(
                                            "[{};{}]",
                                            match argstrs[0].strip_prefix("&mut ") {
                                                Some(v) => v,
                                                None => &argstrs[0],
                                            },
                                            size
                                        ),
                                        out,
                                    ));
                                } else {
                                    return Err(format!("Invalid arguments {} provided for Buffer constructor function, must be either 1 element to fill, or the full size of the buffer", argstrs.join(", ")).into());
                                }
                            }
                            CType::Array(_) => {
                                return Ok((
                                    format!(
                                        "vec![{}]",
                                        argstrs
                                            .iter()
                                            .map(|a| match a.strip_prefix("&mut ") {
                                                Some(v) => v.to_string(),
                                                None => a.clone(),
                                            })
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                    ),
                                    out,
                                ));
                            }
                            CType::Either(ts) => {
                                if argstrs.len() != 1 {
                                    return Err(format!("Invalid arguments {} provided for Either constructor function, must be only one argument", argstrs.join(", ")).into());
                                }
                                let enum_type = &function.args[0].1.degroup();
                                let enum_name = match enum_type {
                                CType::Field(n, _) => Ok(n.clone()),
                                CType::Type(n, _) => Ok(n.clone()),
                                CType::Bound(n, _) => Ok(n.clone()),
                                CType::ResolvedBoundGeneric(n, ..) => Ok(n.clone()),
                                _ => Err(format!("Cannot generate an constructor function for {} type as the input type has no name?", function.name)),
                            }?;
                                for t in &ts {
                                    let inner_type = t.degroup();
                                    match &inner_type {
                                        CType::Field(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
                                                        return Ok(("None".to_string(), out));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                        ));
                                                    }
                                                } else if let CType::Bound(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        if let CType::Bound(..) = t {
                                                            return Ok((
                                                                format!(
                                                                    "Err({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "Ok({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{}::{}({})",
                                                    function.name,
                                                    enum_name,
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                ),
                                                out,
                                            ));
                                        }
                                        CType::Type(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
                                                        return Ok(("None".to_string(), out));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                        ));
                                                    }
                                                } else if let CType::Bound(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        if let CType::Bound(..) = t {
                                                            return Ok((
                                                                format!(
                                                                    "Err({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "Ok({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{}::{}({})",
                                                    function.name,
                                                    enum_name,
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                ),
                                                out,
                                            ));
                                        }
                                        CType::Bound(n, _) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
                                                        return Ok(("None".to_string(), out));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                        ));
                                                    }
                                                } else if let CType::Bound(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        if let CType::Bound(..) = t {
                                                            return Ok((
                                                                format!(
                                                                    "Err({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "Ok({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{}::{}({})",
                                                    function.name,
                                                    enum_name,
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                ),
                                                out,
                                            ));
                                        }
                                        CType::ResolvedBoundGeneric(n, ..) if *n == enum_name => {
                                            // Special-casing for Option and Result mapping. TODO:
                                            // Make this more centralized
                                            if ts.len() == 2 {
                                                if let CType::Void = &ts[1] {
                                                    if let CType::Void = t {
                                                        return Ok(("None".to_string(), out));
                                                    } else {
                                                        return Ok((
                                                            format!(
                                                                "Some({})",
                                                                match argstrs[0]
                                                                    .strip_prefix("&mut ")
                                                                {
                                                                    Some(s) => s,
                                                                    None => &argstrs[0],
                                                                }
                                                            ),
                                                            out,
                                                        ));
                                                    }
                                                } else if let CType::Bound(name, _) = &ts[1] {
                                                    if name == "Error" {
                                                        if let CType::Bound(..) = t {
                                                            return Ok((
                                                                format!(
                                                                    "Err({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                            ));
                                                        } else {
                                                            return Ok((
                                                                format!(
                                                                    "Ok({})",
                                                                    match argstrs[0]
                                                                        .strip_prefix("&mut ")
                                                                    {
                                                                        Some(s) => s,
                                                                        None => &argstrs[0],
                                                                    }
                                                                ),
                                                                out,
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                            return Ok((
                                                format!(
                                                    "{}::{}({})",
                                                    function.name,
                                                    enum_name,
                                                    match argstrs[0].strip_prefix("&mut ") {
                                                        Some(s) => s,
                                                        None => &argstrs[0],
                                                    },
                                                ),
                                                out,
                                            ));
                                        }
                                        _ => {}
                                    }
                                }
                                return Err(format!("Cannot generate a constructor function for {} type as it is not part of the {} type", enum_name, function.name).into());
                            }
                            CType::Tuple(ts) => {
                                // TODO: Better type checking here, but it's *probably* being
                                // done at a higher layer
                                if argstrs.len() == ts.len() {
                                    return Ok((
                                        format!(
                                            "({})",
                                            argstrs
                                                .iter()
                                                .map(|a| match a.strip_prefix("&mut ") {
                                                    Some(s) => s.to_string(),
                                                    None => a.to_string(),
                                                })
                                                .collect::<Vec<String>>()
                                                .join(", ")
                                        ),
                                        out,
                                    ));
                                } else {
                                    return Err(format!(
                                        "{} has {} fields but {} provided",
                                        function.name,
                                        ts.len(),
                                        argstrs.len()
                                    )
                                    .into());
                                }
                            }
                            CType::Bound(_, _) => {
                                // TODO: Is this the right thing to do for aliases to bound
                                // types in all cases?
                                return Ok((argstrs.join(", "), out));
                            }
                            otherwise => {
                                return Err(format!("How did you get here? Trying to create a constructor function for {:?}", otherwise).into());
                            }
                        }
                    }
                    Err(format!(
                        "Trying to create an automatic function for {} but the return type is {}",
                        function.name, ret_name
                    )
                    .into())
                }
            }
        }
        Microstatement::VarCall { name, args, .. } => {
            let mut argstrs = Vec::new();
            for arg in args {
                let (a, o) = from_microstatement(arg, scope, out)?;
                out = o;
                // If the argument is itself a function, this is the only place in Rust
                // where you can't pass by reference, so we check the type and change
                // the argument output accordingly.
                let arg_type = arg.get_type();
                match arg_type {
                    CType::Function(..) => argstrs.push(a.to_string()),
                    _ => argstrs.push(format!("&mut {}", a)),
                }
            }
            Ok((format!("{}({})", name, argstrs.join(", ")).to_string(), out))
        }
        Microstatement::Return { value } => match value {
            Some(val) => {
                let (retval, o) = from_microstatement(val, scope, out)?;
                out = o;
                Ok((
                    format!(
                        "return {}",
                        match retval.strip_prefix("&mut ") {
                            Some(v) => v,
                            None => &retval,
                        }
                    )
                    .to_string(),
                    out,
                ))
            }
            None => Ok(("return".to_string(), out)),
        },
    }
}

pub fn generate(
    rustname: String,
    function: &Function,
    scope: &Scope,
    mut out: OrderedHashMap<String, String>,
) -> Result<OrderedHashMap<String, String>, Box<dyn std::error::Error>> {
    let mut fn_string = "".to_string();
    // First make sure all of the function argument types are defined
    let mut arg_strs = Vec::new();
    for arg in &function.args {
        let (l, t) = arg;
        let (t_str, o) = typen::generate(t, out)?;
        out = o;
        if t_str.starts_with("impl") {
            arg_strs.push(format!("{}: {}", l, t_str));
        } else {
            arg_strs.push(format!("{}: &{}", l, t_str));
        }
    }
    let opt_ret_str = match &function.rettype.degroup() {
        CType::Void => None,
        CType::Type(n, _) if n == "void" => None,
        otherwise => {
            let (t_str, o) = typen::generate(otherwise, out)?;
            out = o;
            Some(t_str)
        }
    };
    // Start generating the function output. We can do this eagerly like this because, at least for
    // now, we inline all other function calls within an "entry" function (the main function, or
    // any function that's attached to an event, or any function that's part of an exported set in
    // a shared library). LLVM *probably* doesn't deduplicate this redundancy, so this will need to
    // be revisited, but it eliminates a whole host of generation problems that I can come back to
    // later.
    fn_string = format!(
        "{}fn {}({}){} {{\n",
        fn_string,
        rustname.clone(),
        arg_strs.join(", "),
        match opt_ret_str {
            Some(rettype) => format!(" -> {}", rettype).to_string(),
            None => "".to_string(),
        },
    )
    .to_string();
    for microstatement in &function.microstatements {
        let (stmt, o) = from_microstatement(microstatement, scope, out)?;
        out = o;
        fn_string = format!("{}    {};\n", fn_string, stmt);
    }
    fn_string = format!("{}}}", fn_string);
    out.insert(rustname, fn_string);
    Ok(out)
}
