use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use super::Backend;
use crate::program::{CType, FnKind, Function, Microstatement, NativeCallKind, Program, Scope};

/// Resolve a function by its representation name and type from scope, with fallback
/// to the parent function's original scope if the current scope doesn't contain it.
pub fn resolve_function_from_scope<'a>(
    representation: &str,
    typen: Arc<CType>,
    scope: &'a Scope<'a>,
    parent_fn: &Function,
) -> Option<Arc<Function>> {
    let f = scope.resolve_function_by_type(&representation.to_string(), typen.clone());
    let f = match f {
        None => {
            if parent_fn.origin_scope_path != scope.path {
                let program = Program::get_program_guard();
                let out = match program
                    .get_ref()
                    .scope_by_file(&parent_fn.origin_scope_path)
                {
                    Ok(original_scope) => {
                        original_scope.resolve_function_by_type(&representation.to_string(), typen)
                    }
                    Err(_) => None,
                };
                out
            } else {
                None
            }
        }
        f => f,
    };
    f
}

/// Check if a representation matches a function-typed argument of the parent function.
pub fn is_function_arg(parent_fn: &Function, representation: &str) -> bool {
    parent_fn
        .args()
        .iter()
        .any(|(name, _, typen)| name == representation && matches!(&**typen, CType::Function(_, _)))
}

/// Generate the mangled function name used for codegen deduplication.
pub fn mangled_function_name(fun: &Function) -> String {
    let arg_strs = fun
        .args()
        .iter()
        .map(|(_, _, t)| t.clone().to_callable_string())
        .collect::<Vec<String>>();
    format!("{}_{}", fun.name, arg_strs.join("_"))
}

/// Strip `&mut ` prefix from a rendered argument expression (Rust ownership model).
pub fn strip_amp_mut(arg: &str) -> &str {
    arg.strip_prefix("&mut ").unwrap_or(arg)
}

/// Determines whether a 2-variant enum constructor call is an Option or Result mapping.
/// Returns `Some(Option)` if the second variant is `Void`, `Some(Result)` if it's `Error`,
/// and `None` for regular enum constructors.
pub fn enum_variant_kind(ts: &[Arc<CType>]) -> Option<EnumVariantKind> {
    if ts.len() != 2 {
        return None;
    }
    match &*ts[1] {
        CType::Void => Some(EnumVariantKind::Option),
        CType::Type(name, _) if name == "Error" => Some(EnumVariantKind::Result),
        _ => None,
    }
}

pub enum EnumVariantKind {
    Option,
    Result,
}

/// Checks if a given variant `t` is the "empty" sentinel of a 2-variant enum
/// (i.e., the `Void` variant of an `Option` or the `Error` variant of a `Result`).
pub fn is_empty_variant(ts: &[Arc<CType>], t: &Arc<CType>) -> bool {
    match enum_variant_kind(ts) {
        Some(EnumVariantKind::Option) => matches!(&**t, CType::Void),
        Some(EnumVariantKind::Result) => matches!(&**t, CType::Type(name, _) if name == "Error"),
        None => false,
    }
}

/// Handles the common pattern of checking if a variant is a sentinel (Empty) or data (Value)
/// for Option/Result types. Returns `Some(result)` if the type is an Option/Result,
/// otherwise `None` to allow the caller to fall back to default rendering.
/// Handles the common Option/Result sentinel-vs-data pattern.
/// `is_empty` is a closure that determines whether the current variant is the empty one.
/// `on_empty` handles the `None`/`Err` case; `on_value` handles the `Some`/`Ok` case.
/// Accepts a mutable reference to `deps` so the closures can update it (e.g., via
/// `typen::ctype_to_rtype`).  Returns `None` if `ts` is not an Option/Result mapping.
pub fn handle_option_result_symmetry<E, F, G>(
    ts: &[Arc<CType>],
    deps: &mut OrderedHashMap<String, String>,
    is_empty: E,
    on_empty: F,
    on_value: G,
) -> Option<Result<String, Box<dyn std::error::Error>>>
where
    E: FnOnce() -> bool,
    F: FnOnce(
        EnumVariantKind,
        &mut OrderedHashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>>,
    G: FnOnce(
        EnumVariantKind,
        &mut OrderedHashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>>,
{
    let kind = enum_variant_kind(ts)?;
    if is_empty() {
        Some(on_empty(kind, deps))
    } else {
        Some(on_value(kind, deps))
    }
}

/// Try single-expression inlining. Returns `Some(inlined_microstatement)` if the function
/// is a `Normal` function that's marked as an inline target and has a single-return body
/// whose parameters can be substituted by the caller's arguments.
pub fn try_single_inline(function: &Function, args: &[Microstatement]) -> Option<Microstatement> {
    if !matches!(function.kind, crate::program::FnKind::Normal) {
        return None;
    }
    if !crate::program::inline::is_inline_target(&crate::program::inline::fn_identity(function)) {
        return None;
    }
    let subs = crate::program::inline::build_inline_substitution(function, args)?;
    let expr = crate::program::inline::single_return_expr(function)?;
    Some(crate::program::inline::substitute(expr, &subs))
}

/// Try multi-statement inlining. Returns `Some((stmts, tail))` if the function has a
/// multi-statement body (assignments + final return) suitable for block-expression inlining.
/// Inner locals are renamed to avoid shadowing caller variables.
pub fn try_multi_inline(
    function: &Function,
    args: &[Microstatement],
) -> Option<(Vec<Microstatement>, Microstatement)> {
    crate::program::inline::build_multi_inline(function, args)
}

/// Sanitize a `TString` for use as an identifier in the target language:
/// keep alphanumeric characters, replace everything else with `_`.
pub fn sanitize_ctype_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0'..='9' | 'a'..='z' | 'A'..='Z' => c,
            _ => '_',
        })
        .collect()
}

/// Returns true if the type is a "static" field — a primitive (`Int`/`Float`/`Bool`/`TString`)
/// or a `Field` whose inner type is a primitive.  Static fields are handled by the
/// compiler's field accessor logic rather than generated as struct members.
pub fn is_static_field(t: &CType) -> bool {
    match t {
        CType::Field(_, inner) => matches!(
            &**inner,
            CType::Int(_) | CType::Float(_) | CType::Bool(_) | CType::TString(_)
        ),
        CType::Int(_) | CType::Float(_) | CType::Bool(_) | CType::TString(_) => true,
        _ => false,
    }
}

/// Filter iterator adapter that removes static fields from a set of variant types.
pub fn filter_static_fields<'a>(
    ts: impl IntoIterator<Item = &'a Arc<CType>>,
) -> impl Iterator<Item = &'a Arc<CType>> {
    ts.into_iter().filter(|t| !is_static_field(t))
}

/// Shared resolution + dispatch for `CType::Function` values in `from_microstatement`.
/// Resolves the function from scope, falls back to `is_function_arg`, then dispatches
/// to the appropriate backend rendering method based on `FnKind`.
#[allow(clippy::type_complexity)]
pub fn resolve_function_value<B: Backend>(
    representation: &str,
    typen: Arc<CType>,
    scope: &Scope,
    parent_fn: &Function,
    out: OrderedHashMap<String, String>,
    deps: OrderedHashMap<String, String>,
) -> super::CodegenResult<String> {
    let f = resolve_function_from_scope(representation, typen.clone(), scope, parent_fn);
    match &f {
        None => {
            if is_function_arg(parent_fn, representation) {
                return Ok((representation.to_string(), out, deps));
            }
            Err(
                format!("Somehow can't find a definition for function {representation}, {typen:?}")
                    .into(),
            )
        }
        Some(fun) => match &fun.kind {
            FnKind::Normal
            | FnKind::External(_)
            | FnKind::Generic(..)
            | FnKind::Derived
            | FnKind::DerivedVariadic
            | FnKind::Static
            | FnKind::Cfn(..)
            | FnKind::CfnRealized(_) => B::render_function_value(fun, scope, out, deps),
            FnKind::Bind(_)
            | FnKind::BoundGeneric(_, _)
            | FnKind::ExternalBind(_, _)
            | FnKind::ExternalGeneric(_, _, _) => B::render_bind_value(fun, out, deps),
        },
    }
}

/// Render a `CType::Binds` name to a string, handling both `TString` and `Import` variants.
/// The `register_dep` closure is called for `Import` variants to register the native dependency.
pub fn render_binds_name<F: FnOnce(&CType)>(name: &CType, register_dep: F) -> String {
    match name {
        CType::TString(s) => s.clone(),
        CType::Import(n, d) => {
            register_dep(d);
            match &**n {
                CType::TString(s) => s.clone(),
                _ => CType::fail("Native import names must be strings"),
            }
        }
        _ => CType::fail("Bound types must be strings or imports"),
    }
}

/// Check if a 2-variant `Either` represents `Option` or `Result` (early-return shortcut).
/// Returns `true` for `Either{T, void}` or `Either{T, Error}`, in which case the backend
/// should render the `Either` as empty (no custom enum needed).
pub fn is_option_or_result_either(ts: &[Arc<CType>]) -> bool {
    if ts.len() != 2 {
        return false;
    }
    matches!(*ts[1], CType::Void) || matches!(&*ts[1], CType::Type(n, _) if n == "Error")
}

/// Build a Rust-style enum variant string from a `CType::Either` variant.
/// Handles `Field`, `Type`, `Void`, `Tuple`, and fallback cases.
pub fn either_variant_to_rust_str(t: &Arc<CType>, rendered: String) -> String {
    match &**t {
        CType::Field(k, _) => format!("{k}({rendered})"),
        CType::Type(n, _) => format!("{n}({rendered})"),
        CType::Void | CType::DerivedVoid(..) => "void".to_string(),
        _ => format!("{}({rendered})", t.clone().to_callable_string()),
    }
}

/// Build the native call expression string from a `NativeCallKind`, call `name`,
/// and pre-rendered arguments. Both backends share this exact formatting logic.
/// Returns `None` if the backend doesn't support the given `NativeCallKind`
/// (e.g. JS doesn't support `Cast`).
pub fn build_native_call(kind: &NativeCallKind, name: &str, rendered: &[String]) -> Option<String> {
    Some(match kind {
        NativeCallKind::Function => format!("{}({})", name, rendered.join(", ")),
        NativeCallKind::Method => {
            let (recv, rest) = rendered.split_first()?;
            format!("{}.{}({})", recv, name, rest.join(", "))
        }
        NativeCallKind::Property => {
            let (recv, _) = rendered.split_first()?;
            format!("{}.{}", recv, name)
        }
        NativeCallKind::Infix => {
            format!("({} {} {})", rendered[0], name, rendered[1])
        }
        NativeCallKind::Prefix => format!("({} {})", name, rendered[0]),
        NativeCallKind::Cast => format!("({} as {})", rendered[0], name),
    })
}

/// Build the native call expression string for backends that don't support `Cast`.
/// Returns `Err` for `NativeCallKind::Cast`.
pub fn build_native_call_no_cast(
    kind: &NativeCallKind,
    name: &str,
    rendered: &[String],
) -> Result<String, Box<dyn std::error::Error>> {
    match kind {
        NativeCallKind::Cast => Err("native casts have no JavaScript form".into()),
        _ => build_native_call(kind, name, rendered)
            .ok_or_else(|| "NativeCall split_first failed".into()),
    }
}

/// Shared helper to render an array of microstatements.
/// Takes a render function (for backend-specific recursion) and a format closure
/// for the backend-specific array syntax (e.g. `vec![..]` vs `[..]`).
pub fn render_array<F, G>(
    vals: &[Microstatement],
    mut out: OrderedHashMap<String, String>,
    mut deps: OrderedHashMap<String, String>,
    mut render: F,
    format: G,
) -> super::CodegenResult<String>
where
    F: FnMut(
        &Microstatement,
        OrderedHashMap<String, String>,
        OrderedHashMap<String, String>,
    ) -> super::CodegenResult<String>,
    G: FnOnce(&[String]) -> String,
{
    let mut val_representations = Vec::new();
    for val in vals {
        let (rep, o, d) = render(val, out, deps)?;
        val_representations.push(rep);
        out = o;
        deps = d;
    }
    Ok((format(&val_representations), out, deps))
}
