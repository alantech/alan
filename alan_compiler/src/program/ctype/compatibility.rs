use std::sync::Arc;

use super::native_call_args;
use super::CType;
use super::Function;
use super::FnKind;
use super::Microstatement;
use super::Program;
use super::Scope;
use crate::program::ArgKind;
use crate::program::function::{type_to_args, type_to_rettype};
use crate::program::NativeCallKind;

impl CType {
    pub fn accepts(self: Arc<CType>, arg: Arc<CType>) -> bool {
        match (&*self, &*arg) {
            // Type{name, t} wrapper is transparent: unwrap to compare inner type
            (_, CType::Type(_, t)) => self.clone().accepts(t.clone()),
            // Shared{T} is transparent: Shared{T} accepts T, and T accepts Shared{T}
            // Shared{T} can also be used where Mut{T} is expected (provides mutable access)
            (CType::Shared(a), CType::Mut(b)) => a.clone().accepts(b.clone()),
            (CType::Mut(a), CType::Shared(b)) => a.clone().accepts(b.clone()),
            (CType::Shared(a), CType::Shared(b)) => a.clone().accepts(b.clone()),
            (CType::Shared(a), other) => a.clone().accepts(Arc::new(other.clone())),
            (self_type, CType::Shared(b)) => Arc::new(self_type.clone()).accepts(b.clone()),
            // Promise{T} is transparent on the argument side: Promise{T} accepts T, and T accepts Promise{T}
            (CType::Promise(a), CType::Promise(b)) => a.clone().accepts(b.clone()),
            (CType::Promise(a), other) => a.clone().accepts(Arc::new(other.clone())),
            (self_type, CType::Promise(b)) => Arc::new(self_type.clone()).accepts(b.clone()),
            // Mut{T} is transparent for function dispatch: Mut{T} accepts T, and T accepts Mut{T}
            (CType::Mut(a), CType::Mut(b)) => a.clone().accepts(b.clone()),
            (CType::Mut(a), other) => a.clone().accepts(Arc::new(other.clone())),
            (self_type, CType::Mut(b)) => Arc::new(self_type.clone()).accepts(b.clone()),
            (_a, CType::AnyOf(ts)) => {
                for t in ts {
                    if self.clone().accepts(t.clone()) {
                        return true;
                    }
                }
                false
            }
            (CType::Function(i1, o1), CType::Function(i2, o2)) => {
                i1.clone().accepts(i2.clone()) && o1.clone().accepts(o2.clone())
            }
            (CType::Function(i1, _), CType::Generic(_, _, t))
                if matches!(&**t, CType::Function(..)) =>
            {
                if let CType::Function(i2, _) = &**t {
                    // TODO: Do this the right way with `infer_generics`, but I need to refactor a
                    // lot to get the scope into this function. For now, let's just assume if the
                    // lengths of the input tuples are the same, we're fine, and if not, we're not.
                    matches!((&**i1, &**i2), (CType::Tuple(ts1, _), CType::Tuple(ts2, _)) if ts1.len() == ts2.len())
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
                    let program = Program::get_program_guard();
                    let other_scope = program.get_ref().scope_by_file(dep_name).unwrap();
                    match &**n {
                        CType::TString(name) => match other_scope.functions.get(name) {
                            None => CType::fail(&format!("{name} not found in {dep_name}")),
                            Some(dep_fs) => {
                                fs.append(&mut dep_fs.clone());
                            }
                        },
                        _ => CType::fail("The name of the import must be a string"),
                    };
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
                        lazy_body: None,
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
                            // A plain native function/macro call (e.g. `format!`),
                            // serialized as `name(args)`. The name and each argument
                            // are kept structural (one `Value` each, literals trimmed
                            // inline) so the wrapper can be inlined. This is the same
                            // for `Normal` and `External` binds: an `External` wrapper
                            // keeps `FnKind::External`, so it is not inlined and its
                            // dependency is still registered at the emitted call site.
                            microstatements.push(Microstatement::Return {
                                value: Some(Box::new(Microstatement::NativeCall {
                                    typen: rettype.clone(),
                                    kind: NativeCallKind::Function,
                                    name: s.clone(),
                                    args: native_call_args(&args, &mut trimmed_args),
                                })),
                            });
                        }
                        CType::Infix(o) => match &**o {
                            CType::TString(s) => {
                                if args.len() != 2 {
                                    CType::fail("Native infix operators may only be bound with two input arguments");
                                }
                                microstatements.push(Microstatement::Return {
                                    value: Some(Box::new(Microstatement::NativeCall {
                                        typen: rettype.clone(),
                                        kind: NativeCallKind::Infix,
                                        name: s.clone(),
                                        args: native_call_args(&args, &mut trimmed_args),
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
                                    value: Some(Box::new(Microstatement::NativeCall {
                                        typen: rettype.clone(),
                                        kind: NativeCallKind::Prefix,
                                        name: s.clone(),
                                        args: native_call_args(&args, &mut trimmed_args),
                                    })),
                                });
                            }
                            otherwise => CType::fail(&format!(
                                "Unsupported native operator declaration {otherwise:?}"
                            )),
                        },
                        CType::Method(f) => match &**f {
                            CType::TString(s) => {
                                // Keep the receiver and arguments structural (one
                                // `Value` each) so they can be substituted later; the
                                // `recv.name(rest)` serialization lives in each codegen
                                // layer. `args[0]` is the receiver.
                                microstatements.push(Microstatement::Return {
                                    value: Some(Box::new(Microstatement::NativeCall {
                                        typen: rettype.clone(),
                                        kind: NativeCallKind::Method,
                                        name: s.clone(),
                                        args: native_call_args(&args, &mut trimmed_args),
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
                                    microstatements.push(Microstatement::Return {
                                        value: Some(Box::new(Microstatement::NativeCall {
                                            typen: rettype.clone(),
                                            kind: NativeCallKind::Property,
                                            name: s.clone(),
                                            args: native_call_args(&args, &mut trimmed_args),
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
                                    value: Some(Box::new(Microstatement::NativeCall {
                                        typen: rettype.clone(),
                                        kind: NativeCallKind::Cast,
                                        name: s.clone(),
                                        args: native_call_args(&args, &mut trimmed_args),
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
                                Vec::new(),
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
                        lazy_body: None,
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
                        lazy_body: None,
                    }));
                }
            }
            CType::Tuple(ts, parents) => {
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
                                        lazy_body: None,
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
                                        lazy_body: None,
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
                                        lazy_body: None,
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
                                        lazy_body: None,
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
                                lazy_body: None,
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
                                lazy_body: None,
                            }));
                        }
                    }
                }
                // Define the constructor function
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(
                        Arc::new(CType::Tuple(actual_ts.clone(), Vec::new())),
                        t.clone(),
                    )),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                    lazy_body: None,
                }));
                // Generate parent constructor functions for each parent type
                for parent in parents {
                    let parent_unwrapped = parent.clone().degroup();
                    let parent_fields = match &*parent_unwrapped {
                        CType::Tuple(pf, _) => pf.clone(),
                        CType::Either(pf, _) => pf.clone(),
                        _ => continue,
                    };
                    // Build the list of field indices in the parent that match our fields
                    let mut indices = Vec::new();
                    for child_field in &actual_ts {
                        let child_key = child_field.clone().degroup().to_callable_string();
                        for (idx, pf) in parent_fields.iter().enumerate() {
                            if pf.clone().degroup().to_callable_string() == child_key {
                                indices.push(idx);
                                break;
                            }
                        }
                    }
                    // Create the parent constructor: fn TypeName(arg: ParentType) -> TypeName
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple(vec![parent.clone()], Vec::new())),
                            t.clone(),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                        lazy_body: None,
                    }));
                }
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
                            lazy_body: None,
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
                            lazy_body: None,
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
                            lazy_body: None,
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
                            lazy_body: None,
                        }));
                    }
                    _ => {
                        fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(t.clone(), f.clone())),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                            lazy_body: None,
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
                    lazy_body: None,
                }));
            }

            CType::Either(ts, parents) => {
                // There are an equal number of constructor functions and accessor
                // functions, one for each inner type of the sum type.
                for e in ts {
                    // Create a constructor fn
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple(
                                vec![Arc::new(CType::Field("arg0".to_string(), e.clone()))],
                                Vec::new(),
                            )),
                            t.clone(),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                        lazy_body: None,
                    }));
                    // Create a store fn to re-assign-and-auto-wrap a value
                    fs.push(Arc::new(Function {
                        name: "store".to_string(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple(vec![t.clone(), e.clone()], Vec::new())),
                            t.clone(),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                        lazy_body: None,
                    }));
                    if let CType::Void = &**e {
                        // Have a zero-arg constructor function produce the void type, if possible.
                        fs.push(Arc::new(Function {
                            name: constructor_fn_name.clone(),
                            typen: Arc::new(CType::Function(Arc::new(CType::Void), t.clone())),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                            lazy_body: None,
                        }));
                    }
                    // Create the accessor function, the name of the function will
                    // depend on the kind of type this is
                    match &**e {
                        CType::Field(n, i) => fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(
                                t.clone(),
                                Arc::new(CType::Either(
                                    vec![i.clone(), Arc::new(CType::Void)],
                                    Vec::new(),
                                )),
                            )),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                            lazy_body: None,
                        })),
                        CType::Type(n, _) => fs.push(Arc::new(Function {
                            name: n.clone(),
                            typen: Arc::new(CType::Function(
                                t.clone(),
                                Arc::new(CType::Either(
                                    vec![e.clone(), Arc::new(CType::Void)],
                                    Vec::new(),
                                )),
                            )),
                            microstatements: Vec::new(),
                            kind: FnKind::Derived,
                            origin_scope_path: scope.path.clone(),
                            lazy_body: None,
                        })),
                        _ => {} // We can't make names for other types
                    }
                }
                // Generate parent constructor functions for each parent type
                for parent in parents {
                    let parent_unwrapped = parent.clone().degroup();
                    let parent_fields = match &*parent_unwrapped {
                        CType::Tuple(pf, _) => pf.clone(),
                        CType::Either(pf, _) => pf.clone(),
                        _ => continue,
                    };
                    // Build the list of field indices in the parent that match our fields
                    let mut indices = Vec::new();
                    for child_field in ts {
                        let child_key = child_field.clone().degroup().to_callable_string();
                        for (idx, pf) in parent_fields.iter().enumerate() {
                            if pf.clone().degroup().to_callable_string() == child_key {
                                indices.push(idx);
                                break;
                            }
                        }
                    }
                    // Create the parent constructor: fn TypeName(arg: ParentType) -> Maybe{TypeName}
                    // Returns Maybe because the parent may hold the excluded variant
                    let maybe_ret = Arc::new(CType::Either(
                        vec![t.clone(), Arc::new(CType::Void)],
                        Vec::new(),
                    ));
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple(vec![parent.clone()], Vec::new())),
                            maybe_ret,
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                        lazy_body: None,
                    }));
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
                    lazy_body: None,
                }));
                let size = match **s {
                    CType::Int(s) => s as usize,
                    _ => 0, // TODO: Make this function fallible, instead?
                };
                if size > 1 {
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple(
                                {
                                    let mut v = Vec::new();
                                    for _ in 0..size {
                                        v.push(b.clone());
                                    }
                                    v
                                },
                                Vec::new(),
                            )),
                            t.clone(),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                        lazy_body: None,
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
                        lazy_body: None,
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
                    lazy_body: None,
                }));
            }
            CType::Shared(s) => {
                // Shared constructor takes one argument of the inner type and
                // wraps it in Arc<RwLock<T>>
                fs.push(Arc::new(Function {
                    name: constructor_fn_name.clone(),
                    typen: Arc::new(CType::Function(s.clone(), t.clone())),
                    microstatements: Vec::new(),
                    kind: FnKind::Derived,
                    origin_scope_path: scope.path.clone(),
                    lazy_body: None,
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
                    lazy_body: None,
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
                    lazy_body: None,
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
                        lazy_body: None,
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
                    lazy_body: None,
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
                        lazy_body: None,
                    }));
                }
            }
            CType::DerivedVoid(parents) => {
                // Generate parent constructor functions that take parent type and return void
                for parent in parents {
                    let parent_unwrapped = parent.clone().degroup();
                    if !matches!(&*parent_unwrapped, CType::Tuple(..) | CType::Either(..)) {
                        continue;
                    }
                    fs.push(Arc::new(Function {
                        name: constructor_fn_name.clone(),
                        typen: Arc::new(CType::Function(
                            Arc::new(CType::Tuple(vec![parent.clone()], Vec::new())),
                            Arc::new(CType::Void),
                        )),
                        microstatements: Vec::new(),
                        kind: FnKind::Derived,
                        origin_scope_path: scope.path.clone(),
                        lazy_body: None,
                    }));
                }
            }
            _ => {} // Don't do anything for other types
        }
        (CType::clone(&t), fs)
    }
}
