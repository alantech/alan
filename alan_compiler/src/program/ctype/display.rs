use std::sync::Arc;

use super::lookup_declared_type_name;
use super::{CType, Scope};
use super::{
    ADD, AND, BAND, BOR, CALLABLE_STRINGS, CLOSE_BRACE, CLOSE_BRACKET, CLOSE_PAREN, COMMA, DEPAT,
    DIV, DOT, EQ, FNARROW, FNCALL, FUNCTIONAL_STRINGS, GT, GTE, IMARROW, LOOSE_STRINGS, LT, LTE,
    MOD, MUL, NAND, NEQ, NOR, OPEN_BRACKET, OR, POW, STRICT_STRINGS, SUB, XNOR, XOR,
};

impl CType {
    pub fn to_string(self: Arc<CType>) -> String {
        self.to_strict_string(true)
    }
    /// Format a type for user-facing error messages: use a declared type name from `scope` when
    /// the type is structurally equivalent to one, otherwise fall back to [`to_functional_string`].
    pub fn to_error_string(self: Arc<CType>, scope: &Scope) -> String {
        lookup_declared_type_name(self.clone(), scope)
            .unwrap_or_else(|| self.to_functional_string())
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
                CType::Void | CType::DerivedVoid(..) => str_parts.push("()"),
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
                CType::Shared(t) => {
                    str_parts.push("Shared{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Promise(t) => {
                    str_parts.push("Promise{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
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
                CType::Tuple(ts, _) => {
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
                CType::Either(ts, _) => {
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
                CType::Exclude(t, p) => {
                    str_parts.push("Exclude{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(p);
                    ctype_stack.push(comma);
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
                CType::Void | CType::DerivedVoid(..) => str_parts.push("void"),
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
                CType::Shared(t) => {
                    str_parts.push("Shared{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
                }
                CType::Promise(t) => {
                    str_parts.push("Promise{");
                    ctype_stack.push(close_brace);
                    ctype_stack.push(t);
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
                CType::Tuple(ts, _) => {
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
                CType::Either(ts, _) => {
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
                CType::Exclude(t, p) => {
                    str_parts.push("Exclude{");
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
            CType::Type(n, t) => match &**t {
                CType::Int(_) | CType::Float(_) => {
                    format!("_{}", self.clone().to_functional_string())
                }
                CType::Binds(..) => n.clone(),
                CType::Exclude(et, ep) => {
                    let resolved = CType::exclude(et.clone(), ep.clone());
                    if matches!(&*resolved, CType::Exclude(..)) {
                        self.clone().to_functional_string()
                    } else {
                        resolved.to_callable_string()
                    }
                }
                CType::Type(_, inner) => {
                    if matches!(&**inner, CType::Exclude(..)) {
                        let (et, ep) = match &**inner {
                            CType::Exclude(et, ep) => (et.clone(), ep.clone()),
                            _ => unreachable!(),
                        };
                        let resolved = CType::exclude(et, ep);
                        if matches!(&*resolved, CType::Exclude(..)) {
                            self.clone().to_functional_string()
                        } else {
                            resolved.to_callable_string()
                        }
                    } else {
                        self.clone().to_functional_string()
                    }
                }
                _ => self.clone().to_functional_string(),
            },
            CType::Exclude(t, p) => {
                let resolved = CType::exclude(t.clone(), p.clone());
                if matches!(&*resolved, CType::Exclude(..)) {
                    self.clone().to_functional_string()
                } else {
                    resolved.to_callable_string()
                }
            }
            CType::Promise(_) => self.clone().to_functional_string(),
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
}
