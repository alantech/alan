use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex, OnceLock, Weak};

use weak_table::PtrWeakKeyHashMap;

use super::function::{type_to_args, type_to_rettype};
use super::ArgKind;
use super::Export;
use super::FnKind;
use super::Function;
use super::Microstatement;
use super::Program;
use super::Scope;
use super::TypeOperatorMapping;
use crate::parse;

#[derive(Clone, Debug, PartialEq)]
pub enum CType {
    Void,
    Infer(String, String), // TODO: Switch to an Interface here once they exist
    Type(String, Arc<CType>),
    Generic(String, Vec<String>, Arc<CType>),
    Binds(Arc<CType>, Vec<Arc<CType>>),
    IntrinsicGeneric(String, usize),
    IntCast(Arc<CType>),
    Int(i128),
    FloatCast(Arc<CType>),
    Float(f64),
    BoolCast(Arc<CType>),
    Bool(bool),
    StringCast(Arc<CType>),
    TString(String),
    Group(Arc<CType>),
    Unwrap(Arc<CType>),
    Function(Arc<CType>, Arc<CType>),
    Call(Arc<CType>, Arc<CType>),
    Infix(Arc<CType>),
    Prefix(Arc<CType>),
    Method(Arc<CType>),
    Property(Arc<CType>),
    Cast(Arc<CType>),
    Own(Arc<CType>),
    Deref(Arc<CType>),
    Mut(Arc<CType>),
    Dependency(Arc<CType>, Arc<CType>),
    Rust(Arc<CType>),
    Nodejs(Arc<CType>),
    From(Arc<CType>),
    Import(Arc<CType>, Arc<CType>),
    Tuple(Vec<Arc<CType>>),
    Field(String, Arc<CType>),
    Either(Vec<Arc<CType>>),
    Prop(Arc<CType>, Arc<CType>),
    AnyOf(Vec<Arc<CType>>),
    Buffer(Arc<CType>, Arc<CType>),
    Array(Arc<CType>),
    Fail(String),
    Add(Vec<Arc<CType>>),
    Sub(Vec<Arc<CType>>),
    Mul(Vec<Arc<CType>>),
    Div(Vec<Arc<CType>>),
    Mod(Vec<Arc<CType>>),
    Pow(Vec<Arc<CType>>),
    Min(Vec<Arc<CType>>),
    Max(Vec<Arc<CType>>),
    Neg(Arc<CType>),
    Len(Arc<CType>),
    Size(Arc<CType>),
    FileStr(Arc<CType>),
    Concat(Arc<CType>, Arc<CType>),
    Env(Vec<Arc<CType>>),
    EnvExists(Arc<CType>),
    TIf(Arc<CType>, Vec<Arc<CType>>),
    And(Vec<Arc<CType>>),
    Or(Vec<Arc<CType>>),
    Xor(Vec<Arc<CType>>),
    Not(Arc<CType>),
    Nand(Vec<Arc<CType>>),
    Nor(Vec<Arc<CType>>),
    Xnor(Vec<Arc<CType>>),
    TEq(Vec<Arc<CType>>),
    Neq(Vec<Arc<CType>>),
    Lt(Vec<Arc<CType>>),
    Lte(Vec<Arc<CType>>),
    Gt(Vec<Arc<CType>>),
    Gte(Vec<Arc<CType>>),
}

static CLOSE_BRACE: OnceLock<Arc<CType>> = OnceLock::new();
static CLOSE_PAREN: OnceLock<Arc<CType>> = OnceLock::new();
static COMMA: OnceLock<Arc<CType>> = OnceLock::new();
static FNARROW: OnceLock<Arc<CType>> = OnceLock::new();
static FNCALL: OnceLock<Arc<CType>> = OnceLock::new();
static DEPAT: OnceLock<Arc<CType>> = OnceLock::new();
static IMARROW: OnceLock<Arc<CType>> = OnceLock::new();
static OR: OnceLock<Arc<CType>> = OnceLock::new();
static DOT: OnceLock<Arc<CType>> = OnceLock::new();
static AND: OnceLock<Arc<CType>> = OnceLock::new();
static OPEN_BRACKET: OnceLock<Arc<CType>> = OnceLock::new();
static CLOSE_BRACKET: OnceLock<Arc<CType>> = OnceLock::new();
static ADD: OnceLock<Arc<CType>> = OnceLock::new();
static SUB: OnceLock<Arc<CType>> = OnceLock::new();
static MUL: OnceLock<Arc<CType>> = OnceLock::new();
static DIV: OnceLock<Arc<CType>> = OnceLock::new();
static MOD: OnceLock<Arc<CType>> = OnceLock::new();
static POW: OnceLock<Arc<CType>> = OnceLock::new();
static BAND: OnceLock<Arc<CType>> = OnceLock::new();
static BOR: OnceLock<Arc<CType>> = OnceLock::new();
static XOR: OnceLock<Arc<CType>> = OnceLock::new();
static NAND: OnceLock<Arc<CType>> = OnceLock::new();
static NOR: OnceLock<Arc<CType>> = OnceLock::new();
static XNOR: OnceLock<Arc<CType>> = OnceLock::new();
static EQ: OnceLock<Arc<CType>> = OnceLock::new();
static NEQ: OnceLock<Arc<CType>> = OnceLock::new();
static LT: OnceLock<Arc<CType>> = OnceLock::new();
static LTE: OnceLock<Arc<CType>> = OnceLock::new();
static GT: OnceLock<Arc<CType>> = OnceLock::new();
static GTE: OnceLock<Arc<CType>> = OnceLock::new();
static FUNCTIONAL_STRINGS: LazyLock<Mutex<PtrWeakKeyHashMap<Weak<CType>, String>>> =
    LazyLock::new(|| Mutex::new(PtrWeakKeyHashMap::<Weak<CType>, String>::new()));
static STRICT_STRINGS: LazyLock<Mutex<PtrWeakKeyHashMap<Weak<CType>, String>>> =
    LazyLock::new(|| Mutex::new(PtrWeakKeyHashMap::<Weak<CType>, String>::new()));
static LOOSE_STRINGS: LazyLock<Mutex<PtrWeakKeyHashMap<Weak<CType>, String>>> =
    LazyLock::new(|| Mutex::new(PtrWeakKeyHashMap::<Weak<CType>, String>::new()));
static CALLABLE_STRINGS: LazyLock<Mutex<PtrWeakKeyHashMap<Weak<CType>, String>>> =
    LazyLock::new(|| Mutex::new(PtrWeakKeyHashMap::<Weak<CType>, String>::new()));

impl CType {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(self: Arc<CType>) -> String {
        self.to_strict_string(true)
    }
    pub fn to_strict_string(self: Arc<CType>, strict: bool) -> String {
        let mut strings = if strict {
            STRICT_STRINGS.lock().unwrap()
        } else {
            LOOSE_STRINGS.lock().unwrap()
        };
        if let Some(string) = strings.get(&self) {
            return string.clone();
        }
        let mut unavoidable_strings = Vec::with_capacity(64);
        let mut str_parts = Vec::with_capacity(1024);
        let mut ctype_stack = Vec::with_capacity(64);
        ctype_stack.push(&self);
        // Hacky re-use of CType::Infer to insert constant strings into the ctype stack
        let close_brace =
            CLOSE_BRACE.get_or_init(|| Arc::new(CType::Infer("}".to_string(), "}".to_string())));
        let close_paren =
            CLOSE_PAREN.get_or_init(|| Arc::new(CType::Infer(")".to_string(), ")".to_string())));
        let comma =
            COMMA.get_or_init(|| Arc::new(CType::Infer(", ".to_string(), ", ".to_string())));
        let fnarrow =
            FNARROW.get_or_init(|| Arc::new(CType::Infer(" -> ".to_string(), " -> ".to_string())));
        let fncall =
            FNCALL.get_or_init(|| Arc::new(CType::Infer(" :: ".to_string(), " :: ".to_string())));
        let depat =
            DEPAT.get_or_init(|| Arc::new(CType::Infer(" @ ".to_string(), " @ ".to_string())));
        let imarrow =
            IMARROW.get_or_init(|| Arc::new(CType::Infer(" <- ".to_string(), " <- ".to_string())));
        let or = OR.get_or_init(|| Arc::new(CType::Infer(" | ".to_string(), " | ".to_string())));
        let dot = DOT.get_or_init(|| Arc::new(CType::Infer(".".to_string(), ".".to_string())));
        let and = AND.get_or_init(|| Arc::new(CType::Infer(" & ".to_string(), " & ".to_string())));
        let open_bracket =
            OPEN_BRACKET.get_or_init(|| Arc::new(CType::Infer("[".to_string(), "[".to_string())));
        let close_bracket =
            CLOSE_BRACKET.get_or_init(|| Arc::new(CType::Infer("]".to_string(), "]".to_string())));
        let add = ADD.get_or_init(|| Arc::new(CType::Infer(" + ".to_string(), " + ".to_string())));
        let sub = SUB.get_or_init(|| Arc::new(CType::Infer(" - ".to_string(), " - ".to_string())));
        let mul = MUL.get_or_init(|| Arc::new(CType::Infer(" * ".to_string(), " * ".to_string())));
        let div = DIV.get_or_init(|| Arc::new(CType::Infer(" / ".to_string(), " / ".to_string())));
        let tmod = MOD.get_or_init(|| Arc::new(CType::Infer(" % ".to_string(), " % ".to_string())));
        let pow =
            POW.get_or_init(|| Arc::new(CType::Infer(" ** ".to_string(), " ** ".to_string())));
        let band =
            BAND.get_or_init(|| Arc::new(CType::Infer(" && ".to_string(), " && ".to_string())));
        let bor = BOR.get_or_init(|| Arc::new(CType::Infer(" ||".to_string(), " ||".to_string())));
        let xor = XOR.get_or_init(|| Arc::new(CType::Infer(" ^ ".to_string(), " ^ ".to_string())));
        let nand =
            NAND.get_or_init(|| Arc::new(CType::Infer(" !& ".to_string(), " !& ".to_string())));
        let nor =
            NOR.get_or_init(|| Arc::new(CType::Infer(" !| ".to_string(), " !| ".to_string())));
        let xnor =
            XNOR.get_or_init(|| Arc::new(CType::Infer(" !^ ".to_string(), " !^ ".to_string())));
        let eq = EQ.get_or_init(|| Arc::new(CType::Infer(" == ".to_string(), " == ".to_string())));
        let neq =
            NEQ.get_or_init(|| Arc::new(CType::Infer(" != ".to_string(), " != ".to_string())));
        let lt = LT.get_or_init(|| Arc::new(CType::Infer(" < ".to_string(), " < ".to_string())));
        let lte =
            LTE.get_or_init(|| Arc::new(CType::Infer(" <= ".to_string(), " <= ".to_string())));
        let gt = GT.get_or_init(|| Arc::new(CType::Infer(" > ".to_string(), " > ".to_string())));
        let gte =
            GTE.get_or_init(|| Arc::new(CType::Infer(" >= ".to_string(), " >= ".to_string())));
        while let Some(element) = ctype_stack.pop() {
            match &**element {
                CType::Void => str_parts.push("()"),
                CType::Infer(s, _) => str_parts.push(s),
                CType::Type(n, t) => match strict {
                    true => str_parts.push(n),
                    false => ctype_stack.push(t),
                },
                CType::Generic(n, gs, _) => {
                    str_parts.push(n);
                    str_parts.push("{");
                    for g in gs {
                        str_parts.push(g);
                        str_parts.push(", ");
                    }
                    str_parts.pop();
                    str_parts.push("}");
                }
                CType::Binds(n, ts) => {
                    str_parts.push("Binds{");
                    ctype_stack.push(close_brace);
                    for t in ts.iter().rev() {
                        ctype_stack.push(t);
                        ctype_stack.push(comma);
                    }
                    ctype_stack.push(n);
                }
                CType::IntrinsicGeneric(s, l) => {
                    str_parts.push(s);
                    str_parts.push("{");
                    for i in 0..*l {
                        let l = unavoidable_strings.len();
                        unavoidable_strings.push(format!("arg{i}"));
                        let p = unavoidable_strings.as_ptr();
                        unsafe {
                            str_parts.push(p.add(l).as_ref().unwrap());
                        }
                        str_parts.push(", ");
                    }
                    str_parts.pop();
                    str_parts.push("}");
                }
                CType::IntCast(t) => {
                    str_parts.push("Int{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Int(i) => {
                    let l = unavoidable_strings.len();
                    unavoidable_strings.push(format!("{i}"));
                    let p = unavoidable_strings.as_ptr();
                    unsafe {
                        str_parts.push(p.add(l).as_ref().unwrap());
                    }
                }
                CType::FloatCast(t) => {
                    str_parts.push("Float{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Float(f) => {
                    let l = unavoidable_strings.len();
                    unavoidable_strings.push(format!("{f}"));
                    let p = unavoidable_strings.as_ptr();
                    unsafe {
                        str_parts.push(p.add(l).as_ref().unwrap());
                    }
                }
                CType::BoolCast(t) => {
                    str_parts.push("Bool{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Bool(b) => match b {
                    true => str_parts.push("true"),
                    false => str_parts.push("false"),
                },
                CType::StringCast(t) => {
                    str_parts.push("String{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::TString(s) => str_parts.push(s),
                CType::Group(t) => match strict {
                    true => {
                        str_parts.push("(");
                        ctype_stack.push(close_paren);
                        ctype_stack.push(t);
                    }
                    false => ctype_stack.push(t),
                },
                CType::Unwrap(t) => {
                    // TODO: Should this path just have it unwrapped?
                    str_parts.push("Unwrap{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Function(i, o) => {
                    ctype_stack.push(o);
                    ctype_stack.push(fnarrow);
                    ctype_stack.push(i);
                }
                CType::Call(n, f) => {
                    ctype_stack.push(f);
                    ctype_stack.push(fncall);
                    ctype_stack.push(n);
                }
                CType::Infix(o) => {
                    str_parts.push("Infix{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(o);
                }
                CType::Prefix(o) => {
                    str_parts.push("Prefix{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(o);
                }
                CType::Method(f) => {
                    str_parts.push("Method{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(f);
                }
                CType::Property(p) => {
                    str_parts.push("Property{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(p);
                }
                CType::Cast(t) => {
                    str_parts.push("Cast{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Own(t) => match strict {
                    true => {
                        str_parts.push("Own{");
                        ctype_stack.push(close_brace);
                        ctype_stack.push(t);
                    }
                    false => ctype_stack.push(t),
                },
                CType::Deref(t) => match strict {
                    true => {
                        str_parts.push("Deref{");
                        ctype_stack.push(close_brace);
                        ctype_stack.push(t);
                    }
                    false => ctype_stack.push(t),
                },
                CType::Mut(t) => match strict {
                    true => {
                        str_parts.push("Mut{");
                        ctype_stack.push(close_brace);
                        ctype_stack.push(t);
                    }
                    false => ctype_stack.push(t),
                },
                CType::Dependency(n, v) => match strict {
                    true => {
                        str_parts.push("Dependency{");
                        ctype_stack.push(close_brace);
                        ctype_stack.push(v);
                        ctype_stack.push(comma);
                        ctype_stack.push(n);
                    }
                    false => {
                        ctype_stack.push(v);
                        ctype_stack.push(depat);
                        ctype_stack.push(n);
                    }
                },
                CType::Rust(d) => {
                    str_parts.push("Rust{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(d);
                }
                CType::Nodejs(d) => {
                    str_parts.push("Nodejs{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(d);
                }
                CType::From(d) => match strict {
                    true => {
                        str_parts.push("From{");
                        ctype_stack.push(close_brace);
                        ctype_stack.push(d);
                    }
                    false => {
                        str_parts.push("<- ");
                        ctype_stack.push(d);
                    }
                },
                CType::Import(n, d) => match strict {
                    true => {
                        str_parts.push("Import{");
                        ctype_stack.push(close_brace);
                        ctype_stack.push(d);
                        ctype_stack.push(comma);
                        ctype_stack.push(n);
                    }
                    false => {
                        ctype_stack.push(d);
                        ctype_stack.push(imarrow);
                        ctype_stack.push(n);
                    }
                },
                CType::Tuple(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Field(l, t) => match strict {
                    true => {
                        str_parts.push(l);
                        str_parts.push(": ");
                        ctype_stack.push(t);
                    }
                    false => ctype_stack.push(t),
                },
                CType::Either(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(or);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Prop(t, p) => {
                    ctype_stack.push(p);
                    ctype_stack.push(dot);
                    ctype_stack.push(t);
                }
                CType::AnyOf(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(and);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Buffer(t, s) => {
                    ctype_stack.push(close_bracket);
                    ctype_stack.push(s);
                    ctype_stack.push(open_bracket);
                    ctype_stack.push(t);
                }
                CType::Array(t) => {
                    ctype_stack.push(close_bracket);
                    ctype_stack.push(open_bracket);
                    ctype_stack.push(t);
                }
                CType::Fail(m) => {
                    str_parts.push("Fail{");
                    str_parts.push(m);
                    str_parts.push("}");
                }
                CType::Add(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(add);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Sub(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(sub);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Mul(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(mul);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Div(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(div);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Mod(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(tmod);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Pow(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(pow);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Min(ts) => {
                    str_parts.push("Min{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Max(ts) => {
                    str_parts.push("Max{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Neg(t) => {
                    str_parts.push("-");
                    ctype_stack.push(t);
                }
                CType::Len(t) => {
                    str_parts.push("Len{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Size(t) => {
                    str_parts.push("Size{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::FileStr(t) => {
                    str_parts.push("FileStr{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Concat(a, b) => {
                    str_parts.push("Concat{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(b);
                    ctype_stack.push(comma);
                    ctype_stack.push(a);
                }
                CType::Env(ts) => {
                    str_parts.push("Env{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::EnvExists(t) => {
                    str_parts.push("EnvExists{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::TIf(t, ts) => {
                    str_parts.push("If{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                    ctype_stack.push(comma);
                    ctype_stack.push(t);
                }
                CType::And(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(band);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Or(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(bor);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Xor(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(xor);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Not(t) => {
                    str_parts.push("!");
                    ctype_stack.push(t);
                }
                CType::Nand(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(nand);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Nor(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(nor);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Xnor(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(xnor);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::TEq(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(eq);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Neq(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(neq);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Lt(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(lt);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Lte(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(lte);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Gt(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(gt);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Gte(ts) => {
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(gte);
                        }
                        ctype_stack.push(t);
                    }
                }
            }
        }
        strings.insert(self.clone(), str_parts.join(""));
        strings.get(&self).unwrap().clone()
    }
    pub fn to_functional_string(self: Arc<CType>) -> String {
        let mut functional_strings = FUNCTIONAL_STRINGS.lock().unwrap();
        if let Some(string) = functional_strings.get(&self) {
            return string.clone();
        }
        let mut unavoidable_strings = Vec::with_capacity(64);
        let mut str_parts = Vec::with_capacity(1024);
        let mut ctype_stack = Vec::with_capacity(64);
        ctype_stack.push(&self);
        let close_brace =
            CLOSE_BRACE.get_or_init(|| Arc::new(CType::Infer("}".to_string(), "}".to_string())));
        let comma =
            COMMA.get_or_init(|| Arc::new(CType::Infer(", ".to_string(), ", ".to_string())));
        while let Some(element) = ctype_stack.pop() {
            match &**element {
                CType::Void => str_parts.push("void"),
                CType::Infer(s, _) => str_parts.push(s),
                CType::Type(_, t) => ctype_stack.push(t),
                CType::Generic(n, gs, _) => {
                    str_parts.push(n);
                    str_parts.push("{");
                    for g in gs {
                        str_parts.push(g);
                        str_parts.push(", ");
                    }
                    str_parts.pop();
                    str_parts.push("}");
                }
                CType::Binds(n, ts) => {
                    str_parts.push("Binds{");
                    ctype_stack.push(close_brace);
                    for t in ts.iter().rev() {
                        ctype_stack.push(t);
                        ctype_stack.push(comma);
                    }
                    ctype_stack.push(n);
                }
                CType::IntrinsicGeneric(s, u) => {
                    str_parts.push(s);
                    str_parts.push("{");
                    for i in 0..(*u as u32) {
                        // TODO: This is dumb
                        str_parts.push(match i {
                            0 => "a",
                            1 => "b",
                            2 => "c",
                            3 => "d",
                            4 => "e",
                            5 => "f",
                            6 => "g",
                            7 => "h",
                            8 => "i",
                            9 => "j",
                            10 => "k",
                            11 => "l",
                            12 => "m",
                            13 => "n",
                            14 => "o",
                            15 => "p",
                            16 => "q",
                            17 => "r",
                            18 => "s",
                            19 => "t",
                            20 => "u",
                            21 => "v",
                            22 => "w",
                            23 => "x",
                            24 => "y",
                            25 => "z",
                            _ => "_",
                        });
                        str_parts.push(", ");
                    }
                    str_parts.pop();
                    str_parts.push("}");
                }
                CType::IntCast(t) => {
                    str_parts.push("Int{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Int(i) => {
                    let l = unavoidable_strings.len();
                    unavoidable_strings.push(format!("{i}"));
                    let p = unavoidable_strings.as_ptr();
                    unsafe {
                        str_parts.push(p.add(l).as_ref().unwrap());
                    }
                }
                CType::FloatCast(t) => {
                    str_parts.push("Float{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Float(f) => {
                    let l = unavoidable_strings.len();
                    unavoidable_strings.push(format!("{f}"));
                    let p = unavoidable_strings.as_ptr();
                    unsafe {
                        str_parts.push(p.add(l).as_ref().unwrap());
                    }
                }
                CType::BoolCast(t) => {
                    str_parts.push("Bool{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Bool(b) => match b {
                    true => str_parts.push("true"),
                    false => str_parts.push("false"),
                },
                CType::StringCast(t) => {
                    str_parts.push("String{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::TString(s) => {
                    str_parts.push("\"");
                    str_parts.push(s);
                    str_parts.push("\"");
                }
                CType::Group(t) => ctype_stack.push(t),
                CType::Unwrap(t) => {
                    str_parts.push("Unwrap{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Function(i, o) => {
                    str_parts.push("Function{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(o);
                    ctype_stack.push(comma);
                    ctype_stack.push(i);
                }
                CType::Call(n, f) => {
                    str_parts.push("Call{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(f);
                    ctype_stack.push(comma);
                    ctype_stack.push(n);
                }
                CType::Infix(o) => {
                    str_parts.push("Infix{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(o);
                }
                CType::Prefix(o) => {
                    str_parts.push("Prefix{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(o);
                }
                CType::Method(f) => {
                    str_parts.push("Method{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(f);
                }
                CType::Property(p) => {
                    str_parts.push("Property{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(p);
                }
                CType::Cast(t) => {
                    str_parts.push("Cast{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Own(t) => {
                    str_parts.push("Own{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Deref(t) => {
                    str_parts.push("Deref{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Mut(t) => {
                    str_parts.push("Mut{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Dependency(n, v) => {
                    str_parts.push("Dependency{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(v);
                    ctype_stack.push(comma);
                    ctype_stack.push(n);
                }
                CType::Rust(d) => {
                    str_parts.push("Rust{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(d);
                }
                CType::Nodejs(d) => {
                    str_parts.push("Nodejs{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(d);
                }
                CType::From(d) => {
                    str_parts.push("From{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(d);
                }
                CType::Import(n, d) => {
                    str_parts.push("Import{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(d);
                    ctype_stack.push(comma);
                    ctype_stack.push(n);
                }
                CType::Tuple(ts) => {
                    str_parts.push("Tuple{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Field(l, t) => {
                    str_parts.push("Field{");
                    str_parts.push(l);
                    str_parts.push(", ");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Either(ts) => {
                    str_parts.push("Either{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Prop(t, p) => {
                    str_parts.push("Prop{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(p);
                    ctype_stack.push(comma);
                    ctype_stack.push(t);
                }
                CType::AnyOf(ts) => {
                    str_parts.push("AnyOf{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Buffer(t, s) => {
                    str_parts.push("Buffer{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(s);
                    ctype_stack.push(comma);
                    ctype_stack.push(t);
                }
                CType::Array(t) => {
                    str_parts.push("Array{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Fail(m) => {
                    str_parts.push("Fail{");
                    str_parts.push(m);
                    str_parts.push("}");
                }
                CType::Add(ts) => {
                    str_parts.push("Add{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Sub(ts) => {
                    str_parts.push("Sub{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Mul(ts) => {
                    str_parts.push("Mul{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Div(ts) => {
                    str_parts.push("Div{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Mod(ts) => {
                    str_parts.push("Mod{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Pow(ts) => {
                    str_parts.push("Pow{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Min(ts) => {
                    str_parts.push("Min{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Max(ts) => {
                    str_parts.push("Max{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Neg(t) => {
                    str_parts.push("Neg{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Len(t) => {
                    str_parts.push("Len{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Size(t) => {
                    str_parts.push("Size{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::FileStr(t) => {
                    str_parts.push("FileStr{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Concat(a, b) => {
                    str_parts.push("Concat{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(b);
                    ctype_stack.push(comma);
                    ctype_stack.push(a);
                }
                CType::Env(ts) => {
                    str_parts.push("Env{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::EnvExists(t) => {
                    str_parts.push("EnvExists{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::TIf(t, ts) => {
                    str_parts.push("If{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                    ctype_stack.push(comma);
                    ctype_stack.push(t);
                }
                CType::And(ts) => {
                    str_parts.push("And{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Or(ts) => {
                    str_parts.push("Or{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Xor(ts) => {
                    str_parts.push("Xor{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Not(t) => {
                    str_parts.push("Not{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Nand(ts) => {
                    str_parts.push("Nand{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Nor(ts) => {
                    str_parts.push("Nor{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Xnor(ts) => {
                    str_parts.push("Xnor{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::TEq(ts) => {
                    str_parts.push("Eq{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Neq(ts) => {
                    str_parts.push("Neq{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Lt(ts) => {
                    str_parts.push("Lt{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Lte(ts) => {
                    str_parts.push("Lte{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Gt(ts) => {
                    str_parts.push("Gt{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
                CType::Gte(ts) => {
                    str_parts.push("Gte{");
                    ctype_stack.push(close_brace);
                    for (i, t) in ts.iter().rev().enumerate() {
                        if i != 0 {
                            ctype_stack.push(comma);
                        }
                        ctype_stack.push(t);
                    }
                }
            }
        }
        functional_strings.insert(self.clone(), str_parts.join(""));
        functional_strings.get(&self).unwrap().clone()
    }
    pub fn to_callable_string(self: Arc<CType>) -> String {
        // TODO: Be more efficient with this later
        let mut callable_strings = CALLABLE_STRINGS.lock().unwrap();
        if let Some(string) = callable_strings.get(&self) {
            return string.clone();
        }
        let string = match &*self {
            CType::Int(_) | CType::Float(_) => format!("_{}", self.clone().to_functional_string()),
            CType::TString(s) if s.starts_with(|c: char| c.is_ascii_digit()) => {
                format!("_{}", self.clone().to_functional_string())
            }
            CType::Type(n, t) => match **t {
                CType::Int(_) | CType::Float(_) => {
                    format!("_{}", self.clone().to_functional_string())
                }
                CType::Binds(..) => n.clone(),
                _ => self.clone().to_functional_string(),
            },
            _ => self.clone().to_functional_string(),
        }
        .chars()
        .map(|c| match c {
            '0'..='9' => c,
            'a'..='z' => c,
            'A'..='Z' => c,
            '!'..='/' => ((c as u8) + 32) as char, // Move to A..=O
            ':'..='@' => ((c as u8) + 22) as char, // Move to P..=W
            '['..='`' => ((c as u8) + 6) as char,  // Move to a..=g
            '|' => 'z',
            '~' => 'y',
            _ => '_',
        })
        .collect::<String>();
        callable_strings.insert(self.clone(), string);
        callable_strings.get(&self).unwrap().clone()
    }

    pub fn has_infer(self: Arc<CType>) -> bool {
        match &*self {
            CType::Void
            | CType::IntrinsicGeneric(..)
            | CType::Int(_)
            | CType::Float(_)
            | CType::Bool(_)
            | CType::TString(_)
            | CType::Fail(_) => false,
            CType::Infer(..) => true,
            CType::Type(_, t)
            | CType::Generic(_, _, t)
            | CType::IntCast(t)
            | CType::FloatCast(t)
            | CType::BoolCast(t)
            | CType::StringCast(t)
            | CType::Group(t)
            | CType::Unwrap(t)
            | CType::Infix(t)
            | CType::Prefix(t)
            | CType::Method(t)
            | CType::Property(t)
            | CType::Cast(t)
            | CType::Own(t)
            | CType::Deref(t)
            | CType::Mut(t)
            | CType::Rust(t)
            | CType::Nodejs(t)
            | CType::From(t)
            | CType::Array(t)
            | CType::Field(_, t)
            | CType::Neg(t)
            | CType::Len(t)
            | CType::Size(t)
            | CType::FileStr(t)
            | CType::EnvExists(t)
            | CType::Not(t) => t.clone().has_infer(),
            CType::Binds(t, ts) | CType::TIf(t, ts) => {
                t.clone().has_infer() || ts.iter().any(|t| t.clone().has_infer())
            }
            CType::Function(a, b)
            | CType::Call(a, b)
            | CType::Dependency(a, b)
            | CType::Import(a, b)
            | CType::Prop(a, b)
            | CType::Buffer(a, b)
            | CType::Concat(a, b) => a.clone().has_infer() || b.clone().has_infer(),
            CType::Tuple(ts)
            | CType::Either(ts)
            | CType::AnyOf(ts)
            | CType::Add(ts)
            | CType::Sub(ts)
            | CType::Mul(ts)
            | CType::Div(ts)
            | CType::Mod(ts)
            | CType::Pow(ts)
            | CType::Min(ts)
            | CType::Max(ts)
            | CType::Env(ts)
            | CType::And(ts)
            | CType::Or(ts)
            | CType::Xor(ts)
            | CType::Nand(ts)
            | CType::Nor(ts)
            | CType::Xnor(ts)
            | CType::TEq(ts)
            | CType::Neq(ts)
            | CType::Lt(ts)
            | CType::Lte(ts)
            | CType::Gt(ts)
            | CType::Gte(ts) => ts.iter().any(|t| t.clone().has_infer()),
        }
    }

    pub fn degroup(self: Arc<CType>) -> Arc<CType> {
        match &*self {
            CType::Void => self,
            CType::Infer(..) => self,
            CType::Type(n, t) => Arc::new(CType::Type(n.clone(), t.clone().degroup())),
            CType::Generic(..) => self,
            CType::Binds(n, ts) => Arc::new(CType::Binds(
                n.clone().degroup(),
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::IntrinsicGeneric(..) => self,
            CType::IntCast(t) => Arc::new(CType::IntCast(t.clone().degroup())),
            CType::Int(_) => self,
            CType::FloatCast(t) => Arc::new(CType::FloatCast(t.clone().degroup())),
            CType::Float(_) => self,
            CType::BoolCast(t) => Arc::new(CType::BoolCast(t.clone().degroup())),
            CType::Bool(_) => self,
            CType::StringCast(t) => Arc::new(CType::StringCast(t.clone().degroup())),
            CType::TString(_) => self,
            CType::Group(t) => t.clone().degroup(),
            CType::Unwrap(t) => Arc::new(CType::Unwrap(t.clone().degroup())),
            CType::Function(i, o) => {
                Arc::new(CType::Function(i.clone().degroup(), o.clone().degroup()))
            }
            CType::Call(n, f) => Arc::new(CType::Call(n.clone().degroup(), f.clone().degroup())),
            CType::Infix(o) => Arc::new(CType::Infix(o.clone().degroup())),
            CType::Prefix(o) => Arc::new(CType::Prefix(o.clone().degroup())),
            CType::Method(f) => Arc::new(CType::Method(f.clone().degroup())),
            CType::Property(p) => Arc::new(CType::Property(p.clone().degroup())),
            CType::Cast(t) => Arc::new(CType::Cast(t.clone().degroup())),
            CType::Own(t) => Arc::new(CType::Own(t.clone().degroup())),
            CType::Deref(t) => Arc::new(CType::Deref(t.clone().degroup())),
            CType::Mut(t) => Arc::new(CType::Mut(t.clone().degroup())),
            CType::Dependency(n, v) => {
                Arc::new(CType::Dependency(n.clone().degroup(), v.clone().degroup()))
            }
            CType::Rust(d) => Arc::new(CType::Rust(d.clone().degroup())),
            CType::Nodejs(d) => Arc::new(CType::Nodejs(d.clone().degroup())),
            CType::From(d) => Arc::new(CType::From(d.clone().degroup())),
            CType::Import(n, d) => {
                Arc::new(CType::Import(n.clone().degroup(), d.clone().degroup()))
            }
            CType::Tuple(ts) => Arc::new(CType::Tuple(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Field(l, t) => Arc::new(CType::Field(l.clone(), t.clone().degroup())),
            CType::Either(ts) => Arc::new(CType::Either(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Prop(t, p) => Arc::new(CType::Prop(t.clone().degroup(), p.clone().degroup())),
            CType::AnyOf(ts) => Arc::new(CType::AnyOf(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Buffer(t, s) => {
                Arc::new(CType::Buffer(t.clone().degroup(), s.clone().degroup()))
            }
            CType::Array(t) => Arc::new(CType::Array(t.clone().degroup())),
            CType::Fail(_) => self,
            CType::Add(ts) => Arc::new(CType::Add(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Sub(ts) => Arc::new(CType::Sub(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Mul(ts) => Arc::new(CType::Mul(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Div(ts) => Arc::new(CType::Div(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Mod(ts) => Arc::new(CType::Mod(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Pow(ts) => Arc::new(CType::Pow(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Min(ts) => Arc::new(CType::Min(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Max(ts) => Arc::new(CType::Max(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Neg(t) => Arc::new(CType::Neg(t.clone().degroup())),
            CType::Len(t) => Arc::new(CType::Len(t.clone().degroup())),
            CType::Size(t) => Arc::new(CType::Size(t.clone().degroup())),
            CType::FileStr(t) => Arc::new(CType::FileStr(t.clone().degroup())),
            CType::Concat(a, b) => {
                Arc::new(CType::Concat(a.clone().degroup(), b.clone().degroup()))
            }
            CType::Env(ts) => Arc::new(CType::Env(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::EnvExists(t) => Arc::new(CType::EnvExists(t.clone().degroup())),
            CType::TIf(t, ts) => Arc::new(CType::TIf(
                t.clone().degroup(),
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::And(ts) => Arc::new(CType::And(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Or(ts) => Arc::new(CType::Or(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Xor(ts) => Arc::new(CType::Xor(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Not(t) => Arc::new(CType::Not(t.clone().degroup())),
            CType::Nand(ts) => Arc::new(CType::Nand(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Nor(ts) => Arc::new(CType::Nor(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Xnor(ts) => Arc::new(CType::Xnor(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::TEq(ts) => Arc::new(CType::TEq(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Neq(ts) => Arc::new(CType::Neq(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Lt(ts) => Arc::new(CType::Lt(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Lte(ts) => Arc::new(CType::Lte(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Gt(ts) => Arc::new(CType::Gt(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Gte(ts) => Arc::new(CType::Gte(
                ts.iter()
                    .map(|t| t.clone().degroup())
                    .collect::<Vec<Arc<CType>>>(),
            )),
        }
    }
    // Given a list of generic type names, a list of argument types provided, and the original type
    // ast of the function, we can infer the generic type mappings by creating a temporary child
    // scope, creating `Infer` CTypes for each generic name and parsing the type ast inside of that
    // scope, then for each input record for the function traverse the tree of the input argument
    // type *and* the created CType tree of the same argument index and we can either error out if
    // they don't match, or reach the `Infer` type on the new tree and assign the corresponding
    // sub-tree of the provided type to the generic in question. If we get a sub-tree for all
    // generic type names, we succeed, otherwise we have to fail on being unable to resolve
    // specific generics.
    pub fn infer_generics_inner_loop(
        scope: &Scope,
        generic_types: &mut HashMap<String, Arc<CType>>,
        arg_type_vec: Vec<(Arc<CType>, Arc<CType>)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (a, i) in arg_type_vec {
            let mut arg = vec![a];
            let mut input = vec![i];
            while let (Some(a), Some(i)) = (arg.pop(), input.pop()) {
                match (&*a, &*i) {
                    (CType::Void, CType::Void) => { /* Do nothing */ }
                    (CType::Infer(s1, _), CType::Infer(s2, _)) if s1 == s2 => {
                        // This is not an error, but we can't garner any useful information here
                    }
                    (CType::Infer(s, _), _) => {
                        return Err(format!(
                            "While attempting to infer generics found an inference type {s} as an input somehow"
                        )
                        .into());
                    }
                    (CType::Type(_, t1), CType::Type(_, t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Type(_, t1), _) if !matches!(&**t1, CType::Binds(..)) => {
                        arg.push(t1.clone());
                        input.push(i.clone());
                    }
                    (_, CType::Type(_, t2)) if !matches!(&**t2, CType::Binds(..)) => {
                        arg.push(a.clone());
                        input.push(t2.clone());
                    }
                    (CType::Generic(_, _, t), CType::Function(..))
                        if matches!(&**t, CType::Function(..)) =>
                    {
                        // TODO: How to get the generic args to compare correctly
                        arg.push(t.clone());
                        input.push(i.clone());
                    }
                    (CType::Generic(..), _) => {
                        return Err(format!(
                            "Ran into an unresolved generic in the arguments list: {arg:?}"
                        )
                        .into());
                    }
                    (CType::Binds(n1, ts1), CType::Binds(n2, ts2)) => {
                        if ts1.len() != ts2.len() {
                            // TODO: Better generic arg matching
                            return Err(format!(
                                "Mismatched resolved bound generic types {}{{{}}} and {}{{{}}} during inference",
                                n1.clone().to_strict_string(false),
                                ts1
                                    .iter()
                                    .map(|t| t.clone().to_strict_string(false))
                                    .collect::<Vec<String>>()
                                    .join(", "),
                                n2.clone().to_strict_string(false),
                                ts2
                                    .iter()
                                    .map(|t| t.clone().to_strict_string(false))
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            ).into());
                        }
                        arg.push(n1.clone());
                        input.push(n2.clone());
                        // Enqueue the bound types for checking purposes
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::IntrinsicGeneric(n1, s1), CType::IntrinsicGeneric(n2, s2)) => {
                        if !(n1 == n2 && s1 == s2) {
                            return Err(format!(
                                "Mismatched generics {n1} and {n2} during inference"
                            )
                            .into());
                        }
                    }
                    (CType::Int(i1), CType::Int(i2)) => {
                        if i1 != i2 {
                            return Err(format!(
                                "Mismatched integers {i1} and {i2} during inference"
                            )
                            .into());
                        }
                    }
                    (_, CType::IntCast(_)) => {
                        // Should only be reachable if there's an `infer` in here
                        // Unfortunately may not infer correctly in this scenario as casting to an
                        // integer is lossy.
                        return Err("Cannot infer an integer cast".into());
                    }
                    (CType::Float(f1), CType::Float(f2)) => {
                        if f1 != f2 {
                            return Err(format!(
                                "Mismatched floats {f1} and {f2} during inference"
                            )
                            .into());
                        }
                    }
                    (_, CType::FloatCast(_)) => {
                        // Should only be reachable if there's an `infer` in here
                        // Unfortunately may not infer correctly in this scenario as casting to a
                        // float is lossy.
                        return Err("Cannot infer a float cast".into());
                    }
                    (CType::Bool(b1), CType::Bool(b2)) => {
                        if b1 != b2 {
                            return Err("Mismatched booleans during inference".into());
                        }
                    }
                    (_, CType::BoolCast(_)) => {
                        // Should only be reachable if there's an `infer` in here
                        // Unfortunately may not infer correctly in this scenario as casting to a
                        // boolean is lossy.
                        return Err("Cannot infer a bool cast".into());
                    }
                    (CType::TString(s1), CType::TString(s2)) => {
                        if s1 != s2 {
                            return Err(format!(
                                "Mismatched strings {s1} and {s2} during inference"
                            )
                            .into());
                        }
                    }
                    (CType::TString(s), CType::StringCast(sc)) => {
                        // Should only be reachable if there's an `infer` in here. Fortunately, we
                        // *can* infer the original type from a string cast by re-parsing the
                        // string as a type declaration
                        match &**sc {
                            CType::Infer(..) => {
                                // We need to parse the string back into a type and then pass that
                                // along
                                let wtol = parse::typeassignables(s).expect("should be impossible");
                                let t = withtypeoperatorslist_to_ctype(&wtol.1, scope)?;
                                arg.push(t.clone());
                                input.push(sc.clone());
                            }
                            _ => {
                                return Err(format!(
                                    "Mismatched string {s} and string cast {sc:?} during inference"
                                )
                                .into());
                            }
                        }
                    }
                    (CType::Group(g1), CType::Group(g2)) => {
                        arg.push(g1.clone());
                        input.push(g2.clone());
                    }
                    (CType::Group(g1), _) => {
                        arg.push(g1.clone());
                        input.push(i.clone());
                    }
                    (_, CType::Group(g2)) => {
                        arg.push(a.clone());
                        input.push(g2.clone());
                    }
                    (CType::Function(i1, o1), CType::Function(i2, o2)) => {
                        match &**i1 {
                            CType::Tuple(ts1) if ts1.len() == 1 => {
                                arg.push(ts1[0].clone());
                            }
                            _otherwise => arg.push(i1.clone()),
                        }
                        arg.push(o1.clone());
                        match &**i2 {
                            CType::Tuple(ts2) if ts2.len() == 1 => {
                                input.push(ts2[0].clone());
                            }
                            _otherwise => input.push(i2.clone()),
                        }
                        input.push(o2.clone());
                    }
                    (CType::Call(n1, f1), CType::Call(n2, f2)) => {
                        arg.push(n1.clone());
                        arg.push(f1.clone());
                        input.push(n2.clone());
                        input.push(f2.clone());
                    }
                    (CType::Infix(o1), CType::Infix(o2)) => {
                        arg.push(o1.clone());
                        input.push(o2.clone());
                    }
                    (CType::Prefix(o1), CType::Prefix(o2)) => {
                        arg.push(o1.clone());
                        input.push(o2.clone());
                    }
                    (CType::Method(f1), CType::Method(f2)) => {
                        arg.push(f1.clone());
                        input.push(f2.clone());
                    }
                    (CType::Property(p1), CType::Property(p2)) => {
                        arg.push(p1.clone());
                        input.push(p2.clone());
                    }
                    (CType::Cast(t1), CType::Cast(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Own(t1), CType::Own(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Deref(t1), CType::Deref(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Mut(t1), CType::Mut(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Dependency(n1, v1), CType::Dependency(n2, v2)) => {
                        arg.push(n1.clone());
                        arg.push(v1.clone());
                        input.push(n2.clone());
                        input.push(v2.clone());
                    }
                    (CType::Rust(d1), CType::Rust(d2)) => {
                        arg.push(d1.clone());
                        input.push(d2.clone());
                    }
                    (CType::Nodejs(d1), CType::Nodejs(d2)) => {
                        arg.push(d1.clone());
                        input.push(d2.clone());
                    }
                    (CType::From(d1), CType::From(d2)) => {
                        arg.push(d1.clone());
                        input.push(d2.clone());
                    }
                    (CType::Import(n1, d1), CType::Import(n2, d2)) => {
                        arg.push(n1.clone());
                        arg.push(d1.clone());
                        input.push(n2.clone());
                        input.push(d2.clone());
                    }
                    (CType::Tuple(ts1), CType::Tuple(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched tuple types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        // TODO: Allow out-of-order listing based on Field labels
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Prop(t1, p1), CType::Prop(t2, p2)) => {
                        arg.push(t1.clone());
                        arg.push(p1.clone());
                        input.push(t2.clone());
                        input.push(p2.clone());
                    }
                    (a, CType::Prop(t, p)) => {
                        // TODO: There's probably a generalized way to handle things like this, but
                        // for now, just hardwire this particular generic resolution used for the
                        // GPGPU `map` function
                        // In this case, the type to infer is the key that gets the
                        // value from the tuple type so we iterate through the values
                        // of the Prop tuple for a value that "accepts" the 'a' value
                        match &**p {
                            CType::StringCast(sc) => match &**sc {
                                CType::Infer(..) => match &**t {
                                    CType::Type(_, it) => match &**it {
                                        CType::Tuple(tp) => {
                                            let mut found = false;
                                            for r in tp {
                                                if let CType::Field(l, v) = &**r {
                                                    if Arc::new(a.clone()).accepts(v.clone()) {
                                                        // We found a match, parse the label back to a
                                                        // type
                                                        let wtol = parse::typeassignables(l)
                                                            .expect("should be impossible");
                                                        let t = withtypeoperatorslist_to_ctype(
                                                            &wtol.1, scope,
                                                        )?;
                                                        arg.push(t.clone());
                                                        input.push(sc.clone());
                                                        found = true;
                                                    }
                                                }
                                            }
                                            if !found {
                                                return Err(
                                                    "Unable to find property during inference"
                                                        .into(),
                                                );
                                            }
                                        }
                                        _ => {
                                            return Err("Property extraction inference only possible on a tuple type".into());
                                        }
                                    },
                                    CType::Tuple(tp) => {
                                        let mut found = false;
                                        for r in tp {
                                            if let CType::Field(l, v) = &**r {
                                                if Arc::new(a.clone()).accepts(v.clone()) {
                                                    // We found a match, parse the label back to a
                                                    // type
                                                    let wtol = parse::typeassignables(l)
                                                        .expect("should be impossible");
                                                    let t = withtypeoperatorslist_to_ctype(
                                                        &wtol.1, scope,
                                                    )?;
                                                    arg.push(t.clone());
                                                    input.push(sc.clone());
                                                    found = true;
                                                }
                                            }
                                        }
                                        if !found {
                                            return Err(
                                                "Unable to find property during inference".into()
                                            );
                                        }
                                    }
                                    _ => {
                                        return Err("Property extraction inference only possible on a tuple type".into());
                                    }
                                },
                                _ => unreachable!(),
                            },
                            // The other path that is being hardwired right now is the version used
                            // for GBufferTagged. TODO: Figure out a general way to handle this
                            // kind of type inference and replace the hacks with it.
                            CType::TString(s) if s == "typeName" => {
                                match &**t {
                                    CType::Prop(t, p) => {
                                        match &**p {
                                            CType::StringCast(sc) => {
                                                match &**sc {
                                                    CType::Infer(..) => match &**t {
                                                        CType::Type(_, it) => match &**it {
                                                            CType::Tuple(tp) => {
                                                                let mut found = false;
                                                                for r in tp {
                                                                    if let CType::Field(l, v) = &**r
                                                                    {
                                                                        if Arc::new(a.clone())
                                                                            .accepts(v.clone())
                                                                        {
                                                                            // We found a match, parse the label back to a
                                                                            // type
                                                                            let wtol = parse::typeassignables(l).expect("should be impossible");
                                                                            let t = withtypeoperatorslist_to_ctype(&wtol.1, scope)?;
                                                                            arg.push(t.clone());
                                                                            input.push(sc.clone());
                                                                            found = true;
                                                                        }
                                                                    }
                                                                }
                                                                if !found {
                                                                    return Err("Unable to find property during inference".into());
                                                                }
                                                            }
                                                            _ => {
                                                                return Err("Property extraction inference only possible on a tuple type".into());
                                                            }
                                                        },
                                                        CType::Tuple(tp) => {
                                                            let mut found = false;
                                                            for r in tp {
                                                                if let CType::Field(l, v) = &**r {
                                                                    if Arc::new(a.clone())
                                                                        .accepts(v.clone())
                                                                    {
                                                                        // We found a match, parse the label back to a
                                                                        // type
                                                                        let wtol = parse::typeassignables(l).expect("should be impossible");
                                                                        let t = withtypeoperatorslist_to_ctype(&wtol.1, scope)?;
                                                                        arg.push(t.clone());
                                                                        input.push(sc.clone());
                                                                        found = true;
                                                                    }
                                                                }
                                                            }
                                                            if !found {
                                                                return Err("Unable to find property during inference".into());
                                                            }
                                                        }
                                                        _ => {
                                                            return Err("Property extraction inference only possible on a tuple type".into());
                                                        }
                                                    },
                                                    _ => unreachable!(),
                                                }
                                            }
                                            _ => {
                                                return Err(format!("Mismatch between {a:?} and {i:?} during inference").into());
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(format!(
                                            "Mismatch between {a:?} and {i:?} during inference"
                                        )
                                        .into());
                                    }
                                }
                            }
                            _ => {
                                return Err(format!(
                                    "Mismatch between {a:?} and {i:?} during inference"
                                )
                                .into());
                            }
                        }
                    }
                    (CType::Field(l1, t1), CType::Field(l2, t2)) => {
                        // TODO: Allow out-of-order listing based on Field labels
                        if l1 != l2 {
                            return Err(format!(
                                "Mismatched fields {l1} and {l2} during inference"
                            )
                            .into());
                        }
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (_, CType::Field(_, t2)) => {
                        arg.push(a.clone());
                        input.push(t2.clone());
                    }
                    (CType::Field(_, t1), _) => {
                        arg.push(t1.clone());
                        input.push(i.clone());
                    }
                    (CType::Either(ts1), CType::Either(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched either types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Buffer(t1, s1), CType::Buffer(t2, s2)) => {
                        arg.push(t1.clone());
                        arg.push(s1.clone());
                        input.push(t2.clone());
                        input.push(s2.clone());
                    }
                    (CType::Array(t1), CType::Array(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::AnyOf(ts), CType::Infer(g, _)) => {
                        // Found an interesting inference situation where more than one answer may
                        // be right. We need to check the existing possible matches (if any) and
                        // intersect it with this AnyOf set, then store the set. If there is only
                        // one match in the set, then we store that match directly, instead.
                        if generic_types.contains_key(g) {
                            let other_type: &CType = generic_types.get(g).unwrap();
                            let mut matches = Vec::new();
                            match other_type {
                                CType::AnyOf(t2s) => {
                                    for t1 in ts {
                                        for t2 in t2s {
                                            if t1.clone().degroup().to_callable_string()
                                                == t2.clone().degroup().to_callable_string()
                                            {
                                                matches.push(t1.clone());
                                            }
                                        }
                                    }
                                }
                                otherwise => {
                                    for t1 in ts {
                                        if t1.clone().degroup().to_callable_string()
                                            == Arc::new(otherwise.clone())
                                                .degroup()
                                                .to_callable_string()
                                        {
                                            matches.push(t1.clone());
                                        }
                                    }
                                }
                            }
                            if matches.is_empty() {
                                // Do nothing
                            } else if matches.len() == 1 {
                                generic_types
                                    .insert(g.clone(), matches.into_iter().nth(0).unwrap().clone());
                            } else {
                                generic_types.insert(g.clone(), Arc::new(CType::AnyOf(matches)));
                            }
                        } else {
                            generic_types.insert(g.clone(), Arc::new(CType::AnyOf(ts.clone())));
                        }
                    }
                    (CType::Fail(m1), CType::Fail(m2)) => {
                        if m1 != m2 {
                            return Err(
                                "The two types want to fail in different ways. How bizarre!".into(),
                            );
                        }
                    }
                    (CType::Add(ts1), CType::Add(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched add types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (
                        CType::Int(_) | CType::Float(_),
                        CType::Add(_)
                        | CType::Sub(_)
                        | CType::Mul(_)
                        | CType::Div(_)
                        | CType::Mod(_)
                        | CType::Pow(_)
                        | CType::Min(_)
                        | CType::Max(_)
                        | CType::Neg(_)
                        | CType::Len(_)
                        | CType::Size(_),
                    ) => {
                        // TODO: This should allow us to constrain which generic values are
                        // possible for each generic to infer on the right-hand-side, but for now
                        // we're just going to ignore this path and require the components are
                        // inferred separately in the type system
                    }
                    (
                        CType::Int(_) | CType::Bool(_),
                        CType::And(_)
                        | CType::Or(_)
                        | CType::Xor(_)
                        | CType::Not(_)
                        | CType::Nand(_)
                        | CType::Nor(_)
                        | CType::Xnor(_),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (
                        CType::Int(_) | CType::Float(_) | CType::TString(_) | CType::Bool(_),
                        CType::TEq(_) | CType::Neq(_),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (
                        CType::Int(_) | CType::Float(_) | CType::TString(_),
                        CType::Lt(_) | CType::Lte(_) | CType::Gt(_) | CType::Gte(_),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (CType::Sub(ts1), CType::Sub(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched sub types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Mul(ts1), CType::Mul(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched mul types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Div(ts1), CType::Div(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched div types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Mod(ts1), CType::Mod(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched div types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Pow(ts1), CType::Pow(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched pow types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Min(ts1), CType::Min(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched min types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Max(ts1), CType::Max(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched max types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Neg(t1), CType::Neg(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Len(t1), CType::Len(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Size(t1), CType::Size(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::FileStr(t1), CType::FileStr(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Env(ts1), CType::Env(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched env types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::EnvExists(t1), CType::EnvExists(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::TIf(t1, ts1), CType::TIf(t2, ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched env types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        arg.push(t1.clone());
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        input.push(t2.clone());
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::And(ts1), CType::And(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched and types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Or(ts1), CType::Or(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched or types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Xor(ts1), CType::Xor(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched xor types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Not(t1), CType::Not(t2)) => {
                        arg.push(t1.clone());
                        input.push(t2.clone());
                    }
                    (CType::Nand(ts1), CType::Nand(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched nand types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Nor(ts1), CType::Nor(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched nor types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Xnor(ts1), CType::Xnor(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched xnor types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::TEq(ts1), CType::TEq(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched eq types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Neq(ts1), CType::Neq(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched neq types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Lt(ts1), CType::Lt(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched lt types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Lte(ts1), CType::Lte(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched lte types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Gt(ts1), CType::Gt(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched gt types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (CType::Gte(ts1), CType::Gte(ts2)) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched gte types {} and {} found during inference",
                                a.clone().to_string(),
                                i.clone().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1.clone());
                        }
                        for t2 in ts2 {
                            input.push(t2.clone());
                        }
                    }
                    (_, CType::Infer(g, _)) => {
                        // Found the normal path to infer. If there's already a match, check if the
                        // existing match is an AnyOf and intersect the set, otherwise a simple
                        // comparison
                        if generic_types.contains_key(g) {
                            // Possible found the same thing, already, let's confirm that we aren't
                            // in an impossible scenario.
                            let other_type: &Arc<CType> = generic_types.get(g).unwrap();
                            let mut matched = false;
                            match &**other_type {
                                CType::AnyOf(ts) => {
                                    for t1 in ts {
                                        if CType::tunwrap(t1.clone().degroup())
                                            .to_functional_string()
                                            == CType::tunwrap(a.clone().degroup())
                                                .to_functional_string()
                                        {
                                            matched = true;
                                        }
                                    }
                                }
                                otherwise => {
                                    if CType::tunwrap(Arc::new(otherwise.clone()).degroup())
                                        .to_functional_string()
                                        == CType::tunwrap(a.clone().degroup())
                                            .to_functional_string()
                                    {
                                        matched = true;
                                    }
                                }
                            }
                            if matched {
                                generic_types.insert(g.clone(), a.clone());
                            } else {
                                return Err(format!(
                                    "Generic {} matched both {} and {}",
                                    g,
                                    other_type.clone().to_functional_string(),
                                    a.to_functional_string()
                                )
                                .into());
                            }
                        } else {
                            generic_types.insert(g.clone(), a.clone());
                        }
                    }
                    (CType::AnyOf(ts), _) => {
                        // Multiple of these `AnyOf` types may be viable. Accept all that are, and
                        // later on something should hopefully work as a tiebreaker.
                        let mut success = false;
                        let inner_results = ts
                            .iter()
                            .map(|t| {
                                let mut generic_types_inner = generic_types.clone();
                                if CType::infer_generics_inner_loop(
                                    scope,
                                    &mut generic_types_inner,
                                    vec![(t.clone(), i.clone())],
                                )
                                .is_ok()
                                {
                                    success = true;
                                } else {
                                    // Reset it on failure, just in case
                                    generic_types_inner = generic_types.clone();
                                }
                                generic_types_inner
                            })
                            .collect::<Vec<HashMap<String, Arc<CType>>>>();
                        if !success {
                            return Err(format!(
                                "None of {} matches {}",
                                ts.iter()
                                    .map(|t| t.clone().to_strict_string(false))
                                    .collect::<Vec<String>>()
                                    .join(" & "),
                                i.clone().to_strict_string(false)
                            )
                            .into());
                        }
                        // Merge the results into a singular set to check. If there are multiple
                        // values for the same key, merge them as an `AnyOf`.
                        let mut combined_types = HashMap::new();
                        for gti in inner_results {
                            for (k, v) in &gti {
                                match combined_types.get(k) {
                                    None => {
                                        combined_types.insert(k.clone(), v.clone());
                                    }
                                    Some(other_v) => match (&**other_v, &**v) {
                                        (CType::AnyOf(ots), nt) => {
                                            let mut preexists = false;
                                            for t in ots {
                                                if t.clone().to_functional_string()
                                                    == Arc::new(nt.clone()).to_functional_string()
                                                {
                                                    preexists = true;
                                                }
                                            }
                                            if !preexists {
                                                let mut nts = ots.clone();
                                                nts.push(Arc::new(nt.clone()));
                                                combined_types
                                                    .insert(k.clone(), Arc::new(CType::AnyOf(nts)));
                                            }
                                        }
                                        (_, _) => {
                                            combined_types.insert(
                                                k.clone(),
                                                Arc::new(CType::AnyOf(vec![
                                                    other_v.clone(),
                                                    v.clone(),
                                                ])),
                                            );
                                        }
                                    },
                                }
                            }
                        }
                        // Now comparing the combined resolved types with what was in the original
                        // set, anything new gets included, but we attempt to *narrow* the `AnyOf`
                        // to as few as possible, when possible
                        for (k, v) in &combined_types {
                            match generic_types.get(k) {
                                None => {
                                    generic_types.insert(k.clone(), v.clone());
                                }
                                Some(old_v) => match (&**old_v, &**v) {
                                    (CType::AnyOf(oldts), CType::AnyOf(newts)) => {
                                        let mut outts = Vec::new();
                                        for ot in oldts {
                                            for nt in newts {
                                                if ot.clone().to_functional_string()
                                                    == nt.clone().to_functional_string()
                                                {
                                                    outts.push(nt.clone());
                                                }
                                            }
                                        }
                                        generic_types
                                            .insert(k.clone(), Arc::new(CType::AnyOf(outts)));
                                    }
                                    (_, CType::AnyOf(newts)) => {
                                        let mut success = false;
                                        for nt in newts {
                                            if old_v.clone().to_functional_string()
                                                == nt.clone().to_functional_string()
                                            {
                                                success = true;
                                                break;
                                            }
                                        }
                                        if !success {
                                            return Err(format!(
                                                "None of {} matches {}",
                                                newts
                                                    .iter()
                                                    .map(|t| t.clone().to_strict_string(false))
                                                    .collect::<Vec<String>>()
                                                    .join(" & "),
                                                old_v.clone().to_strict_string(false)
                                            )
                                            .into());
                                        }
                                    }
                                    (CType::AnyOf(oldts), _) => {
                                        let mut success = false;
                                        for ot in oldts {
                                            if ot.clone().to_functional_string()
                                                == v.clone().to_functional_string()
                                            {
                                                success = true;
                                                break;
                                            }
                                        }
                                        if !success {
                                            return Err(format!(
                                                "None of {} matches {}",
                                                oldts
                                                    .iter()
                                                    .map(|t| t.clone().to_strict_string(false))
                                                    .collect::<Vec<String>>()
                                                    .join(" & "),
                                                v.clone().to_strict_string(false)
                                            )
                                            .into());
                                        }
                                        generic_types.insert(k.clone(), v.clone());
                                    }
                                    (_, _) => {
                                        if old_v.clone().to_functional_string()
                                            != v.clone().to_functional_string()
                                        {
                                            return Err(format!(
                                                "{} does not match {}",
                                                old_v.clone().to_strict_string(false),
                                                v.clone().to_strict_string(false),
                                            )
                                            .into());
                                        }
                                    }
                                },
                            }
                        }
                    }
                    _ => {
                        return Err(format!("Mismatch between {a:?} and {i:?}").into());
                    }
                }
            }
        }
        Ok(())
    }
    pub fn infer_generics(
        scope: &Scope,
        generics: &[(String, Arc<CType>)],
        fn_args: &[(String, ArgKind, Arc<CType>)],
        call_args: &[Arc<CType>],
    ) -> Result<Vec<Arc<CType>>, Box<dyn std::error::Error>> {
        let mut temp_scope = scope.child();
        for (generic_name, generic_type) in generics {
            temp_scope
                .types
                .insert(generic_name.clone(), generic_type.clone());
        }
        let input_types = fn_args
            .iter()
            .map(|(_, _, t)| t.clone())
            .collect::<Vec<Arc<CType>>>();
        let mut generic_types: HashMap<String, Arc<CType>> = HashMap::new();
        CType::infer_generics_inner_loop(
            &temp_scope,
            &mut generic_types,
            call_args
                .iter()
                .zip(input_types.iter())
                .map(|(a, b)| (a.clone(), b.clone()))
                .collect::<Vec<(Arc<CType>, Arc<CType>)>>(),
        )?;
        let mut output_types = Vec::new();
        for (generic_name, _) in generics {
            output_types.push(match generic_types.get(generic_name) {
                Some(t) => Ok(t.clone()),
                None => Err(format!("No inferred type found for {generic_name}")),
            }?);
        }
        Ok(output_types)
    }
    pub fn accepts(self: Arc<CType>, arg: Arc<CType>) -> bool {
        match (&*self, &*arg) {
            (_a, CType::AnyOf(ts)) => {
                for t in ts {
                    if self.clone().accepts(t.clone()) {
                        return true;
                    }
                }
                false
            }
            (CType::Function(i1, _), CType::Generic(_, _, t))
                if matches!(&**t, CType::Function(..)) =>
            {
                if let CType::Function(i2, _) = &**t {
                    // TODO: Do this the right way with `infer_generics`, but I need to refactor a
                    // lot to get the scope into this function. For now, let's just assume if the
                    // lengths of the input tuples are the same, we're fine, and if not, we're not.
                    matches!((&**i1, &**i2), (CType::Tuple(ts1), CType::Tuple(ts2)) if ts1.len() == ts2.len())
                } else {
                    // Should be impossible
                    false
                }
            }
            // TODO: Do this without stringification
            (_a, _b) => self.clone().to_strict_string(false) == arg.clone().to_strict_string(false),
        }
    }

    pub fn to_functions(
        self: Arc<CType>,
        name: String,
        scope: &Scope,
    ) -> (CType, Vec<Arc<Function>>) {
        let t = Arc::new(CType::Type(name.clone(), self.clone()));
        let constructor_fn_name = t.clone().to_callable_string();
        let mut fs = Vec::new();
        match &*self {
            CType::Import(n, d) => match &**d {
                CType::TString(dep_name) => {
                    let program = Program::get_program();
                    let other_scope = program.scope_by_file(dep_name).unwrap();
                    match &**n {
                        CType::TString(name) => match other_scope.functions.get(name) {
                            None => CType::fail(&format!("{name} not found in {dep_name}")),
                            Some(dep_fs) => {
                                fs.append(&mut dep_fs.clone());
                            }
                        },
                        _ => CType::fail("The name of the import must be a string"),
                    };
                    Program::return_program(program);
                }
                _ => CType::fail("TODO: Support imports beyond local directories"),
            },
            CType::Call(n, f) => {
                let mut typen = f.clone().degroup();
                let args = type_to_args(typen.clone());
                let rettype = type_to_rettype(typen.clone());
                // Short-circuit for "normal" function binding with "normal" arguments only
                if args.iter().all(|(_, k, t)| {
                    matches!(k, ArgKind::Ref)
                        && !matches!(
                            &**t,
                            CType::Int(_) | CType::Float(_) | CType::Bool(_) | CType::TString(_)
                        )
                }) && matches!(&**n, CType::TString(_))
                {
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen,
                        microstatements: Vec::new(),
                        kind: FnKind::Bind(match &**n {
                            CType::TString(s) => s.clone(),
                            _ => unreachable!(),
                        }),
                        origin_scope_path: scope.path.clone(),
                    }));
                } else {
                    let mut microstatements = Vec::new();
                    let mut trimmed_args = false;
                    let mut kind = FnKind::Normal;
                    for (name, arg_kind, typen) in args.iter() {
                        match arg_kind {
                            ArgKind::Deref => {
                                microstatements.push(Microstatement::Assignment {
                                    mutable: true, // TODO: Determine this correctly
                                    name: name.clone(),
                                    value: Box::new(Microstatement::Value {
                                        typen: typen.clone(),
                                        representation: format!("*{name}"),
                                    }),
                                })
                            }
                            ArgKind::Own | ArgKind::Mut | ArgKind::Ref => {}
                        }
                    }
                    let call_name = match &**n {
                        CType::Import(n, d) => {
                            kind = FnKind::External(d.clone());
                            &**n
                        }
                        otherwise => otherwise,
                    };
                    match call_name {
                        CType::TString(s) => {
                            microstatements.push(Microstatement::Return {
                                value: Some(Box::new(Microstatement::Value {
                                    typen: rettype.clone(),
                                    representation: format!(
                                        "{}({})",
                                        s,
                                        args.iter()
                                            .map(|(name, _, typen)| {
                                                match &**typen {
                                                    CType::Int(i) => {
                                                        trimmed_args = true;
                                                        format!("{i}")
                                                    }
                                                    CType::Float(f) => {
                                                        trimmed_args = true;
                                                        format!("{f}")
                                                    }
                                                    CType::Bool(b) => {
                                                        trimmed_args = true;
                                                        match b {
                                                            true => "true".to_string(),
                                                            false => "false".to_string(),
                                                        }
                                                    }
                                                    CType::TString(s) => {
                                                        trimmed_args = true;
                                                        format!("\"{}\"", s.replace("\"", "\\\""))
                                                    }
                                                    _ => name.clone(),
                                                }
                                            })
                                            .collect::<Vec<String>>()
                                            .join(", ")
                                    ),
                                })),
                            });
                        }
                        CType::Infix(o) => match &**o {
                            CType::TString(s) => {
                                if args.len() != 2 {
                                    CType::fail("Native infix operators may only be bound with two input arguments");
                                }
                                microstatements.push(Microstatement::Return {
                                    value: Some(Box::new(Microstatement::Value {
                                        typen: rettype.clone(),
                                        representation: format!(
                                            "({} {} {})",
                                            match &*args[0].2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{i}")
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{f}")
                                                }
                                                CType::Bool(b) => {
                                                    trimmed_args = true;
                                                    match b {
                                                        true => "true".to_string(),
                                                        false => "false".to_string(),
                                                    }
                                                }
                                                CType::TString(s) => {
                                                    trimmed_args = true;
                                                    format!("\"{}\"", s.replace("\"", "\\\""))
                                                }
                                                _ => args[0].0.clone(),
                                            },
                                            s,
                                            match &*args[1].2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{i}")
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{f}")
                                                }
                                                CType::Bool(b) => {
                                                    trimmed_args = true;
                                                    match b {
                                                        true => "true".to_string(),
                                                        false => "false".to_string(),
                                                    }
                                                }
                                                CType::TString(s) => {
                                                    trimmed_args = true;
                                                    format!("\"{}\"", s.replace("\"", "\\\""))
                                                }
                                                _ => args[1].0.clone(),
                                            },
                                        ),
                                    })),
                                });
                            }
                            otherwise => CType::fail(&format!(
                                "Unsupported native operator declaration {otherwise:?}"
                            )),
                        },
                        CType::Prefix(o) => match &**o {
                            CType::TString(s) => {
                                if args.len() != 1 {
                                    CType::fail("Native prefix operators may only be bound with one input argument");
                                }
                                microstatements.push(Microstatement::Return {
                                    value: Some(Box::new(Microstatement::Value {
                                        typen: rettype.clone(),
                                        representation: format!(
                                            "({} {})",
                                            s,
                                            match &*args[0].2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{i}")
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{f}")
                                                }
                                                CType::Bool(b) => {
                                                    trimmed_args = true;
                                                    match b {
                                                        true => "true".to_string(),
                                                        false => "false".to_string(),
                                                    }
                                                }
                                                CType::TString(s) => {
                                                    trimmed_args = true;
                                                    format!("\"{}\"", s.replace("\"", "\\\""))
                                                }
                                                _ => args[0].0.clone(),
                                            },
                                        ),
                                    })),
                                });
                            }
                            otherwise => CType::fail(&format!(
                                "Unsupported native operator declaration {otherwise:?}"
                            )),
                        },
                        CType::Method(f) => match &**f {
                            CType::TString(s) => {
                                let arg_car = args[0].clone();
                                let arg_cdr = args.clone().split_off(1);
                                microstatements.push(Microstatement::Return {
                                    value: Some(Box::new(Microstatement::Value {
                                        typen: rettype.clone(),
                                        representation: format!(
                                            "{}.{}({})",
                                            match &*arg_car.2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{i}")
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{f}")
                                                }
                                                CType::Bool(b) => {
                                                    trimmed_args = true;
                                                    match b {
                                                        true => "true".to_string(),
                                                        false => "false".to_string(),
                                                    }
                                                }
                                                CType::TString(s) => {
                                                    trimmed_args = true;
                                                    format!("\"{}\"", s.replace("\"", "\\\""))
                                                }
                                                _ => arg_car.0.clone(),
                                            },
                                            s,
                                            arg_cdr
                                                .into_iter()
                                                .map(|a| match &*a.2 {
                                                    CType::Int(i) => {
                                                        trimmed_args = true;
                                                        format!("{i}")
                                                    }
                                                    CType::Float(f) => {
                                                        trimmed_args = true;
                                                        format!("{f}")
                                                    }
                                                    CType::Bool(b) => {
                                                        trimmed_args = true;
                                                        match b {
                                                            true => "true".to_string(),
                                                            false => "false".to_string(),
                                                        }
                                                    }
                                                    CType::TString(s) => {
                                                        trimmed_args = true;
                                                        format!("\"{}\"", s.replace("\"", "\\\""))
                                                    }
                                                    _ => a.0.clone(),
                                                })
                                                .collect::<Vec<String>>()
                                                .join(", ")
                                        ),
                                    })),
                                });
                            }
                            otherwise => CType::fail(&format!(
                                "Unsupported native method declaration {otherwise:?}"
                            )),
                        },
                        CType::Property(p) => match &**p {
                            CType::TString(s) => {
                                if args.len() > 1 {
                                    CType::fail(&format!("Property bindings may only have one argument, the value the property is accessed from. Not {args:?}"))
                                } else {
                                    let arg_car = args[0].clone();
                                    microstatements.push(Microstatement::Return {
                                        value: Some(Box::new(Microstatement::Value {
                                            typen: rettype.clone(),
                                            representation: format!(
                                                "{}.{}",
                                                match &*arg_car.2 {
                                                    CType::Int(i) => {
                                                        trimmed_args = true;
                                                        format!("{i}")
                                                    }
                                                    CType::Float(f) => {
                                                        trimmed_args = true;
                                                        format!("{f}")
                                                    }
                                                    CType::Bool(b) => {
                                                        trimmed_args = true;
                                                        match b {
                                                            true => "true".to_string(),
                                                            false => "false".to_string(),
                                                        }
                                                    }
                                                    CType::TString(s) => {
                                                        trimmed_args = true;
                                                        format!("\"{}\"", s.replace("\"", "\\\""))
                                                    }
                                                    _ => arg_car.0.clone(),
                                                },
                                                s,
                                            ),
                                        })),
                                    });
                                }
                            }
                            otherwise => CType::fail(&format!(
                                "Unsupported native method declaration {otherwise:?}"
                            )),
                        },
                        CType::Cast(t) => match &**t {
                            CType::TString(s) => {
                                if args.len() != 1 {
                                    CType::fail(
                                        "Native casting may only be bound with one input argument",
                                    );
                                }
                                microstatements.push(Microstatement::Return {
                                    value: Some(Box::new(Microstatement::Value {
                                        typen: rettype.clone(),
                                        representation: format!(
                                            "({} as {})",
                                            match &*args[0].2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{i}")
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{f}")
                                                }
                                                CType::Bool(b) => {
                                                    trimmed_args = true;
                                                    match b {
                                                        true => "true".to_string(),
                                                        false => "false".to_string(),
                                                    }
                                                }
                                                CType::TString(s) => {
                                                    trimmed_args = true;
                                                    format!("\"{}\"", s.replace("\"", "\\\""))
                                                }
                                                _ => args[0].0.clone(),
                                            },
                                            s
                                        ),
                                    })),
                                });
                            }
                            otherwise => CType::fail(&format!(
                                "Unsupported native cast declaration {otherwise:?}"
                            )),
                        },
                        otherwise => CType::fail(&format!(
                            "Unsupported native operator declaration {otherwise:?}"
                        )),
                    }
                    if trimmed_args {
                        typen = Arc::new(CType::Function(
                            Arc::new(CType::Tuple(
                                args.into_iter()
                                    .filter(|(_, _, typen)| {
                                        !matches!(
                                            &**typen,
                                            CType::Int(_)
                                                | CType::Float(_)
                                                | CType::Bool(_)
                                                | CType::TString(_)
                                        )
                                    })
                                    .map(|(n, k, t)| {
                                        Arc::new(CType::Field(
                                            n,
                                            match k {
                                                ArgKind::Own => Arc::new(CType::Own(t)),
                                                ArgKind::Deref => Arc::new(CType::Deref(t)),
                                                ArgKind::Mut => Arc::new(CType::Mut(t)),
                                                ArgKind::Ref => t,
                                            },
                                        ))
                                    })
                                    .collect::<Vec<Arc<CType>>>(),
                            )),
                            rettype,
                        ));
                    }
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen,
                        microstatements,
                        kind,
                        origin_scope_path: scope.path.clone(),
                    }));
                }
            }
            CType::Type(n, _) => {
                // This is just an alias, but avoid circular derives
                if name != constructor_fn_name {
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Field(n.clone(), self.clone())),
                            t.clone(),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                    }));
                }
            }
            CType::Tuple(ts) => {
                // The constructor function needs to grab the types from all
                // arguments to construct the desired product type. For any type
                // that is marked as a field, we also want to create an accessor
                // function for it to simulate structs better.
                // Create accessor functions for static tag values in the tuple, if any exist
                let mut actual_ts = Vec::new();
                for ti in ts.iter().filter(|t1| match &***t1 {
                    CType::Field(_, t2) => matches!(
                        &**t2,
                        CType::TString(_) | CType::Int(_) | CType::Float(_) | CType::Bool(_)
                    ),
                    CType::TString(_) | CType::Int(_) | CType::Float(_) | CType::Bool(_) => true,
                    _ => false,
                }) {
                    match &**ti {
                        CType::Field(n, f) => {
                            match &**f {
                                CType::TString(s) => {
                                    // Create an accessor function for this value, but do not add
                                    // it to the args array to construct it. The accessor function
                                    // will return this value as a string.
                                    let string = scope.resolve_type("string").unwrap().clone();
                                    fs.push(Arc::new(Function {
                                        name: n.clone(),
                                        typen: Arc::new(CType::Function(t.clone(), string.clone())),
                                        microstatements: vec![Microstatement::Value {
                                            typen: string,
                                            representation: format!(
                                                "\"{}\"",
                                                s.replace("\"", "\\\"")
                                            ),
                                        }],
                                        kind: FnKind::Static,
                                        origin_scope_path: scope.path.clone(),
                                    }));
                                }
                                CType::Int(i) => {
                                    // Create an accessor function for this value, but do not add
                                    // it to the args array to construct it. The accessor function
                                    // will return this value as an i64.
                                    let int64 = scope.resolve_type("i64").unwrap().clone();
                                    fs.push(Arc::new(Function {
                                        name: n.clone(),
                                        typen: Arc::new(CType::Function(t.clone(), int64.clone())),
                                        microstatements: vec![Microstatement::Value {
                                            typen: int64,
                                            representation: format!("{i}"),
                                        }],
                                        kind: FnKind::Static,
                                        origin_scope_path: scope.path.clone(),
                                    }));
                                }
                                CType::Float(f) => {
                                    // Create an accessor function for this value, but do not add
                                    // it to the args array to construct it. The accessor function
                                    // will return this value as an f64.
                                    let float64 = scope.resolve_type("f64").unwrap().clone();
                                    fs.push(Arc::new(Function {
                                        name: n.clone(),
                                        typen: Arc::new(CType::Function(
                                            t.clone(),
                                            float64.clone(),
                                        )),
                                        microstatements: vec![Microstatement::Value {
                                            typen: float64,
                                            representation: format!("{f}"),
                                        }],
                                        kind: FnKind::Static,
                                        origin_scope_path: scope.path.clone(),
                                    }));
                                }
                                CType::Bool(b) => {
                                    // Create an accessor function for this value, but do not add
                                    // it to the args array to construct it. The accessor function
                                    // will return this value as a bool.
                                    let booln = scope.resolve_type("bool").unwrap().clone();
                                    fs.push(Arc::new(Function {
                                        name: n.clone(),
                                        typen: Arc::new(CType::Function(t.clone(), booln.clone())),
                                        microstatements: vec![Microstatement::Value {
                                            typen: booln,
                                            representation: match b {
                                                true => "true".to_string(),
                                                false => "false".to_string(),
                                            },
                                        }],
                                        kind: FnKind::Static,
                                        origin_scope_path: scope.path.clone(),
                                    }));
                                }
                                _ => { /* Do nothing */ }
                            }
                        }
                        _ => { /* Do nothing */ }
                    }
                }
                for (i, ti) in ts
                    .iter()
                    .filter(|t1| match &***t1 {
                        CType::Field(_, t2) => !matches!(
                            &**t2,
                            CType::TString(_) | CType::Int(_) | CType::Float(_) | CType::Bool(_)
                        ),
                        CType::TString(_) | CType::Int(_) | CType::Float(_) | CType::Bool(_) => {
                            false
                        }
                        _ => true,
                    })
                    .enumerate()
                {
                    actual_ts.push(ti.clone());
                    match &**ti {
                        CType::Field(n, f) => {
                            // Create an accessor function
                            fs.push(Arc::new(Function {
                                name: n.clone(),
                                typen: Arc::new(CType::Function(t.clone(), f.clone())),
                                microstatements: Vec::new(),
                                kind: FnKind::Derived,
                                origin_scope_path: scope.path.clone(),
                            }));
                        }
                        _otherwise => {
                            // Create an `<N>` function accepting the tuple by field number
                            fs.push(Arc::new(Function {
                                name: format!("{i}"),
                                typen: Arc::new(CType::Function(t.clone(), ti.clone())),
                                microstatements: Vec::new(),
                                kind: FnKind::Derived,
                                origin_scope_path: scope.path.clone(),
                            }));
                        }
                    }
                }
                // Define the constructor function
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(
                        Arc::new(CType::Tuple(actual_ts.clone())),
                        t.clone(),
                    )),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                }));
            }
            CType::Field(n, f) => {
                // This is a "baby tuple" of just one value. So we follow the Tuple logic, but
                // simplified.
                match &**f {
                    CType::TString(s) => {
                        // Create an accessor function for this value, but do not add
                        // it to the args array to construct it. The accessor function
                        // will return this value as a string.
                        let string = scope.resolve_type("string").unwrap().clone();
                        fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(t.clone(), string.clone())),
                            microstatements: vec![Microstatement::Value {
                                typen: string,
                                representation: s.clone(),
                            }],
                            kind: FnKind::Static,
                            origin_scope_path: scope.path.clone(),
                        }));
                    }
                    CType::Int(i) => {
                        // Create an accessor function for this value, but do not add
                        // it to the args array to construct it. The accessor function
                        // will return this value as an i64.
                        let int64 = scope.resolve_type("i64").unwrap().clone();
                        fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(t.clone(), int64.clone())),
                            microstatements: vec![Microstatement::Value {
                                typen: int64,
                                representation: format!("{i}"),
                            }],
                            kind: FnKind::Static,
                            origin_scope_path: scope.path.clone(),
                        }));
                    }
                    CType::Float(f) => {
                        // Create an accessor function for this value, but do not add
                        // it to the args array to construct it. The accessor function
                        // will return this value as an f64.
                        let float64 = scope.resolve_type("f64").unwrap().clone();
                        fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(t.clone(), float64.clone())),
                            microstatements: vec![Microstatement::Value {
                                typen: float64,
                                representation: format!("{f}"),
                            }],
                            kind: FnKind::Static,
                            origin_scope_path: scope.path.clone(),
                        }));
                    }
                    CType::Bool(b) => {
                        // Create an accessor function for this value, but do not add
                        // it to the args array to construct it. The accessor function
                        // will return this value as a bool.
                        let booln = scope.resolve_type("bool").unwrap().clone();
                        fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(t.clone(), booln.clone())),
                            microstatements: vec![Microstatement::Value {
                                typen: booln,
                                representation: match b {
                                    true => "true".to_string(),
                                    false => "false".to_string(),
                                },
                            }],
                            kind: FnKind::Static,
                            origin_scope_path: scope.path.clone(),
                        }));
                    }
                    _ => {
                        fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(t.clone(), f.clone())),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                        }));
                    }
                }
                // Define the constructor function
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(f.clone(), t.clone())),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                }));
            }
            CType::Either(ts) => {
                // There are an equal number of constructor functions and accessor
                // functions, one for each inner type of the sum type.
                for e in ts {
                    // Create a constructor fn
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple(vec![Arc::new(CType::Field(
                                "arg0".to_string(),
                                e.clone(),
                            ))])),
                            t.clone(),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                    }));
                    // Create a store fn to re-assign-and-auto-wrap a value
                    fs.push(Arc::new(Function {
                        name: "store".to_string(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple(vec![t.clone(), e.clone()])),
                            t.clone(),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                    }));
                    if let CType::Void = &**e {
                        // Have a zero-arg constructor function produce the void type, if possible.
                        fs.push(Arc::new(Function {
                            name: constructor_fn_name.clone(),
                            typen: Arc::new(CType::Function(Arc::new(CType::Void), t.clone())),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                        }));
                    }
                    // Create the accessor function, the name of the function will
                    // depend on the kind of type this is
                    match &**e {
                        CType::Field(n, i) => fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(
                                t.clone(),
                                Arc::new(CType::Either(vec![i.clone(), Arc::new(CType::Void)])),
                            )),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                        })),
                        CType::Type(n, _) => fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(
                                t.clone(),
                                Arc::new(CType::Either(vec![e.clone(), Arc::new(CType::Void)])),
                            )),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                        })),
                        _ => {} // We can't make names for other types
                    }
                }
            }
            CType::Buffer(b, s) => {
                // For Buffers we can create up to two types, one that takes a
                // single value to fill in for all records in the buffer, and one
                // that takes a distinct value for each possible value in the
                // buffer. If the buffer size is just one element, we only
                // implement one of these
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(b.clone(), t.clone())),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                }));
                let size = match **s {
                    CType::Int(s) => s as usize,
                    _ => 0, // TODO: Make this function fallible, instead?
                };
                if size > 1 {
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple({
                                let mut v = Vec::new();
                                for _ in 0..size {
                                    v.push(b.clone());
                                }
                                v
                            })),
                            t.clone(),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                    }));
                }
                // Also include accessor functions for each
                for i in 0..size {
                    fs.push(Arc::new(Function {
                        name: format!("{i}"),
                        typen: Arc::new(CType::Function(t.clone(), b.clone())),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                    }))
                }
            }
            CType::Array(a) => {
                // For Arrays we create only one kind of array, one that takes any
                // number of the input type. Until there's better support in the
                // language for variadic functions, this is faked with a special
                // DerivedVariadic function type that repeats the first and only
                // arg for all input arguments. We also need to create `get` and
                // `set` functions for this type (TODO: This is probably true for
                // other types, too.
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(a.clone(), t.clone())),
                    microstatements: Vec::new(),
                    kind: FnKind::DerivedVariadic,
                    origin_scope_path: scope.path.clone(),
                }));
            }
            CType::Int(i) => {
                // TODO: Support construction of other integer types
                let int64 = scope.resolve_type("i64").unwrap().clone();
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(Arc::new(CType::Void), int64.clone())),
                    microstatements: vec![Microstatement::Return {
                        value: Some(Box::new(Microstatement::Value {
                            typen: int64,
                            representation: format!("{i}"),
                        })),
                    }],
                    kind: FnKind::Normal,
                    origin_scope_path: scope.path.clone(),
                }));
            }
            CType::Float(f) => {
                // TODO: Support construction of other float types
                let float64 = scope.resolve_type("f64").unwrap().clone();
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(Arc::new(CType::Void), float64.clone())),
                    microstatements: vec![Microstatement::Return {
                        value: Some(Box::new(Microstatement::Value {
                            typen: float64,
                            representation: format!("{f}"),
                        })),
                    }],
                    kind: FnKind::Normal,
                    origin_scope_path: scope.path.clone(),
                }));
            }
            CType::Bool(b) => {
                // A special exception exists for a few booleans that are created *before* the bool
                // type is created in the root scope. TODO: Find a better solution for this so they
                // have accessor functions defined for run-time code to use them.
                if let Some(boolt) = scope.resolve_type("bool") {
                    let booln = boolt.clone();
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(Arc::new(CType::Void), booln.clone())),
                        microstatements: vec![Microstatement::Return {
                            value: Some(Box::new(Microstatement::Value {
                                typen: booln,
                                representation: match b {
                                    true => "true".to_string(),
                                    false => "false".to_string(),
                                },
                            })),
                        }],
                        kind: FnKind::Normal,
                        origin_scope_path: scope.path.clone(),
                    }));
                }
            }
            CType::TString(s) => {
                let string = scope.resolve_type("string").unwrap().clone();
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(Arc::new(CType::Void), string.clone())),
                    microstatements: vec![Microstatement::Return {
                        value: Some(Box::new(Microstatement::Value {
                            typen: string.clone(),
                            representation: format!("\"{}\"", s.replace("\"", "\\\"")),
                        })),
                    }],
                    kind: FnKind::Normal,
                    origin_scope_path: scope.path.clone(),
                }));
                // Also include the original name if it doesn't match. TODO: Figure out why these
                // aren't resolving in the same way
                if constructor_fn_name != name {
                    fs.push(Arc::new(Function {
                        name: name.clone(),
                        typen: Arc::new(CType::Function(Arc::new(CType::Void), string.clone())),
                        microstatements: vec![Microstatement::Return {
                            value: Some(Box::new(Microstatement::Value {
                                typen: string,
                                representation: format!("\"{}\"", s.replace("\"", "\\\"")),
                            })),
                        }],
                        kind: FnKind::Normal,
                        origin_scope_path: scope.path.clone(),
                    }));
                }
            }
            _ => {} // Don't do anything for other types
        }
        (CType::clone(&t), fs)
    }
    pub fn from_ast<'a>(
        mut scope: Scope<'a>,
        type_ast: &parse::Types,
        is_export: bool,
    ) -> Result<(Scope<'a>, CType), Box<dyn std::error::Error>> {
        let name = type_ast.fulltypename.typename.clone();
        if let Some(generics) = &type_ast.opttypegenerics {
            // We are going to conditionally compile this type declaration. If the we get true, we
            // continue, if we get false, we don't compile and return a Fail type that isn't added
            // to the scope to cause compilation to crash *if* something tries to use this, and if
            // we don't get a boolean at all or we get multiple inner values in the generic call,
            // we bail out immediately because of a syntax error.
            let generic_call = withtypeoperatorslist_to_ctype(&generics.typecalllist, &scope)?;
            match &*generic_call {
                CType::Bool(b) => match b {
                    false => return Ok((scope, CType::Fail(format!("{name} is not supposed to be compiled because the conditional compilation generic value is false")))),
                    true => { /* Do nothing */ }
                },
                CType::Type(n, c) => match &**c {
                    CType::Bool(b) => match b {
                        false => return Ok((scope, CType::Fail(format!("{name} is not supposed to be compiled because {n} is false")))),
                        true => { /* Do nothing */ }
                    },
                    _ => {
                        return Err(format!(
                        "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                        name,
                        generics.to_string()
                    )
                        .into())
                    }
                },
                _ => {
                    return Err(format!(
                    "Invalid conditional compilation for type {}, {} does not resolve to a boolean",
                    name,
                    generics.to_string()
                )
                    .into())
                }
            }
        }

        let (t, fs) = match &type_ast.fulltypename.opttypegenerics {
            None => {
                // This is a "normal" type
                // When creating a "normal" type, we also create constructor and optionally
                // accessor functions. This is not done for bound types nor done for
                // generics until the generic type has been constructed. We create a set of
                // `derived` Function objects and add it to the scope that a later stage of
                // the compiler is responsible for actually creating. All of the types get
                // one or more constructor functions, while struct-like Tuples and Either
                // get accessor functions to dig into the sub-types.
                let mut inner_type =
                    withtypeoperatorslist_to_ctype(&type_ast.typedef.typeassignables, &scope)?;
                // Unwrap a Group type, if any exists, we don't want it here.
                while matches!(&*inner_type, CType::Group(_)) {
                    inner_type = match &*inner_type {
                        CType::Group(t) => t.clone(),
                        _t => inner_type,
                    };
                }
                // Let's just avoid the "bare field" type definition and auto-wrap into a tuple
                if let CType::Field(..) = &*inner_type {
                    inner_type = Arc::new(CType::Tuple(vec![inner_type]));
                }
                // Magic hackery to convert a `From` type into an `Import` type if it's the top-level type
                inner_type = match &*inner_type {
                    CType::From(t) => {
                        CType::import(Arc::new(CType::TString(name.clone())), t.clone())
                    }
                    _t => inner_type,
                };
                // If we've got an `Import` type, we need to grab the actual type definition from
                // the other file and pull it in here.
                if let CType::Import(name, dep) = &*inner_type {
                    match &**dep {
                        CType::TString(dep_name) => {
                            let program = Program::get_program();
                            let scope = program.scope_by_file(dep_name)?;
                            match &**name {
                                CType::TString(n) => {
                                    inner_type = match scope.types.get(n) {
                                        None => {
                                            CType::fail(&format!("{n} not found in {dep_name}"))
                                        }
                                        Some(t) => match &**t {
                                            CType::Type(_, t) => t.clone(),
                                            _t => t.clone(),
                                        },
                                    }
                                }
                                _ => CType::fail("The name of the import must be a string"),
                            };
                            Program::return_program(program);
                        }
                        _ => CType::fail("TODO: Support imports beyond local directories"),
                    }
                }
                inner_type.to_functions(name.clone(), &scope)
            }
            Some(g) => {
                // This is a "generic" type
                // TODO: Stronger checking on the usage here
                let args = g
                    .typecalllist
                    .iter()
                    .map(|tc| tc.to_string())
                    .collect::<Vec<String>>()
                    .join("")
                    .split(", ")
                    .map(|r| r.trim().to_string())
                    .collect::<Vec<String>>();
                let mut temp_scope = scope.child();
                for arg in &args {
                    temp_scope.types.insert(
                        arg.clone(),
                        Arc::new(CType::Infer(arg.clone(), "Any".to_string())),
                    );
                }
                let generic_call =
                    withtypeoperatorslist_to_ctype(&type_ast.typedef.typeassignables, &temp_scope)?;
                (CType::Generic(name.clone(), args, generic_call), Vec::new())
            }
        };
        if is_export {
            scope.exports.insert(name.clone(), Export::Type);
            if !fs.is_empty() {
                let mut names = HashSet::new();
                for f in &fs {
                    names.insert(f.name.clone());
                }
                for name in names {
                    scope.exports.insert(name.clone(), Export::Function);
                }
            }
        }
        let insert_t = Arc::new(t.clone());
        scope.types.insert(name.clone(), insert_t.clone());
        scope
            .types
            .insert(insert_t.clone().to_callable_string(), insert_t.clone());
        if !fs.is_empty() {
            let mut name_fn_pairs = HashMap::new();
            for f in fs {
                if name_fn_pairs.contains_key(&f.name) {
                    let v: &mut Vec<Arc<Function>> = name_fn_pairs.get_mut(&f.name).unwrap();
                    v.push(f.clone());
                } else {
                    name_fn_pairs.insert(f.name.clone(), vec![f.clone()]);
                }
            }
            for (name, fns) in name_fn_pairs.drain() {
                if scope.functions.contains_key(&name) {
                    let func_vec = scope.functions.get_mut(&name).unwrap();
                    func_vec.splice(0..0, fns);
                } else {
                    scope.functions.insert(name, fns);
                }
            }
        }
        Ok((scope, t))
    }

    pub fn from_ctype(mut scope: Scope, name: String, ctype: Arc<CType>) -> Scope {
        scope.exports.insert(name.clone(), Export::Type);
        let (_, fs) = ctype.clone().to_functions(name.clone(), &scope);
        scope.types.insert(name, ctype.clone());
        scope
            .types
            .insert(ctype.clone().to_callable_string(), ctype);
        if !fs.is_empty() {
            let mut name_fn_pairs = HashMap::new();
            for f in fs {
                // We need to similarly load all of the return types from the functions created by
                // this from_ctype call if they don't already exist
                let mut contains_rettype = false;
                let retstr = f.rettype().to_functional_string();
                for t in scope.types.values() {
                    if retstr == t.clone().to_functional_string() {
                        contains_rettype = true;
                    }
                }
                if !contains_rettype {
                    scope = CType::from_ctype(scope, retstr, f.rettype().clone());
                }
                if name_fn_pairs.contains_key(&f.name) {
                    let v: &mut Vec<Arc<Function>> = name_fn_pairs.get_mut(&f.name).unwrap();
                    v.push(f.clone());
                } else {
                    name_fn_pairs.insert(f.name.clone(), vec![f.clone()]);
                }
            }
            for (name, fns) in name_fn_pairs.drain() {
                if scope.functions.contains_key(&name) {
                    let func_vec = scope.functions.get_mut(&name).unwrap();
                    func_vec.splice(0..0, fns);
                } else {
                    scope.functions.insert(name, fns);
                }
            }
        }
        scope
    }

    pub fn from_generic<'a>(scope: Scope<'a>, name: &str, arglen: usize) -> Scope<'a> {
        CType::from_ctype(
            scope,
            name.to_string(),
            Arc::new(CType::IntrinsicGeneric(name.to_string(), arglen)),
        )
    }
    pub fn swap_subtype(
        self: Arc<CType>,
        old_type: Arc<CType>,
        new_type: Arc<CType>,
    ) -> Arc<CType> {
        // Implemented recursively to be easier to follow. It would be nice to avoid all of the
        // cloning if the old type is not anywhere in the CType tree, but that would be a lot
        // harder to detect ahead of time.
        if self == old_type {
            return new_type;
        }
        match &*self {
            CType::Void
            | CType::Infer(..)
            | CType::Generic(..)
            | CType::IntrinsicGeneric(..)
            | CType::Int(_)
            | CType::Float(_)
            | CType::Bool(_)
            | CType::TString(_)
            | CType::Fail(_) => self.clone(),
            CType::Type(name, ct) => Arc::new(CType::Type(
                name.clone(),
                ct.clone().swap_subtype(old_type, new_type),
            )),
            CType::Binds(name, gen_type_resolved) => Arc::new(CType::Binds(
                name.clone()
                    .swap_subtype(old_type.clone(), new_type.clone()),
                gen_type_resolved
                    .iter()
                    .map(|gtr| gtr.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::IntCast(i) => CType::intcast(i.clone().swap_subtype(old_type, new_type)),
            CType::FloatCast(f) => CType::floatcast(f.clone().swap_subtype(old_type, new_type)),
            CType::BoolCast(b) => CType::boolcast(b.clone().swap_subtype(old_type, new_type)),
            CType::StringCast(s) => CType::stringcast(s.clone().swap_subtype(old_type, new_type)),
            CType::Group(g) => g.clone().swap_subtype(old_type, new_type),
            CType::Unwrap(t) => CType::tunwrap(t.clone().swap_subtype(old_type, new_type)),
            CType::Function(i, o) => Arc::new(CType::Function(
                i.clone().swap_subtype(old_type.clone(), new_type.clone()),
                o.clone().swap_subtype(old_type, new_type),
            )),
            CType::Call(n, f) => Arc::new(CType::Call(
                n.clone().swap_subtype(old_type.clone(), new_type.clone()),
                f.clone().swap_subtype(old_type, new_type),
            )),
            CType::Infix(o) => Arc::new(CType::Infix(o.clone().swap_subtype(old_type, new_type))),
            CType::Prefix(o) => Arc::new(CType::Prefix(o.clone().swap_subtype(old_type, new_type))),
            CType::Method(f) => Arc::new(CType::Method(f.clone().swap_subtype(old_type, new_type))),
            CType::Property(p) => {
                Arc::new(CType::Property(p.clone().swap_subtype(old_type, new_type)))
            }
            CType::Cast(t) => Arc::new(CType::Cast(t.clone().swap_subtype(old_type, new_type))),
            CType::Own(t) => Arc::new(CType::Own(t.clone().swap_subtype(old_type, new_type))),
            CType::Deref(t) => Arc::new(CType::Deref(t.clone().swap_subtype(old_type, new_type))),
            CType::Mut(t) => Arc::new(CType::Mut(t.clone().swap_subtype(old_type, new_type))),
            CType::Dependency(n, v) => Arc::new(CType::Dependency(
                n.clone().swap_subtype(old_type.clone(), new_type.clone()),
                v.clone().swap_subtype(old_type, new_type),
            )),
            CType::Rust(d) => Arc::new(CType::Rust(d.clone().swap_subtype(old_type, new_type))),
            CType::Nodejs(d) => Arc::new(CType::Nodejs(d.clone().swap_subtype(old_type, new_type))),
            CType::From(d) => Arc::new(CType::From(d.clone().swap_subtype(old_type, new_type))),
            CType::Import(n, d) => Arc::new(CType::Import(
                n.clone().swap_subtype(old_type.clone(), new_type.clone()),
                d.clone().swap_subtype(old_type, new_type),
            )),
            CType::Tuple(ts) => Arc::new(CType::Tuple(
                ts.iter()
                    .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Field(name, t) => Arc::new(CType::Field(
                name.clone(),
                t.clone().swap_subtype(old_type, new_type),
            )),
            CType::Either(ts) => Arc::new(CType::Either(
                ts.iter()
                    .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Prop(t, p) => CType::prop(
                t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                p.clone().swap_subtype(old_type, new_type),
            ),
            CType::AnyOf(ts) => Arc::new(CType::AnyOf(
                ts.iter()
                    .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                    .collect::<Vec<Arc<CType>>>(),
            )),
            CType::Buffer(t, size) => Arc::new(CType::Buffer(
                t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                size.clone().swap_subtype(old_type, new_type),
            )),
            CType::Array(t) => Arc::new(CType::Array(t.clone().swap_subtype(old_type, new_type))),
            // For these when we swap, we check to see if we can "condense" them down into simpler
            // types (eg `Add{N, 1}` swapping `N` for `3` should just yield `4`)
            CType::Add(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::add)
                .unwrap(),
            CType::Sub(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::sub)
                .unwrap(),
            CType::Mul(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::mul)
                .unwrap(),
            CType::Div(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::div)
                .unwrap(),
            CType::Mod(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::cmod)
                .unwrap(),
            CType::Pow(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::pow)
                .unwrap(),
            CType::Min(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::min)
                .unwrap(),
            CType::Max(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::max)
                .unwrap(),
            CType::Neg(t) => CType::neg(t.clone().swap_subtype(old_type, new_type)),
            CType::Len(t) => CType::len(t.clone().swap_subtype(old_type, new_type)),
            CType::Size(t) => CType::size(t.clone().swap_subtype(old_type, new_type)),
            CType::FileStr(t) => CType::filestr(t.clone().swap_subtype(old_type, new_type)),
            CType::Concat(a, b) => CType::concat(
                a.clone().swap_subtype(old_type.clone(), new_type.clone()),
                b.clone().swap_subtype(old_type, new_type),
            ),
            CType::Env(ts) => {
                if ts.len() == 1 {
                    CType::env(ts[0].clone().swap_subtype(old_type, new_type))
                } else if ts.len() == 2 {
                    CType::envdefault(
                        ts[0]
                            .clone()
                            .swap_subtype(old_type.clone(), new_type.clone()),
                        ts[1]
                            .clone()
                            .swap_subtype(old_type.clone(), new_type.clone()),
                    )
                } else {
                    CType::fail("Somehow gave Env{..} an incorrect number of args and caught during generic resolution")
                }
            }
            CType::EnvExists(t) => CType::envexists(t.clone().swap_subtype(old_type, new_type)),
            CType::TIf(t, ts) => {
                if ts.len() == 1 {
                    CType::tupleif(
                        t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                        ts[0].clone().swap_subtype(old_type, new_type),
                    )
                } else if ts.len() == 2 {
                    CType::cif(
                        t.clone().swap_subtype(old_type.clone(), new_type.clone()),
                        ts[0]
                            .clone()
                            .swap_subtype(old_type.clone(), new_type.clone()),
                        ts[1].clone().swap_subtype(old_type, new_type),
                    )
                } else {
                    CType::fail("Somehow gave If{..} an incorrect number of args and caught during generic resolution")
                }
            }
            CType::And(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::and)
                .unwrap(),
            CType::Or(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::or)
                .unwrap(),
            CType::Xor(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::xor)
                .unwrap(),
            CType::Not(t) => CType::not(t.clone().swap_subtype(old_type, new_type)),
            CType::Nand(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::nand)
                .unwrap(),
            CType::Nor(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::nor)
                .unwrap(),
            CType::Xnor(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::xnor)
                .unwrap(),
            CType::TEq(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::eq)
                .unwrap(),
            CType::Neq(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::neq)
                .unwrap(),
            CType::Lt(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::lt)
                .unwrap(),
            CType::Lte(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::lte)
                .unwrap(),
            CType::Gt(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::gt)
                .unwrap(),
            CType::Gte(ts) => ts
                .iter()
                .map(|t| t.clone().swap_subtype(old_type.clone(), new_type.clone()))
                .reduce(CType::gte)
                .unwrap(),
        }
    }

    pub fn binds(args: Vec<Arc<CType>>) -> Arc<CType> {
        let base_type = args[0].clone();
        if matches!(
            &*base_type,
            CType::TString(_) | CType::Import(..) | CType::From(_)
        ) {
            let mut out_vec = Vec::new();
            #[allow(clippy::needless_range_loop)] // It's not needless
            for i in 1..args.len() {
                out_vec.push(args[i].clone());
            }
            Arc::new(CType::Binds(base_type, out_vec))
        } else {
            CType::fail(
                "Binds{T, ...} must be given a string or an import for the base type to bind",
            );
        }
    }

    pub fn intcast(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::IntCast(arg))
        } else {
            match &*arg {
                CType::Int(_) => arg,
                CType::Float(f) => Arc::new(CType::Int(*f as i128)),
                CType::Bool(b) => Arc::new(if *b { CType::Int(1) } else { CType::Int(0) }),
                CType::TString(s) => match s.parse::<i128>() {
                    Ok(v) => Arc::new(CType::Int(v)),
                    Err(e) => Arc::new(CType::Fail(format!("{e:?}"))),
                },
                _ => Arc::new(CType::Fail("Not implemented".into())),
            }
        }
    }

    pub fn floatcast(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::FloatCast(arg))
        } else {
            match &*arg {
                CType::Float(_) => arg,
                CType::Int(i) => Arc::new(CType::Float(*i as f64)),
                CType::Bool(b) => Arc::new(if *b {
                    CType::Float(1.0)
                } else {
                    CType::Float(0.0)
                }),
                CType::TString(s) => match s.parse::<f64>() {
                    Ok(v) => Arc::new(CType::Float(v)),
                    Err(e) => Arc::new(CType::Fail(format!("{e:?}"))),
                },
                _ => Arc::new(CType::Fail("Not implemented".into())),
            }
        }
    }

    pub fn boolcast(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::BoolCast(arg))
        } else {
            match &*arg {
                CType::Bool(_) => arg,
                CType::Float(f) => Arc::new(CType::Bool(*f != 0.0)),
                CType::Int(i) => Arc::new(CType::Bool(*i != 0)),
                CType::TString(s) => Arc::new(CType::Bool(s == "true")),
                _ => Arc::new(CType::Fail("Not implemented".into())),
            }
        }
    }

    pub fn stringcast(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::StringCast(arg))
        } else {
            Arc::new(CType::TString(CType::to_functional_string(arg)))
        }
    }

    pub fn tunwrap(arg: Arc<CType>) -> Arc<CType> {
        if arg.clone().has_infer() {
            Arc::new(CType::Unwrap(arg))
        } else {
            match &*arg {
                CType::Type(_, t) | CType::Group(t) | CType::Unwrap(t) => t.clone(),
                _ => arg,
            }
        }
    }

    pub fn import(name: Arc<CType>, dep: Arc<CType>) -> Arc<CType> {
        if let CType::Infer(..) = &*name {
            Arc::new(CType::Import(name, dep))
        } else if let CType::Infer(..) = &*dep {
            Arc::new(CType::Import(name, dep))
        } else if !matches!(&*name, CType::TString(_)) {
            CType::fail("The Import{N, D} N parameter must be a string")
        } else {
            match &*dep {
                CType::TString(s) => {
                    // Load the dependency
                    if let Err(e) = Program::load(s.clone()) {
                        CType::fail(&format!("Failed to load dependency {s}: {e:?}"))
                    } else {
                        let program = Program::get_program();
                        let out = match program.scope_by_file(s) {
                            Err(e) => CType::fail(&format!("Failed to load dependency {s}: {e:?}")),
                            Ok(dep_scope) => {
                                // Currently can only import types and functions. Constants and
                                // operator mappings don't have a syntax to express this. TODO:
                                // Figure out how to tackle this syntactically, and then update
                                // this logic.
                                if let CType::TString(n) = &*name {
                                    let found = dep_scope.types.contains_key(n)
                                        || dep_scope.functions.contains_key(n);
                                    if !found {
                                        CType::fail(&format!("{n} not found in {s}"))
                                    } else {
                                        // We're good
                                        Arc::new(CType::Import(name, dep))
                                    }
                                } else {
                                    CType::fail("The Import{N, D} N parameter must be a string")
                                }
                            }
                        };
                        Program::return_program(program);
                        out
                    }
                }
                CType::Dependency(..) => CType::fail("TODO: Alan package import support"),
                CType::Nodejs(_) | CType::Rust(_) => Arc::new(CType::Import(name, dep)),
                CType::Type(_, t) if matches!(**t, CType::Nodejs(_) | CType::Rust(_)) => {
                    Arc::new(CType::Import(name, dep))
                }
                otherwise => CType::fail(&format!(
                    "Invalid import defined {} <- {}",
                    name.clone().to_functional_string(),
                    Arc::new(otherwise.clone()).to_functional_string()
                )),
            }
        }
    }
    // Special implementation for the tuple and either types since they *are* CTypes, but if one of
    // the provided input types *is* the same kind of CType, it should produce a merged version.
    pub fn tuple(args: Vec<Arc<CType>>) -> Arc<CType> {
        let mut out_vec = Vec::new();
        for arg in args {
            match &*arg {
                CType::Tuple(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                _other => out_vec.push(arg),
            }
        }
        Arc::new(CType::Tuple(out_vec))
    }
    pub fn either(args: Vec<Arc<CType>>) -> Arc<CType> {
        let mut out_vec = Vec::new();
        for arg in args {
            match &*arg {
                CType::Either(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                _other => out_vec.push(arg),
            }
        }
        Arc::new(CType::Either(out_vec))
    }
    pub fn prop(t: Arc<CType>, p: Arc<CType>) -> Arc<CType> {
        // Check the arguments first to see if they're to be inferred
        if t.clone().has_infer() || p.clone().has_infer() {
            return Arc::new(CType::Prop(t, p));
        }
        match &*t {
            CType::Infer(..) => unreachable!(),
            CType::Type(_, t) => CType::prop(t.clone(), p),
            CType::Group(t) => CType::prop(t.clone(), p),
            CType::Field(n, f) => match &*p {
                CType::TString(s) => {
                    if n == s {
                        f.clone()
                    } else {
                        Arc::new(CType::Fail(format!(
                            "Property {} not found on type {:?}",
                            s, &t
                        )))
                    }
                }
                CType::Int(i) => match i {
                    0 => Arc::new(CType::TString(n.to_string())),
                    1 => f.clone(),
                    _ => Arc::new(CType::Fail(
                        "Only 0 or 1 are valid integer accesses on a field".into(),
                    )),
                },
                otherwise => Arc::new(CType::Fail(format!(
                    "Properties must be a name or integer location, not {otherwise:?}",
                ))),
            },
            CType::Tuple(ts) | CType::Either(ts) => match &*p {
                CType::TString(s) => {
                    for inner in ts {
                        if let CType::Field(n, f) = &**inner {
                            if n == s {
                                return f.clone();
                            }
                        }
                    }
                    Arc::new(CType::Fail(format!("Property {s} not found on type {t:?}")))
                }
                CType::Int(i) => {
                    if (0..ts.len()).contains(&(*i as usize)) {
                        ts[*i as usize].clone()
                    } else {
                        Arc::new(CType::Fail(format!("{i} is out of bounds for type {t:?}")))
                    }
                }
                otherwise => Arc::new(CType::Fail(format!(
                    "Properties must be a name or integer location, not {otherwise:?}",
                ))),
            },
            CType::TIf(_, tf) => {
                match &*p {
                    CType::TString(s) => {
                        // TODO: Is this path reachable?
                        if s == "true" {
                            tf[0].clone()
                        } else if s == "false" {
                            tf[1].clone()
                        } else {
                            CType::fail("Only true or false (or 1 or 0) are valid for accessing the types from an If{C, A, B} type")
                        }
                    }
                    CType::Bool(b) => {
                        if *b {
                            tf[0].clone()
                        } else {
                            tf[1].clone()
                        }
                    }
                    CType::Int(i) => {
                        if (0..2).contains(i) {
                            tf[*i as usize].clone()
                        } else {
                            CType::fail("Only true or false (or 1 or 0) are valid for accessing the types from an If{C, A, B} type")
                        }
                    }
                    otherwise => CType::fail(&format!(
                        "Properties must be a name or integer location, not {otherwise:?}",
                    )),
                }
            }
            otherwise => CType::fail(&format!(
                "Properties cannot be accessed from type {otherwise:?}"
            )),
        }
    }
    pub fn anyof(args: Vec<Arc<CType>>) -> Arc<CType> {
        let mut out_vec = Vec::new();
        for arg in args {
            match &*arg {
                CType::AnyOf(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                _other => out_vec.push(arg),
            }
        }
        Arc::new(CType::Either(out_vec))
    }
    pub fn field(mut args: Vec<Arc<CType>>) -> Arc<CType> {
        if args.len() != 2 {
            CType::fail("Field{K, V} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            match (&*arg0, &*arg1) {
                (CType::TString(key), anything) => {
                    Arc::new(CType::Field(key.clone(), Arc::new(anything.clone())))
                }
                _ => CType::fail("The field key must be a quoted string at this time"),
            }
        }
    }
    // Some validation for buffer creation, too
    pub fn buffer(mut args: Vec<Arc<CType>>) -> Arc<CType> {
        if args.len() != 2 {
            CType::fail("Buffer{T, S} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap().degroup();
            let arg0 = args.pop().unwrap().degroup();
            match (&*arg0, &*arg1) {
                (CType::Infer(..), _) => Arc::new(CType::Buffer(arg0.clone(), arg1.clone())),
                (_, CType::Infer(..)) => Arc::new(CType::Buffer(arg0.clone(), arg1.clone())),
                (_, CType::Int(size)) => {
                    if *size < 0 {
                        CType::fail("The buffer size must be a positive integer")
                    } else {
                        Arc::new(CType::Buffer(arg0, Arc::new(CType::Int(*size))))
                    }
                }
                otherwise => CType::fail(&format!(
                    "The buffer size must be a positive integer {otherwise:?}"
                )),
            }
        }
    }
    // Implementation of the ctypes that aren't storage but compute into another CType
    pub fn fail(message: &str) -> ! {
        // TODO: Include more information on where this compiler exit is coming from
        eprintln!("{message}");
        std::process::exit(1);
    }
    pub fn cfail(message: Arc<CType>) -> Arc<CType> {
        match &*message {
            CType::TString(s) => Arc::new(CType::Fail(s.clone())),
            _ => CType::fail("Fail passed a type that does not resolve into a message string"),
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn neg(t: Arc<CType>) -> Arc<CType> {
        match &*t {
            CType::Int(v) => Arc::new(CType::Int(-v)),
            CType::Float(v) => Arc::new(CType::Float(-v)),
            CType::Infer(..) => Arc::new(CType::Neg(t)),
            _ => CType::fail("Attempting to negate non-integer or non-float types at compile time"),
        }
    }
    pub fn len(t: Arc<CType>) -> Arc<CType> {
        match &*t {
            CType::Tuple(tup) => Arc::new(CType::Int(tup.len() as i128)),
            CType::Buffer(_, l) => match **l {
                CType::Int(l) => Arc::new(CType::Int(l)),
                _ => {
                    CType::fail("Cannot get a compile time length for an invalid Buffer definition")
                }
            },
            CType::Either(eit) => Arc::new(CType::Int(eit.len() as i128)),
            CType::Array(_) => {
                CType::fail("Cannot get a compile time length for a variable-length array")
            }
            CType::Infer(..) => Arc::new(CType::Len(t)),
            _ => Arc::new(CType::Int(1)),
        }
    }
    pub fn size(t: Arc<CType>) -> Arc<CType> {
        // TODO: Implementing this might require all types be made C-style structs under the hood,
        // and probably some weird hackery to find out the size including padding on aligned
        // architectures, so I might take it back out before its actually implemented, but I can
        // think of several places where knowing the actual size of the type could be useful,
        // particularly for writing to disk or interfacing with network protocols, etc, so I'd
        // prefer to keep it and have some compile-time guarantees we don't normally see.
        match &*t {
            CType::Void => Arc::new(CType::Int(0)),
            CType::Infer(..) => Arc::new(CType::Size(t.clone())),
            CType::Type(_, t) => CType::size(t.clone()),
            CType::Generic(..) => CType::fail("Cannot determine the size of an unbound generic"),
            CType::Binds(t, ts) => {
                if !ts.is_empty() {
                    CType::fail("Cannot determine the size of an unbound generic")
                } else {
                    Arc::new(match &**t {
                        CType::TString(n) if n == "i8" => CType::Int(1),
                        CType::TString(n) if n == "u8" => CType::Int(1),
                        CType::TString(n) if n == "i16" => CType::Int(2),
                        CType::TString(n) if n == "u16" => CType::Int(2),
                        CType::TString(n) if n == "i32" => CType::Int(4),
                        CType::TString(n) if n == "u32" => CType::Int(4),
                        CType::TString(n) if n == "f32" => CType::Int(4),
                        CType::TString(n) if n == "i64" => CType::Int(8),
                        CType::TString(n) if n == "u64" => CType::Int(8),
                        CType::TString(n) if n == "f64" => CType::Int(8),
                        CType::TString(n) => {
                            CType::fail(&format!("Cannot determine the size of {n}"))
                        }
                        _ => CType::fail(&format!(
                            "Cannot determine the size of {}",
                            t.clone().to_functional_string()
                        )),
                    })
                }
            }
            CType::IntrinsicGeneric(..) => {
                CType::fail("Cannot determine the size of an unbound generic")
            }
            CType::Int(_) | CType::Float(_) => Arc::new(CType::Int(8)),
            CType::Bool(_) => Arc::new(CType::Int(1)),
            CType::TString(s) => Arc::new(CType::Int(s.capacity() as i128)),
            CType::Group(t) | CType::Field(_, t) => CType::size(t.clone()),
            CType::Tuple(ts) => {
                let sizes = ts
                    .clone()
                    .into_iter()
                    .map(CType::size)
                    .collect::<Vec<Arc<CType>>>();
                let mut out_size = 0;
                for t in sizes {
                    match &*t {
                        CType::Int(s) => out_size += s,
                        _ => unreachable!(),
                    }
                }
                Arc::new(CType::Int(out_size))
            }
            CType::Either(ts) => {
                let sizes = ts
                    .clone()
                    .into_iter()
                    .map(CType::size)
                    .collect::<Vec<Arc<CType>>>();
                let mut out_size = 0;
                for t in sizes {
                    match &*t {
                        CType::Int(s) => out_size = i128::max(out_size, *s),
                        _ => unreachable!(),
                    }
                }
                Arc::new(CType::Int(out_size))
            }
            CType::Buffer(b, s) => {
                let base_size = CType::size(b.clone());
                match (&*base_size, &**s) {
                    (CType::Int(a), CType::Int(b)) => Arc::new(CType::Int(a + b)),
                    (CType::Infer(..), _) | (_, CType::Infer(..)) => {
                        Arc::new(CType::Size(b.clone()))
                    }
                    _ => unreachable!(),
                }
            }
            CType::Array(_) => {
                CType::fail("Cannot determine the size of an array, it's length is not static")
            }
            CType::Function(..)
            | CType::Call(..)
            | CType::Infix(_)
            | CType::Prefix(_)
            | CType::Method(_)
            | CType::Property(_) => CType::fail("Cannot determine the size of a function"),
            _ => CType::fail(&format!(
                "Getting the size of {} doesn't make any sense",
                t.to_functional_string()
            )),
        }
    }
    pub fn filestr(f: Arc<CType>) -> Arc<CType> {
        match &*f {
            CType::TString(s) => match std::fs::read_to_string(s) {
                Err(e) => CType::fail(&format!("Failed to read {s}: {e:?}")),
                Ok(s) => Arc::new(CType::TString(s)),
            },
            CType::Infer(..) => f,
            _ => CType::fail("FileStr{F} must be given a string path to load"),
        }
    }
    pub fn concat(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        match (&*a, &*b) {
            (CType::Infer(..), _) | (_, CType::Infer(..)) => Arc::new(CType::Concat(a, b)),
            (CType::TString(a), CType::TString(b)) => Arc::new(CType::TString(format!("{a}{b}"))),
            _ => CType::fail("Concat{A, B} must be given strings to concatenate"),
        }
    }
    pub fn env(k: Arc<CType>) -> Arc<CType> {
        let program = Program::get_program();
        let out = match &*k {
            CType::TString(s) => match program.env.get(s) {
                None => CType::fail(&format!("Failed to load environment variable {s}",)),
                Some(s) => CType::TString(s.clone()),
            },
            CType::Infer(..) => CType::Env(vec![k.clone()]),
            _ => CType::fail("Env{K} must be given a key as a string to load"),
        };
        Program::return_program(program);
        Arc::new(out)
    }
    pub fn envexists(k: Arc<CType>) -> Arc<CType> {
        let program = Program::get_program();
        let out = match &*k {
            CType::TString(s) => CType::Bool(program.env.contains_key(s)),
            CType::Infer(..) => CType::EnvExists(k),
            _ => CType::fail("EnvExists{K} must be given a key as a string to check"),
        };
        Program::return_program(program);
        Arc::new(out)
    }
    #[allow(clippy::should_implement_trait)]
    pub fn not(b: Arc<CType>) -> Arc<CType> {
        Arc::new(match &*b {
            CType::Bool(b) => CType::Bool(!*b),
            CType::Int(b) => CType::Int(!*b),
            CType::Infer(..) => CType::Not(b),
            _ => CType::fail("Not{B} must be provided a boolean or integer type to invert"),
        })
    }
    pub fn min(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(if a < b { a } else { b }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(if a < b { a } else { b }),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Min(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Min(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to min non-integer or non-float types together at compile time",
            ),
        })
    }
    pub fn max(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(if a > b { a } else { b }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(if a > b { a } else { b }),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Max(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Max(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to max non-integer or non-float types together at compile time",
            ),
        })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn add(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a + b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a + b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Add(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Add(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to add non-integer or non-float types together at compile time",
            ),
        })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn sub(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a - b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a - b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Sub(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Sub(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to subtract non-integer or non-float types together at compile time",
            ),
        })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn mul(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a * b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Mul(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Mul(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to multiply non-integer or non-float types together at compile time",
            ),
        })
    }
    #[allow(clippy::should_implement_trait)]
    pub fn div(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a / b),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a / b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Div(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Div(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to divide non-integer or non-float types together at compile time",
            ),
        })
    }
    pub fn cmod(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_)) => {
                CType::Mod(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_), &CType::Infer(..)) => CType::Mod(vec![a.clone(), b.clone()]),
            _ => CType::fail("Attempting to modulus non-integer types together at compile time"),
        })
    }
    pub fn pow(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        Arc::new(match (&*a, &*b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(match a.checked_pow(b as u32) {
                Some(c) => c,
                None => CType::fail("Compile time exponentiation too large"),
            }),
            (&CType::Float(a), &CType::Float(b)) => CType::Float(a.powf(b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_)) => {
                CType::Pow(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_), &CType::Infer(..)) => {
                CType::Pow(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Attempting to divide non-integer or non-float types together at compile time",
            ),
        })
    }
    pub fn cif(c: Arc<CType>, a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        match &*CType::tunwrap(c.clone().degroup()) {
            CType::Bool(cond) => match cond {
                true => a.clone(),
                false => b.clone(),
            },
            CType::Infer(..) => Arc::new(CType::TIf(c.clone(), vec![a.clone(), b.clone()])),
            _ => CType::fail("If{C, A, B} must be given a boolean value as the condition"),
        }
    }
    pub fn tupleif(c: Arc<CType>, t: Arc<CType>) -> Arc<CType> {
        match &*c {
            CType::Bool(cond) => {
                match &*t {
                    CType::Tuple(tup) => {
                        if tup.len() == 2 {
                            match cond {
                                true => tup[0].clone(),
                                false => tup[1].clone(),
                            }
                        } else {
                            CType::fail("The tuple type provided to If{C, T} must have exactly two elements")
                        }
                    }
                    _ => CType::fail(
                        "The second type provided to If{C, T} must be a tuple of two types",
                    ),
                }
            }
            CType::Infer(..) => Arc::new(CType::TIf(c.clone(), vec![t.clone()])),
            _ => CType::fail("The first type provided to If{C, T} must be a boolean type"),
        }
    }
    pub fn envdefault(k: Arc<CType>, d: Arc<CType>) -> Arc<CType> {
        let program = Program::get_program();
        let out = match (&*k, &*d) {
            (CType::TString(s), CType::TString(def)) => match program.env.get(s) {
                None => CType::TString(def.clone()),
                Some(v) => CType::TString(v.clone()),
            },
            (CType::Infer(..), CType::TString(_))
            | (CType::TString(_), CType::Infer(..))
            | (CType::Infer(..), CType::Infer(..)) => CType::Env(vec![k.clone(), d.clone()]),
            _ => CType::fail("Env{K, D} must be provided a string for each type"),
        };
        Program::return_program(program);
        Arc::new(out)
    }
    pub fn and(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::and(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::and(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a & *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a && *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::And(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::And(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "And{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        })
    }
    pub fn or(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::or(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::or(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a | *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a || *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Or(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Or(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Or{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        })
    }
    pub fn xor(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::xor(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::xor(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a ^ *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a ^ *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Xor(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Xor(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Xor{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        })
    }
    pub fn nand(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::nand(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::nand(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a & *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a && *b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Nand(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Nand(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Nand{A, B} must be provided two values of the same type, either integer or boolean")
        })
    }
    pub fn nor(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::nor(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::nor(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a | *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a || *b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Nor(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Nor(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Nor{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        })
    }
    pub fn xnor(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::xnor(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::xnor(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a ^ *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a ^ *b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Xnor(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Xnor(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Xnor{A, B} must be provided two values of the same type, either integer or boolean")
        })
    }
    pub fn eq(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::eq(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::eq(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a == *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a == *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a == *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a == *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_) | &CType::Bool(_)) => {
                CType::TEq(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::TEq(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Eq{A, B} must be provided two values of the same type, one of: integer, float, string, boolean"),
        })
    }
    pub fn neq(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::neq(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::neq(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a != *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a != *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a != *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a != *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_) | &CType::Bool(_)) => {
                CType::Neq(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Neq(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Neq{A, B} must be provided two values of the same type, one of: integer, float, string, boolean"),
        })
    }
    pub fn lt(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::lt(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::lt(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a < *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a < *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a < *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_)) => {
                CType::Lt(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_), &CType::Infer(..)) => {
                CType::Lt(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Lt{A, B} must be provided two values of the same type, one of: integer, float, string"),
        })
    }
    pub fn lte(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::lte(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::lte(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a <= *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a <= *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a <= *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_)) => {
                CType::Lte(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_), &CType::Infer(..)) => {
                CType::Lte(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Lte{A, B} must be provided two values of the same type, one of: integer, float, string"),
        })
    }
    pub fn gt(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::gt(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::gt(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a > *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a > *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a > *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_)) => {
                CType::Gt(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_), &CType::Infer(..)) => {
                CType::Gt(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Gt{A, B} must be provided two values of the same type, one of: integer, float, string"),
        })
    }
    pub fn gte(a: Arc<CType>, b: Arc<CType>) -> Arc<CType> {
        if let CType::Type(_, t) = &*a {
            return CType::gte(t.clone(), b);
        }
        if let CType::Type(_, t) = &*b {
            return CType::gte(a, t.clone());
        }
        Arc::new(match (&*a, &*b) {
            (CType::Int(a), CType::Int(b)) => CType::Bool(*a >= *b),
            (CType::Float(a), CType::Float(b)) => CType::Bool(*a >= *b),
            (CType::TString(a), CType::TString(b)) => CType::Bool(*a >= *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Float(_) | &CType::TString(_)) => {
                CType::Gte(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Float(_) | &CType::TString(_), &CType::Infer(..)) => {
                CType::Gte(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Gte{A, B} must be provided two values of the same type, one of: integer, float, string"),
        })
    }
}

// TODO: I really hoped these two would share more code. Figure out how to DRY this out later, if
// possible
pub fn withtypeoperatorslist_to_ctype(
    withtypeoperatorslist: &Vec<parse::WithTypeOperators>,
    scope: &Scope,
) -> Result<Arc<CType>, Box<dyn std::error::Error>> {
    // To properly linearize the operations here, we need to scan through all of the operators,
    // determine which is the highest precedence, whether it is infix or prefix (or maybe postfix
    // in the future?) and then process them and whichever of the baseassignables surrounding them
    // are associated, then put those results in the same "slot" as before and check again. Because
    // users can define these operators, that makes it theoretically possible for the same operator
    // to be used in both an infix or prefix manner, or with different precedence levels, depending
    // on the types of the data involved, which makes things *really* complicated here. TODO:
    // Actually implement that complexity, for now, just pretend operators have only one binding.
    let mut queue = withtypeoperatorslist.clone();
    let mut out_ctype = None;
    while !queue.is_empty() {
        let mut largest_operator_level: i8 = -1;
        let mut largest_operator_index: i64 = -1;
        for (i, assignable_or_operator) in queue.iter().enumerate() {
            if let parse::WithTypeOperators::Operators(o) = assignable_or_operator {
                let operatorname = &o.op;
                let operator = match scope.resolve_typeoperator(operatorname) {
                    Some(o) => Ok(o),
                    None => Err(format!("Operator {operatorname} not found")),
                }?;
                let level = match &operator {
                    TypeOperatorMapping::Prefix { level, .. } => level,
                    TypeOperatorMapping::Infix { level, .. } => level,
                    TypeOperatorMapping::Postfix { level, .. } => level,
                };
                if level > &largest_operator_level {
                    largest_operator_level = *level;
                    largest_operator_index = i as i64;
                }
            }
        }
        if largest_operator_index > -1 {
            // We have at least one operator, and this is the one to dig into
            let operatorname = match &queue[largest_operator_index as usize] {
                parse::WithTypeOperators::Operators(o) => &o.op,
                _ => unreachable!(),
            };
            let operator = match scope.resolve_typeoperator(operatorname) {
                Some(o) => Ok(o),
                None => Err(format!("Operator {operatorname} not found")),
            }?;
            let functionname = match operator {
                TypeOperatorMapping::Prefix { functionname, .. } => functionname.clone(),
                TypeOperatorMapping::Infix { functionname, .. } => functionname.clone(),
                TypeOperatorMapping::Postfix { functionname, .. } => functionname.clone(),
            };
            let is_infix = match operator {
                TypeOperatorMapping::Prefix { .. } => false,
                TypeOperatorMapping::Postfix { .. } => false,
                TypeOperatorMapping::Infix { .. } => true,
            };
            let is_prefix = match operator {
                TypeOperatorMapping::Prefix { .. } => true,
                TypeOperatorMapping::Postfix { .. } => false,
                TypeOperatorMapping::Infix { .. } => false,
            };
            if is_infix {
                // Confirm that we have records before and after the operator and that they are
                // baseassignables.
                let first_arg = match match queue.get(largest_operator_index as usize - 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {operatorname} is an infix operator but missing a left-hand side value"
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is an infix operator but preceded by another operator {}",
                        operatorname, o.op
                    )),
                }?;
                let second_arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!("Operator {operatorname} is an infix operator but missing a right-hand side value")),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!("Operator{} is an infix operator but followed by a lower precedence operator {}", operatorname, o.op)),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `a + b` and turn it into `add(a, b)`
                let rewrite = parse::WithTypeOperators::TypeBaseList(vec![
                    parse::TypeBase::Variable(functionname),
                    parse::TypeBase::GnCall(parse::GnCall {
                        opencurly: "{".to_string(),
                        a: "".to_string(),
                        typecalllist: vec![
                            parse::WithTypeOperators::TypeBaseList(first_arg.to_vec()),
                            parse::WithTypeOperators::Operators(
                                parse::TypeOperatorsWithWhitespace {
                                    a: " ".to_string(),
                                    op: ",".to_string(),
                                    b: " ".to_string(),
                                },
                            ),
                            parse::WithTypeOperators::TypeBaseList(second_arg.to_vec()),
                        ],
                        b: "".to_string(),
                        closecurly: "}".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithTypeOperators> = queue
                    .splice(
                        (largest_operator_index as usize - 1)
                            ..(largest_operator_index as usize + 2),
                        vec![rewrite],
                    )
                    .collect();
            } else if is_prefix {
                // Confirm that we have a record after the operator and that it's a baseassignables
                let arg = match match queue.get(largest_operator_index as usize + 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {operatorname} is a prefix operator but missing a right-hand side value"
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is a prefix operator but followed by another operator {}",
                        operatorname, o.op
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `#array` and turn it into `len(array)`
                let rewrite = parse::WithTypeOperators::TypeBaseList(vec![
                    parse::TypeBase::Variable(functionname),
                    parse::TypeBase::GnCall(parse::GnCall {
                        opencurly: "{".to_string(),
                        a: "".to_string(),
                        typecalllist: vec![parse::WithTypeOperators::TypeBaseList(arg.to_vec())],
                        b: "".to_string(),
                        closecurly: "}".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithTypeOperators> = queue
                    .splice(
                        (largest_operator_index as usize)..(largest_operator_index as usize + 2),
                        vec![rewrite],
                    )
                    .collect();
            } else {
                let arg = match match queue.get(largest_operator_index as usize - 1) {
                    Some(val) => Ok(val),
                    None => Err(format!(
                        "Operator {operatorname} is a postfix operator but missing a left-hand side value"
                    )),
                }? {
                    parse::WithTypeOperators::TypeBaseList(typebaselist) => Ok(typebaselist),
                    parse::WithTypeOperators::Operators(o) => Err(format!(
                        "Operator {} is a postfix operator but preceded by another operator {}",
                        operatorname, o.op
                    )),
                }?;
                // We're gonna rewrite the operator and base assignables into a function call, eg
                // we take `type?` and turn it into `Maybe{type}`
                let rewrite = parse::WithTypeOperators::TypeBaseList(vec![
                    parse::TypeBase::Variable(functionname),
                    parse::TypeBase::GnCall(parse::GnCall {
                        opencurly: "{".to_string(),
                        a: "".to_string(),
                        typecalllist: vec![parse::WithTypeOperators::TypeBaseList(arg.to_vec())],
                        b: "".to_string(),
                        closecurly: "}".to_string(),
                    }),
                ]);
                // Splice the new record into the processing queue
                let _: Vec<parse::WithTypeOperators> = queue
                    .splice(
                        (largest_operator_index as usize - 1)
                            ..(largest_operator_index as usize + 1),
                        vec![rewrite],
                    )
                    .collect();
            }
        } else {
            // We have no more typeoperators, there should only be one reworked typebaselist now
            if queue.len() != 1 {
                // No idea how such a wonky thing could occur. TODO: Improve error message
                return Err(format!("Invalid syntax: {withtypeoperatorslist:?}").into());
            }
            let typebaselist = match match queue.pop() {
                Some(v) => Ok(v),
                None => Err(format!(
                    "Somehow we collapsed the statement into nothing? {withtypeoperatorslist:?}"
                )),
            }? {
                parse::WithTypeOperators::TypeBaseList(b) => Ok(b),
                _ => Err(format!(
                    "Somehow we collapse the statement into a solitary operator? {withtypeoperatorslist:?}"
                )),
            }?;
            out_ctype = Some(typebaselist_to_ctype(&typebaselist, scope)?);
        }
    }
    match out_ctype {
        Some(ctype) => Ok(ctype),
        None => Err(format!("Never resolved a type from {withtypeoperatorslist:?}").into()),
    }
}

// TODO: This similarly shares a lot of structure with baseassignablelist_to_microstatements, see
// if there is any way to DRY this up, or is it just doomed to be like this?
pub fn typebaselist_to_ctype(
    typebaselist: &[parse::TypeBase],
    scope: &Scope,
) -> Result<Arc<CType>, Box<dyn std::error::Error>> {
    let mut i = 0;
    let mut prior_value = None;
    while i < typebaselist.len() {
        let typebase = &typebaselist[i];
        let nexttypebase = typebaselist.get(i + 1);
        match typebase {
            parse::TypeBase::Constants(c) => {
                // With constants, there are a few situations where they are allowed:
                // 1) When they're used within a `GnCall` as the sole value passed in
                // 2) When they're used as the property of a type, but only in certain scenarios.
                // 2a) If it's an integer indexing into a tuple type or an either type, it returns
                // the type of that element in the tuple or either.
                // 2b) If it's a string indexing into a labeled tuple type (aka a struct), it
                // returns the type of that element in the tuple.
                // 2c) If it's a string that is specifically "input" or "output" indexing on a
                // function type, it returns the input or output type (function types could
                // internally have been a struct-like type with two fields, but they're special for
                // now)
                if let Some(next) = nexttypebase {
                    match next {
                        parse::TypeBase::Variable(_) => {
                            return Err(format!("A constant cannot be directly before a variable without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::TypeBase::GnCall(_) => {
                            return Err(format!("A constant cannot be directly before a generic function call without an operator and type name between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::TypeBase::TypeGroup(_) => {
                            return Err(format!("A constant cannot be directly before a parenthetical grouping without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::TypeBase::Constants(_) => {
                            return Err(format!("A constant cannot be directly before another constant without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                    }
                }
                if prior_value.is_none() {
                    match c {
                        parse::Constants::Bool(b) => {
                            prior_value = Some(Arc::new(CType::Bool(b.as_str() == "true")))
                        }
                        parse::Constants::Strn(s) => {
                            prior_value = Some(Arc::new(CType::TString(if s.starts_with('"') {
                                s.split("\\\"")
                                    .map(|sub| sub.replace("\"", ""))
                                    .collect::<Vec<String>>()
                                    .join("\"")
                            } else {
                                s.split("\\'")
                                    .map(|sub| sub.replace("'", ""))
                                    .collect::<Vec<String>>()
                                    .join("'")
                            })))
                        }
                        parse::Constants::Num(n) => match n {
                            parse::Number::RealNum(r) => {
                                prior_value = Some(Arc::new(CType::Float(
                                    r.replace('_', "").parse::<f64>().unwrap(), // This should never fail if the
                                                                                // parser says it's a float
                                )))
                            }
                            parse::Number::IntNum(i) => {
                                prior_value = Some(Arc::new(CType::Int(
                                    i.replace('_', "").parse::<i128>().unwrap(), // Same deal here
                                )))
                            }
                        },
                    }
                } else {
                    // There are broadly two cases where this can be reasonable: tuple-like access
                    // with integers and struct-like access with strings
                    match c {
                        parse::Constants::Bool(_) => {
                            return Err(format!("A boolean cannot follow another value without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                        }
                        parse::Constants::Strn(s) => {
                            prior_value = Some(match &*prior_value.unwrap() {
                                CType::Tuple(ts) => {
                                    let mut out = None;
                                    for t in ts {
                                        if let CType::Field(f, c) = &**t {
                                            if f.as_str() == s.as_str() {
                                                out = Some(c.clone());
                                            }
                                        }
                                    }
                                    match out {
                                        Some(o) => o,
                                        None => CType::fail(&format!("{ts:?} does not have a property named {s}")),
                                    }
                                }
                                CType::Function(i, o) => match s.as_str() {
                                    "input" => i.clone(),
                                    "output" => o.clone(),
                                    _ => CType::fail("Function types only have \"input\" and \"output\" properties"),
                                }
                                other => CType::fail(&format!("String properties are not allowed on {other:?}")),
                            });
                        }
                        parse::Constants::Num(n) => {
                            match n {
                                parse::Number::RealNum(_) => {
                                    return Err(format!("A floating point number cannot follow another value without an operator between them: {}", typebaselist.iter().map(|tb| tb.to_string()).collect::<Vec<String>>().join("")).into());
                                }
                                parse::Number::IntNum(i) => {
                                    let idx = match i.parse::<usize>() {
                                    Ok(idx) => idx,
                                    Err(_) => CType::fail("Indexing into a type must be done with positive integers"),
                                };
                                    prior_value = Some(match &*prior_value.unwrap() {
                                        CType::Tuple(ts) => match ts.get(idx) {
                                            Some(t) => t.clone(),
                                            None => CType::fail(&format!(
                                                "{idx} is larger than the size of {ts:?}"
                                            )),
                                        },
                                        CType::Either(ts) => match ts.get(idx) {
                                            Some(t) => t.clone(),
                                            None => CType::fail(&format!(
                                                "{idx} is larger than the size of {ts:?}"
                                            )),
                                        },
                                        other => CType::fail(&format!(
                                            "{other:?} cannot be indexed by an integer"
                                        )),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            parse::TypeBase::Variable(var) => {
                // Variables can be used to access sub-types in a type, or used as method-style
                // execution of a prior value. For method access, if the function takes only one
                // argument, it should still work even if the follow-on curly braces are not
                // written, so there's a little bit of extra logic added here for that situation,
                // otherwise it's handled by the GnCall path. When it's a property access, it
                // replaces the prior CType with the sub-type of the prior value.
                // For the simpler case when it's *just* a reference to a prior variable, it just
                // becomes a `Type` CType providing an alias for the named type.
                let mut args = Vec::new();
                if let Some(val) = &prior_value {
                    args.push(val.clone())
                };
                prior_value = Some(match scope.resolve_type(var) {
                    Some(t) => {
                        // TODO: Once interfaces are a thing, there needs to be a built-in
                        // interface called `Label` that we can use here to mark the first argument
                        // to `Field` as a `Label` and turn this logic into something regularized
                        // For now, we're just special-casing the `Field` built-in generic type.
                        match &*t {
                            CType::IntrinsicGeneric(p, 2) if p == "Prop" => {
                                match nexttypebase {
                                    None => {},
                                    Some(next) => match next {
                                        parse::TypeBase::GnCall(g) => {
                                            // There should be only two args, the first arg is
                                            // coerced from a variable to a string, the second arg
                                            // is treated like normal
                                            if g.typecalllist.len() != 3 {
                                                CType::fail("The Prop generic type accepts only two parameters");
                                            }
                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[0].clone()], scope)?);
                                            match g.typecalllist[0].to_string().parse::<i128>() {
                                                Ok(i) => args.push(Arc::new(CType::Int(i))),
                                                Err(_) => {
                                                    if let parse::WithTypeOperators::TypeBaseList(tbl) = &g.typecalllist[2] {
                                                        if tbl.len() > 1 {
                                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[2].clone()], scope)?);
                                                        } else {
                                                            let argstr = g.typecalllist[2].to_string();
                                                            match argstr.as_str() {
                                                                "true" => args.push(Arc::new(CType::Bool(true))),
                                                                "false" => args.push(Arc::new(CType::Bool(false))),
                                                                _ => args.push(Arc::new(CType::TString(argstr)))
                                                            }
                                                        }
                                                    } else {
                                                        CType::fail("huh?")
                                                    }
                                                }
                                            }
                                        }
                                        _ => CType::fail("Cannot follow method style syntax without an operator in between"),
                                    }
                                }
                            }
                            CType::IntrinsicGeneric(f, 2) if f == "Field" => {
                                match nexttypebase {
                                    None => {},
                                    Some(next) => match next {
                                        parse::TypeBase::GnCall(g) => {
                                            // There should be only two args, the first arg is
                                            // coerced from a variable to a string, the second arg
                                            // is treated like normal
                                            if g.typecalllist.len() != 3 {
                                                CType::fail("The Field generic type accepts only two parameters");
                                            }
                                            // Special hack to de-stringify the field label if and
                                            // only if it's trying to cast to a string here
                                            let label = g.typecalllist[0].to_string();
                                            if label.starts_with("String{") {
                                                args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[0].clone()], scope)?);
                                            } else {
                                                args.push(Arc::new(CType::TString(label)));
                                            }
                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[2].clone()], scope)?);
                                        }
                                        _ => CType::fail("Cannot follow method style syntax without an operator in between"),
                                    }
                                }
                            }
                            _ => {
                                match nexttypebase {
                                    None => {},
                                    Some(next) => match next {
                                        parse::TypeBase::GnCall(g) => {
                                            // Unfortunately ambiguous, but commas behave
                                            // differently in here, so we're gonna chunk this,
                                            // split by commas, then iterate on those chunks
                                            let mut temp_args = Vec::new();
                                            for ta in &g.typecalllist {
                                                temp_args.push(ta.clone());
                                            }
                                            let mut arg_block = Vec::new();
                                            for arg in temp_args {
                                                if let parse::WithTypeOperators::Operators(o) = &arg {
                                                    if o.op == "," {
                                                        // Process the arg block that has
                                                        // accumulated
                                                        args.push(withtypeoperatorslist_to_ctype(&arg_block, scope)?);
                                                        arg_block.clear();
                                                        continue;
                                                    }
                                                }
                                                arg_block.push(arg);
                                            }
                                            if !arg_block.is_empty() {
                                                args.push(withtypeoperatorslist_to_ctype(&arg_block, scope)?);
                                            }
                                        }
                                        _ => CType::fail("Cannot follow method style syntax without an operator in between"),
                                    }
                                }
                            }
                        }
                        // Now, we need to validate that the resolved type *is* a generic
                        // type that can be called, and that we have the correct number of
                        // arguments for it, then we can call it and return the resulting
                        // type
                        match &*t {
                            CType::Generic(_name, params, generic_type) => {
                                if params.len() != args.len() {
                                    CType::fail(&format!(
                                        "Generic type {} takes {} arguments but {} given",
                                        var,
                                        params.len(),
                                        args.len()
                                    ))
                                } else {
                                    // We use a temporary scope to resolve the
                                    // arguments to the generic function as the
                                    // specified names
                                    let mut out_type = generic_type.clone();
                                    for i in 0..params.len() {
                                        let generic_arg = Arc::new(CType::Infer(
                                            params[i].clone(),
                                            "Any".to_string(),
                                        ));
                                        out_type =
                                            out_type.swap_subtype(generic_arg, args[i].clone());
                                    }
                                    // Now we return the type we resolve within this
                                    // scope
                                    out_type
                                }
                            }
                            CType::IntrinsicGeneric(name, len) => {
                                if *len != 0 && args.len() != *len {
                                    CType::fail(&format!(
                                        "Generic type {} takes {} arguments but {} given",
                                        var,
                                        len,
                                        args.len()
                                    ))
                                } else {
                                    // TODO: Is there a better way to do this?
                                    match name.as_str() {
                                        "Binds" => CType::binds(args),
                                        "Int" => CType::intcast(args[0].clone()),
                                        "Float" => CType::floatcast(args[0].clone()),
                                        "Bool" => CType::boolcast(args[0].clone()),
                                        "String" => CType::stringcast(args[0].clone()),
                                        "Group" => Arc::new(CType::Group(args[0].clone())),
                                        "Unwrap" => CType::tunwrap(args[0].clone()),
                                        "Function" => Arc::new(CType::Function(
                                            args[0].clone(),
                                            args[1].clone(),
                                        )),
                                        "Call" => {
                                            Arc::new(CType::Call(args[0].clone(), args[1].clone()))
                                        }
                                        "Infix" => Arc::new(CType::Infix(args[0].clone())),
                                        "Prefix" => Arc::new(CType::Prefix(args[0].clone())),
                                        "Method" => Arc::new(CType::Method(args[0].clone())),
                                        "Property" => Arc::new(CType::Property(args[0].clone())),
                                        "Cast" => Arc::new(CType::Cast(args[0].clone())),
                                        "Own" => Arc::new(CType::Own(args[0].clone())),
                                        "Deref" => Arc::new(CType::Deref(args[0].clone())),
                                        "Mut" => Arc::new(CType::Mut(args[0].clone())),
                                        "Dependency" => Arc::new(CType::Dependency(
                                            args[0].clone(),
                                            args[1].clone(),
                                        )),
                                        "Rust" => Arc::new(CType::Rust(args[0].clone())),
                                        "Nodejs" => Arc::new(CType::Nodejs(args[0].clone())),
                                        "From" => Arc::new(CType::From(args[0].clone())),
                                        "Import" => CType::import(args[0].clone(), args[1].clone()),
                                        "Tuple" => CType::tuple(args.clone()),
                                        "Field" => CType::field(args.clone()),
                                        "Either" => CType::either(args.clone()),
                                        "Prop" => CType::prop(args[0].clone(), args[1].clone()),
                                        "AnyOf" => CType::anyof(args.clone()),
                                        "Buffer" => CType::buffer(args.clone()),
                                        "Array" => Arc::new(CType::Array(args[0].clone())),
                                        "Fail" => CType::cfail(args[0].clone()),
                                        "Min" => CType::min(args[0].clone(), args[1].clone()),
                                        "Max" => CType::max(args[0].clone(), args[1].clone()),
                                        "Neg" => CType::neg(args[0].clone()),
                                        "Len" => CType::len(args[0].clone()),
                                        "Size" => CType::size(args[0].clone()),
                                        "FileStr" => CType::filestr(args[0].clone()),
                                        "Concat" => CType::concat(args[0].clone(), args[1].clone()),
                                        "Env" => CType::env(args[0].clone()),
                                        "EnvExists" => CType::envexists(args[0].clone()),
                                        "Not" => CType::not(args[0].clone()),
                                        "Add" => CType::add(args[0].clone(), args[1].clone()),
                                        "Sub" => CType::sub(args[0].clone(), args[1].clone()),
                                        "Mul" => CType::mul(args[0].clone(), args[1].clone()),
                                        "Div" => CType::div(args[0].clone(), args[1].clone()),
                                        "Mod" => CType::cmod(args[0].clone(), args[1].clone()),
                                        "Pow" => CType::pow(args[0].clone(), args[1].clone()),
                                        "If" => {
                                            if args.len() == 2 {
                                                CType::tupleif(args[0].clone(), args[1].clone())
                                            } else if args.len() == 3 {
                                                CType::cif(
                                                    args[0].clone(),
                                                    args[1].clone(),
                                                    args[2].clone(),
                                                )
                                            } else {
                                                CType::fail(&format!("Invalid arguments provided to `If{{...}}`: {args:?}"))
                                            }
                                        }
                                        "And" => CType::and(args[0].clone(), args[1].clone()),
                                        "Or" => CType::or(args[0].clone(), args[1].clone()),
                                        "Xor" => CType::xor(args[0].clone(), args[1].clone()),
                                        "Nand" => CType::nand(args[0].clone(), args[1].clone()),
                                        "Nor" => CType::nor(args[0].clone(), args[1].clone()),
                                        "Xnor" => CType::xnor(args[0].clone(), args[1].clone()),
                                        "Eq" => CType::eq(args[0].clone(), args[1].clone()),
                                        "Neq" => CType::neq(args[0].clone(), args[1].clone()),
                                        "Lt" => CType::lt(args[0].clone(), args[1].clone()),
                                        "Lte" => CType::lte(args[0].clone(), args[1].clone()),
                                        "Gt" => CType::gt(args[0].clone(), args[1].clone()),
                                        "Gte" => CType::gte(args[0].clone(), args[1].clone()),
                                        unknown => CType::fail(&format!(
                                            "Unknown ctype {unknown} accessed. How did this happen?"
                                        )),
                                    }
                                }
                            }
                            others => {
                                // If we hit this branch, then the `args` vector needs to have a
                                // length of zero, and then we just bubble up the type as-is
                                if args.is_empty() {
                                    Arc::new(others.clone())
                                } else {
                                    CType::fail(&format!(
                                        "{var} is used as a generic type but is not one: {others:?}, {prior_value:?}",
                                    ))
                                }
                            }
                        }
                    }
                    None => CType::fail(&format!("{var} is not a valid type name")),
                })
            }
            parse::TypeBase::GnCall(_) => { /* We always process GnCall in the Variable path */ }
            parse::TypeBase::TypeGroup(g) => {
                if g.typeassignables.is_empty() {
                    // It's a void type!
                    prior_value = Some(Arc::new(CType::Group(Arc::new(CType::Void))));
                } else {
                    // Simply wrap the returned type in a `CType::Group`
                    prior_value = Some(Arc::new(CType::Group(withtypeoperatorslist_to_ctype(
                        &g.typeassignables,
                        scope,
                    )?)));
                }
            }
        };
        i += 1;
    }
    match prior_value {
        Some(p) => Ok(p),
        None => Err("Somehow did not resolve the type definition into anything".into()),
    }
}
