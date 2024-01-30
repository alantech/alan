use nom::{
  branch::alt,
  bytes::complete::{tag, is_not, take_while},
  combinator::{consumed, opt, recognize},
  error::{Error, ErrorKind},
  multi::{many0, many1, separated_list1},
  sequence::{delimited, tuple},
  IResult,
  Err,
};

/// Macros to make building nom functions nicer (for me). For now they always make everything
/// public for easier usage of this file. Also `#[derive(Debug)]` is added to all structs and enums

/// The `build` macro provides the function wrapper and naming for the function in question
macro_rules! build {
  ( $name:ident, $body:expr $(,)? ) => {
    pub fn $name(input: &str) -> IResult<&str, &str> {
      $body(input)
    }
  };
  ( $name:ident: $ret:ty, $body:expr $(,)? ) => {
    pub fn $name(input: &str) -> IResult<&str, $ret> {
      $body(input)
    }
  }
}

/// The `token` macro matches an exact string
macro_rules! token {
  ( $str:expr ) => {tag($str)}
}

/// The `not` macro matches anything except the string in question
macro_rules! not {
  ( $str:expr ) => {is_not($str)}
}

/// The `one_or_more` macro matches one or more instances of a given rule, returning the whole
/// string matched
macro_rules! one_or_more {
  ( $rule:expr ) => {recognize(many1($rule))}
}

/// The `zero_or_more` macro matches zero or more instances of a given rule, returning a string,
/// even if empty
macro_rules! zero_or_more {
  ( $rule:expr ) => {recognize(many0($rule))}
}

/// The `zero_or_one` macro matches the given rule zero or one time, returning a string,
/// potentially empty
macro_rules! zero_or_one {
  ( $rule:expr ) => {recognize(opt($rule))}
}

/// The `or` macro matches one of the given rules in a tuple, returning the first match
macro_rules! or {
  ( $($rule:expr),+ $(,)? ) => {alt(($($rule,)+))}
}

/// The `and` macro matches all of the given rules in a tuple, returning them all as a single
/// string
macro_rules! and {
  ( $($rule:expr),+ $(,)? ) => {recognize(tuple(($($rule,)+)))}
}

/// The `charset` macro matches as long as the given character ranges are found. Multiple ranges
/// can be concatenated in the same charset with normal `|` syntax. Eg `'_' | 'a'..='z'`
macro_rules! charset {
  ( $ranges:pat ) => {
    take_while(|c| match c {
      $ranges => true,
      _ => false,
    })
  }
}

/// The `named_and` macro matches all of the given rules just like `and`, but it also defines a
/// struct and field names for these matches. For simplicity, the `str`s are copied into `String`s
/// Because defining a struct needs to happen in the top-level of the source file, this macro
/// implicitly `build`s itself and cannot be wrapped like the other macros can.
macro_rules! named_and {
  ( $fn_name:ident: $struct_name:ident => $( $field_name:ident: $field_type:ty as $rule:expr ),+ $(,)? ) => {
    #[derive(Debug)]
    pub struct $struct_name {
      $($field_name: $field_type,)+
    }

    pub fn $fn_name(input: &str) -> IResult<&str, $struct_name> {
      let mut i = input;
      let out = $struct_name {
        $( $field_name: {
          let res = $rule(i)?;
          i = res.0;
          res.1.into()
        },)+
      };
      return Ok((i, out));
    }
  }
}

/// The `named_or` macro is similar to the above, but constructs an enum instead of a struct. It
/// also cannot be used with `build`. This variant is much more useful when you want to know *what*
/// kind of match you're dealing with, and considering Rust has a built-in `match` syntax, I am
/// surprised that I can't find something like this in `nom` already?
macro_rules! named_or {
  ( $fn_name:ident: $enum_name:ident => $( $option_name:ident: $option_type:ty as $rule:expr ),+ $(,)? ) => {
    #[derive(Debug)]
    pub enum $enum_name {
      $($option_name($option_type),)+
    }

    pub fn $fn_name(input: &str) -> IResult<&str, $enum_name> {
      $(if let Ok((i, val)) = $rule(input) {
        return Ok((i, $enum_name::$option_name(val.into())));
      })+
      // Reaching this point is an error. For now, just return a generic one
      Err(Err::Error(Error::new(input, ErrorKind::Fail)))
    }
  }
}

/// `left_subset` matches if the first argument (the left side) matches, but the second argument
/// (the right side) does not, when viewing these operations like a venn diagram. `and` would be
/// the intersection of the two sets, `or` is all three sections of the two sets, `xor` would be
/// the first and third sections, and `left_subset` is just the first section. (A `right_subset`
/// would be redundant since you can just swap the order of the arguments, and the first and second
/// sections just reduces down to checking the left side only so there's no need for such an
/// operation)
macro_rules! left_subset {
  ( $left:expr, $right:expr $(,)? ) => {
    (|input| {
      let left_match = consumed($left)(input)?;
      let right_match = $right(left_match.1.0);
      if let Ok(_) = right_match {
        return Ok((left_match.0, left_match.1.1));
      } else {
        return right_match;
      }
    })
  }
}

/// `list` didn't exist in the prior code, but there's a nice primitive in `nom` that makes this
/// easier (and I can avoid complicating the typing of the other primitives for now)
macro_rules! list {
  // Special path for Strings because str/String split is annoying
  ( $fn_name:ident: String => $rule:expr, $sep:expr $(,)? ) => {
    pub fn $fn_name(input: &str) -> IResult<&str, Vec<String>> {
      let res = separated_list1($sep, $rule)(input)?;
      Ok((res.0, res.1.iter().map(|s| s.to_string()).collect()))
    }
  };
  // Normal path
  ( $fn_name:ident: $type:ty => $rule:expr, $sep:expr $(,)? ) => {
    pub fn $fn_name(input: &str) -> IResult<&str, Vec<$type>> {
      separated_list1($sep, $rule)(input)
    }
  };
  // Path for no separators
  ( $fn_name: ident: $type:ty => $rule:expr $(,)? ) => {
    pub fn $fn_name(input: &str) -> IResult<&str, Vec<$type>> {
      many1($rule)(input)
    }
  }
}

/// Because `opt` returns an `Option<&str>` for &str sources, but you can't easily put that inside
/// of structs and enums, this macro handles that situation
macro_rules! opt_string {
  ( $rule:expr ) => {
    opt(|input| match $rule(input) {
      Ok((i, o)) => Ok((i, o.to_string())),
      Err(e) => Err(e),
    })
  }
}

build!(space, token!(" "));
build!(blank, one_or_more!(space));
build!(optblank, zero_or_one!(blank));
build!(newline, or!(token!("\n"), token!("\r")));
build!(notnewline, and!(not!("\n"), not!("\r")));
build!(singlelinecomment, and!(token!("//"), zero_or_more!(notnewline), newline));
build!(star, token!("*"));
build!(notstar, not!("*"));
build!(notslash, not!("/"));
build!(multilinecomment, and!(
    token!("/*"),
    zero_or_more!(or!(notstar, and!(star, notslash))),
    token!("*/"),
));
build!(whitespace, one_or_more!(or!(space, newline, singlelinecomment, multilinecomment)));
build!(optwhitespace, zero_or_one!(whitespace));
build!(colon, token!(":"));
build!(under, token!("_"));
build!(negate, token!("-"));
build!(dot, token!("."));
build!(par, token!(".."));
build!(eq, token!("="));
build!(openparen, token!("("));
build!(closeparen, token!(")"));
build!(opencurly, token!("{"));
build!(closecurly, token!("}"));
build!(opencaret, token!("<"));
build!(closecaret, token!(">"));
build!(openarr, token!("["));
build!(closearr, token!("]"));
build!(comma, token!(","));
build!(semicolon, token!(";"));
build!(optsemicolon, zero_or_one!(semicolon));
build!(at, token!("@"));
build!(slash, token!("/"));
build!(base10, charset!('0'..='9'));
build!(natural, one_or_more!(base10));
build!(integer, and!(zero_or_one!(negate), natural));
build!(real, and!(integer, zero_or_one!(and!(dot, natural))));
named_or!(num: Number => RealNum: String as real, IntNum: String as integer);
build!(t, token!("true"));
build!(f, token!("false"));
build!(booln, or!(t, f));
build!(lower, charset!('a'..='z'));
build!(upper, charset!('A'..='Z'));
build!(variable, left_subset!(
    and!(one_or_more!(or!(under, lower, upper)), zero_or_more!(or!(under, lower, upper, base10))),
    booln,
));
build!(operators, one_or_more!(or!(
    token!("+"),
    token!("-"),
    token!("/"),
    token!("\\"),
    token!("*"),
    token!("^"),
    token!("."),
    token!("~"),
    token!("`"),
    token!("!"),
    token!("@"),
    token!("#"),
    token!("$"),
    token!("%"),
    token!("&"),
    token!("|"),
    token!(":"),
    token!("<"),
    token!(">"),
    token!("?"),
    token!("="),
)));
build!(interface, token!("interface"));
build!(new, token!("new"));
build!(ifn, token!("if"));
build!(elsen, token!("else"));
build!(precedence, token!("precedence"));
build!(infix, token!("infix"));
build!(prefix, token!("prefix"));
build!(asn, token!("as"));
build!(exit, token!("exit"));
build!(emit, token!("emit"));
build!(letn, token!("let"));
build!(constn, token!("const"));
build!(on, token!("on"));
build!(event, token!("event"));
build!(export, token!("export"));
build!(typen, token!("type"));
build!(import, token!("import"));
build!(from, token!("from"));
build!(fnn, token!("fn"));
build!(quote, token!("'"));
build!(doublequote, token!("\""));
build!(escapequote, token!("\\'"));
build!(escapedoublequote, token!("\\\""));
build!(notquote, not!("'"));
build!(notdoublequote, not!("\""));
build!(sep, and!(optwhitespace, comma, optwhitespace));
build!(optsep, zero_or_one!(sep));
build!(strn, or!(
    and!(quote, zero_or_more!(or!(escapequote, notquote)), quote),
    and!(doublequote, zero_or_more!(or!(escapedoublequote, notdoublequote)), doublequote),
));
build!(arrayaccess: Vec<WithOperators>, delimited(and!(openarr, optwhitespace), assignables, and!(optwhitespace, closearr)));
named_or!(varsegment: VarSegment =>
    Variable: String as variable,
    MethodSep: String as and!(optwhitespace, dot, optwhitespace),
    ArrayAccess: Vec<WithOperators> as arrayaccess,
);
build!(var, one_or_more!(varsegment));
named_or!(varop: VarOp => Variable: String as variable, Operator: String as operators);
named_and!(renamed: Renamed =>
    a: String as blank,
    asn: String as asn,
    b: String as blank,
    varop: VarOp as varop,
);
named_and!(renameablevar: RenameableVar => varop: VarOp as varop, optrenamed: Option<Renamed> as opt(renamed));
list!(varlist: RenameableVar => renameablevar, sep);
list!(depsegments: String => variable, slash);
named_and!(curdir: CurDir =>
    dot: String as dot,
    slash: String as slash,
    depsegments: Vec<String> as depsegments,
);
named_and!(pardir: ParDir =>
    par: String as par,
    slash: String as slash,
    depsegments: Vec<String> as depsegments,
);
named_or!(localdependency: LocalDependency => CurDir: CurDir as curdir, ParDir: ParDir as pardir);
named_and!(globaldependency: GlobalDependency =>
    at: String as at,
    depsegments: Vec<String> as depsegments,
);
named_or!(dependency: Dependency =>
    Local: LocalDependency as localdependency,
    Global: GlobalDependency as globaldependency,
);
named_and!(standardimport: StandardImport =>
    import: String as import,
    a: String as blank,
    dependency: Dependency as dependency,
    renamed: Option<Renamed> as opt(renamed),
    b: String as optblank,
    c: String as newline,
    d: String as optwhitespace,
);
named_and!(fromimport: FromImport =>
    from: String as from,
    a: String as blank,
    dependency: Dependency as dependency,
    b: String as blank,
    import: String as import,
    c: String as blank,
    varlist: Vec<RenameableVar> as varlist,
    d: String as optblank,
    e: String as newline,
    f: String as optwhitespace,
);
named_or!(importstatement: ImportStatement =>
    Standard: StandardImport as standardimport,
    From: FromImport as fromimport,
);
list!(imports: ImportStatement => importstatement);
// Function aliases don't seem to exist in Rust, so just redefining this, it's the same as 'var'
build!(typename, one_or_more!(varsegment));
named_and!(typegenerics: TypeGenerics =>
    a: String as opencaret,
    b: String as optwhitespace,
    generics: Vec<FullTypename> as generics,
    c: String as optwhitespace,
    d: String as closecaret,
);
named_and!(fulltypename: FullTypename =>
    typename: String as typename, // TODO: Maybe we want to keep this in a tree form in the future?
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics)
);
list!(generics: FullTypename => fulltypename, sep);
named_and!(typeline: Typeline =>
    variable: String as variable,
    a: String as optwhitespace,
    colon: String as colon,
    b: String as optwhitespace,
    fulltypename: FullTypename as fulltypename,
);
list!(typelist: Typeline => typeline, sep);
named_and!(typebody: TypeBody =>
    a: String as opencurly,
    b: String as optwhitespace,
    typelist: Vec<Typeline> as typelist,
    c: String as optwhitespace,
    d: String as closecurly,
);
named_and!(typealias: TypeAlias =>
    a: String as eq,
    b: String as blank,
    fulltypename: FullTypename as fulltypename,
);
named_or!(typedef: TypeDef => TypeBody: TypeBody as typebody, TypeAlias: TypeAlias as typealias);
named_and!(types: Types =>
    a: String as typen,
    b: String as blank,
    fulltypename: FullTypename as fulltypename,
    c: String as optwhitespace,
    typedef: TypeDef as typedef,
);
named_or!(constants: Constants =>
    Bool: String as booln,
    Num: Number as num,
    Strn: String as strn,
);
named_or!(baseassignable: BaseAssignable =>
    ObjectLiterals: ObjectLiterals as objectliterals,
    Functions: Functions as functions,
    FnCall: FnCall as fncall,
    Variable: String as variable,
    Constants: Constants as constants,
    MethodSep: String as and!(optwhitespace, dot, optwhitespace),
);
list!(baseassignablelist: BaseAssignable => baseassignable);
named_or!(withoperators: WithOperators =>
    BaseAssignableList: Vec<BaseAssignable> as baseassignablelist,
    Operators: String as and!(optwhitespace, operators, optwhitespace),
);
list!(assignables: WithOperators => withoperators);
named_and!(typedec: TypeDec =>
    colon: String as colon,
    a: String as optwhitespace,
    fulltypename: FullTypename as fulltypename,
    b: String as optwhitespace,
);
named_and!(constdeclaration: ConstDeclaration =>
    constn: String as constn,
    whitespace: String as whitespace,
    variable: String as variable,
    a: String as optwhitespace,
    typedec: TypeDec as typedec,
    eq: String as eq,
    b: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
named_and!(letdeclaration: LetDeclaration =>
    letn: String as letn,
    whitespace: String as whitespace,
    variable: String as variable,
    a: String as optwhitespace,
    typedec: TypeDec as typedec,
    eq: String as eq,
    b: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
named_or!(declarations: Declarations =>
    Const: ConstDeclaration as constdeclaration,
    Let: LetDeclaration as letdeclaration,
);
named_and!(assignments: Assignments =>
    var: String as var,
    a: String as optwhitespace,
    eq: String as eq,
    b: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
named_and!(retval: RetVal =>
    assignables: Vec<WithOperators> as assignables,
    a: String as optwhitespace,
);
build!(optretval: Option<RetVal>, opt(retval));
named_and!(exits: Exits =>
    exit: String as exit,
    a: String as optwhitespace,
    retval: Option<RetVal> as optretval,
    semicolon: String as semicolon,
);
named_and!(emits: Emits =>
    emit: String as emit,
    a: String as optwhitespace,
    eventname: String as var,
    b: String as optwhitespace,
    retval: Option<RetVal> as optretval,
    semicolon: String as semicolon,
);
named_and!(arg: Arg =>
    variable: String as variable,
    a: String as optblank,
    colon: String as colon,
    b: String as optblank,
    fulltypename: FullTypename as fulltypename,
);
list!(arglist: Arg => arg, sep);
named_and!(functionbody: FunctionBody =>
    opencurly: String as opencurly,
    statements: Vec<Statement> as statements,
    a: String as optwhitespace,
    closecurly:String as closecurly,
);
named_and!(assignfunction: AssignFunction =>
    eq: String as eq,
    a: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    b: String as optsemicolon,
);
named_or!(fullfunctionbody: FullFunctionBody =>
    FunctionBody: FunctionBody as functionbody,
    AssignFunction: AssignFunction as assignfunction,
);
named_and!(args: Args =>
    openparen: String as openparen,
    a: String as optwhitespace,
    arglist: Vec<Arg> as arglist,
    b: String as optwhitespace,
    closeparen: String as closeparen,
    c: String as optwhitespace,
);
named_and!(returntype: ReturnType =>
    colon: String as colon,
    a: String as optwhitespace,
    fulltypename: FullTypename as fulltypename,
    b: String as optwhitespace,
);
named_and!(functions: Functions =>
    fnn: String as fnn,
    a: String as optwhitespace,
    optname: Option<String> as opt_string!(variable),
    b: String as optwhitespace,
    optargs: Option<Args> as opt(args),
    optreturntype: Option<ReturnType> as opt(returntype),
    fullfunctionbody: FullFunctionBody as fullfunctionbody,
);
named_or!(blocklike: Blocklike =>
    Functions: Functions as functions,
    FunctionBody: FunctionBody as functionbody,
    FnName: String as var,
);
named_or!(condorblock: CondOrBlock =>
    Conditional: Conditional as conditional,
    Blocklike: Blocklike as blocklike,
);
named_and!(elsebranch: ElseBranch =>
    a: String as whitespace,
    elsen: String as elsen,
    b: String as optwhitespace,
    condorblock: Box<CondOrBlock> as condorblock,
);
named_and!(conditional: Conditional =>
    ifn: String as ifn,
    a: String as whitespace,
    assignables: Vec<WithOperators> as assignables,
    b: String as optwhitespace,
    blocklike: Blocklike as blocklike,
    optelsebranch: Option<ElseBranch> as opt(elsebranch),
);
named_and!(assignablestatement: AssignableStatement =>
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
named_or!(statement: Statement =>
    Declarations: Declarations as declarations,
    Exits: Exits as exits,
    Emits: Emits as emits,
    Assignments: Assignments as assignments,
    Conditional: Conditional as conditional,
    Assignables: AssignableStatement as assignablestatement,
);
list!(statements: Statement => statement, optwhitespace);
named_and!(literaldec: LiteralDec =>
    new: String as new,
    a: String as blank,
    fulltypename: FullTypename as fulltypename,
    b: String as optblank,
);
list!(assignablelist: Vec<WithOperators> => assignables, sep);
named_and!(arraybase: ArrayBase =>
    openarr: String as openarr,
    a: String as optwhitespace,
    assignablelist: Vec<Vec<WithOperators>> as assignablelist,
    b: String as optwhitespace,
    closearr: String as closearr,
);
named_and!(fullarrayliteral: FullArrayLiteral =>
    literaldec: LiteralDec as literaldec,
    arraybase: ArrayBase as arraybase,
);
named_or!(arrayliteral: ArrayLiteral =>
    ArrayBase: ArrayBase as arraybase,
    FullArrayLiteral: FullArrayLiteral as fullarrayliteral,
);
named_and!(typeassign: TypeAssign =>
    variable: String as variable,
    a: String as optwhitespace,
    colon: String as colon,
    b: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    c: String as optwhitespace
);
list!(typeassignlist: TypeAssign => typeassign, sep);
named_and!(typebase: TypeBase =>
    opencurly: String as opencurly,
    a: String as optwhitespace,
    typeassignlist: Vec<TypeAssign> as typeassignlist,
    b: String as optwhitespace,
    closecurly: String as closecurly,
);
named_and!(typeliteral: TypeLiteral =>
    literaldec: LiteralDec as literaldec,
    typebase: TypeBase as typebase,
);
named_or!(objectliterals: ObjectLiterals =>
    ArrayLiteral: ArrayLiteral as arrayliteral,
    TypeLiteral: TypeLiteral as typeliteral,
);
named_and!(fncall: FnCall =>
    openparen: String as openparen,
    a: String as optwhitespace,
    assignablelist: Vec<Vec<WithOperators>> as assignablelist,
    b: String as optwhitespace,
    closeparen: String as closeparen,
);
named_and!(fntoop: FnToOp =>
    fnname: String as variable,
    a: String as blank,
    asn: String as asn,
    b: String as blank,
    operators: String as operators,
);
named_and!(opprecedence: OpPrecedence =>
    precedence: String as precedence,
    blank: String as blank,
    num: String as integer,
);
named_or!(fix: Fix =>
    Prefix: String as prefix,
    Infix: String as infix,
);
named_and!(fnopprecedence: FnOpPrecedence =>
    fntoop: FnToOp as fntoop,
    blank: String as blank,
    opprecedence: OpPrecedence as opprecedence,
);
named_and!(precedencefnop: PrecedenceFnOp =>
    opprecedence: OpPrecedence as opprecedence,
    blank: String as blank,
    fntoop: FnToOp as fntoop,
);
// TODO: Maybe I can drop the enum here and make a unified type
named_or!(opmap: OpMap =>
    FnOpPrecedence: FnOpPrecedence as fnopprecedence,
    PrecedenceFnOp: PrecedenceFnOp as precedencefnop,
);
named_and!(operatormapping: OperatorMapping =>
    fix: Fix as fix,
    blank: String as blank,
    opmap: OpMap as opmap,
);
named_and!(events: Events =>
    event: String as event,
    a: String as whitespace,
    variable: String as variable,
    b: String as optwhitespace,
    colon: String as colon,
    c: String as optwhitespace,
    fulltypename: FullTypename as fulltypename,
);
named_and!(propertytypeline: PropertyTypeline =>
    variable: String as variable,
    a: String as blank,
    colon: String as colon,
    b: String as blank,
    fulltypename: FullTypename as fulltypename,
);
named_and!(leftarg: LeftArg =>
    leftarg: FullTypename as fulltypename,
    whitespace: String as whitespace,
);
named_and!(operatortypeline: OperatorTypeline =>
    optleftarg: Option<LeftArg> as opt(leftarg),
    operators: String as operators,
    whitespace: String as whitespace,
    rightarg: FullTypename as fulltypename,
    a: String as optwhitespace,
    colon: String as colon,
    b: String as optwhitespace,
    fulltypename: FullTypename as fulltypename,
);
list!(argtypelist: FullTypename => fulltypename, sep);
named_and!(functiontype: FunctionType =>
    openparen: String as openparen,
    a: String as optwhitespace,
    argtypelist: Vec<FullTypename> as argtypelist,
    optsep: String as optsep,
    b: String as optwhitespace,
    closeparen: String as closeparen,
    c: String as optwhitespace,
    colon: String as colon,
    d: String as optwhitespace,
    returntype: FullTypename as fulltypename,
);
named_and!(functiontypeline: FunctionTypeline =>
    variable: String as variable,
    a: String as optblank,
    functiontype: FunctionType as functiontype,
);
named_or!(interfaceline: InterfaceLine =>
    FunctionTypeLine: FunctionTypeline as functiontypeline,
    OperatorTypeLine: OperatorTypeline as operatortypeline,
    PropertyTypeLine: PropertyTypeline as propertytypeline,
);
list!(interfacelist: InterfaceLine => interfaceline, sep);
named_and!(interfacebody: InterfaceBody =>
    opencurly: String as opencurly,
    interfacelist: Vec<InterfaceLine> as interfacelist,
    a: String as optwhitespace,
    closecurly: String as closecurly,
);
named_and!(interfacealias: InterfaceAlias =>
    eq: String as eq,
    blank: String as blank,
    variable: String as variable,
);
named_or!(interfacedef: InterfaceDef =>
    InterfaceBody: InterfaceBody as interfacebody,
    InterfaceAlias: InterfaceAlias as interfacealias,
);
named_and!(interfaces: Interfaces =>
    interface: String as interface,
    a: String as optblank,
    variable: String as variable,
    b: String as optblank,
    interfacedef: InterfaceDef as interfacedef,
);
named_or!(exportable: Exportable =>
    Functions: Functions as functions,
    ConstDeclaration: ConstDeclaration as constdeclaration,
    Types: Types as types,
    Intefaces: Interfaces as interfaces,
    OperatorMapping: OperatorMapping as operatormapping,
    Events: Events as events,
    Ref: String as variable,
);
named_and!(exports: Exports =>
    export: String as export,
    a: String as blank,
    exportable: Exportable as exportable,
);
named_or!(handler: Handler =>
    Functions: Functions as functions,
    FunctionBody: FunctionBody as functionbody,
    FnName: String as variable,
);
named_and!(handlers: Handlers =>
    on: String as on,
    a: String as whitespace,
    eventname: String as var,
    b: String as whitespace,
    handler: Handler as handler,
);
named_or!(rootelements: RootElements =>
    Whitespace: String as whitespace,
    Exports: Exports as exports,
    Handlers: Handlers as handlers,
    Functions: Functions as functions,
    Types: Types as types,
    ConstDeclaration: ConstDeclaration as constdeclaration,
    OperatorMapping: OperatorMapping as operatormapping,
    Events: Events as events,
    Interfaces: Interfaces as interfaces,
);
list!(body: RootElements => rootelements);
named_and!(ln: Ln =>
    a: String as optwhitespace,
    imports: Vec<ImportStatement> as imports,
    body: Vec<RootElements> as body,
);