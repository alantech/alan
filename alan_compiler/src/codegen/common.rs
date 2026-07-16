use std::sync::Arc;

use crate::program::{CType, Function, Microstatement, Program, Scope};

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
