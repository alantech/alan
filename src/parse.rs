use std::path::PathBuf;

use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::satisfy,
    combinator::{all_consuming, opt, peek, recognize},
    error::{Error, ErrorKind},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::tuple,
    IResult,
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
    };
}

/// The `token` macro matches an exact string
macro_rules! token {
    ( $str:expr ) => {
        tag($str)
    };
}

/// The `not` macro matches anything except the string in question for the same length as the
/// string. It behaves *very differently* to nom's `is_not` function, which is more like an
/// inverse `charset`
macro_rules! not {
    ( $str:expr ) => {
        (|input| match tag::<&str, &str, Error<&str>>($str)(input) {
            Ok(_) => Err(nom::Err::Error(Error::new(input, ErrorKind::Fail))),
            Err(_) => take($str.len())(input),
        })
    };
}

/// The `one_or_more` macro matches one or more instances of a given rule, returning the whole
/// string matched
macro_rules! one_or_more {
    ( $rule:expr ) => {
        recognize(many1($rule))
    };
}

/// The `zero_or_more` macro matches zero or more instances of a given rule, returning a string,
/// even if empty
macro_rules! zero_or_more {
    ( $rule:expr ) => {
        recognize(many0($rule))
    };
}

/// The `zero_or_one` macro matches the given rule zero or one time, returning a string,
/// potentially empty
macro_rules! zero_or_one {
    ( $rule:expr ) => {
        recognize(opt($rule))
    };
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

/// The `charset` macro matches a single character in the given character set. Multiple such sets
/// can be concatenated in the same charset with normal `|` syntax. Eg `'_' | 'a'..='z'`
macro_rules! charset {
    ( $ranges:pat ) => {
        recognize(satisfy(|c| match c {
            $ranges => true,
            _ => false,
        }))
    };
}

/// The `named_and` macro matches all of the given rules just like `and`, but it also defines a
/// struct and field names for these matches. For simplicity, the `str`s are copied into `String`s
/// Because defining a struct needs to happen in the top-level of the source file, this macro
/// implicitly `build`s itself and cannot be wrapped like the other macros can.
macro_rules! named_and {
  ( $fn_name:ident: $struct_name:ident => $( $field_name:ident: $field_type:ty as $rule:expr ),+ $(,)? ) => {
    #[derive(Debug, PartialEq, Clone)]
    pub struct $struct_name {
      $(pub $field_name: $field_type,)+
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
    #[derive(Debug, PartialEq, Clone)]
    pub enum $enum_name {
      $($option_name($option_type),)+
    }

    pub fn $fn_name(input: &str) -> IResult<&str, $enum_name> {
      $(if let Ok((i, val)) = $rule(input) {
        return Ok((i, $enum_name::$option_name(val.into())));
      })+
      // Reaching this point is an error. For now, just return a generic one
      Err(nom::Err::Error(Error::new(input, ErrorKind::Fail)))
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
            let left_match = $left(input)?;
            let right_match = $right(left_match.1);
            if let Ok((i, _)) = right_match {
                if i.len() == 0 {
                    return Err(nom::Err::Error(Error::new(input, ErrorKind::Fail)));
                }
            }
            return Ok(left_match);
        })
    };
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
    };
    // Normal path where an empty vector is fine
    ( opt $fn_name:ident: $type:ty => $rule:expr, $sep:expr $(,)? ) => {
        pub fn $fn_name(input: &str) -> IResult<&str, Vec<$type>> {
            separated_list0($sep, $rule)(input)
        }
    };
    // Path for no separators where an empty vector is fine
    ( opt $fn_name: ident: $type:ty => $rule:expr $(,)? ) => {
        pub fn $fn_name(input: &str) -> IResult<&str, Vec<$type>> {
            many0($rule)(input)
        }
    };
}

/// Because `opt` returns an `Option<&str>` for &str sources, but you can't easily put that inside
/// of structs and enums, this macro handles that situation
macro_rules! opt_string {
    ( $rule:expr ) => {
        opt(|input| match $rule(input) {
            Ok((i, o)) => Ok((i, o.to_string())),
            Err(e) => Err(e),
        })
    };
}

/// If I'm gonna have in-file unit tests, let's have them as absolutely close as possible to the
/// relevant functions, yeah? This macro makes a mod and a unit test a single statement. The
/// following macros help make super brief unit tests. Usage:
/// test!(parser_function =>
///   fail "input that causes failure",
///   pass "input that causes success",
///   pass "another successful input" => "the new input it generates", "the output it generates",
/// );
macro_rules! test {
  ( $rule:ident => $( $type:ident $test_val:expr $(=> $i:expr $(, $o:expr)?)? );+ $(;)? ) => {
    #[cfg(test)]
    mod $rule {
      #[test]
      fn $rule() {
        let cmd = |input| super::$rule(input);
        $( $type!(cmd, $test_val $(, $i $(, $o)?)?); )+
      }
    }
  }
}
#[cfg(test)]
macro_rules! pass {
    ( $cmd:ident, $input:expr ) => {
        match $cmd($input) {
            Err(_) => panic!("Did not parse {} correctly!", $input),
            Ok(_) => {}
        }
    };
    ( $cmd:ident, $input:expr, $i:expr, $o:expr) => {
        match $cmd($input) {
            Err(_) => panic!("Did not parse {} correctly!", $input),
            Ok((i, o)) => {
                assert_eq!(i, $i);
                assert_eq!(o, $o);
            }
        }
    };
    ( $cmd:ident, $input:expr, $i:expr) => {
        match $cmd($input) {
            Err(_) => panic!("Did not parse {} correctly!", $input),
            Ok((i, _)) => {
                assert_eq!(i, $i);
            }
        }
    };
}
#[cfg(test)]
macro_rules! fail {
    ( $cmd:ident, $input:expr ) => {
        match $cmd($input) {
            Ok(_) => panic!("Unexpectedly parsed the input! {}", $input),
            Err(_) => {}
        }
    };
}

// Begin defining the nom functions to parse Alan code. This is pretty dense code. If you read it
// top-to-bottom, it mostly follows a leaf-to-root ordering of nodes (but some cycles exist in this
// grammar, so it's not just a simple DAG). You may want to scroll to the bottom of the file and
// start from the `get_ast` function to see how it all comes together conceptually.

build!(space, token!(" "));
// There won't be a test case for *every* token function, just validating they work as expected
test!(space =>
    fail "";
    fail "a";
    pass " " => "", " ";
    pass "  " => " ", " ";
    pass "   ";
);
// Similarly validating one_or_more behaves as expected here
build!(blank, one_or_more!(space));
test!(blank =>
    fail "";
    fail "a";
    pass " " => "", " ";
    pass "  " => "", "  ";
    pass "  a" => "a", "  ";
);
// And validating zero_or_one here
build!(optblank, zero_or_one!(blank));
test!(optblank =>
    pass "" => "", "";
    pass "  " => "", "  ";
    pass "  a" => "a", "  ";
    pass "a" => "a", "";
);
// Validating or
build!(newline, or!(token!("\n"), token!("\r")));
test!(newline =>
    fail "";
    fail " ";
    pass "\n" => "", "\n";
    pass "\r" => "", "\r";
    pass "\r\n" => "\n", "\r";
);
// Validating not
build!(notnewline, not!("\n")); // TODO: Properly support windows newlines here
test!(notnewline =>
    fail "\n";
    pass " " => "", " ";
    pass "   " => "  ", " ";
);
// Validating and
build!(
    singlelinecomment,
    and!(token!("//"), zero_or_more!(notnewline), newline)
);
test!(singlelinecomment =>
    fail "";
    fail "/";
    fail "//";
    pass "//\n";
    pass "// This is a comment\n" => "", "// This is a comment\n";
);
build!(star, token!("*"));
build!(notstar, not!("*"));
build!(notslash, not!("/"));
// Adding a test here just because of the complexity
build!(
    multilinecomment,
    and!(
        token!("/*"),
        zero_or_more!(or!(notstar, and!(star, peek(notslash)))),
        token!("*/"),
    )
);
test!(multilinecomment =>
    fail "";
    pass "/**/" => "", "/**/";
    pass "/*\n This is a basic multi-line comment.\n*/";
    pass "/***\n * This is a multi-line comment.\n */";
    pass "/***\n * This is a multi-line comment with a standard style we now support.\n **/";
);
build!(
    whitespace,
    one_or_more!(or!(space, newline, singlelinecomment, multilinecomment))
);
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
// Validating charset
build!(base10, charset!('0'..='9'));
test!(base10 =>
    fail "";
    fail "a";
    pass "0" => "", "0";
    pass "00" => "0", "0";
);
build!(natural, one_or_more!(base10));
build!(integer, and!(zero_or_one!(negate), natural));
build!(real, and!(integer, dot, natural));
// Validating named_or
named_or!(num: Number => RealNum: String as real, IntNum: String as integer);
impl Number {
    pub fn to_string(&self) -> String {
        match self {
            Number::RealNum(r) => r.clone(),
            Number::IntNum(i) => i.clone(),
        }
    }
}
test!(num =>
    fail "";
    fail "a";
    pass "0" => "", super::Number::IntNum("0".to_string());
    pass "0.5" => "", super::Number::RealNum("0.5".to_string());
    pass "-5" => "", super::Number::IntNum("-5".to_string());
    pass "-5.5" => "", super::Number::RealNum("-5.5".to_string());
);
build!(t, token!("true"));
build!(f, token!("false"));
build!(booln, or!(t, f));
build!(lower, charset!('a'..='z'));
build!(upper, charset!('A'..='Z'));
// Validating left_subset
build!(
    variable,
    left_subset!(
        and!(
            one_or_more!(or!(under, lower, upper)),
            zero_or_more!(or!(under, lower, upper, base10))
        ),
        booln,
    )
);
test!(variable =>
    fail "";
    fail "123abc";
    fail "true";
    fail "false";
    pass "falsetto";
    pass "_123abc";
    pass "variable after_variable" => " after_variable", "variable";
);
build!(
    operators,
    one_or_more!(or!(
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
    ))
);
build!(interface, token!("interface"));
build!(new, token!("new"));
build!(ifn, token!("if"));
build!(elsen, token!("else"));
build!(precedence, token!("precedence"));
build!(infix, token!("infix"));
build!(prefix, token!("prefix"));
build!(postfix, token!("postfix"));
build!(asn, token!("as"));
build!(returnn, token!("return"));
build!(letn, token!("let"));
build!(constn, token!("const"));
build!(export, token!("export"));
build!(typen, token!("type"));
build!(import, token!("import"));
build!(from, token!("from"));
build!(fnn, token!("fn"));
build!(binds, token!("binds"));
build!(quote, token!("'"));
build!(doublequote, token!("\""));
build!(escapequote, token!("\\'"));
build!(escapedoublequote, token!("\\\""));
build!(notquote, not!("'"));
build!(notdoublequote, not!("\""));
// This is used a lot, so let's test it
build!(sep, and!(optwhitespace, comma, optwhitespace));
test!(sep =>
    fail "";
    pass ",";
    pass " ,";
    pass "  ,";
    pass " , ";
    pass "\n,\n";
    pass ",something" => "something", ",";
);
build!(optsep, zero_or_one!(sep));
// Also complex, let's check it
build!(
    strn,
    or!(
        and!(quote, zero_or_more!(or!(escapequote, notquote)), quote),
        and!(
            doublequote,
            zero_or_more!(or!(escapedoublequote, notdoublequote)),
            doublequote
        ),
    )
);
test!(strn =>
    fail "bare text";
    pass "'str'";
    pass "\"str2\"";
    pass "'str\\'3'";
    pass "\"str\\\"4\"";
);
named_or!(varop: VarOp => Variable: String as variable, Operator: String as operators);
impl VarOp {
    pub fn to_string(&self) -> String {
        match self {
            VarOp::Variable(v) => v.clone(),
            VarOp::Operator(o) => o.clone(),
        }
    }
}
// Validating named_and
named_and!(renamed: Renamed =>
    a: String as blank,
    asn: String as asn,
    b: String as blank,
    varop: VarOp as varop,
);
test!(renamed =>
    fail "";
    fail "as";
    fail " as ";
    pass " as foo" => "", super::Renamed{
        a: " ".to_string(),
        asn: "as".to_string(),
        b: " ".to_string(),
        varop: super::VarOp::Variable("foo".to_string())
    };
    pass " as +";
    pass " as foo bar" => " bar", super::Renamed{
        a: " ".to_string(),
        asn: "as".to_string(),
        b: " ".to_string(),
        varop: super::VarOp::Variable("foo".to_string())
    };
);
// Validate optional fields
named_and!(renameablevar: RenameableVar => varop: VarOp as varop, optrenamed: Option<Renamed> as opt(renamed));
test!(renameablevar =>
    fail "";
    pass "foo" => "", super::RenameableVar{
        varop: super::VarOp::Variable("foo".to_string()),
        optrenamed: None,
    };
    pass "foo as bar" => "", super::RenameableVar{
        varop: super::VarOp::Variable("foo".to_string()),
        optrenamed: Some(super::Renamed{
            a: " ".to_string(),
            asn: "as".to_string(),
            b: " ".to_string(),
            varop: super::VarOp::Variable("bar".to_string())
        })
    };
);
// Validating list
list!(varlist: RenameableVar => renameablevar, sep);
test!(varlist =>
    fail "";
    pass "foo" => "", vec![super::RenameableVar{
        varop: super::VarOp::Variable("foo".to_string()),
        optrenamed: None,
    }];
    pass "foo, bar" => "", vec![
        super::RenameableVar{
            varop: super::VarOp::Variable("foo".to_string()),
            optrenamed: None,
        },
        super::RenameableVar{
            varop: super::VarOp::Variable("bar".to_string()),
            optrenamed: None,
        }
    ];
);
named_and!(typegenerics: TypeGenerics =>
    a: String as opencaret,
    b: String as optwhitespace,
    generics: Vec<FullTypename> as generics,
    c: String as optwhitespace,
    d: String as closecaret,
);
impl TypeGenerics {
    pub fn to_string(&self) -> String {
        format!(
            "<{}>",
            self.generics
                .iter()
                .map(|gen| gen.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
named_and!(fulltypename: FullTypename =>
    typename: String as variable, // TODO: Add support for method syntax on type names (for
                                  // imported types)
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics)
);
test!(fulltypename =>
    pass "Array<Array<int64>>" => "";
);
list!(depsegments: String => variable, slash);
named_and!(curdir: CurDir =>
    dot: String as dot,
    slash: String as slash,
    depsegments: Vec<String> as depsegments,
);
impl CurDir {
    fn to_string(&self) -> String {
        format!("./{}", self.depsegments.join("/"))
    }
}
named_and!(pardir: ParDir =>
    par: String as par,
    slash: String as slash,
    depsegments: Vec<String> as depsegments,
);
impl ParDir {
    fn to_string(&self) -> String {
        format!("../{}", self.depsegments.join("/"))
    }
}
named_or!(localdependency: LocalDependency => CurDir: CurDir as curdir, ParDir: ParDir as pardir);
impl LocalDependency {
    fn to_string(&self) -> String {
        match &self {
            LocalDependency::CurDir(c) => c.to_string(),
            LocalDependency::ParDir(p) => p.to_string(),
        }
    }
}
named_and!(globaldependency: GlobalDependency =>
    at: String as at,
    depsegments: Vec<String> as depsegments,
);
impl GlobalDependency {
    fn to_string(&self) -> String {
        format!("@{}", self.depsegments.join("/"))
    }
}
// This one is kinda complex, so let's take a look at it
named_or!(dependency: Dependency =>
    Local: LocalDependency as localdependency,
    Global: GlobalDependency as globaldependency,
);
test!(dependency =>
    fail "";
    fail "foo";
    pass "./foo" => "", super::Dependency::Local(super::LocalDependency::CurDir(super::CurDir{
        dot: ".".to_string(),
        slash: "/".to_string(),
        depsegments: vec!["foo".to_string()],
    }));
    pass "./foo/bar";
    pass "../foo";
    pass "../foo/bar";
    pass "@foo";
    pass "@foo/bar";
);
impl Dependency {
    pub fn resolve(&self, curr_file: String) -> Result<String, Box<dyn std::error::Error>> {
        match &self {
            Dependency::Local(l) => {
                let path = PathBuf::from(curr_file)
                    .parent()
                    .unwrap()
                    .join(l.to_string())
                    .canonicalize()?;
                Ok(path.to_string_lossy().to_string())
            }
            Dependency::Global(g) => {
                if g.depsegments[0] == "std" {
                    // Keep the `@std/...` imports as-is for the Program level to know it should
                    // pull from the embedded strings
                    Ok(g.to_string())
                } else {
                    // For everything else, let's assume it's in the `./dependencies` directory
                    let path = PathBuf::from("./dependencies")
                        .join(g.depsegments.join("/"))
                        .canonicalize()?;
                    Ok(path.to_string_lossy().to_string())
                }
            }
        }
    }
}
named_and!(standardimport: StandardImport =>
    import: String as import,
    a: String as optblank,
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    b: String as optblank,
    dependency: Dependency as dependency,
    renamed: Option<Renamed> as opt(renamed),
    c: String as optblank,
    d: String as newline,
    e: String as optwhitespace,
);
named_and!(fromimport: FromImport =>
    from: String as from,
    a: String as optblank,
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    b: String as optblank,
    dependency: Dependency as dependency,
    c: String as blank,
    import: String as import,
    d: String as blank,
    varlist: Vec<RenameableVar> as varlist,
    e: String as optblank,
    f: String as newline,
    g: String as optwhitespace,
);
named_or!(importstatement: ImportStatement =>
    Standard: StandardImport as standardimport,
    From: FromImport as fromimport,
);
list!(opt imports: ImportStatement => importstatement);
impl FullTypename {
    pub fn to_string(&self) -> String {
        format!(
            "{}{}",
            self.typename,
            match &self.opttypegenerics {
                None => "".to_string(),
                Some(g) => g.to_string(),
            }
        )
        .to_string()
    }
}
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
    c: String as optsep,
    d: String as optwhitespace,
    e: String as closecurly,
);
test!(typebody =>
    pass "{\n  arr: Array<T>,\n  initial: U,\n}";
);
named_and!(typealias: TypeAlias =>
    a: String as eq,
    b: String as blank,
    fulltypename: FullTypename as fulltypename,
);
list!(rustpath: String => variable, token!("::"));
named_and!(typebind: TypeBind =>
    binds: String as binds,
    a: String as blank,
    rustpath: Vec<String> as rustpath,
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics)
);
named_or!(typedef: TypeDef =>
    TypeBody: TypeBody as typebody,
    TypeAlias: TypeAlias as typealias,
    TypeBind: TypeBind as typebind,
);
named_and!(types: Types =>
    typen: String as typen,
    a: String as optblank,
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    b: String as optblank,
    fulltypename: FullTypename as fulltypename,
    c: String as optwhitespace,
    typedef: TypeDef as typedef,
    optsemicolon: String as optsemicolon,
);
test!(types =>
    pass "type Foo {\n  bar: string,\n}";
    pass "type Foo = Bar";
    pass "type Result<T, Error> binds Result<T, Error>";
    pass "type ExitCode binds std::process::ExitCode;";
    pass "type<Windows> Path {\n driveLetter: string, pathsegments: Array<string>, \n}";
);
named_or!(constants: Constants =>
    Bool: String as booln,
    Num: Number as num,
    Strn: String as strn,
);
impl Constants {
    pub fn to_string(&self) -> String {
        match self {
            Constants::Bool(b) => b.clone(),
            Constants::Num(n) => n.to_string(),
            Constants::Strn(s) => s.clone(),
        }
    }
}
named_or!(baseassignable: BaseAssignable =>
    ObjectLiterals: ObjectLiterals as objectliterals,
    Functions: Functions as functions,
    FnCall: FnCall as fncall,
    Variable: String as variable,
    MethodSep: String as and!(optwhitespace, dot, optwhitespace),
    Constants: Constants as constants,
);
impl BaseAssignable {
    pub fn to_string(&self) -> String {
        match self {
            BaseAssignable::ObjectLiterals(_ol) => "todo".to_string(),
            BaseAssignable::Functions(_f) => "todo".to_string(),
            BaseAssignable::FnCall(fc) => format!("({})", fc.assignablelist.iter().map(|bal| bal.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("")).collect::<Vec<String>>().join(", ")).to_string(),
            BaseAssignable::Variable(v) => v.clone(),
            BaseAssignable::MethodSep(_) => ".".to_string(),
            BaseAssignable::Constants(c) => c.to_string(),
        }
    }
}
test!(baseassignable =>
    pass "new Foo{}" => "", super::BaseAssignable::ObjectLiterals(super::ObjectLiterals::TypeLiteral(super::TypeLiteral{
      literaldec: super::LiteralDec{
        new: "new".to_string(),
        a: " ".to_string(),
        fulltypename: super::FullTypename{
          typename: "Foo".to_string(),
          opttypegenerics: None,
        },
        b: "".to_string(),
      },
      typebase: super::TypeBase{
        opencurly: "{".to_string(),
        a: "".to_string(),
        typeassignlist: Vec::new(),
        b: "".to_string(),
        c: "".to_string(),
        closecurly: "}".to_string(),
      }
    }));
);
list!(baseassignablelist: BaseAssignable => baseassignable);
test!(baseassignablelist =>
    pass "new Foo{}" => "", vec![super::BaseAssignable::ObjectLiterals(super::ObjectLiterals::TypeLiteral(super::TypeLiteral{
      literaldec: super::LiteralDec{
        new: "new".to_string(),
        a: " ".to_string(),
        fulltypename: super::FullTypename{
          typename: "Foo".to_string(),
          opttypegenerics: None,
        },
        b: "".to_string(),
      },
      typebase: super::TypeBase{
        opencurly: "{".to_string(),
        a: "".to_string(),
        typeassignlist: Vec::new(),
        b: "".to_string(),
        c: "".to_string(),
        closecurly: "}".to_string(),
      }
    }))];
);
named_or!(withoperators: WithOperators =>
    BaseAssignableList: Vec<BaseAssignable> as baseassignablelist,
    Operators: String as and!(optwhitespace, operators, optwhitespace),
);
impl WithOperators {
    pub fn to_string(&self) -> String {
        match self {
            WithOperators::BaseAssignableList(bal) => bal.iter().map(|ba| ba.to_string()).collect::<Vec<String>>().join("").to_string(),
            WithOperators::Operators(o) => o.clone(),
        }
    }
}
test!(withoperators =>
    pass "new Foo{}" => "", super::WithOperators::BaseAssignableList(vec![super::BaseAssignable::ObjectLiterals(super::ObjectLiterals::TypeLiteral(super::TypeLiteral{
      literaldec: super::LiteralDec{
        new: "new".to_string(),
        a: " ".to_string(),
        fulltypename: super::FullTypename{
          typename: "Foo".to_string(),
          opttypegenerics: None,
        },
        b: "".to_string(),
      },
      typebase: super::TypeBase{
        opencurly: "{".to_string(),
        a: "".to_string(),
        typeassignlist: Vec::new(),
        b: "".to_string(),
        c: "".to_string(),
        closecurly: "}".to_string(),
      }
    }))]);
    pass "new Array<Array<int64>> [ new Array<int64> [], ] * lookupLen;" => " * lookupLen;";
);
list!(assignables: WithOperators => withoperators);
test!(assignables =>
    pass "maybe.isSome()";
    pass "new Foo{}" => "", vec![super::WithOperators::BaseAssignableList(vec![super::BaseAssignable::ObjectLiterals(super::ObjectLiterals::TypeLiteral(super::TypeLiteral{
      literaldec: super::LiteralDec{
        new: "new".to_string(),
        a: " ".to_string(),
        fulltypename: super::FullTypename{
          typename: "Foo".to_string(),
          opttypegenerics: None,
        },
        b: "".to_string(),
      },
      typebase: super::TypeBase{
        opencurly: "{".to_string(),
        a: "".to_string(),
        typeassignlist: Vec::new(),
        b: "".to_string(),
        c: "".to_string(),
        closecurly: "}".to_string(),
      }
    }))])];
    pass "new InitialReduce<any, anythingElse> {\n    arr: arr,\n    initial: initial,\n  }";
    pass "new Array<Array<int64>> [ new Array<int64> [], ] * lookupLen;" => ";";
);
named_and!(typedec: TypeDec =>
    colon: String as colon,
    a: String as optwhitespace,
    fulltypename: FullTypename as fulltypename,
    b: String as optwhitespace,
);
named_and!(constdeclaration: ConstDeclaration =>
    constn: String as constn,
    a: String as optblank,
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    whitespace: String as optwhitespace,
    variable: String as variable,
    b: String as optwhitespace,
    typedec: Option<TypeDec> as opt(typedec),
    c: String as optwhitespace,
    eq: String as eq,
    d: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
test!(constdeclaration =>
    pass "const args = new Foo{};";
    pass "const args = new InitialReduce<any, anythingElse> {\n    arr: arr,\n    initial: initial,\n  };";
    pass "const<Test> args = 'test val';";
);
named_and!(letdeclaration: LetDeclaration =>
    letn: String as letn,
    a: String as optblank,
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    whitespace: String as optwhitespace,
    variable: String as variable,
    b: String as optwhitespace,
    typedec: Option<TypeDec> as opt(typedec),
    eq: String as eq,
    c: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
named_or!(declarations: Declarations =>
    Const: ConstDeclaration as constdeclaration,
    Let: LetDeclaration as letdeclaration,
);
named_and!(assignments: Assignments =>
    var: Vec<BaseAssignable> as baseassignablelist,
    a: String as optwhitespace,
    eq: String as eq,
    b: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
test!(assignments =>
    pass "hm.lookup = new Array<Array<int64>> [ new Array<int64> [], ] * lookupLen;" => "";
);
named_and!(retval: RetVal =>
    assignables: Vec<WithOperators> as assignables,
    a: String as optwhitespace,
);
build!(optretval: Option<RetVal>, opt(retval));
named_and!(returns: Returns =>
    returnn: String as returnn,
    a: String as optwhitespace,
    retval: Option<RetVal> as optretval,
    semicolon: String as semicolon,
);
test!(returns =>
    pass "return maybe.getMaybe().toString();";
);
named_and!(arg: Arg =>
    variable: String as variable,
    a: String as optblank,
    colon: String as colon,
    b: String as optblank,
    fulltypename: FullTypename as fulltypename,
);
list!(opt arglist: Arg => arg, sep);
named_and!(functionbody: FunctionBody =>
    opencurly: String as opencurly,
    a: String as optwhitespace,
    statements: Vec<Statement> as statements,
    b: String as optwhitespace,
    closecurly:String as closecurly,
);
test!(functionbody =>
    pass "{  if maybe.isSome() {\n    return maybe.getMaybe().toString();\n  } else {\n    return 'none';\n  } }";
    pass "{  const args = new InitialReduce<any, anythingElse> {\n    arr: arr,\n    initial: initial,\n  };\n  return foldl(args, cb);\n}";
    pass "{ // TODO: Rust-like fn::<typeA, typeB> syntax?\n  let hm = new HashMap<Hashable, any> {\n    keyVal: new Array<KeyVal<Hashable, any>> [],\n    lookup: new Array<Array<int64>> [ new Array<int64> [] ] * 128, // 1KB of space\n  };\n  return hm.set(firstKey, firstVal);\n}" => "";
);
named_and!(assignfunction: AssignFunction =>
    eq: String as eq,
    a: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    b: String as optsemicolon,
);
named_and!(bindfunction: BindFunction =>
    binds: String as binds,
    a: String as blank,
    rustfunc: String as and!(variable, opt(token!("!"))), // TODO: Support methods for a particular type somehow?
    b: String as optsemicolon,
);
named_or!(fullfunctionbody: FullFunctionBody =>
    FunctionBody: FunctionBody as functionbody,
    AssignFunction: AssignFunction as assignfunction,
    BindFunction: BindFunction as bindfunction,
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
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    b: String as optwhitespace,
    optname: Option<String> as opt_string!(variable),
    c: String as optwhitespace,
    optargs: Option<Args> as opt(args),
    optreturntype: Option<ReturnType> as opt(returntype),
    fullfunctionbody: FullFunctionBody as fullfunctionbody,
);
test!(functions =>
    pass "fn newHashMap(firstKey: Hashable, firstVal: any): HashMap<Hashable, any> { // TODO: Rust-like fn::<typeA, typeB> syntax?\n  let hm = new HashMap<Hashable, any> {\n    keyVal: new Array<KeyVal<Hashable, any>> [],\n    lookup: new Array<Array<int64>> [ new Array<int64> [] ] * 128, // 1KB of space\n  };\n  return hm.set(firstKey, firstVal);\n}" => "";
    pass "fn foo binds foo;" => "";
    pass "fn print(val: String) binds println!;" => "";
    pass "fn<Test> foo binds foo_test;" => "";
);
named_or!(blocklike: Blocklike =>
    Functions: Functions as functions,
    FunctionBody: FunctionBody as functionbody,
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
test!(conditional =>
    pass "if maybe.isSome() {\n    return maybe.getMaybe().toString();\n  } else {\n    return 'none';\n  }";
);
named_and!(assignablestatement: AssignableStatement =>
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
named_or!(statement: Statement =>
    Declarations: Declarations as declarations,
    Returns: Returns as returns,
    Assignments: Assignments as assignments,
    Conditional: Conditional as conditional,
    Assignables: AssignableStatement as assignablestatement,
    A: String as whitespace,
);
test!(statement =>
    pass "return maybe.getMaybe().toString();";
    pass "if maybe.isSome() {\n    return maybe.getMaybe().toString();\n  } else {\n    return 'none';\n  }";
);
list!(statements: Statement => statement);
test!(statements =>
    pass "return maybe.getMaybe().toString();";
    pass "if maybe.isSome() {\n    return maybe.getMaybe().toString();\n  } else {\n    return 'none';\n  }";
    pass "let hm = new HashMap<Hashable, any> {\n    keyVal: new Array<KeyVal<Hashable, any>> [],\n    lookup: new Array<Array<int64>> [ new Array<int64> [] ] * 128, // 1KB of space\n  };\n  return hm.set(firstKey, firstVal);" => "";
);
named_and!(literaldec: LiteralDec =>
    new: String as new,
    a: String as blank,
    fulltypename: FullTypename as fulltypename,
    b: String as optblank,
);
test!(literaldec =>
    pass "new Foo" => "", super::LiteralDec{
      new: "new".to_string(),
      a: " ".to_string(),
      fulltypename: super::FullTypename{
        typename: "Foo".to_string(),
        opttypegenerics: None,
      },
      b: "".to_string(),
    };
);
list!(opt assignablelist: Vec<WithOperators> => assignables, sep);
named_and!(arraybase: ArrayBase =>
    openarr: String as openarr,
    a: String as optwhitespace,
    assignablelist: Vec<Vec<WithOperators>> as assignablelist,
    b: String as optsep,
    c: String as optwhitespace,
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
list!(opt typeassignlist: TypeAssign => typeassign, sep);
named_and!(typebase: TypeBase =>
    opencurly: String as opencurly,
    a: String as optwhitespace,
    typeassignlist: Vec<TypeAssign> as typeassignlist,
    b: String as optsep,
    c: String as optwhitespace,
    closecurly: String as closecurly,
);
named_and!(typeliteral: TypeLiteral =>
    literaldec: LiteralDec as literaldec,
    typebase: TypeBase as typebase,
);
test!(typeliteral =>
    pass "new Foo{}" => "", super::TypeLiteral{
      literaldec: super::LiteralDec{
        new: "new".to_string(),
        a: " ".to_string(),
        fulltypename: super::FullTypename{
          typename: "Foo".to_string(),
          opttypegenerics: None,
        },
        b: "".to_string(),
      },
      typebase: super::TypeBase{
        opencurly: "{".to_string(),
        a: "".to_string(),
        typeassignlist: Vec::new(),
        b: "".to_string(),
        c: "".to_string(),
        closecurly: "}".to_string(),
      }
    };
);
named_or!(objectliterals: ObjectLiterals =>
    ArrayLiteral: ArrayLiteral as arrayliteral,
    TypeLiteral: TypeLiteral as typeliteral,
);
test!(objectliterals =>
    pass "new Foo{}" => "", super::ObjectLiterals::TypeLiteral(super::TypeLiteral{
      literaldec: super::LiteralDec{
        new: "new".to_string(),
        a: " ".to_string(),
        fulltypename: super::FullTypename{
          typename: "Foo".to_string(),
          opttypegenerics: None,
        },
        b: "".to_string(),
      },
      typebase: super::TypeBase{
        opencurly: "{".to_string(),
        a: "".to_string(),
        typeassignlist: Vec::new(),
        b: "".to_string(),
        c: "".to_string(),
        closecurly: "}".to_string(),
      }
    });
    pass "new Array<Array<int64>> [ new Array<int64> [], ]" => "";
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
    operator: String as operators,
);
named_and!(opprecedence: OpPrecedence =>
    precedence: String as precedence,
    blank: String as blank,
    num: String as integer,
);
named_or!(fix: Fix =>
    Prefix: String as prefix,
    Infix: String as infix,
    Postfix: String as postfix,
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
impl OpMap {
    pub fn get_fntoop(&self) -> &FnToOp {
        match self {
            OpMap::FnOpPrecedence(fop) => &fop.fntoop,
            OpMap::PrecedenceFnOp(pfo) => &pfo.fntoop,
        }
    }
    pub fn get_opprecedence(&self) -> &OpPrecedence {
        match self {
            OpMap::FnOpPrecedence(fop) => &fop.opprecedence,
            OpMap::PrecedenceFnOp(pfo) => &pfo.opprecedence,
        }
    }
}
named_and!(operatormapping: OperatorMapping =>
    fix: Fix as fix,
    a: String as optblank,
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    blank: String as optblank,
    opmap: OpMap as opmap,
    optsemicolon: String as optsemicolon,
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
list!(opt argtypelist: FullTypename => fulltypename, sep);
test!(argtypelist =>
    pass "" => "", Vec::new();
    pass "foo" => "", vec![super::FullTypename{
      typename: "foo".to_string(),
      opttypegenerics: None,
    }];
    pass "foo, bar" => "", vec![super::FullTypename{
      typename: "foo".to_string(),
      opttypegenerics: None,
    }, super::FullTypename{
      typename: "bar".to_string(),
      opttypegenerics: None,
    }];
);
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
test!(functiontype =>
    fail "()";
    pass "():void";
    pass "(Foo): Bar";
    pass "(Foo, Bar): Baz";
);
named_and!(functiontypeline: FunctionTypeline =>
    variable: String as variable,
    a: String as optblank,
    functiontype: FunctionType as functiontype,
);
test!(functiontypeline =>
    pass "toString(Stringifiable): string," => ",", super::FunctionTypeline{
      variable: "toString".to_string(),
      a: "".to_string(),
      functiontype: super::FunctionType{
        openparen: "(".to_string(),
        a: "".to_string(),
        argtypelist: vec![super::FullTypename{
          typename: "Stringifiable".to_string(),
          opttypegenerics: None,
        }],
        optsep: "".to_string(),
        b: "".to_string(),
        closeparen: ")".to_string(),
        c: "".to_string(),
        colon: ":".to_string(),
        d: " ".to_string(),
        returntype: super::FullTypename{
          typename: "string".to_string(),
          opttypegenerics: None,
        }
      }
    };
);
named_or!(interfaceline: InterfaceLine =>
    FunctionTypeline: FunctionTypeline as functiontypeline,
    OperatorTypeline: OperatorTypeline as operatortypeline,
    PropertyTypeline: PropertyTypeline as propertytypeline,
);
test!(interfaceline =>
    pass "toString(Stringifiable): string," => ",", super::InterfaceLine::FunctionTypeline(super::FunctionTypeline{
      variable: "toString".to_string(),
      a: "".to_string(),
      functiontype: super::FunctionType{
        openparen: "(".to_string(),
        a: "".to_string(),
        argtypelist: vec![super::FullTypename{
          typename: "Stringifiable".to_string(),
          opttypegenerics: None,
        }],
        optsep: "".to_string(),
        b: "".to_string(),
        closeparen: ")".to_string(),
        c: "".to_string(),
        colon: ":".to_string(),
        d: " ".to_string(),
        returntype: super::FullTypename{
          typename: "string".to_string(),
          opttypegenerics: None,
        }
      }
    });
);
list!(opt interfacelist: InterfaceLine => interfaceline, sep);
test!(interfacelist =>
    pass "toString(Stringifiable): string," => ",", vec![super::InterfaceLine::FunctionTypeline(super::FunctionTypeline{
      variable: "toString".to_string(),
      a: "".to_string(),
      functiontype: super::FunctionType{
        openparen: "(".to_string(),
        a: "".to_string(),
        argtypelist: vec![super::FullTypename{
          typename: "Stringifiable".to_string(),
          opttypegenerics: None,
        }],
        optsep: "".to_string(),
        b: "".to_string(),
        closeparen: ")".to_string(),
        c: "".to_string(),
        colon: ":".to_string(),
        d: " ".to_string(),
        returntype: super::FullTypename{
          typename: "string".to_string(),
          opttypegenerics: None,
        }
      }
    })];
);
named_and!(interfacebody: InterfaceBody =>
    opencurly: String as opencurly,
    a: String as optwhitespace,
    interfacelist: Vec<InterfaceLine> as interfacelist,
    b: String as optsep,
    c: String as optwhitespace,
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
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    b: String as optblank,
    variable: String as variable,
    c: String as optblank,
    interfacedef: InterfaceDef as interfacedef,
);
test!(interfaces =>
    pass "interface any {}";
    pass "interface anythingElse = any";
    pass "interface Stringifiable {\ntoString(Stringifiable): string,\n}";
);
named_or!(exportable: Exportable =>
    Functions: Functions as functions,
    ConstDeclaration: ConstDeclaration as constdeclaration,
    Types: Types as types,
    Intefaces: Interfaces as interfaces,
    OperatorMapping: OperatorMapping as operatormapping,
    Ref: String as variable,
);
named_and!(exports: Exports =>
    export: String as export,
    a: String as optblank,
    opttypegenerics: Option<TypeGenerics> as opt(typegenerics),
    b: String as optblank,
    exportable: Exportable as exportable,
);
test!(exports =>
    pass "export fn newHashMap(firstKey: Hashable, firstVal: any): HashMap<Hashable, any> { // TODO: Rust-like fn::<typeA, typeB> syntax?\n  let hm = new HashMap<Hashable, any> {\n    keyVal: new Array<KeyVal<Hashable, any>> [],\n    lookup: new Array<Array<int64>> [ new Array<int64> [] ] * 128, // 1KB of space\n  };\n  return hm.set(firstKey, firstVal);\n}" => "";
    pass "export<Test> fn main() { let foo = 'bar'; // TODO: Add tests\n }" => "";
);
named_or!(rootelements: RootElements =>
    Whitespace: String as whitespace,
    Exports: Exports as exports,
    Functions: Functions as functions,
    Types: Types as types,
    ConstDeclaration: ConstDeclaration as constdeclaration,
    OperatorMapping: OperatorMapping as operatormapping,
    Interfaces: Interfaces as interfaces,
);
list!(opt body: RootElements => rootelements);
named_and!(ln: Ln =>
    a: String as optwhitespace,
    imports: Vec<ImportStatement> as imports,
    body: Vec<RootElements> as body,
);
test!(ln =>
    pass "";
    pass " " => "", super::Ln{
        a: " ".to_string(),
        imports: Vec::new(),
        body: Vec::new(),
    };
    pass "import ./foo" => "import ./foo", super::Ln{
        a: "".to_string(),
        imports: Vec::new(),
        body: Vec::new(),
    };
    pass "import ./foo\n" => "", super::Ln{
        a: "".to_string(),
        imports: vec![super::ImportStatement::Standard(super::StandardImport{
            import: "import".to_string(),
            a: " ".to_string(),
            opttypegenerics: None,
            b: "".to_string(),
            dependency: super::Dependency::Local(super::LocalDependency::CurDir(super::CurDir{
                dot: ".".to_string(),
                slash: "/".to_string(),
                depsegments: vec!["foo".to_string()],
            })),
            renamed: None,
            c: "".to_string(),
            d: "\n".to_string(),
            e: "".to_string()
        })],
        body: Vec::new(),
    };
);

pub fn get_ast(input: &str) -> Result<Ln, nom::Err<nom::error::Error<&str>>> {
    // We wrap the `ln` root parser in `all_consuming` to cause an error if there's unexpected
    // cruft at the end of the input, which we consider a syntax error at compile time. An LSP
    // would probably use `ln` directly, instead, so new lines/functions/etc the user is currently
    // writing don't trip things up.
    match all_consuming(ln)(input) {
        Ok((_, out)) => Ok(out),
        Err(e) => Err(e),
    }
}
// TODO: Modify the test macro to allow for functions without nom-like signatures to have
// assertions
test!(get_ast =>
    pass "";
    pass " ";
    fail "import ./foo";
    pass "import ./foo\n";
    pass "export fn main {\n  print('Hello, World!');\n}";
);
