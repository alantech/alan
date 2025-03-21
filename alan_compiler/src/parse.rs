use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::satisfy,
    combinator::{all_consuming, opt, peek, recognize},
    error::{Error, ErrorKind},
    multi::{many0, many1, separated_list0},
    IResult, Parser,
};

// Macros to make building nom functions nicer (for me). For now they always make everything
// public for easier usage of this file. Also `#[derive(Debug)]` is added to all structs and enums

/// The `build` macro provides the function wrapper and naming for the function in question
macro_rules! build {
    ( $name:ident, $body:expr $(,)? ) => {
        pub fn $name(input: &str) -> IResult<&str, &str> {
            $body.parse(input)
        }
    };
    ( $name:ident: $ret:ty, $body:expr $(,)? ) => {
        pub fn $name(input: &str) -> IResult<&str, $ret> {
            $body.parse(input)
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
            Err(_) => take($str.len()).parse(input),
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
  ( $($rule:expr),+ $(,)? ) => {recognize((($($rule,)+)))}
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

/// The `not_charset` macro matches the *inverse* of the charset provided. And can also concatenate
/// multiple characters to avoid with `|`
macro_rules! not_charset {
    ( $ranges:pat ) => {
        recognize(satisfy(|c| match c {
            $ranges => false,
            _ => true,
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
          let res = $rule.parse(i)?;
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
    #[allow(clippy::large_enum_variant)]
    #[derive(Debug, PartialEq, Clone)]
    pub enum $enum_name {
      $($option_name($option_type),)+
    }

    pub fn $fn_name(input: &str) -> IResult<&str, $enum_name> {
      $(if let Ok((i, val)) = $rule.parse(input) {
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
            let left_match = $left.parse(input)?;
            let right_match = $right.parse(left_match.1);
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
            separated_list1($sep, $rule).parse(input)
        }
    };
    // Path for no separators
    ( $fn_name: ident: $type:ty => $rule:expr $(,)? ) => {
        pub fn $fn_name(input: &str) -> IResult<&str, Vec<$type>> {
            many1($rule).parse(input)
        }
    };
    // Normal path where an empty vector is fine
    ( opt $fn_name:ident: $type:ty => $rule:expr, $sep:expr $(,)? ) => {
        pub fn $fn_name(input: &str) -> IResult<&str, Vec<$type>> {
            separated_list0($sep, $rule).parse(input)
        }
    };
    // Path for no separators where an empty vector is fine
    ( opt $fn_name: ident: $type:ty => $rule:expr $(,)? ) => {
        pub fn $fn_name(input: &str) -> IResult<&str, Vec<$type>> {
            many0($rule).parse(input)
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
build!(notnewline, not_charset!('\r' | '\n'));
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
build!(eq, token!("="));
build!(openparen, token!("("));
build!(closeparen, token!(")"));
build!(opencurly, token!("{"));
build!(closecurly, token!("}"));
build!(openarr, token!("["));
build!(closearr, token!("]"));
build!(comma, token!(","));
build!(semicolon, token!(";"));
build!(optsemicolon, zero_or_one!(semicolon));
// Validating charset
build!(base2, or!(charset!('0'..='1'), under));
build!(base8, or!(charset!('0'..='7'), under));
build!(base10, or!(charset!('0'..='9'), under));
test!(base10 =>
    fail "";
    fail "a";
    pass "0" => "", "0";
    pass "00" => "0", "0";
    pass "_" => "", "_";
);
build!(
    base16,
    or!(
        charset!('0'..='9'),
        charset!('a'..='f'),
        charset!('A'..='F'),
        under
    )
);
build!(natural, one_or_more!(base10));
test!(natural =>
    fail "";
    fail "a";
    pass "0" => "", "0";
    pass "00" => "", "00";
    pass "1_000_000" => "", "1_000_000";
);
build!(normalints, and!(zero_or_one!(negate), natural));
build!(
    integer,
    or!(
        and!(token!("0b"), one_or_more!(base2)),
        and!(token!("0o"), one_or_more!(base8)),
        and!(token!("0x"), one_or_more!(base16)),
        normalints, // Needs to be last or it will parse the leading zero as a zero
    )
);
test!(integer =>
    pass "0";
    pass "0b0";
    pass "0b10";
    pass "0b2" => "b2", "0"; // It grabs just the leading zero
    pass "0o34";
    pass "0o777";
    pass "0o800" => "o800", "0";
    pass "0xdeadbeef";
    pass "0xDAD_B0D";
    pass "5";
    pass "-5";
    pass "-0b1" => "b1", "-0";
    pass "-0o3" => "o3", "-0";
    pass "-0xAAA" => "xAAA", "-0";
);
build!(
    real,
    or!(
        and!(
            normalints,
            dot,
            natural,
            opt(and!(or!(token!("e"), token!("E")), normalints))
        ),
        and!(normalints, or!(token!("e"), token!("E")), normalints),
    )
);
test!(real =>
    pass "0.0";
    pass "-1.3";
    pass "-5.2e15";
    pass "1.23E456";
    pass "1e100";
);
// Validating named_or
named_or!(num: Number => RealNum: String as real, IntNum: String as integer);
impl Number {
    #[allow(clippy::inherent_to_string)]
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
    pass "T";
);
build!(
    typeoperators,
    or!(
        and!(
            one_or_more!(or!(
                // You can't have too many args here because nom's type shenanigans explode, so
                // breaking this up into two blocks
                or!(
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
                ),
                or!(
                    token!("#"),
                    token!("$"),
                    token!("%"),
                    token!("&"),
                    token!("|"),
                    token!(":"),
                    token!("?"),
                    token!("["),
                    token!("]"),
                    token!("<"),
                    token!(">"),
                ),
            )),
            zero_or_more!(token!("=")),
        ),
        // Also allow the following whitelisted symbols
        token!("=="),
        token!(","),
    ),
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
build!(ctype, token!("ctype"));
build!(typen, token!("type"));
build!(fnn, token!("fn"));
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
named_and!(fulltypename: FullTypename =>
    typename: String as variable,
    opttypegenerics: Option<GnCall> as opt(gncall)
);
test!(fulltypename =>
    pass "Array{Array{int64}}" => "";
);
impl FullTypename {
    #[allow(clippy::inherent_to_string)]
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
named_and!(typeoperatorswithwhitespace: TypeOperatorsWithWhitespace =>
    a: String as optwhitespace,
    op: String as typeoperators,
    b: String as optwhitespace,
);
named_or!(withtypeoperators: WithTypeOperators =>
    TypeBaseList: Vec<TypeBase> as typebaselist,
    Operators: TypeOperatorsWithWhitespace as typeoperatorswithwhitespace,
);
impl WithTypeOperators {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            WithTypeOperators::TypeBaseList(tbl) => tbl
                .iter()
                .map(|tb| tb.to_string())
                .collect::<Vec<String>>()
                .join("")
                .to_string(),
            WithTypeOperators::Operators(o) => format!(" {} ", o.op),
        }
    }
}
test!(withtypeoperators =>
    pass ":";
);
list!(opt typeassignables: WithTypeOperators => withtypeoperators);
test!(typeassignables =>
    pass "(foo: Foo) -> Bar";
    pass "(i: i8) -> Result{i8}";
    pass "(firstKey: Hashable, firstVal: any) -> HashMap{Hashable, any}";
    pass "() -> void";
);
named_and!(gncall: GnCall =>
    opencurly: String as opencurly,
    a: String as optwhitespace,
    typecalllist: Vec<WithTypeOperators> as typeassignables,
    b: String as optwhitespace,
    closecurly: String as closecurly,
);
impl GnCall {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        format!(
            "{{{}}}",
            self.typecalllist
                .iter()
                .map(|ta| ta.to_string())
                .collect::<Vec<String>>()
                .join("")
        )
        .to_string()
    }
}
test!(gncall =>
    pass "{T}" => "", super::GnCall{
        opencurly: "{".to_string(),
        a: "".to_string(),
        typecalllist: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("T".to_string())])],
        b: "".to_string(),
        closecurly: "}".to_string(),
    };
);
named_and!(typegroup: TypeGroup =>
    openparen: String as openparen,
    a: String as optwhitespace,
    typeassignables: Vec<WithTypeOperators> as typeassignables,
    b: String as optwhitespace,
    closeparen: String as closeparen,
);
test!(typegroup =>
    pass "(T)" => "", super::TypeGroup{
        openparen: "(".to_string(),
        a: "".to_string(),
        typeassignables: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("T".to_string())])],
        b: "".to_string(),
        closeparen: ")".to_string(),
    };
);
named_or!(typebase: TypeBase =>
    GnCall: GnCall as gncall,
    TypeGroup: TypeGroup as typegroup,
    Constants: Constants as constants,
    Variable: String as variable,
);
impl TypeBase {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            TypeBase::GnCall(gc) => gc.to_string(),
            TypeBase::TypeGroup(tg) => format!(
                "({})",
                tg.typeassignables
                    .iter()
                    .map(|ta| ta.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            )
            .to_string(),
            TypeBase::Variable(v) => v.clone(),
            TypeBase::Constants(c) => c.to_string(),
        }
    }
}
test!(typebase =>
    pass "Foo" => "", super::TypeBase::Variable("Foo".to_string());
);
list!(typebaselist: TypeBase => typebase);
test!(typebaselist =>
    pass "Foo{T}" => "", vec![
        super::TypeBase::Variable("Foo".to_string()),
        super::TypeBase::GnCall(super::GnCall{
            opencurly: "{".to_string(),
            a: "".to_string(),
            typecalllist: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("T".to_string())])],
            b: "".to_string(),
            closecurly: "}".to_string(),
        }),
    ];
);
named_and!(typedef: TypeDef =>
    a: Option<String> as opt_string!(eq),
    b: String as optwhitespace,
    typeassignables: Vec<WithTypeOperators> as typeassignables,
);
named_and!(types: Types =>
    typen: String as typen,
    a: String as optwhitespace,
    opttypegenerics: Option<GnCall> as opt(gncall),
    b: String as optwhitespace,
    fulltypename: FullTypename as fulltypename,
    c: String as optwhitespace,
    typedef: TypeDef as typedef,
    optsemicolon: String as optsemicolon,
);
test!(types =>
    pass "type Foo = bar: string;";
    pass "type Foo = Bar";
    pass "type Result{T, Error} = Binds{'Result', T, Error};";
    pass "type ExitCode = Binds{'std::process::ExitCode'};";
    pass "type{Windows} Path = driveLetter: string, pathsegments: Array{string};";
    pass "type add <- 'my_dep' @ '1.0.0';";
);
named_and!(ctypes: CTypes =>
    ctype: String as ctype,
    a: String as blank,
    name: String as variable,
    b: String as optblank,
    opttypegenerics: Option<GnCall> as opt(gncall),
    c: String as optblank,
    optsemicolon: String as optsemicolon,
);
named_or!(constants: Constants =>
    Bool: String as booln,
    Num: Number as num,
    Strn: String as strn,
);
impl Constants {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            Constants::Bool(b) => b.clone(),
            Constants::Num(n) => n.to_string(),
            Constants::Strn(s) => s.clone(),
        }
    }
}
named_or!(baseassignable: BaseAssignable =>
    Functions: Functions as functions,
    FnCall: FnCall as fncall,
    GnCall: GnCall as gncall,
    Array: ArrayBase as arraybase,
    Variable: String as variable,
    MethodSep: String as and!(optwhitespace, dot, optwhitespace),
    Constants: Constants as constants,
);
impl BaseAssignable {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            BaseAssignable::Functions(_f) => "todo".to_string(),
            BaseAssignable::Array(a) => format!(
                "[{}]",
                a.assignablelist
                    .iter()
                    .map(|bal| bal
                        .iter()
                        .map(|ba| ba.to_string())
                        .collect::<Vec<String>>()
                        .join(""))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            BaseAssignable::FnCall(fc) => format!(
                "({})",
                fc.assignablelist
                    .iter()
                    .map(|bal| bal
                        .iter()
                        .map(|ba| ba.to_string())
                        .collect::<Vec<String>>()
                        .join(""))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            BaseAssignable::GnCall(gc) => format!(
                "{{{}}}",
                gc.typecalllist
                    .iter()
                    .map(|ta| ta.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            ),
            BaseAssignable::Variable(v) => v.clone(),
            BaseAssignable::MethodSep(_) => ".".to_string(),
            BaseAssignable::Constants(c) => c.to_string(),
        }
    }
}
test!(baseassignable =>
    pass "Foo" => "", super::BaseAssignable::Variable("Foo".to_string());
);
list!(baseassignablelist: BaseAssignable => baseassignable);
test!(baseassignablelist =>
    pass "Foo()" => "", vec![super::BaseAssignable::Variable("Foo".to_string()), super::BaseAssignable::FnCall(super::FnCall{
        openparen: "(".to_string(),
        a: "".to_string(),
        assignablelist: Vec::new(),
        b: "".to_string(),
        closeparen: ")".to_string(),
    })];
);
named_or!(withoperators: WithOperators =>
    BaseAssignableList: Vec<BaseAssignable> as baseassignablelist,
    Operators: String as and!(optwhitespace, operators, optwhitespace),
);
impl WithOperators {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            WithOperators::BaseAssignableList(bal) => bal
                .iter()
                .map(|ba| ba.to_string())
                .collect::<Vec<String>>()
                .join("")
                .to_string(),
            WithOperators::Operators(o) => o.clone(),
        }
    }
}
test!(withoperators =>
    pass "Foo()" => "", super::WithOperators::BaseAssignableList(vec![super::BaseAssignable::Variable("Foo".to_string()), super::BaseAssignable::FnCall(super::FnCall{
        openparen: "(".to_string(),
        a: "".to_string(),
        assignablelist: Vec::new(),
        b: "".to_string(),
        closeparen: ")".to_string(),
    })]);
    pass "Array{Array{int64}}(Array{int64}()) * lookupLen;" => " * lookupLen;";
);
list!(assignables: WithOperators => withoperators);
test!(assignables =>
    pass "maybe.isSome()";
    pass "InitialReduce{any, anythingElse}(arr, initial)";
    pass "Array{Array{int64}}(Array{int64}()) * lookupLen;" => ";";
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
    opttypegenerics: Option<GnCall> as opt(gncall),
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
    pass "const args = Foo();";
    pass "const args = InitialReduce{any, anythingElse}(arr, initial);";
    pass "const{Test} args = 'test val';";
);
named_and!(letdeclaration: LetDeclaration =>
    letn: String as letn,
    a: String as optblank,
    opttypegenerics: Option<GnCall> as opt(gncall),
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
named_and!(functionbody: FunctionBody =>
    opencurly: String as opencurly,
    a: String as optwhitespace,
    statements: Vec<Statement> as statements,
    b: String as optwhitespace,
    closecurly: String as closecurly,
);
test!(functionbody =>
    pass "{  if maybe.isSome() {\n    return maybe.getMaybe().toString();\n  } else {\n    return 'none';\n  } }";
    pass "{  const args = InitialReduce{any, anythingElse}(arr, initial);\n  return foldl(args, cb);\n}";
    pass "{ // TODO: Rust-like fn::<typeA, typeB> syntax?\n  let hm = HashMap{Hashable, any}(Array{KeyVal{Hashable, any}}(), Array{Array{int64}}(Array{int64}()) * 128 // 1KB of space\n  );\n  return hm.set(firstKey, firstVal);\n}" => "";
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
    DecOnly: String as semicolon,
);
named_and!(functions: Functions =>
    fnn: String as fnn,
    a: String as optwhitespace,
    opttypegenerics: Option<GnCall> as opt(gncall),
    b: String as optwhitespace,
    optname: Option<String> as opt_string!(variable),
    c: String as optwhitespace,
    optgenerics: Option<GnCall> as opt(gncall),
    d: String as optwhitespace,
    opttype: Option<Vec<WithTypeOperators>> as opt(typeassignables),
    e: String as optwhitespace,
    fullfunctionbody: FullFunctionBody as fullfunctionbody,
);
test!(functions =>
    pass "fn foo 'foo' :: () -> ();" => "";
    pass "fn print 'println!' :: string;" => "";
    pass "fn{Test} foo 'foo_test' :: () -> ();" => "";
    pass "fn newHashMap(firstKey: Hashable, firstVal: any) -> HashMap{Hashable, any} { // TODO: Rust-like fn::<typeA, typeB> syntax?\n  let hm = HashMap{Hashable, any}(Array{KeyVal{Hashable, any}}(), Array{Array{int64}}(Array{int64}()) * 128 // 1KB of space\n);\n  return hm.set(firstKey, firstVal);\n}" => "";
    pass "fn cast{T, U}(t: T) -> U = U(t);" => "";
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
// TODO: Can I do this better inside of the normal Assignables path?
named_and!(arrayassignment: ArrayAssignment =>
    name: BaseAssignable as baseassignable,
    a: String as optwhitespace,
    array: ArrayBase as arraybase,
    b: String as optwhitespace,
    eq: String as eq,
    c: String as optwhitespace,
    assignables: Vec<WithOperators> as assignables,
    semicolon: String as semicolon,
);
named_or!(statement: Statement =>
    Declarations: Declarations as declarations,
    Returns: Returns as returns,
    Conditional: Conditional as conditional,
    ArrayAssignment: ArrayAssignment as arrayassignment,
    Assignables: AssignableStatement as assignablestatement,
    A: String as whitespace,
);
test!(statement =>
    pass "return maybe.getMaybe().toString();";
    pass "if maybe.isSome() {\n    return maybe.getMaybe().toString();\n  } else {\n    return 'none';\n  }";
);
list!(opt statements: Statement => statement);
test!(statements =>
    pass "return maybe.getMaybe().toString();";
    pass "if maybe.isSome() {\n    return maybe.getMaybe().toString();\n  } else {\n    return 'none';\n  }";
    pass "let hm = HashMap{Hashable, any}(Array{KeyVal{Hashable, any}}(), Array{Array{int64}}(Array{int64}()) * 128 // 1KB of space\n  );\n  return hm.set(firstKey, firstVal);" => "";
    pass "";
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
named_and!(typefntoop: TypeFnToOp =>
    fnname: String as variable,
    a: String as blank,
    asn: String as asn,
    b: String as blank,
    operator: String as typeoperators,
);
named_and!(typefnopprecedence: TypeFnOpPrecedence =>
    fntoop: TypeFnToOp as typefntoop,
    blank: String as blank,
    opprecedence: OpPrecedence as opprecedence,
);
named_and!(typeprecedencefnop: TypePrecedenceFnOp =>
    opprecedence: OpPrecedence as opprecedence,
    blank: String as blank,
    fntoop: TypeFnToOp as typefntoop,
);
named_or!(typeopmap: TypeOpMap =>
    FnOpPrecedence: TypeFnOpPrecedence as typefnopprecedence,
    PrecedenceFnOp: TypePrecedenceFnOp as typeprecedencefnop,
);
impl TypeOpMap {
    pub fn get_fntoop(&self) -> &TypeFnToOp {
        match self {
            TypeOpMap::FnOpPrecedence(fop) => &fop.fntoop,
            TypeOpMap::PrecedenceFnOp(pfo) => &pfo.fntoop,
        }
    }
    pub fn get_opprecedence(&self) -> &OpPrecedence {
        match self {
            TypeOpMap::FnOpPrecedence(fop) => &fop.opprecedence,
            TypeOpMap::PrecedenceFnOp(pfo) => &pfo.opprecedence,
        }
    }
}
named_and!(operatormapping: OperatorMapping =>
    fix: Fix as fix,
    a: String as optblank,
    opttypegenerics: Option<GnCall> as opt(gncall),
    blank: String as optblank,
    opmap: OpMap as opmap,
    optsemicolon: String as optsemicolon,
);
named_and!(typeoperatormapping: TypeOperatorMapping =>
    typen: String as typen,
    a: String as blank,
    fix: Fix as fix,
    b: String as optblank,
    opttypegenerics: Option<GnCall> as opt(gncall),
    blank: String as optblank,
    opmap: TypeOpMap as typeopmap,
    optsemicolon: String as optsemicolon,
);
named_and!(functiontypeline: FunctionTypeline =>
    variable: String as variable,
    a: String as optblank,
    functiontype: Vec<WithTypeOperators> as typeassignables,
);
test!(functiontypeline =>
    pass "toString(Stringifiable) -> string" => "", super::FunctionTypeline{
      variable: "toString".to_string(),
      a: "".to_string(),
      functiontype: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::TypeGroup(super::TypeGroup {
          openparen: "(".to_string(),
          a: "".to_string(),
          typeassignables: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("Stringifiable".to_string())])],
          b: "".to_string(),
          closeparen: ")".to_string(),
        })]),
        super::WithTypeOperators::Operators(super::TypeOperatorsWithWhitespace {
            a: " ".to_string(),
            op: "->".to_string(),
            b: " ".to_string(),
        }),
        super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("string".to_string())]),
      ]};
);
named_or!(interfaceline: InterfaceLine =>
    FunctionTypeline: FunctionTypeline as functiontypeline,
);
test!(interfaceline =>
    pass "toString(Stringifiable) -> string" => "", super::InterfaceLine::FunctionTypeline(super::FunctionTypeline{
      variable: "toString".to_string(),
      a: "".to_string(),
      functiontype: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::TypeGroup(super::TypeGroup {
          openparen: "(".to_string(),
          a: "".to_string(),
          typeassignables: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("Stringifiable".to_string())])],
          b: "".to_string(),
          closeparen: ")".to_string(),
        })]),
        super::WithTypeOperators::Operators(super::TypeOperatorsWithWhitespace {
            a: " ".to_string(),
            op: "->".to_string(),
            b: " ".to_string(),
        }),
        super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("string".to_string())]),
      ]});
);
list!(opt interfacelist: InterfaceLine => interfaceline, newline);
test!(interfacelist =>
    pass "toString(Stringifiable) -> string" => "", vec![super::InterfaceLine::FunctionTypeline(super::FunctionTypeline{
      variable: "toString".to_string(),
      a: "".to_string(),
      functiontype: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::TypeGroup(super::TypeGroup {
          openparen: "(".to_string(),
          a: "".to_string(),
          typeassignables: vec![super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("Stringifiable".to_string())])],
          b: "".to_string(),
          closeparen: ")".to_string(),
        })]),
        super::WithTypeOperators::Operators(super::TypeOperatorsWithWhitespace {
            a: " ".to_string(),
            op: "->".to_string(),
            b: " ".to_string(),
        }),
        super::WithTypeOperators::TypeBaseList(vec![super::TypeBase::Variable("string".to_string())]),
      ]})];
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
    opttypegenerics: Option<GnCall> as opt(gncall),
    b: String as optblank,
    variable: String as variable,
    c: String as optblank,
    interfacedef: InterfaceDef as interfacedef,
);
test!(interfaces =>
    pass "interface any {}";
    pass "interface anythingElse = any";
    pass "interface Stringifiable {\ntoString(Stringifiable) -> string,\n}";
);
named_or!(exportable: Exportable =>
    OperatorMapping: OperatorMapping as operatormapping,
    TypeOperatorMapping: TypeOperatorMapping as typeoperatormapping,
    Functions: Functions as functions,
    ConstDeclaration: ConstDeclaration as constdeclaration,
    Types: Types as types,
    Intefaces: Interfaces as interfaces,
    Ref: String as variable,
);
named_and!(exports: Exports =>
    export: String as export,
    a: String as optblank,
    opttypegenerics: Option<GnCall> as opt(gncall),
    b: String as optblank,
    exportable: Exportable as exportable,
);
test!(exports =>
    pass "export fn newHashMap(firstKey: Hashable, firstVal: any) -> HashMap{Hashable, any} { // TODO: Rust-like fn::<typeA, typeB> syntax?\n  let hm = HashMap{Hashable, any}(Array{KeyVal{Hashable, any}}(), Array{Array{int64}}(Array{int64}()) * 128 // 1KB of space\n  );\n  return hm.set(firstKey, firstVal);\n}" => "";
    pass "export{Test} fn main() { let foo = 'bar'; // TODO: Add tests\n }" => "";
);
named_or!(rootelements: RootElements =>
    Whitespace: String as whitespace,
    Exports: Exports as exports,
    OperatorMapping: OperatorMapping as operatormapping,
    TypeOperatorMapping: TypeOperatorMapping as typeoperatormapping,
    Functions: Functions as functions,
    Types: Types as types,
    CTypes: CTypes as ctypes,
    ConstDeclaration: ConstDeclaration as constdeclaration,
    Interfaces: Interfaces as interfaces,
);
list!(opt body: RootElements => rootelements);
named_and!(ln: Ln =>
    a: String as optwhitespace,
    body: Vec<RootElements> as body,
);
test!(ln =>
    pass "";
    pass " " => "", super::Ln{
        a: " ".to_string(),
        body: Vec::new(),
    };
    pass "const test = 5;";
);

pub fn get_ast(input: &str) -> Result<Ln, nom::Err<nom::error::Error<&str>>> {
    // We wrap the `ln` root parser in `all_consuming` to cause an error if there's unexpected
    // cruft at the end of the input, which we consider a syntax error at compile time. An LSP
    // would probably use `ln` directly, instead, so new lines/functions/etc the user is currently
    // writing don't trip things up.
    match all_consuming(ln).parse(input) {
        Ok((_, out)) => Ok(out),
        Err(e) => Err(e),
    }
}
test!(get_ast =>
    pass "";
    pass " ";
    pass "export fn main {\n  print('Hello, World!');\n}";
);
