use std::collections::{HashMap, HashSet};

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
    Type(String, Box<CType>),
    Generic(String, Vec<String>, Box<CType>),
    Binds(Box<CType>, Vec<CType>),
    IntrinsicGeneric(String, usize),
    Int(i128),
    Float(f64),
    Bool(bool),
    TString(String),
    Group(Box<CType>),
    Function(Box<CType>, Box<CType>),
    Call(Box<CType>, Box<CType>),
    Infix(Box<CType>),
    Prefix(Box<CType>),
    Method(Box<CType>),
    Property(Box<CType>),
    Cast(Box<CType>),
    Own(Box<CType>),
    Deref(Box<CType>),
    Mut(Box<CType>),
    Dependency(Box<CType>, Box<CType>),
    Rust(Box<CType>),
    Node(Box<CType>),
    From(Box<CType>),
    Import(Box<CType>, Box<CType>),
    Tuple(Vec<CType>),
    Field(String, Box<CType>),
    Either(Vec<CType>),
    Prop(Box<CType>, Box<CType>),
    AnyOf(Vec<CType>),
    Buffer(Box<CType>, Box<CType>),
    Array(Box<CType>),
    Fail(String),
    Add(Vec<CType>),
    Sub(Vec<CType>),
    Mul(Vec<CType>),
    Div(Vec<CType>),
    Mod(Vec<CType>),
    Pow(Vec<CType>),
    Min(Vec<CType>),
    Max(Vec<CType>),
    Neg(Box<CType>),
    Len(Box<CType>),
    Size(Box<CType>),
    FileStr(Box<CType>),
    Concat(Box<CType>, Box<CType>),
    Env(Vec<CType>),
    EnvExists(Box<CType>),
    TIf(Box<CType>, Vec<CType>),
    And(Vec<CType>),
    Or(Vec<CType>),
    Xor(Vec<CType>),
    Not(Box<CType>),
    Nand(Vec<CType>),
    Nor(Vec<CType>),
    Xnor(Vec<CType>),
    TEq(Vec<CType>),
    Neq(Vec<CType>),
    Lt(Vec<CType>),
    Lte(Vec<CType>),
    Gt(Vec<CType>),
    Gte(Vec<CType>),
}

impl CType {
    // TODO: Find a better way to handle these primitive types
    pub fn i64() -> CType {
        let program = Program::get_program();
        let output_lang = program.env.get("ALAN_OUTPUT_LANG").unwrap().clone();
        Program::return_program(program);
        match output_lang.as_str() {
            "rs" => CType::Type(
                "i64".to_string(),
                Box::new(CType::Binds(
                    Box::new(CType::TString("i64".to_string())),
                    Vec::new(),
                )),
            ),
            "js" => CType::Type(
                "i64".to_string(),
                Box::new(CType::Binds(
                    Box::new(CType::Import(
                        Box::new(CType::TString("alan_std.I64".to_string())),
                        Box::new(CType::Type(
                            "RootBacking".to_string(),
                            Box::new(CType::Node(Box::new(CType::Dependency(
                                Box::new(CType::TString("alan_std".to_string())),
                                Box::new(CType::TString(
                                    "https://github.com/alantech/alan.git".to_string(),
                                )),
                            )))),
                        )),
                    )),
                    Vec::new(),
                )),
            ),
            _ => unreachable!(),
        }
    }
    pub fn f64() -> CType {
        let program = Program::get_program();
        let output_lang = program.env.get("ALAN_OUTPUT_LANG").unwrap().clone();
        Program::return_program(program);
        match output_lang.as_str() {
            "rs" => CType::Type(
                "f64".to_string(),
                Box::new(CType::Binds(
                    Box::new(CType::TString("f64".to_string())),
                    Vec::new(),
                )),
            ),
            "js" => CType::Type(
                "f64".to_string(),
                Box::new(CType::Binds(
                    Box::new(CType::Import(
                        Box::new(CType::TString("alan_std.F64".to_string())),
                        Box::new(CType::Type(
                            "RootBacking".to_string(),
                            Box::new(CType::Node(Box::new(CType::Dependency(
                                Box::new(CType::TString("alan_std".to_string())),
                                Box::new(CType::TString(
                                    "https://github.com/alantech/alan.git".to_string(),
                                )),
                            )))),
                        )),
                    )),
                    Vec::new(),
                )),
            ),
            _ => unreachable!(),
        }
    }
    pub fn bool() -> CType {
        let program = Program::get_program();
        let output_lang = program.env.get("ALAN_OUTPUT_LANG").unwrap().clone();
        Program::return_program(program);
        match output_lang.as_str() {
            "rs" => CType::Type(
                "bool".to_string(),
                Box::new(CType::Binds(
                    Box::new(CType::TString("bool".to_string())),
                    Vec::new(),
                )),
            ),
            "js" => CType::Type(
                "bool".to_string(),
                Box::new(CType::Binds(
                    Box::new(CType::Import(
                        Box::new(CType::TString("alan_std.Bool".to_string())),
                        Box::new(CType::Type(
                            "RootBacking".to_string(),
                            Box::new(CType::Node(Box::new(CType::Dependency(
                                Box::new(CType::TString("alan_std".to_string())),
                                Box::new(CType::TString(
                                    "https://github.com/alantech/alan.git".to_string(),
                                )),
                            )))),
                        )),
                    )),
                    Vec::new(),
                )),
            ),
            _ => unreachable!(),
        }
    }
    pub fn string() -> CType {
        let program = Program::get_program();
        let output_lang = program.env.get("ALAN_OUTPUT_LANG").unwrap().clone();
        Program::return_program(program);
        match output_lang.as_str() {
            "rs" => CType::Type(
                "string".to_string(),
                Box::new(CType::Binds(
                    Box::new(CType::TString("String".to_string())),
                    Vec::new(),
                )),
            ),
            "js" => CType::Type(
                "string".to_string(),
                Box::new(CType::Binds(
                    Box::new(CType::Import(
                        Box::new(CType::TString("alan_std.Str".to_string())),
                        Box::new(CType::Type(
                            "RootBacking".to_string(),
                            Box::new(CType::Node(Box::new(CType::Dependency(
                                Box::new(CType::TString("alan_std".to_string())),
                                Box::new(CType::TString(
                                    "https://github.com/alantech/alan.git".to_string(),
                                )),
                            )))),
                        )),
                    )),
                    Vec::new(),
                )),
            ),
            _ => unreachable!(),
        }
    }
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.to_strict_string(true)
    }
    pub fn to_strict_string(&self, strict: bool) -> String {
        match self {
            CType::Void => "()".to_string(),
            CType::Infer(s, _) => s.clone(), // TODO: Replace this
            CType::Type(n, t) => match strict {
                true => n.to_string(),
                false => t.to_strict_string(strict),
            },
            CType::Generic(n, a, _) => format!("{}{{{}}}", n, a.join(", ")),
            CType::Binds(n, ts) => format!(
                "Binds{{{}{}{}}}",
                n.to_strict_string(strict),
                if ts.is_empty() { "" } else { ", " },
                ts.iter()
                    .map(|t| t.to_strict_string(strict))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::IntrinsicGeneric(s, l) => format!(
                "{}{{{}}}",
                s,
                (0..*l)
                    .map(|b| format!("arg{}", b))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Int(i) => format!("{}", i),
            CType::Float(f) => format!("{}", f),
            CType::Bool(b) => match b {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            CType::TString(s) => s.clone(),
            CType::Group(t) => match strict {
                true => format!("({})", t.to_strict_string(strict)),
                false => t.to_strict_string(strict),
            },
            CType::Function(i, o) => format!(
                "{} -> {}",
                i.to_strict_string(strict),
                o.to_strict_string(strict)
            ),
            CType::Call(n, f) => format!(
                "{} :: {}",
                n.to_strict_string(strict),
                f.to_strict_string(strict)
            ),
            CType::Infix(o) => format!("Infix{{{}}}", o.to_strict_string(strict)),
            CType::Prefix(o) => format!("Prefix{{{}}}", o.to_strict_string(strict)),
            CType::Method(f) => format!("Method{{{}}}", f.to_strict_string(strict)),
            CType::Property(p) => format!("Property{{{}}}", p.to_strict_string(strict)),
            CType::Cast(t) => format!("Cast{{{}}}", t.to_strict_string(strict)),
            CType::Own(t) => {
                if strict {
                    format!("Own{{{}}}", t.to_strict_string(strict))
                } else {
                    t.to_strict_string(strict)
                }
            }
            CType::Deref(t) => {
                if strict {
                    format!("Deref{{{}}}", t.to_strict_string(strict))
                } else {
                    t.to_strict_string(strict)
                }
            }
            CType::Mut(t) => {
                if strict {
                    format!("Mut{{{}}}", t.to_strict_string(strict))
                } else {
                    t.to_strict_string(strict)
                }
            }
            CType::Dependency(n, v) => {
                if strict {
                    format!(
                        "Dependency{{{}, {}}}",
                        n.to_strict_string(strict),
                        v.to_strict_string(strict)
                    )
                } else {
                    format!(
                        "{} @ {}",
                        n.to_strict_string(strict),
                        v.to_strict_string(strict)
                    )
                }
            }
            CType::Rust(d) => format!("Rust{{{}}}", d.to_strict_string(strict)),
            CType::Node(d) => format!("Node{{{}}}", d.to_strict_string(strict)),
            CType::From(d) => {
                if strict {
                    format!("From{{{}}}", d.to_strict_string(strict))
                } else {
                    format!("<- {}", d.to_strict_string(strict))
                }
            }
            CType::Import(n, d) => {
                if strict {
                    format!(
                        "Import{{{}, {}}}",
                        n.to_strict_string(strict),
                        d.to_strict_string(strict)
                    )
                } else {
                    format!(
                        "{} <- {}",
                        n.to_strict_string(strict),
                        d.to_strict_string(strict)
                    )
                }
            }
            CType::Tuple(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(", "),
            CType::Field(l, t) => match strict {
                true => format!("{}: {}", l, t.to_strict_string(strict)),
                false => t.to_strict_string(strict),
            },
            CType::Either(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" | "),
            CType::Prop(t, p) => format!(
                "{}.{}",
                t.to_strict_string(strict),
                p.to_strict_string(strict)
            ),
            CType::AnyOf(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" & "),
            CType::Buffer(t, s) => format!(
                "{}[{}]",
                t.to_strict_string(strict),
                s.to_strict_string(strict)
            ),
            CType::Array(t) => format!("{}[]", t.to_strict_string(strict)),
            CType::Fail(m) => format!("Fail{{{}}}", m),
            CType::Add(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" + "),
            CType::Sub(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" - "),
            CType::Mul(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" * "),
            CType::Div(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" / "),
            CType::Mod(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" % "),
            CType::Pow(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" ** "),
            CType::Min(ts) => format!(
                "Min{{{}}}",
                ts.iter()
                    .map(|t| t.to_strict_string(strict))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Max(ts) => format!(
                "Max{{{}}}",
                ts.iter()
                    .map(|t| t.to_strict_string(strict))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Neg(t) => format!("-{}", t.to_strict_string(strict)),
            CType::Len(t) => format!("Len{{{}}}", t.to_strict_string(strict)),
            CType::Size(t) => format!("Size{{{}}}", t.to_strict_string(strict)),
            CType::FileStr(t) => format!("FileStr{{{}}}", t.to_strict_string(strict)),
            CType::Concat(a, b) => format!(
                "Concat{{{}, {}}}",
                a.to_strict_string(strict),
                b.to_strict_string(strict)
            ),
            CType::Env(ts) => format!(
                "Env{{{}}}",
                ts.iter()
                    .map(|t| t.to_strict_string(strict))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::EnvExists(t) => format!("EnvExists{{{}}}", t.to_strict_string(strict)),
            CType::TIf(t, ts) => format!(
                "If{{{}, {}}}",
                t.to_strict_string(strict),
                ts.iter()
                    .map(|t| t.to_strict_string(strict))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::And(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" && "),
            CType::Or(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" || "),
            CType::Xor(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" ^ "),
            CType::Not(t) => format!("!{}", t.to_strict_string(strict)),
            CType::Nand(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" !& "),
            CType::Nor(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" !| "),
            CType::Xnor(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" !^ "),
            CType::TEq(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" == "),
            CType::Neq(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" != "),
            CType::Lt(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" < "),
            CType::Lte(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" <= "),
            CType::Gt(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" > "),
            CType::Gte(ts) => ts
                .iter()
                .map(|t| t.to_strict_string(strict))
                .collect::<Vec<String>>()
                .join(" >= "),
        }
    }
    pub fn to_functional_string(&self) -> String {
        match self {
            CType::Void => "void".to_string(),
            CType::Infer(s, _) => s.clone(), // TODO: What to do here?
            CType::Type(_, t) => t.to_functional_string(),
            CType::Generic(n, gs, _) => format!("{}{{{}}}", n, gs.join(", ")),
            CType::Binds(n, ts) => format!(
                "Binds{{{}{}{}}}",
                n.to_functional_string(),
                if ts.is_empty() { "" } else { ", " },
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::IntrinsicGeneric(s, u) => format!("{}{{{}}}", s, {
                let mut out = Vec::new();
                for i in 0..(*u as u32) {
                    let a = 'a' as u32;
                    let c = char::from_u32(a + i).unwrap();
                    out.push(c.to_string());
                }
                out.join(", ")
            }),
            CType::Int(i) => format!("{}", i),
            CType::Float(f) => format!("{}", f),
            CType::Bool(b) => match b {
                true => "true".to_string(),
                false => "false".to_string(),
            },
            CType::TString(s) => s.clone(),
            CType::Group(t) => t.to_functional_string(),
            CType::Function(i, o) => format!(
                "Function{{{}, {}}}",
                i.to_functional_string(),
                o.to_functional_string()
            ),
            CType::Call(n, f) => format!(
                "Call{{{}, {}}}",
                n.to_functional_string(),
                f.to_functional_string()
            ),
            CType::Infix(o) => format!("Infix{{{}}}", o.to_functional_string()),
            CType::Prefix(o) => format!("Prefix{{{}}}", o.to_functional_string()),
            CType::Method(f) => format!("Method{{{}}}", f.to_functional_string()),
            CType::Property(p) => format!("Property{{{}}}", p.to_functional_string()),
            CType::Cast(t) => format!("Cast{{{}}}", t.to_functional_string()),
            CType::Own(t) => format!("Own{{{}}}", t.to_functional_string()),
            CType::Deref(t) => format!("Deref{{{}}}", t.to_functional_string()),
            CType::Mut(t) => format!("Mut{{{}}}", t.to_functional_string()),
            CType::Dependency(n, v) => format!(
                "Dependency{{{}, {}}}",
                n.to_functional_string(),
                v.to_functional_string()
            ),
            CType::Rust(d) => format!("Rust{{{}}}", d.to_functional_string()),
            CType::Node(d) => format!("Node{{{}}}", d.to_functional_string()),
            CType::From(d) => format!("From{{{}}}", d.to_functional_string()),
            CType::Import(n, d) => format!(
                "Import{{{}, {}}}",
                n.to_functional_string(),
                d.to_functional_string(),
            ),
            CType::Tuple(ts) => format!(
                "Tuple{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Field(l, t) => format!("Field{{{}, {}}}", l, t.to_functional_string()),
            CType::Either(ts) => format!(
                "Either{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Prop(t, p) => format!(
                "Prop{{{}, {}}}",
                t.to_functional_string(),
                p.to_functional_string()
            ),
            CType::AnyOf(ts) => format!(
                "AnyOf{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Buffer(t, s) => format!(
                "Buffer{{{}, {}}}",
                t.to_functional_string(),
                s.to_functional_string()
            ),
            CType::Array(t) => format!("Array{{{}}}", t.to_functional_string()),
            CType::Fail(m) => format!("Fail{{{}}}", m),
            CType::Add(ts) => format!(
                "Add{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Sub(ts) => format!(
                "Sub{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Mul(ts) => format!(
                "Mul{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Div(ts) => format!(
                "Div{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Mod(ts) => format!(
                "Mod{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Pow(ts) => format!(
                "Pow{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Min(ts) => format!(
                "Min{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Max(ts) => format!(
                "Max{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Neg(t) => format!("Neg{{{}}}", t.to_functional_string()),
            CType::Len(t) => format!("Len{{{}}}", t.to_functional_string()),
            CType::Size(t) => format!("Size{{{}}}", t.to_functional_string()),
            CType::FileStr(t) => format!("FileStr{{{}}}", t.to_functional_string()),
            CType::Concat(a, b) => format!(
                "Concat{{{}, {}}}",
                a.to_functional_string(),
                b.to_functional_string()
            ),
            CType::Env(ts) => format!(
                "Env{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::EnvExists(t) => format!("EnvExists{{{}}}", t.to_functional_string()),
            CType::TIf(t, ts) => format!(
                "If{{{}, {}}}",
                t.to_functional_string(),
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::And(ts) => format!(
                "And{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Or(ts) => format!(
                "Or{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Xor(ts) => format!(
                "Xor{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Not(t) => format!("Not{{{}}}", t.to_functional_string()),
            CType::Nand(ts) => format!(
                "Nand{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Nor(ts) => format!(
                "Nor{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Xnor(ts) => format!(
                "Xnor{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::TEq(ts) => format!(
                "Eq{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Neq(ts) => format!(
                "Neq{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Lt(ts) => format!(
                "Lt{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Lte(ts) => format!(
                "Lte{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Gt(ts) => format!(
                "Gt{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            CType::Gte(ts) => format!(
                "Gte{{{}}}",
                ts.iter()
                    .map(|t| t.to_functional_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
    pub fn to_callable_string(&self) -> String {
        // TODO: Be more efficient with this later
        match self {
            CType::Int(_) | CType::Float(_) => format!("_{}", self.to_functional_string()),
            CType::Type(n, t) => match **t {
                CType::Int(_) | CType::Float(_) => format!("_{}", self.to_functional_string()),
                CType::Binds(..) => n.clone(),
                _ => self.to_functional_string(),
            },
            _ => self.to_functional_string(),
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
        .collect::<String>()
    }
    pub fn degroup(&self) -> CType {
        match self {
            CType::Void => CType::Void,
            CType::Infer(s, i) => CType::Infer(s.clone(), i.clone()),
            CType::Type(n, t) => CType::Type(n.clone(), Box::new((*t).degroup())),
            CType::Generic(n, gs, wtos) => CType::Generic(n.clone(), gs.clone(), wtos.clone()),
            CType::Binds(n, ts) => CType::Binds(
                Box::new(n.degroup()),
                ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>(),
            ),
            CType::IntrinsicGeneric(n, s) => CType::IntrinsicGeneric(n.clone(), *s),
            CType::Int(i) => CType::Int(*i),
            CType::Float(f) => CType::Float(*f),
            CType::Bool(b) => CType::Bool(*b),
            CType::TString(s) => CType::TString(s.clone()),
            CType::Group(t) => t.degroup(),
            CType::Function(i, o) => {
                CType::Function(Box::new((*i).degroup()), Box::new((*o).degroup()))
            }
            CType::Call(n, f) => CType::Call(Box::new((*n).degroup()), Box::new((*f).degroup())),
            CType::Infix(o) => CType::Infix(Box::new((*o).degroup())),
            CType::Prefix(o) => CType::Prefix(Box::new((*o).degroup())),
            CType::Method(f) => CType::Method(Box::new((*f).degroup())),
            CType::Property(p) => CType::Property(Box::new((*p).degroup())),
            CType::Cast(t) => CType::Cast(Box::new((*t).degroup())),
            CType::Own(t) => CType::Own(Box::new((*t).degroup())),
            CType::Deref(t) => CType::Deref(Box::new((*t).degroup())),
            CType::Mut(t) => CType::Mut(Box::new((*t).degroup())),
            CType::Dependency(n, v) => {
                CType::Dependency(Box::new((*n).degroup()), Box::new((*v).degroup()))
            }
            CType::Rust(d) => CType::Rust(Box::new((*d).degroup())),
            CType::Node(d) => CType::Node(Box::new((*d).degroup())),
            CType::From(d) => CType::From(Box::new((*d).degroup())),
            CType::Import(n, d) => {
                CType::Import(Box::new((*n).degroup()), Box::new((*d).degroup()))
            }
            CType::Tuple(ts) => {
                CType::Tuple(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>())
            }
            CType::Field(l, t) => CType::Field(l.clone(), Box::new((*t).degroup())),
            CType::Either(ts) => {
                CType::Either(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>())
            }
            CType::Prop(t, p) => CType::Prop(Box::new(t.degroup()), Box::new(p.degroup())),
            CType::AnyOf(ts) => {
                CType::AnyOf(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>())
            }
            CType::Buffer(t, s) => {
                CType::Buffer(Box::new((*t).degroup()), Box::new((*s).degroup()))
            }
            CType::Array(t) => CType::Array(Box::new((*t).degroup())),
            CType::Fail(m) => CType::Fail(m.clone()),
            CType::Add(ts) => CType::Add(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Sub(ts) => CType::Sub(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Mul(ts) => CType::Mul(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Div(ts) => CType::Div(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Mod(ts) => CType::Mod(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Pow(ts) => CType::Pow(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Min(ts) => CType::Min(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Max(ts) => CType::Max(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Neg(t) => CType::Neg(Box::new((*t).degroup())),
            CType::Len(t) => CType::Len(Box::new((*t).degroup())),
            CType::Size(t) => CType::Size(Box::new((*t).degroup())),
            CType::FileStr(t) => CType::FileStr(Box::new((*t).degroup())),
            CType::Concat(a, b) => {
                CType::Concat(Box::new((*a).degroup()), Box::new((*b).degroup()))
            }
            CType::Env(ts) => CType::Env(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::EnvExists(t) => CType::EnvExists(Box::new((*t).degroup())),
            CType::TIf(t, ts) => CType::TIf(
                Box::new((*t).degroup()),
                ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>(),
            ),
            CType::And(ts) => CType::And(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Or(ts) => CType::Or(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Xor(ts) => CType::Xor(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Not(t) => CType::Not(Box::new((*t).degroup())),
            CType::Nand(ts) => CType::Nand(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Nor(ts) => CType::Nor(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Xnor(ts) => CType::Xnor(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::TEq(ts) => CType::TEq(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Neq(ts) => CType::Neq(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Lt(ts) => CType::Lt(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Lte(ts) => CType::Lte(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Gt(ts) => CType::Gt(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
            CType::Gte(ts) => CType::Gte(ts.iter().map(|t| t.degroup()).collect::<Vec<CType>>()),
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
        generic_types: &mut HashMap<String, CType>,
        arg_type_vec: Vec<(&CType, &CType)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (a, i) in arg_type_vec {
            let mut arg = vec![a];
            let mut input = vec![i];
            while !arg.is_empty() {
                let a = arg.pop();
                let i = input.pop();
                match (a, i) {
                    (Some(CType::Void), Some(CType::Void)) => { /* Do nothing */ }
                    (Some(CType::Infer(s, _)), _) => {
                        return Err(format!(
                            "While attempting to infer generics found an inference type {} as an input somehow",
                            s
                        )
                        .into());
                    }
                    (Some(CType::Type(_, t1)), Some(CType::Type(_, t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Type(_, t1)), Some(b)) if !matches!(&**t1, CType::Binds(..)) => {
                        arg.push(t1);
                        input.push(b);
                    }
                    (Some(a), Some(CType::Type(_, t2))) if !matches!(&**t2, CType::Binds(..)) => {
                        arg.push(a);
                        input.push(t2);
                    }
                    (Some(CType::Generic(..)), _) => {
                        return Err(format!(
                            "Ran into an unresolved generic in the arguments list: {:?}",
                            arg
                        )
                        .into());
                    }
                    (Some(CType::Binds(n1, ts1)), Some(CType::Binds(n2, ts2))) => {
                        if ts1.len() != ts2.len() {
                            // TODO: Better generic arg matching
                            return Err(format!("Mismatched resolved bound generic types {}{{{}}} and {}{{{}}} during inference", n1.to_strict_string(false), ts1.iter().map(|t| t.to_strict_string(false)).collect::<Vec<String>>().join(", "), n2.to_strict_string(false), ts2.iter().map(|t| t.to_strict_string(false)).collect::<Vec<String>>().join(", ")).into());
                        }
                        arg.push(n1);
                        input.push(n2);
                        // Enqueue the bound types for checking purposes
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (
                        Some(CType::IntrinsicGeneric(n1, s1)),
                        Some(CType::IntrinsicGeneric(n2, s2)),
                    ) => {
                        if !(n1 == n2 && s1 == s2) {
                            return Err(format!(
                                "Mismatched generics {} and {} during inference",
                                n1, n2
                            )
                            .into());
                        }
                    }
                    (Some(CType::Int(i1)), Some(CType::Int(i2))) => {
                        if i1 != i2 {
                            return Err(format!(
                                "Mismatched integers {} and {} during inference",
                                i1, i2
                            )
                            .into());
                        }
                    }
                    (Some(CType::Float(f1)), Some(CType::Float(f2))) => {
                        if f1 != f2 {
                            return Err(format!(
                                "Mismatched floats {} and {} during inference",
                                f1, f2
                            )
                            .into());
                        }
                    }
                    (Some(CType::Bool(b1)), Some(CType::Bool(b2))) => {
                        if b1 != b2 {
                            return Err("Mismatched booleans during inference".to_string().into());
                        }
                    }
                    (Some(CType::TString(s1)), Some(CType::TString(s2))) => {
                        if s1 != s2 {
                            return Err(format!(
                                "Mismatched strings {} and {} during inference",
                                s1, s2
                            )
                            .into());
                        }
                    }
                    (Some(CType::Group(g1)), Some(CType::Group(g2))) => {
                        arg.push(g1);
                        input.push(g2);
                    }
                    (Some(CType::Group(g1)), Some(b)) => {
                        arg.push(g1);
                        input.push(b);
                    }
                    (Some(a), Some(CType::Group(g2))) => {
                        arg.push(a);
                        input.push(g2);
                    }
                    (Some(CType::Function(i1, o1)), Some(CType::Function(i2, o2))) => {
                        match &**i1 {
                            CType::Tuple(ts1) if ts1.len() == 1 => {
                                arg.push(&ts1[0]);
                            }
                            otherwise => arg.push(otherwise),
                        }
                        arg.push(o1);
                        match &**i2 {
                            CType::Tuple(ts2) if ts2.len() == 1 => {
                                input.push(&ts2[0]);
                            }
                            otherwise => input.push(otherwise),
                        }
                        input.push(o2);
                    }
                    (Some(CType::Call(n1, f1)), Some(CType::Call(n2, f2))) => {
                        arg.push(n1);
                        arg.push(f1);
                        input.push(n2);
                        input.push(f2);
                    }
                    (Some(CType::Infix(o1)), Some(CType::Infix(o2))) => {
                        arg.push(o1);
                        input.push(o2);
                    }
                    (Some(CType::Prefix(o1)), Some(CType::Prefix(o2))) => {
                        arg.push(o1);
                        input.push(o2);
                    }
                    (Some(CType::Method(f1)), Some(CType::Method(f2))) => {
                        arg.push(f1);
                        input.push(f2);
                    }
                    (Some(CType::Property(p1)), Some(CType::Property(p2))) => {
                        arg.push(p1);
                        input.push(p2);
                    }
                    (Some(CType::Cast(t1)), Some(CType::Cast(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Own(t1)), Some(CType::Own(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Deref(t1)), Some(CType::Deref(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Mut(t1)), Some(CType::Mut(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Dependency(n1, v1)), Some(CType::Dependency(n2, v2))) => {
                        arg.push(n1);
                        arg.push(v1);
                        input.push(n2);
                        input.push(v2);
                    }
                    (Some(CType::Rust(d1)), Some(CType::Rust(d2))) => {
                        arg.push(d1);
                        input.push(d2);
                    }
                    (Some(CType::Node(d1)), Some(CType::Node(d2))) => {
                        arg.push(d1);
                        input.push(d2);
                    }
                    (Some(CType::From(d1)), Some(CType::From(d2))) => {
                        arg.push(d1);
                        input.push(d2);
                    }
                    (Some(CType::Import(n1, d1)), Some(CType::Import(n2, d2))) => {
                        arg.push(n1);
                        arg.push(d1);
                        input.push(n2);
                        input.push(d2);
                    }
                    (Some(CType::Tuple(ts1)), Some(CType::Tuple(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched tuple types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        // TODO: Allow out-of-order listing based on Field labels
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Field(l1, t1)), Some(CType::Field(l2, t2))) => {
                        // TODO: Allow out-of-order listing based on Field labels
                        if l1 != l2 {
                            return Err(format!(
                                "Mismatched fields {} and {} during inference",
                                l1, l2
                            )
                            .into());
                        }
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(a), Some(CType::Field(_, t2))) => {
                        arg.push(a);
                        input.push(t2);
                    }
                    (Some(CType::Field(_, t1)), Some(b)) => {
                        arg.push(t1);
                        input.push(b);
                    }
                    (Some(CType::Either(ts1)), Some(CType::Either(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched either types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Buffer(t1, s1)), Some(CType::Buffer(t2, s2))) => {
                        arg.push(t1);
                        arg.push(s1);
                        input.push(t2);
                        input.push(s2);
                    }
                    (Some(CType::Array(t1)), Some(CType::Array(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::AnyOf(ts)), Some(CType::Infer(g, _))) => {
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
                                            if t1.degroup().to_callable_string()
                                                == t2.degroup().to_callable_string()
                                            {
                                                matches.push(t1.clone());
                                            }
                                        }
                                    }
                                }
                                otherwise => {
                                    for t1 in ts {
                                        if t1.degroup().to_callable_string()
                                            == otherwise.degroup().to_callable_string()
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
                                    .insert(g.clone(), matches.into_iter().nth(0).unwrap());
                            } else {
                                generic_types.insert(g.clone(), CType::AnyOf(matches));
                            }
                        } else {
                            generic_types.insert(g.clone(), CType::AnyOf(ts.clone()));
                        }
                    }
                    (Some(CType::Fail(m1)), Some(CType::Fail(m2))) => {
                        if m1 != m2 {
                            return Err(
                                "The two types want to fail in different ways. How bizarre!".into(),
                            );
                        }
                    }
                    (Some(CType::Add(ts1)), Some(CType::Add(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched add types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (
                        Some(CType::Int(_) | CType::Float(_)),
                        Some(
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
                        ),
                    ) => {
                        // TODO: This should allow us to constrain which generic values are
                        // possible for each generic to infer on the right-hand-side, but for now
                        // we're just going to ignore this path and require the components are
                        // inferred separately in the type system
                    }
                    (
                        Some(CType::Int(_) | CType::Bool(_)),
                        Some(
                            CType::And(_)
                            | CType::Or(_)
                            | CType::Xor(_)
                            | CType::Not(_)
                            | CType::Nand(_)
                            | CType::Nor(_)
                            | CType::Xnor(_),
                        ),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (
                        Some(CType::Int(_) | CType::Float(_) | CType::TString(_) | CType::Bool(_)),
                        Some(CType::TEq(_) | CType::Neq(_)),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (
                        Some(CType::Int(_) | CType::Float(_) | CType::TString(_)),
                        Some(CType::Lt(_) | CType::Lte(_) | CType::Gt(_) | CType::Gte(_)),
                    ) => {
                        // TODO: Also skipping this for now
                    }
                    (Some(CType::Sub(ts1)), Some(CType::Sub(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched sub types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Mul(ts1)), Some(CType::Mul(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched mul types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Div(ts1)), Some(CType::Div(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched div types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Mod(ts1)), Some(CType::Mod(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched div types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Pow(ts1)), Some(CType::Pow(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched pow types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Min(ts1)), Some(CType::Min(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched min types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Max(ts1)), Some(CType::Max(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched max types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Neg(t1)), Some(CType::Neg(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Len(t1)), Some(CType::Len(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Size(t1)), Some(CType::Size(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::FileStr(t1)), Some(CType::FileStr(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Env(ts1)), Some(CType::Env(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched env types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::EnvExists(t1)), Some(CType::EnvExists(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::TIf(t1, ts1)), Some(CType::TIf(t2, ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched env types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        arg.push(t1);
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        input.push(t2);
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::And(ts1)), Some(CType::And(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched and types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Or(ts1)), Some(CType::Or(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched or types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Xor(ts1)), Some(CType::Xor(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched xor types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Not(t1)), Some(CType::Not(t2))) => {
                        arg.push(t1);
                        input.push(t2);
                    }
                    (Some(CType::Nand(ts1)), Some(CType::Nand(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched nand types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Nor(ts1)), Some(CType::Nor(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched nor types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Xnor(ts1)), Some(CType::Xnor(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched xnor types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::TEq(ts1)), Some(CType::TEq(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched eq types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Neq(ts1)), Some(CType::Neq(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched neq types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Lt(ts1)), Some(CType::Lt(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched lt types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Lte(ts1)), Some(CType::Lte(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched lte types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Gt(ts1)), Some(CType::Gt(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched gt types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(CType::Gte(ts1)), Some(CType::Gte(ts2))) => {
                        if ts1.len() != ts2.len() {
                            return Err(format!(
                                "Mismatched gte types {} and {} found during inference",
                                a.unwrap().to_string(),
                                i.unwrap().to_string()
                            )
                            .into());
                        }
                        for t1 in ts1 {
                            arg.push(t1);
                        }
                        for t2 in ts2 {
                            input.push(t2);
                        }
                    }
                    (Some(a), Some(CType::Infer(g, _))) => {
                        // Found the normal path to infer. If there's already a match, check if the
                        // existing match is an AnyOf and intersect the set, otherwise a simple
                        // comparison
                        if generic_types.contains_key(g) {
                            // Possible found the same thing, already, let's confirm that we aren't
                            // in an impossible scenario.
                            let other_type: &CType = generic_types.get(g).unwrap();
                            let mut matched = false;
                            match other_type {
                                CType::AnyOf(ts) => {
                                    for t1 in ts {
                                        if t1.degroup().to_callable_string()
                                            == a.degroup().to_callable_string()
                                        {
                                            matched = true;
                                        }
                                    }
                                }
                                otherwise => {
                                    if otherwise.degroup().to_callable_string()
                                        == a.degroup().to_callable_string()
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
                                    other_type.to_strict_string(false),
                                    a.to_strict_string(false)
                                )
                                .into());
                            }
                        } else {
                            generic_types.insert(g.clone(), a.clone());
                        }
                    }
                    (Some(CType::AnyOf(ts)), Some(b)) => {
                        let mut success = false;
                        for t in ts {
                            // We need to check each of these and accept the one that passes, or
                            // fail if none of them pass. It's expected that most of them will
                            // fail, so we can't just push them onto the queue, as those mismatches
                            // will fail out of the function. Instead we clone the hashmap and add
                            // each of these as a singular element to push through, merging the
                            // hashmap on success and exiting the loop.
                            let mut generic_types_inner = generic_types.clone();
                            if CType::infer_generics_inner_loop(
                                &mut generic_types_inner,
                                vec![(t, b)],
                            )
                            .is_ok()
                            {
                                // If there's a conflict between the inferred types, we skip
                                let mut matches = true;
                                for (k, v) in &generic_types_inner {
                                    match generic_types.get(k) {
                                        Some(old_v) => {
                                            if old_v != v {
                                                matches = false;
                                            }
                                        }
                                        None => { /* Do nothing */ }
                                    }
                                }
                                if !matches {
                                    continue;
                                }
                                success = true;
                                for (k, v) in &generic_types_inner {
                                    generic_types.insert(k.clone(), v.clone());
                                }
                            }
                        }
                        if !success {
                            return Err(format!(
                                "None of {} matches {}",
                                ts.iter()
                                    .map(|t| t.to_strict_string(false))
                                    .collect::<Vec<String>>()
                                    .join(" & "),
                                b.to_strict_string(false)
                            )
                            .into());
                        }
                    }
                    _ => {
                        return Err(format!("Mismatch between {:?} and {:?}", a, i).into());
                    }
                }
            }
        }
        Ok(())
    }
    pub fn infer_generics(
        scope: &Scope,
        generics: &[(String, CType)],
        fn_args: &[(String, ArgKind, CType)],
        call_args: &[CType],
    ) -> Result<Vec<CType>, Box<dyn std::error::Error>> {
        let mut temp_scope = scope.child();
        for (generic_name, generic_type) in generics {
            temp_scope
                .types
                .insert(generic_name.clone(), generic_type.clone());
        }
        let input_types = fn_args
            .iter()
            .map(|(_, _, t)| t.clone())
            .collect::<Vec<CType>>();
        let mut generic_types: HashMap<String, CType> = HashMap::new();
        CType::infer_generics_inner_loop(
            &mut generic_types,
            call_args
                .iter()
                .zip(input_types.iter())
                .collect::<Vec<(&CType, &CType)>>(),
        )?;
        let mut output_types = Vec::new();
        for (generic_name, _) in generics {
            output_types.push(match generic_types.get(generic_name) {
                Some(t) => Ok(t.clone()),
                None => Err(format!("No inferred type found for {}", generic_name)),
            }?);
        }
        Ok(output_types)
    }
    pub fn accepts(&self, arg: &CType) -> bool {
        match (self, arg) {
            (a, CType::AnyOf(ts)) => {
                for t in ts {
                    if a.accepts(t) {
                        return true;
                    }
                }
                false
            }
            // TODO: Do this without stringification
            (a, b) => a.degroup().to_strict_string(false) == b.degroup().to_strict_string(false),
        }
    }

    pub fn to_functions(&self, name: String, scope: &Scope) -> (CType, Vec<Function>) {
        let t = CType::Type(name.clone(), Box::new(self.clone()));
        let constructor_fn_name = t.to_callable_string();
        let mut fs = Vec::new();
        match self {
            CType::Import(n, d) => match &**d {
                CType::TString(dep_name) => {
                    let program = Program::get_program();
                    let other_scope = program.scope_by_file(dep_name).unwrap();
                    match &**n {
                        CType::TString(name) => match other_scope.functions.get(name) {
                            None => CType::fail(&format!("{} not found in {}", name, dep_name)),
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
                let mut typen = f.degroup();
                let args = type_to_args(&typen);
                let rettype = type_to_rettype(&typen);
                // Short-circuit for "normal" function binding with "normal" arguments only
                if args.iter().all(|(_, k, t)| {
                    matches!(k, ArgKind::Ref)
                        && !matches!(
                            t,
                            CType::Int(_) | CType::Float(_) | CType::Bool(_) | CType::TString(_)
                        )
                }) && matches!(&**n, CType::TString(_))
                {
                    fs.push(Function {
                        name: constructor_fn_name.clone(),
                        typen,
                        microstatements: Vec::new(),
                        kind: FnKind::Bind(match &**n {
                            CType::TString(s) => s.clone(),
                            _ => unreachable!(),
                        }),
                        origin_scope_path: scope.path.clone(),
                    });
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
                                        representation: format!("*{}", name),
                                    }),
                                })
                            }
                            ArgKind::Own | ArgKind::Mut | ArgKind::Ref => {}
                        }
                    }
                    let call_name = match &**n {
                        CType::Import(n, d) => {
                            kind = FnKind::External((&**d).clone());
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
                                                match &typen {
                                                    CType::Int(i) => {
                                                        trimmed_args = true;
                                                        format!("{}", i)
                                                    }
                                                    CType::Float(f) => {
                                                        trimmed_args = true;
                                                        format!("{}", f)
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
                                            match &args[0].2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{}", i)
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{}", f)
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
                                            match &args[1].2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{}", i)
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{}", f)
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
                                "Unsupported native operator declaration {:?}",
                                otherwise
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
                                            match &args[0].2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{}", i)
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{}", f)
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
                                "Unsupported native operator declaration {:?}",
                                otherwise
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
                                            match &arg_car.2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{}", i)
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{}", f)
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
                                                .map(|a| match &a.2 {
                                                    CType::Int(i) => {
                                                        trimmed_args = true;
                                                        format!("{}", i)
                                                    }
                                                    CType::Float(f) => {
                                                        trimmed_args = true;
                                                        format!("{}", f)
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
                                "Unsupported native method declaration {:?}",
                                otherwise
                            )),
                        },
                        CType::Property(p) => match &**p {
                            CType::TString(s) => {
                                if args.len() > 1 {
                                    CType::fail(&format!("Property bindings may only have one argument, the value the property is accessed from. Not {:?}", args))
                                } else {
                                    let arg_car = args[0].clone();
                                    microstatements.push(Microstatement::Return {
                                        value: Some(Box::new(Microstatement::Value {
                                            typen: rettype.clone(),
                                            representation: format!(
                                                "{}.{}",
                                                match &arg_car.2 {
                                                    CType::Int(i) => {
                                                        trimmed_args = true;
                                                        format!("{}", i)
                                                    }
                                                    CType::Float(f) => {
                                                        trimmed_args = true;
                                                        format!("{}", f)
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
                                "Unsupported native method declaration {:?}",
                                otherwise
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
                                            match &args[0].2 {
                                                CType::Int(i) => {
                                                    trimmed_args = true;
                                                    format!("{}", i)
                                                }
                                                CType::Float(f) => {
                                                    trimmed_args = true;
                                                    format!("{}", f)
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
                                "Unsupported native cast declaration {:?}",
                                otherwise
                            )),
                        },
                        otherwise => CType::fail(&format!(
                            "Unsupported native operator declaration {:?}",
                            otherwise
                        )),
                    }
                    if trimmed_args {
                        typen = CType::Function(
                            Box::new(CType::Tuple(
                                args.into_iter()
                                    .filter(|(_, _, typen)| {
                                        !matches!(
                                            &typen,
                                            CType::Int(_)
                                                | CType::Float(_)
                                                | CType::Bool(_)
                                                | CType::TString(_)
                                        )
                                    })
                                    .map(|(n, k, t)| {
                                        CType::Field(
                                            n,
                                            Box::new(match k {
                                                ArgKind::Own => CType::Own(Box::new(t)),
                                                ArgKind::Deref => CType::Deref(Box::new(t)),
                                                ArgKind::Mut => CType::Mut(Box::new(t)),
                                                ArgKind::Ref => t,
                                            }),
                                        )
                                    })
                                    .collect::<Vec<CType>>(),
                            )),
                            Box::new(rettype),
                        );
                    }
                    fs.push(Function {
                        name: constructor_fn_name.clone(),
                        typen,
                        microstatements,
                        kind,
                        origin_scope_path: scope.path.clone(),
                    });
                }
            }
            CType::Type(n, _) => {
                // This is just an alias
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(
                        Box::new(CType::Field(n.clone(), Box::new(self.clone()))),
                        Box::new(t.clone()),
                    ),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                });
            }
            CType::Tuple(ts) => {
                // The constructor function needs to grab the types from all
                // arguments to construct the desired product type. For any type
                // that is marked as a field, we also want to create an accessor
                // function for it to simulate structs better.
                // Create accessor functions for static tag values in the tuple, if any exist
                let mut actual_ts = Vec::new();
                for ti in ts.iter().filter(|t1| match t1 {
                    CType::Field(_, t2) => matches!(
                        &**t2,
                        CType::TString(_) | CType::Int(_) | CType::Float(_) | CType::Bool(_)
                    ),
                    CType::TString(_) | CType::Int(_) | CType::Float(_) | CType::Bool(_) => true,
                    _ => false,
                }) {
                    match ti {
                        CType::Field(n, f) => {
                            match &**f {
                                CType::TString(s) => {
                                    // Create an accessor function for this value, but do not add
                                    // it to the args array to construct it. The accessor function
                                    // will return this value as a string.
                                    let string = CType::string();
                                    fs.push(Function {
                                        name: n.clone(),
                                        typen: CType::Function(
                                            Box::new(t.clone()),
                                            Box::new(string.clone()),
                                        ),
                                        microstatements: vec![Microstatement::Value {
                                            typen: string,
                                            representation: format!(
                                                "\"{}\"",
                                                s.replace("\"", "\\\"")
                                            ),
                                        }],
                                        kind: FnKind::Static,
                                        origin_scope_path: scope.path.clone(),
                                    });
                                }
                                CType::Int(i) => {
                                    // Create an accessor function for this value, but do not add
                                    // it to the args array to construct it. The accessor function
                                    // will return this value as an i64.
                                    let int64 = CType::i64();
                                    fs.push(Function {
                                        name: n.clone(),
                                        typen: CType::Function(
                                            Box::new(t.clone()),
                                            Box::new(int64.clone()),
                                        ),
                                        microstatements: vec![Microstatement::Value {
                                            typen: int64,
                                            representation: format!("{}", i),
                                        }],
                                        kind: FnKind::Static,
                                        origin_scope_path: scope.path.clone(),
                                    });
                                }
                                CType::Float(f) => {
                                    // Create an accessor function for this value, but do not add
                                    // it to the args array to construct it. The accessor function
                                    // will return this value as an f64.
                                    let float64 = CType::f64();
                                    fs.push(Function {
                                        name: n.clone(),
                                        typen: CType::Function(
                                            Box::new(t.clone()),
                                            Box::new(float64.clone()),
                                        ),
                                        microstatements: vec![Microstatement::Value {
                                            typen: float64,
                                            representation: format!("{}", f),
                                        }],
                                        kind: FnKind::Static,
                                        origin_scope_path: scope.path.clone(),
                                    });
                                }
                                CType::Bool(b) => {
                                    // Create an accessor function for this value, but do not add
                                    // it to the args array to construct it. The accessor function
                                    // will return this value as a bool.
                                    let booln = CType::bool();
                                    fs.push(Function {
                                        name: n.clone(),
                                        typen: CType::Function(
                                            Box::new(t.clone()),
                                            Box::new(booln.clone()),
                                        ),
                                        microstatements: vec![Microstatement::Value {
                                            typen: booln,
                                            representation: match b {
                                                true => "true".to_string(),
                                                false => "false".to_string(),
                                            },
                                        }],
                                        kind: FnKind::Static,
                                        origin_scope_path: scope.path.clone(),
                                    });
                                }
                                _ => { /* Do nothing */ }
                            }
                        }
                        _ => { /* Do nothing */ }
                    }
                }
                for (i, ti) in ts
                    .iter()
                    .filter(|t1| match t1 {
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
                    match ti {
                        CType::Field(n, f) => {
                            // Create an accessor function
                            fs.push(Function {
                                name: n.clone(),
                                typen: CType::Function(Box::new(t.clone()), Box::new(*f.clone())),
                                microstatements: Vec::new(),
                                kind: FnKind::Derived,
                                origin_scope_path: scope.path.clone(),
                            });
                        }
                        otherwise => {
                            // Create an `<N>` function accepting the tuple by field number
                            fs.push(Function {
                                name: format!("{}", i),
                                typen: CType::Function(
                                    Box::new(t.clone()),
                                    Box::new(otherwise.clone()),
                                ),
                                microstatements: Vec::new(),
                                kind: FnKind::Derived,
                                origin_scope_path: scope.path.clone(),
                            });
                        }
                    }
                }
                // Define the constructor function
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(
                        Box::new(CType::Tuple(actual_ts.clone())),
                        Box::new(t.clone()),
                    ),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                });
            }
            CType::Field(n, f) => {
                // This is a "baby tuple" of just one value. So we follow the Tuple logic, but
                // simplified.
                match &**f {
                    CType::TString(s) => {
                        // Create an accessor function for this value, but do not add
                        // it to the args array to construct it. The accessor function
                        // will return this value as a string.
                        let string = CType::string();
                        fs.push(Function {
                            name: n.clone(),
                            typen: CType::Function(Box::new(t.clone()), Box::new(string.clone())),
                            microstatements: vec![Microstatement::Value {
                                typen: string,
                                representation: s.clone(),
                            }],
                            kind: FnKind::Static,
                            origin_scope_path: scope.path.clone(),
                        });
                    }
                    CType::Int(i) => {
                        // Create an accessor function for this value, but do not add
                        // it to the args array to construct it. The accessor function
                        // will return this value as an i64.
                        let int64 = CType::i64();
                        fs.push(Function {
                            name: n.clone(),
                            typen: CType::Function(Box::new(t.clone()), Box::new(int64.clone())),
                            microstatements: vec![Microstatement::Value {
                                typen: int64,
                                representation: format!("{}", i),
                            }],
                            kind: FnKind::Static,
                            origin_scope_path: scope.path.clone(),
                        });
                    }
                    CType::Float(f) => {
                        // Create an accessor function for this value, but do not add
                        // it to the args array to construct it. The accessor function
                        // will return this value as an f64.
                        let float64 = CType::f64();
                        fs.push(Function {
                            name: n.clone(),
                            typen: CType::Function(Box::new(t.clone()), Box::new(float64.clone())),
                            microstatements: vec![Microstatement::Value {
                                typen: float64,
                                representation: format!("{}", f),
                            }],
                            kind: FnKind::Static,
                            origin_scope_path: scope.path.clone(),
                        });
                    }
                    CType::Bool(b) => {
                        // Create an accessor function for this value, but do not add
                        // it to the args array to construct it. The accessor function
                        // will return this value as a bool.
                        let booln = CType::bool();
                        fs.push(Function {
                            name: n.clone(),
                            typen: CType::Function(Box::new(t.clone()), Box::new(booln.clone())),
                            microstatements: vec![Microstatement::Value {
                                typen: booln,
                                representation: match b {
                                    true => "true".to_string(),
                                    false => "false".to_string(),
                                },
                            }],
                            kind: FnKind::Static,
                            origin_scope_path: scope.path.clone(),
                        });
                    }
                    _ => {
                        fs.push(Function {
                            name: n.clone(),
                            typen: CType::Function(Box::new(t.clone()), Box::new(*f.clone())),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                        });
                    }
                }
                // Define the constructor function
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(Box::new(*f.clone()), Box::new(t.clone())),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                });
            }
            CType::Either(ts) => {
                // There are an equal number of constructor functions and accessor
                // functions, one for each inner type of the sum type.
                for e in ts {
                    // Create a constructor fn
                    fs.push(Function {
                        name: constructor_fn_name.clone(),
                        typen: CType::Function(Box::new(e.clone()), Box::new(t.clone())),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                    });
                    // Create a store fn to re-assign-and-auto-wrap a value
                    fs.push(Function {
                        name: "store".to_string(),
                        typen: CType::Function(
                            Box::new(CType::Tuple(vec![t.clone(), e.clone()])),
                            Box::new(t.clone()),
                        ),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                    });
                    if let CType::Void = &e {
                        // Have a zero-arg constructor function produce the void type, if possible.
                        fs.push(Function {
                            name: constructor_fn_name.clone(),
                            typen: CType::Function(Box::new(CType::Void), Box::new(t.clone())),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                        });
                    }
                    // Create the accessor function, the name of the function will
                    // depend on the kind of type this is
                    match e {
                        CType::Field(n, i) => fs.push(Function {
                            name: n.clone(),
                            typen: CType::Function(
                                Box::new(t.clone()),
                                Box::new(CType::Either(vec![*i.clone(), CType::Void])),
                            ),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                        }),
                        CType::Type(n, _) => fs.push(Function {
                            name: n.clone(),
                            typen: CType::Function(
                                Box::new(t.clone()),
                                Box::new(CType::Either(vec![e.clone(), CType::Void])),
                            ),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                        }),
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
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(Box::new(*b.clone()), Box::new(t.clone())),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                });
                let size = match **s {
                    CType::Int(s) => s as usize,
                    _ => 0, // TODO: Make this function fallible, instead?
                };
                if size > 1 {
                    fs.push(Function {
                        name: constructor_fn_name.clone(),
                        typen: CType::Function(
                            Box::new(CType::Tuple({
                                let mut v = Vec::new();
                                for _ in 0..size {
                                    v.push(*b.clone());
                                }
                                v
                            })),
                            Box::new(t.clone()),
                        ),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                    });
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
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(Box::new(*a.clone()), Box::new(t.clone())),
                    microstatements: Vec::new(),
                    kind: FnKind::DerivedVariadic,
                    origin_scope_path: scope.path.clone(),
                });
            }
            CType::Int(i) => {
                // TODO: Support construction of other integer types
                let int64 = CType::i64();
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(Box::new(CType::Void), Box::new(int64.clone())),
                    microstatements: vec![Microstatement::Return {
                        value: Some(Box::new(Microstatement::Value {
                            typen: int64,
                            representation: format!("{}", i),
                        })),
                    }],
                    kind: FnKind::Normal,
                    origin_scope_path: scope.path.clone(),
                });
            }
            CType::Float(f) => {
                // TODO: Support construction of other float types
                let float64 = CType::f64();
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(Box::new(CType::Void), Box::new(float64.clone())),
                    microstatements: vec![Microstatement::Return {
                        value: Some(Box::new(Microstatement::Value {
                            typen: float64,
                            representation: format!("{}", f),
                        })),
                    }],
                    kind: FnKind::Normal,
                    origin_scope_path: scope.path.clone(),
                });
            }
            CType::Bool(b) => {
                let booln = CType::bool();
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(Box::new(CType::Void), Box::new(booln.clone())),
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
                });
            }
            CType::TString(s) => {
                let string = CType::string();
                fs.push(Function {
                    name: constructor_fn_name.clone(),
                    typen: CType::Function(Box::new(CType::Void), Box::new(string.clone())),
                    microstatements: vec![Microstatement::Return {
                        value: Some(Box::new(Microstatement::Value {
                            typen: string,
                            representation: format!("\"{}\"", s.replace("\"", "\\\"")),
                        })),
                    }],
                    kind: FnKind::Normal,
                    origin_scope_path: scope.path.clone(),
                });
            }
            _ => {} // Don't do anything for other types
        }
        (t, fs)
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
            match generic_call {
                CType::Bool(b) => match b {
                    false => return Ok((scope, CType::Fail(format!("{} is not supposed to be compiled because the conditional compilation generic value is false", name)))),
                    true => { /* Do nothing */ }
                },
                CType::Type(n, c) => match *c {
                    CType::Bool(b) => match b {
                        false => return Ok((scope, CType::Fail(format!("{} is not supposed to be compiled because {} is false", name, n)))),
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
                while matches!(&inner_type, CType::Group(_)) {
                    inner_type = match inner_type {
                        CType::Group(t) => *t,
                        t => t,
                    };
                }
                // Let's just avoid the "bare field" type definition and auto-wrap into a tuple
                if let CType::Field(..) = &inner_type {
                    inner_type = CType::Tuple(vec![inner_type]);
                }
                // Magic hackery to convert a `From` type into an `Import` type if it's the top-level type
                inner_type = match inner_type {
                    CType::From(t) => CType::import(CType::TString(name.clone()), *t),
                    t => t,
                };
                // If we've got an `Import` type, we need to grab the actual type definition from
                // the other file and pull it in here.
                if let CType::Import(name, dep) = &inner_type {
                    match &**dep {
                        CType::TString(dep_name) => {
                            let program = Program::get_program();
                            let scope = program.scope_by_file(dep_name)?;
                            match &**name {
                                CType::TString(n) => {
                                    inner_type = match scope.types.get(n) {
                                        None => {
                                            CType::fail(&format!("{} not found in {}", n, dep_name))
                                        }
                                        Some(t) => match t {
                                            CType::Type(_, t) => (**t).clone(),
                                            t => t.clone(),
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
                    temp_scope
                        .types
                        .insert(arg.clone(), CType::Infer(arg.clone(), "Any".to_string()));
                }
                let generic_call =
                    withtypeoperatorslist_to_ctype(&type_ast.typedef.typeassignables, &temp_scope)?;
                (
                    CType::Generic(name.clone(), args, Box::new(generic_call)),
                    Vec::new(),
                )
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
        scope.types.insert(name.clone(), t.clone());
        scope.types.insert(t.to_callable_string(), t.clone());
        if !fs.is_empty() {
            let mut name_fn_pairs = HashMap::new();
            for f in fs {
                if name_fn_pairs.contains_key(&f.name) {
                    let v: &mut Vec<Function> = name_fn_pairs.get_mut(&f.name).unwrap();
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

    pub fn from_ctype(mut scope: Scope, name: String, ctype: CType) -> Scope {
        scope.exports.insert(name.clone(), Export::Type);
        let (_, fs) = ctype.to_functions(name.clone(), &scope);
        scope.types.insert(name, ctype.clone());
        scope.types.insert(ctype.to_callable_string(), ctype);
        if !fs.is_empty() {
            let mut name_fn_pairs = HashMap::new();
            for f in fs {
                // We need to similarly load all of the return types from the functions created by
                // this from_ctype call if they don't already exist
                let mut contains_rettype = false;
                for t in scope.types.values() {
                    if f.rettype().to_callable_string() == t.to_callable_string() {
                        contains_rettype = true;
                    }
                }
                if !contains_rettype {
                    scope = CType::from_ctype(
                        scope,
                        f.rettype().to_callable_string(),
                        f.rettype().clone(),
                    );
                }
                if name_fn_pairs.contains_key(&f.name) {
                    let v: &mut Vec<Function> = name_fn_pairs.get_mut(&f.name).unwrap();
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
            CType::IntrinsicGeneric(name.to_string(), arglen),
        )
    }
    pub fn swap_subtype(&self, old_type: &CType, new_type: &CType) -> CType {
        // Implemented recursively to be easier to follow. It would be nice to avoid all of the
        // cloning if the old type is not anywhere in the CType tree, but that would be a lot
        // harder to detect ahead of time.
        if self == old_type {
            return new_type.clone();
        }
        match self {
            CType::Void
            | CType::Infer(..)
            | CType::Generic(..)
            | CType::IntrinsicGeneric(..)
            | CType::Int(_)
            | CType::Float(_)
            | CType::Bool(_)
            | CType::TString(_)
            | CType::Fail(_) => self.clone(),
            CType::Type(name, ct) => {
                CType::Type(name.clone(), Box::new(ct.swap_subtype(old_type, new_type)))
            }
            CType::Binds(name, gen_type_resolved) => CType::Binds(
                Box::new(name.swap_subtype(old_type, new_type)),
                gen_type_resolved
                    .iter()
                    .map(|gtr| gtr.swap_subtype(old_type, new_type))
                    .collect::<Vec<CType>>(),
            ),
            CType::Group(g) => g.swap_subtype(old_type, new_type),
            CType::Function(i, o) => CType::Function(
                Box::new(i.swap_subtype(old_type, new_type)),
                Box::new(o.swap_subtype(old_type, new_type)),
            ),
            CType::Call(n, f) => CType::Call(
                Box::new(n.swap_subtype(old_type, new_type)),
                Box::new(f.swap_subtype(old_type, new_type)),
            ),
            CType::Infix(o) => CType::Infix(Box::new(o.swap_subtype(old_type, new_type))),
            CType::Prefix(o) => CType::Prefix(Box::new(o.swap_subtype(old_type, new_type))),
            CType::Method(f) => CType::Method(Box::new(f.swap_subtype(old_type, new_type))),
            CType::Property(p) => CType::Property(Box::new(p.swap_subtype(old_type, new_type))),
            CType::Cast(t) => CType::Cast(Box::new(t.swap_subtype(old_type, new_type))),
            CType::Own(t) => CType::Own(Box::new(t.swap_subtype(old_type, new_type))),
            CType::Deref(t) => CType::Deref(Box::new(t.swap_subtype(old_type, new_type))),
            CType::Mut(t) => CType::Mut(Box::new(t.swap_subtype(old_type, new_type))),
            CType::Dependency(n, v) => CType::Dependency(
                Box::new(n.swap_subtype(old_type, new_type)),
                Box::new(v.swap_subtype(old_type, new_type)),
            ),
            CType::Rust(d) => CType::Rust(Box::new(d.swap_subtype(old_type, new_type))),
            CType::Node(d) => CType::Node(Box::new(d.swap_subtype(old_type, new_type))),
            CType::From(d) => CType::From(Box::new(d.swap_subtype(old_type, new_type))),
            CType::Import(n, d) => CType::Import(
                Box::new(n.swap_subtype(old_type, new_type)),
                Box::new(d.swap_subtype(old_type, new_type)),
            ),
            CType::Tuple(ts) => CType::Tuple(
                ts.iter()
                    .map(|t| t.swap_subtype(old_type, new_type))
                    .collect::<Vec<CType>>(),
            ),
            CType::Field(name, t) => {
                CType::Field(name.clone(), Box::new(t.swap_subtype(old_type, new_type)))
            }
            CType::Either(ts) => CType::Either(
                ts.iter()
                    .map(|t| t.swap_subtype(old_type, new_type))
                    .collect::<Vec<CType>>(),
            ),
            CType::Prop(t, p) => CType::Prop(
                Box::new(t.swap_subtype(old_type, new_type)),
                Box::new(p.swap_subtype(old_type, new_type)),
            ),
            CType::AnyOf(ts) => CType::AnyOf(
                ts.iter()
                    .map(|t| t.swap_subtype(old_type, new_type))
                    .collect::<Vec<CType>>(),
            ),
            CType::Buffer(t, size) => CType::Buffer(
                Box::new(t.swap_subtype(old_type, new_type)),
                Box::new(size.swap_subtype(old_type, new_type)),
            ),
            CType::Array(t) => CType::Array(Box::new(t.swap_subtype(old_type, new_type))),
            // For these when we swap, we check to see if we can "condense" them down into simpler
            // types (eg `Add{N, 1}` swapping `N` for `3` should just yield `4`)
            CType::Add(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::add(&a, &b))
                .unwrap(),
            CType::Sub(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::sub(&a, &b))
                .unwrap(),
            CType::Mul(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::mul(&a, &b))
                .unwrap(),
            CType::Div(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::div(&a, &b))
                .unwrap(),
            CType::Mod(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::cmod(&a, &b))
                .unwrap(),
            CType::Pow(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::pow(&a, &b))
                .unwrap(),
            CType::Min(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::min(&a, &b))
                .unwrap(),
            CType::Max(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::max(&a, &b))
                .unwrap(),
            CType::Neg(t) => CType::neg(&t.swap_subtype(old_type, new_type)),
            CType::Len(t) => CType::len(&t.swap_subtype(old_type, new_type)),
            CType::Size(t) => CType::size(&t.swap_subtype(old_type, new_type)),
            CType::FileStr(t) => CType::filestr(&t.swap_subtype(old_type, new_type)),
            CType::Concat(a, b) => CType::concat(
                &a.swap_subtype(old_type, new_type),
                &b.swap_subtype(old_type, new_type),
            ),
            CType::Env(ts) => {
                if ts.len() == 1 {
                    CType::env(&ts[0].swap_subtype(old_type, new_type))
                } else if ts.len() == 2 {
                    CType::envdefault(
                        &ts[0].swap_subtype(old_type, new_type),
                        &ts[1].swap_subtype(old_type, new_type),
                    )
                } else {
                    CType::fail("Somehow gave Env{..} an incorrect number of args and caught during generic resolution")
                }
            }
            CType::EnvExists(t) => CType::envexists(&t.swap_subtype(old_type, new_type)),
            CType::TIf(t, ts) => {
                if ts.len() == 1 {
                    CType::tupleif(
                        &t.swap_subtype(old_type, new_type),
                        &ts[0].swap_subtype(old_type, new_type),
                    )
                } else if ts.len() == 2 {
                    CType::cif(
                        &t.swap_subtype(old_type, new_type),
                        &ts[0].swap_subtype(old_type, new_type),
                        &ts[1].swap_subtype(old_type, new_type),
                    )
                } else {
                    CType::fail("Somehow gave If{..} an incorrect number of args and caught during generic resolution")
                }
            }
            CType::And(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::and(&a, &b))
                .unwrap(),
            CType::Or(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::or(&a, &b))
                .unwrap(),
            CType::Xor(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::xor(&a, &b))
                .unwrap(),
            CType::Not(t) => CType::not(&t.swap_subtype(old_type, new_type)),
            CType::Nand(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::nand(&a, &b))
                .unwrap(),
            CType::Nor(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::nor(&a, &b))
                .unwrap(),
            CType::Xnor(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::xnor(&a, &b))
                .unwrap(),
            CType::TEq(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::eq(&a, &b))
                .unwrap(),
            CType::Neq(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::neq(&a, &b))
                .unwrap(),
            CType::Lt(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::lt(&a, &b))
                .unwrap(),
            CType::Lte(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::lte(&a, &b))
                .unwrap(),
            CType::Gt(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::gt(&a, &b))
                .unwrap(),
            CType::Gte(ts) => ts
                .iter()
                .map(|t| t.swap_subtype(old_type, new_type))
                .reduce(|a, b| CType::gte(&a, &b))
                .unwrap(),
        }
    }
    pub fn binds(args: Vec<CType>) -> CType {
        let base_type = args[0].clone();
        if matches!(
            base_type,
            CType::TString(_) | CType::Import(..) | CType::From(_)
        ) {
            let mut out_vec = Vec::new();
            #[allow(clippy::needless_range_loop)] // It's not needless
            for i in 1..args.len() {
                out_vec.push(args[i].clone());
            }
            CType::Binds(Box::new(base_type), out_vec)
        } else {
            CType::fail(
                "Binds{T, ...} must be given a string or an import for the base type to bind",
            );
        }
    }
    pub fn import(name: CType, dep: CType) -> CType {
        if let CType::Infer(..) = &name {
            CType::Import(Box::new(name), Box::new(dep))
        } else if let CType::Infer(..) = &dep {
            CType::Import(Box::new(name), Box::new(dep))
        } else if !matches!(name, CType::TString(_)) {
            CType::fail("The Import{N, D} N parameter must be a string")
        } else {
            match &dep {
                CType::TString(s) => {
                    // Load the dependency
                    if let Err(e) = Program::load(s.clone()) {
                        CType::fail(&format!("Failed to load dependency {}: {:?}", s, e))
                    } else {
                        let program = Program::get_program();
                        let out = match program.scope_by_file(s) {
                            Err(e) => {
                                CType::fail(&format!("Failed to load dependency {}: {:?}", s, e))
                            }
                            Ok(dep_scope) => {
                                // Currently can only import types and functions. Constants and
                                // operator mappings don't have a syntax to express this. TODO:
                                // Figure out how to tackle this syntactically, and then update
                                // this logic.
                                if let CType::TString(n) = &name {
                                    let found = dep_scope.types.contains_key(n)
                                        || dep_scope.functions.contains_key(n);
                                    if !found {
                                        CType::fail(&format!("{} not found in {}", n, s))
                                    } else {
                                        // We're good
                                        CType::Import(Box::new(name), Box::new(dep))
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
                CType::Node(_) | CType::Rust(_) => CType::Import(Box::new(name), Box::new(dep)),
                CType::Type(_, t) if matches!(**t, CType::Node(_) | CType::Rust(_)) => {
                    CType::Import(Box::new(name), Box::new(dep))
                }
                otherwise => CType::fail(&format!(
                    "Invalid import defined {:?} <- {:?}",
                    name, otherwise
                )),
            }
        }
    }
    // Special implementation for the tuple and either types since they *are* CTypes, but if one of
    // the provided input types *is* the same kind of CType, it should produce a merged version.
    pub fn tuple(args: Vec<CType>) -> CType {
        let mut out_vec = Vec::new();
        for arg in args {
            match arg {
                CType::Tuple(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                other => out_vec.push(other),
            }
        }
        CType::Tuple(out_vec)
    }
    pub fn either(args: Vec<CType>) -> CType {
        let mut out_vec = Vec::new();
        for arg in args {
            match arg {
                CType::Either(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                other => out_vec.push(other),
            }
        }
        CType::Either(out_vec)
    }
    pub fn prop(t: CType, p: CType) -> CType {
        // Checking the p property type first if it's to be inferred because short-circuiting on
        // that will simplify follow-up logic
        if let CType::Infer(..) = &p {
            return CType::Prop(Box::new(t), Box::new(p));
        }
        match t.clone() {
            CType::Infer(..) => CType::Prop(Box::new(t), Box::new(p)),
            CType::Type(_, t) => CType::prop(*t, p),
            CType::Group(t) => CType::prop(*t, p),
            CType::Field(n, f) => match p {
                CType::TString(s) => {
                    if n == s {
                        *f.clone()
                    } else {
                        CType::fail(&format!("Property {} not found on type {:?}", s, &t))
                    }
                }
                CType::Int(i) => match i {
                    0 => CType::TString(n.to_string()),
                    1 => *f.clone(),
                    _ => CType::fail("Only 0 or 1 are valid integer accesses on a field"),
                },
                otherwise => CType::fail(&format!(
                    "Properties must be a name or integer location, not {:?}",
                    otherwise,
                )),
            },
            CType::Tuple(ts) | CType::Either(ts) => match p {
                CType::TString(s) => {
                    for inner in ts {
                        if let CType::Field(n, f) = inner {
                            if n == s {
                                return *f.clone();
                            }
                        }
                    }
                    CType::fail(&format!("Property {} not found on type {:?}", s, t))
                }
                CType::Int(i) => {
                    if (0..ts.len()).contains(&(i as usize)) {
                        ts[i as usize].clone()
                    } else {
                        CType::fail(&format!("{} is out of bounds for type {:?}", i, t))
                    }
                }
                otherwise => CType::fail(&format!(
                    "Properties must be a name or integer location, not {:?}",
                    otherwise,
                )),
            },
            CType::TIf(_, tf) => {
                match p {
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
                        if b {
                            tf[0].clone()
                        } else {
                            tf[1].clone()
                        }
                    }
                    CType::Int(i) => {
                        if (0..2).contains(&i) {
                            tf[i as usize].clone()
                        } else {
                            CType::fail("Only true or false (or 1 or 0) are valid for accessing the types from an If{C, A, B} type")
                        }
                    }
                    otherwise => CType::fail(&format!(
                        "Properties must be a name or integer location, not {:?}",
                        otherwise,
                    )),
                }
            }
            otherwise => CType::fail(&format!(
                "Properties cannot be accessed from type {:?}",
                otherwise
            )),
        }
    }
    pub fn anyof(args: Vec<CType>) -> CType {
        let mut out_vec = Vec::new();
        for arg in args {
            match arg {
                CType::AnyOf(ts) => {
                    for t in ts {
                        out_vec.push(t.clone());
                    }
                }
                other => out_vec.push(other),
            }
        }
        CType::Either(out_vec)
    }
    pub fn field(mut args: Vec<CType>) -> CType {
        if args.len() != 2 {
            CType::fail("Field{K, V} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap();
            let arg0 = args.pop().unwrap();
            match (arg0, arg1) {
                (CType::TString(key), anything) => {
                    CType::Field(key.clone(), Box::new(anything.clone()))
                }
                _ => CType::fail("The field key must be a quoted string at this time"),
            }
        }
    }
    // Some validation for buffer creation, too
    pub fn buffer(mut args: Vec<CType>) -> CType {
        if args.len() != 2 {
            CType::fail("Buffer{T, S} only accepts two sub-types")
        } else {
            let arg1 = args.pop().unwrap().degroup();
            let arg0 = args.pop().unwrap().degroup();
            match (&arg0, &arg1) {
                (CType::Infer(..), _) => {
                    CType::Buffer(Box::new(arg0.clone()), Box::new(arg1.clone()))
                }
                (_, CType::Infer(..)) => {
                    CType::Buffer(Box::new(arg0.clone()), Box::new(arg1.clone()))
                }
                (anything, CType::Int(size)) => {
                    if *size < 0 {
                        CType::fail("The buffer size must be a positive integer")
                    } else {
                        CType::Buffer(Box::new(anything.clone()), Box::new(CType::Int(*size)))
                    }
                }
                otherwise => CType::fail(&format!(
                    "The buffer size must be a positive integer {:?}",
                    otherwise
                )),
            }
        }
    }
    // Implementation of the ctypes that aren't storage but compute into another CType
    pub fn fail(message: &str) -> ! {
        // TODO: Include more information on where this compiler exit is coming from
        eprintln!("{}", message);
        std::process::exit(1);
    }
    pub fn cfail(message: &CType) -> CType {
        match message {
            CType::TString(s) => CType::Fail(s.clone()),
            _ => CType::fail("Fail passed a type that does not resolve into a message string"),
        }
    }
    pub fn neg(t: &CType) -> CType {
        match *t {
            CType::Int(v) => CType::Int(-v),
            CType::Float(v) => CType::Float(-v),
            CType::Infer(..) => CType::Neg(Box::new(t.clone())),
            _ => CType::fail("Attempting to negate non-integer or non-float types at compile time"),
        }
    }
    pub fn len(t: &CType) -> CType {
        match t {
            CType::Tuple(tup) => CType::Int(tup.len() as i128),
            CType::Buffer(_, l) => match **l {
                CType::Int(l) => CType::Int(l),
                _ => {
                    CType::fail("Cannot get a compile time length for an invalid Buffer definition")
                }
            },
            CType::Either(eit) => CType::Int(eit.len() as i128),
            CType::Array(_) => {
                CType::fail("Cannot get a compile time length for a variable-length array")
            }
            CType::Infer(..) => CType::Len(Box::new(t.clone())),
            _ => CType::Int(1),
        }
    }
    pub fn size(t: &CType) -> CType {
        // TODO: Implementing this might require all types be made C-style structs under the hood,
        // and probably some weird hackery to find out the size including padding on aligned
        // architectures, so I might take it back out before its actually implemented, but I can
        // think of several places where knowing the actual size of the type could be useful,
        // particularly for writing to disk or interfacing with network protocols, etc, so I'd
        // prefer to keep it and have some compile-time guarantees we don't normally see.
        match t {
            CType::Infer(..) => CType::Size(Box::new(t.clone())),
            _ => CType::fail("TODO: Implement Size{T}!"),
        }
    }
    pub fn filestr(f: &CType) -> CType {
        match f {
            CType::TString(s) => match std::fs::read_to_string(s) {
                Err(e) => CType::fail(&format!("Failed to read {}: {:?}", s, e)),
                Ok(s) => CType::TString(s),
            },
            CType::Infer(..) => CType::FileStr(Box::new(f.clone())),
            _ => CType::fail("FileStr{F} must be given a string path to load"),
        }
    }
    pub fn concat(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Infer(..), _) | (_, CType::Infer(..)) => {
                CType::Concat(Box::new(a.clone()), Box::new(b.clone()))
            }
            (CType::TString(a), CType::TString(b)) => CType::TString(format!("{}{}", a, b)),
            _ => CType::fail("Concat{A, B} must be given strings to concatenate"),
        }
    }
    pub fn env(k: &CType) -> CType {
        let program = Program::get_program();
        let out = match k {
            CType::TString(s) => match program.env.get(s) {
                None => CType::fail(&format!("Failed to load environment variable {}", s,)),
                Some(s) => CType::TString(s.clone()),
            },
            CType::Infer(..) => CType::Env(vec![k.clone()]),
            _ => CType::fail("Env{K} must be given a key as a string to load"),
        };
        Program::return_program(program);
        out
    }
    pub fn envexists(k: &CType) -> CType {
        let program = Program::get_program();
        let out = match k {
            CType::TString(s) => CType::Bool(program.env.contains_key(s)),
            CType::Infer(..) => CType::EnvExists(Box::new(k.clone())),
            _ => CType::fail("EnvExists{K} must be given a key as a string to check"),
        };
        Program::return_program(program);
        out
    }
    pub fn not(b: &CType) -> CType {
        match b {
            CType::Bool(b) => CType::Bool(!*b),
            CType::Infer(..) => CType::Not(Box::new(b.clone())),
            _ => CType::fail("Not{B} must be provided a boolean type to invert"),
        }
    }
    pub fn min(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn max(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn add(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn sub(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn mul(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn div(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn cmod(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (&CType::Int(a), &CType::Int(b)) => CType::Int(a * b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_)) => {
                CType::Mod(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_), &CType::Infer(..)) => CType::Mod(vec![a.clone(), b.clone()]),
            _ => CType::fail("Attempting to modulus non-integer types together at compile time"),
        }
    }
    pub fn pow(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn cif(c: &CType, a: &CType, b: &CType) -> CType {
        match c {
            CType::Bool(cond) => match cond {
                true => a.clone(),
                false => b.clone(),
            },
            CType::Infer(..) => CType::TIf(Box::new(c.clone()), vec![a.clone(), b.clone()]),
            _ => CType::fail("If{C, A, B} must be given a boolean value as the condition"),
        }
    }
    pub fn tupleif(c: &CType, t: &CType) -> CType {
        match c {
            CType::Bool(cond) => {
                match t {
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
            CType::Infer(..) => CType::TIf(Box::new(c.clone()), vec![t.clone()]),
            _ => CType::fail("The first type provided to If{C, T} must be a boolean type"),
        }
    }
    pub fn envdefault(k: &CType, d: &CType) -> CType {
        let program = Program::get_program();
        let out = match (k, d) {
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
        out
    }
    pub fn and(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn or(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn xor(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(*a ^ *b),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(*a ^ *b),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Xor(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Xor(vec![a.clone(), b.clone()])
            }
            _ => CType::fail(
                "Or{A, B} must be provided two values of the same type, either integer or boolean",
            ),
        }
    }
    pub fn nand(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a & *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a && *b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Nand(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Nand(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Nand{A, B} must be provided two values of the same type, either integer or boolean")
        }
    }
    pub fn nor(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn xnor(a: &CType, b: &CType) -> CType {
        match (a, b) {
            (CType::Int(a), CType::Int(b)) => CType::Int(!(*a ^ *b)),
            (CType::Bool(a), CType::Bool(b)) => CType::Bool(!(*a ^ *b)),
            (&CType::Infer(..), &CType::Infer(..) | &CType::Int(_) | &CType::Bool(_)) => {
                CType::Xnor(vec![a.clone(), b.clone()])
            }
            (&CType::Int(_) | &CType::Bool(_), &CType::Infer(..)) => {
                CType::Xnor(vec![a.clone(), b.clone()])
            }
            _ => CType::fail("Xnor{A, B} must be provided two values of the same type, either integer or boolean")
        }
    }
    pub fn eq(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn neq(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn lt(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn lte(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn gt(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
    pub fn gte(a: &CType, b: &CType) -> CType {
        match (a, b) {
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
        }
    }
}

// TODO: I really hoped these two would share more code. Figure out how to DRY this out later, if
// possible
pub fn withtypeoperatorslist_to_ctype(
    withtypeoperatorslist: &Vec<parse::WithTypeOperators>,
    scope: &Scope,
) -> Result<CType, Box<dyn std::error::Error>> {
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
                    None => Err(format!("Operator {} not found", operatorname)),
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
                None => Err(format!("Operator {} not found", operatorname)),
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
                        "Operator {} is an infix operator but missing a left-hand side value",
                        operatorname
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
                    None => Err(format!("Operator {} is an infix operator but missing a right-hand side value", operatorname)),
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
                        "Operator {} is a prefix operator but missing a right-hand side value",
                        operatorname
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
                        "Operator {} is a postfix operator but missing a left-hand side value",
                        operatorname
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
                return Err(format!("Invalid syntax: {:?}", withtypeoperatorslist).into());
            }
            let typebaselist = match match queue.pop() {
                Some(v) => Ok(v),
                None => Err(format!(
                    "Somehow we collapsed the statement into nothing? {:?}",
                    withtypeoperatorslist
                )),
            }? {
                parse::WithTypeOperators::TypeBaseList(b) => Ok(b),
                _ => Err(format!(
                    "Somehow we collapse the statement into a solitary operator? {:?}",
                    withtypeoperatorslist
                )),
            }?;
            out_ctype = Some(typebaselist_to_ctype(&typebaselist, scope)?);
        }
    }
    match out_ctype {
        Some(ctype) => Ok(ctype),
        None => Err(format!("Never resolved a type from {:?}", withtypeoperatorslist).into()),
    }
}

// TODO: This similarly shares a lot of structure with baseassignablelist_to_microstatements, see
// if there is any way to DRY this up, or is it just doomed to be like this?
pub fn typebaselist_to_ctype(
    typebaselist: &[parse::TypeBase],
    scope: &Scope,
) -> Result<CType, Box<dyn std::error::Error>> {
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
                            prior_value = Some(CType::Bool(b.as_str() == "true"))
                        }
                        parse::Constants::Strn(s) => {
                            prior_value = Some(CType::TString(if s.starts_with('"') {
                                s.split("\\\"")
                                    .map(|sub| sub.replace("\"", ""))
                                    .collect::<Vec<String>>()
                                    .join("\"")
                            } else {
                                s.split("\\'")
                                    .map(|sub| sub.replace("'", ""))
                                    .collect::<Vec<String>>()
                                    .join("'")
                            }))
                        }
                        parse::Constants::Num(n) => match n {
                            parse::Number::RealNum(r) => {
                                prior_value = Some(CType::Float(
                                    r.replace('_', "").parse::<f64>().unwrap(), // This should never fail if the
                                                                                // parser says it's a float
                                ))
                            }
                            parse::Number::IntNum(i) => {
                                prior_value = Some(CType::Int(
                                    i.replace('_', "").parse::<i128>().unwrap(), // Same deal here
                                ))
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
                            prior_value = Some(match prior_value.unwrap() {
                                CType::Tuple(ts) => {
                                    let mut out = None;
                                    for t in &ts {
                                        if let CType::Field(f, c) = t {
                                            if f.as_str() == s.as_str() {
                                                out = Some(*c.clone());
                                            }
                                        }
                                    }
                                    match out {
                                        Some(o) => o,
                                        None => CType::fail(&format!("{:?} does not have a property named {}", ts, s)),
                                    }
                                }
                                CType::Function(i, o) => match s.as_str() {
                                    "input" => *i.clone(),
                                    "output" => *o.clone(),
                                    _ => CType::fail("Function types only have \"input\" and \"output\" properties"),
                                }
                                other => CType::fail(&format!("String properties are not allowed on {:?}", other)),
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
                                    prior_value = Some(match prior_value.unwrap() {
                                        CType::Tuple(ts) => match ts.get(idx) {
                                            Some(t) => t.clone(),
                                            None => CType::fail(&format!(
                                                "{} is larger than the size of {:?}",
                                                idx, ts
                                            )),
                                        },
                                        CType::Either(ts) => match ts.get(idx) {
                                            Some(t) => t.clone(),
                                            None => CType::fail(&format!(
                                                "{} is larger than the size of {:?}",
                                                idx, ts
                                            )),
                                        },
                                        other => CType::fail(&format!(
                                            "{:?} cannot be indexed by an integer",
                                            other
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
                match &prior_value {
                    Some(val) => args.push(val.clone()),
                    None => {}
                };
                prior_value = Some(match scope.resolve_type(var) {
                    Some(t) => {
                        // TODO: Once interfaces are a thing, there needs to be a built-in
                        // interface called `Label` that we can use here to mark the first argument
                        // to `Field` as a `Label` and turn this logic into something regularized
                        // For now, we're just special-casing the `Field` built-in generic type.
                        match &t {
                            CType::IntrinsicGeneric(p, 2) if p == "Prop" => {
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
                                            args.push(withtypeoperatorslist_to_ctype(&vec![g.typecalllist[0].clone()], scope)?);
                                            match g.typecalllist[0].to_string().parse::<i128>() {
                                                Ok(i) => args.push(CType::Int(i)),
                                                Err(_) => {
                                                    let argstr = g.typecalllist[2].to_string();
                                                    match argstr.as_str() {
                                                        "true" => args.push(CType::Bool(true)),
                                                        "false" => args.push(CType::Bool(false)),
                                                        _ => args.push(CType::TString(argstr))
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
                                            args.push(CType::TString(g.typecalllist[0].to_string()));
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
                        match t {
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
                                    let mut out_type = *generic_type.clone();
                                    for i in 0..params.len() {
                                        let generic_arg =
                                            CType::Infer(params[i].clone(), "Any".to_string());
                                        out_type = out_type.swap_subtype(&generic_arg, &args[i]);
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
                                        "Binds" => CType::binds(args.clone()),
                                        "Group" => CType::Group(Box::new(args[0].clone())),
                                        "Function" => CType::Function(
                                            Box::new(args[0].clone()),
                                            Box::new(args[1].clone()),
                                        ),
                                        "Call" => CType::Call(
                                            Box::new(args[0].clone()),
                                            Box::new(args[1].clone()),
                                        ),
                                        "Infix" => CType::Infix(Box::new(args[0].clone())),
                                        "Prefix" => CType::Prefix(Box::new(args[0].clone())),
                                        "Method" => CType::Method(Box::new(args[0].clone())),
                                        "Property" => CType::Property(Box::new(args[0].clone())),
                                        "Cast" => CType::Cast(Box::new(args[0].clone())),
                                        "Own" => CType::Own(Box::new(args[0].clone())),
                                        "Deref" => CType::Deref(Box::new(args[0].clone())),
                                        "Mut" => CType::Mut(Box::new(args[0].clone())),
                                        "Dependency" => CType::Dependency(
                                            Box::new(args[0].clone()),
                                            Box::new(args[1].clone()),
                                        ),
                                        "Rust" => CType::Rust(Box::new(args[0].clone())),
                                        "Node" => CType::Node(Box::new(args[0].clone())),
                                        "From" => CType::From(Box::new(args[0].clone())),
                                        "Import" => CType::import(args[0].clone(), args[1].clone()),
                                        "Tuple" => CType::tuple(args.clone()),
                                        "Field" => CType::field(args.clone()),
                                        "Either" => CType::either(args.clone()),
                                        "Prop" => CType::prop(args[0].clone(), args[1].clone()),
                                        "AnyOf" => CType::anyof(args.clone()),
                                        "Buffer" => CType::buffer(args.clone()),
                                        "Array" => CType::Array(Box::new(args[0].clone())),
                                        "Fail" => CType::cfail(&args[0]),
                                        "Min" => CType::min(&args[0], &args[1]),
                                        "Max" => CType::max(&args[0], &args[1]),
                                        "Neg" => CType::neg(&args[0]),
                                        "Len" => CType::len(&args[0]),
                                        "Size" => CType::size(&args[0]),
                                        "FileStr" => CType::filestr(&args[0]),
                                        "Concat" => CType::concat(&args[0], &args[1]),
                                        "Env" => CType::env(&args[0]),
                                        "EnvExists" => CType::envexists(&args[0]),
                                        "Not" => CType::not(&args[0]),
                                        "Add" => CType::add(&args[0], &args[1]),
                                        "Sub" => CType::sub(&args[0], &args[1]),
                                        "Mul" => CType::mul(&args[0], &args[1]),
                                        "Div" => CType::div(&args[0], &args[1]),
                                        "Mod" => CType::cmod(&args[0], &args[1]),
                                        "Pow" => CType::pow(&args[0], &args[1]),
                                        "If" => {
                                            if args.len() == 2 {
                                                CType::tupleif(&args[0], &args[1])
                                            } else if args.len() == 3 {
                                                CType::cif(&args[0], &args[1], &args[2])
                                            } else {
                                                CType::fail(&format!("Invalid arguments provided to `If{{...}}`: {:?}", args))
                                            }
                                        }
                                        "And" => CType::and(&args[0], &args[1]),
                                        "Or" => CType::or(&args[0], &args[1]),
                                        "Xor" => CType::xor(&args[0], &args[1]),
                                        "Nand" => CType::nand(&args[0], &args[1]),
                                        "Nor" => CType::nor(&args[0], &args[1]),
                                        "Xnor" => CType::xnor(&args[0], &args[1]),
                                        "Eq" => CType::eq(&args[0], &args[1]),
                                        "Neq" => CType::neq(&args[0], &args[1]),
                                        "Lt" => CType::lt(&args[0], &args[1]),
                                        "Lte" => CType::lte(&args[0], &args[1]),
                                        "Gt" => CType::gt(&args[0], &args[1]),
                                        "Gte" => CType::gte(&args[0], &args[1]),
                                        unknown => CType::fail(&format!(
                                            "Unknown ctype {} accessed. How did this happen?",
                                            unknown
                                        )),
                                    }
                                }
                            }
                            others => {
                                // If we hit this branch, then the `args` vector needs to have a
                                // length of zero, and then we just bubble up the type as-is
                                if args.is_empty() {
                                    others.clone()
                                } else {
                                    CType::fail(&format!(
                                        "{} is used as a generic type but is not one: {:?}, {:?}",
                                        var, others, prior_value,
                                    ))
                                }
                            }
                        }
                    }
                    None => CType::fail(&format!("{} is not a valid type name", var)),
                })
            }
            parse::TypeBase::GnCall(_) => { /* We always process GnCall in the Variable path */ }
            parse::TypeBase::TypeGroup(g) => {
                if g.typeassignables.is_empty() {
                    // It's a void type!
                    prior_value = Some(CType::Group(Box::new(CType::Void)));
                } else {
                    // Simply wrap the returned type in a `CType::Group`
                    prior_value = Some(CType::Group(Box::new(withtypeoperatorslist_to_ctype(
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
