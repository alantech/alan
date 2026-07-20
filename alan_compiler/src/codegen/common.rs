use std::sync::Arc;

use ordered_hash_map::OrderedHashMap;

use super::Backend;
use crate::program::{CType, FnKind, Function, Microstatement, NativeCallKind, Program, Scope};

/// Shared `render_bind_value`: register external dep, return bind name.
pub fn shared_render_bind_value(
    fun: &Arc<Function>,
    out: OrderedHashMap<String, String>,
    mut deps: OrderedHashMap<String, String>,
    register_dep: impl FnOnce(&CType, &mut OrderedHashMap<String, String>),
) -> super::CodegenResult<String> {
    if let FnKind::ExternalGeneric(_, _, d) | FnKind::ExternalBind(_, d) = &fun.kind {
        register_dep(d, &mut deps);
    }
    match &fun.kind {
        FnKind::Bind(name)
        | FnKind::BoundGeneric(_, name)
        | FnKind::ExternalBind(name, _)
        | FnKind::ExternalGeneric(_, name, _) => Ok((name.clone(), out, deps)),
        _ => Err("render_bind_value called on non-bind function kind".into()),
    }
}

/// Resolve function from scope, with fallback to parent's origin scope.
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

/// Does representation match a function-typed parent arg?
pub fn is_function_arg(parent_fn: &Function, representation: &str) -> bool {
    parent_fn
        .args()
        .iter()
        .any(|(name, _, typen)| name == representation && matches!(&**typen, CType::Function(_, _)))
}

/// Mangled function name for codegen deduplication.
pub fn mangled_function_name(fun: &Function) -> String {
    let arg_strs = fun
        .args()
        .iter()
        .map(|(_, _, t)| t.clone().to_callable_string())
        .collect::<Vec<String>>();
    format!("{}_{}", fun.name, arg_strs.join("_"))
}

/// Strip `&mut ` prefix.
pub fn strip_amp_mut(arg: &str) -> &str {
    arg.strip_prefix("&mut ").unwrap_or(arg)
}

/// Is a 2-variant enum `Option` (void sentinel) or `Result` (Error sentinel)?
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

/// Is variant `t` the empty sentinel of an Option/Result?
pub fn is_empty_variant(ts: &[Arc<CType>], t: &Arc<CType>) -> bool {
    match enum_variant_kind(ts) {
        Some(EnumVariantKind::Option) => matches!(&**t, CType::Void),
        Some(EnumVariantKind::Result) => matches!(&**t, CType::Type(name, _) if name == "Error"),
        None => false,
    }
}

/// Option/Result sentinel-vs-data handler. Returns `None` if not Option/Result.
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

/// Try single-expression inline: substitute params with caller args.
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

/// Try multi-statement inline: returns (stmts, tail) for block-expression inlining.
pub fn try_multi_inline(
    function: &Function,
    args: &[Microstatement],
) -> Option<(Vec<Microstatement>, Microstatement)> {
    crate::program::inline::build_multi_inline(function, args)
}

/// Sanitize for identifier: keep alphanumeric, replace rest with `_`.
pub fn sanitize_ctype_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0'..='9' | 'a'..='z' | 'A'..='Z' => c,
            _ => '_',
        })
        .collect()
}

/// Is a primitive or `Field` wrapping a primitive (handled by field accessor logic)?
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

/// Resolve function value and dispatch to backend render based on FnKind.
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

/// Render Binds name; register dep for Import variants.
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

/// Is a 2-variant Either an Option or Result?
pub fn is_option_or_result_either(ts: &[Arc<CType>]) -> bool {
    if ts.len() != 2 {
        return false;
    }
    matches!(*ts[1], CType::Void) || matches!(&*ts[1], CType::Type(n, _) if n == "Error")
}

/// Build Rust enum variant string from Either variant.
pub fn either_variant_to_rust_str(t: &Arc<CType>, rendered: String) -> String {
    match &**t {
        CType::Field(k, _) => format!("{k}({rendered})"),
        CType::Type(n, _) => format!("{n}({rendered})"),
        CType::Void | CType::DerivedVoid(..) => "void".to_string(),
        _ => format!("{}({rendered})", t.clone().to_callable_string()),
    }
}

/// Format native call expression. Returns `None` if kind unsupported (e.g. Cast in JS).
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

/// Native call expression that errors on Cast (for JS backend).
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

/// Render array of microstatements; backend provides format closure.
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
