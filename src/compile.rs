use std::fs::{remove_file, write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::lntors::lntors;

/// The `compile` function is a very thin wrapper on top of `lntors`, just handling the file
/// loading and temporary file storage and removal on the path to generating the binary.
pub fn compile(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    // Fail if rustc is not present
    Command::new("which").arg("rustc").output()?;
    // Generate the rust code to compile
    let rs_str = lntors(source_file.clone())?;
    // Shove it into a temp file for rustc
    let tmp_file = match PathBuf::from(source_file).file_stem() {
        Some(pb) => format!("{}.rs", pb.to_string_lossy().to_string()),
        None => {
            return Err("Invalid path".into());
        }
    };
    write(&tmp_file, rs_str)?;
    // Build the executable
    Command::new("rustc")
        .arg(&tmp_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    // Drop the temp file
    remove_file(tmp_file)?;
    Ok(())
}

/// The `tors` function is an even thinner wrapper on top of `lntors` that shoves the output into a
/// `.rs` file.
pub fn to_rs(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    // Generate the rust code to compile
    let rs_str = lntors(source_file.clone())?;
    // Shove it into a temp file for rustc
    let out_file = match PathBuf::from(source_file).file_stem() {
        Some(pb) => format!("{}.rs", pb.to_string_lossy().to_string()),
        None => {
            return Err("Invalid path".into());
        }
    };
    write(&out_file, rs_str)?;
    Ok(())
}

/// The majority of this file is dedicated to a comprehensive test suite, converted from the prior
/// test suite using macros to make it a bit more dense than it would have been otherwise.
/// The macro here is composed of three parts: The test program source, the expected exit code
/// (usually 0 so it is optional), and the expected stdout or stderr text (also optional).
/// The syntax to use it is something like:
/// test!(hello_world => r#"
///     on start {
///         print("Hello, World!");
///     }
///     "#;
///     stdout "Hello, World!\n";
///     status 0;
/// );
macro_rules! test {
    ( $rule: ident => $code:expr; $( $type:ident $test_val:expr);+ $(;)? ) => {
        #[cfg(test)]
        mod $rule {
            #[test]
            fn $rule() -> Result<(), Box<dyn std::error::Error>> {
                let filename = format!("{}.ln", stringify!($rule));
                super::write(&filename, $code)?;
                assert_eq!((), super::compile(filename.to_string())?);
                let run = super::Command::new(format!("./{}", stringify!($rule))).output()?;
                $( $type!($test_val, &run); )+
                // Cleanup the temp files. TODO: Make this happen regardless of test failure?
                super::remove_file(&filename)?;
                super::remove_file(stringify!($rule))?;
                Ok(())
            }
        }
    }
}
#[cfg(test)]
macro_rules! stdout {
    ( $test_val:expr, $real_val:expr ) => {
        let std_out = String::from_utf8($real_val.stdout.clone())?;
        assert_eq!($test_val, &std_out);
    };
}
#[cfg(test)]
macro_rules! stderr {
    ( $test_val:expr, $real_val:expr ) => {
        let std_err = String::from_utf8($real_val.stderr.clone())?;
        assert_eq!($test_val, &std_err);
    };
}
#[cfg(test)]
macro_rules! status {
    ( $test_val:expr, $real_val:expr ) => {
        let status = $real_val.status.code().unwrap();
        assert_eq!($test_val, status);
    };
}

// The gold standard test. If you can't do this, are you even a language at all? :P
test!(hello_world => r#"
    export fn main {
        print('Hello, World!');
    }"#;
    stdout "Hello, World!\n";
    status 0;
);
test!(multi_line_hello_world => r#"
    export fn main() {
      print(
"Hello,
World!");
    }"#;
    stdout r#"Hello,
World!
"#;
    status 0;
);

// Event Tests

test!(normal_exit_code => r#"
    export fn main(): ExitCode {
        return ExitCode(0);
    }"#;
    status 0;
);
test!(error_exit_code => r#"
    export fn main(): ExitCode = ExitCode(1);"#;
    status 1;
);
test!(non_global_memory_exit_code => r#"
    export fn main(): ExitCode {
      let x: i64 = 0;
      return ExitCode(x);
    }"#;
    status 0;
);
test!(passing_ints_from_global_memory => r#"
    from @std/app import start, print, exit

    event aNumber: int64;

    on aNumber fn(num: int64) {
      print('I got a number! ' + num.toString());
      emit exit 0;
    }

    on start {
      emit aNumber 5;
    }"#;
    stdout "I got a number! 5\n";
    status 0;
);

// Printing Tests

// This one will replace the hello_world test above once the syntax is updated
test!(print_function => r#"
    export fn main(): ExitCode {
      print('Hello, World');
      return ExitCode(0);
    }"#;
    stdout "Hello, World\n";
    status 0;
);
test!(stdout_event => r#"
    export fn main(): ExitCode {
      emit stdout 'Hello, World';
      wait(10); // Because emits run on another thread, we need to wait to be sure it actually runs
      return ExitCode(0);
    }"#;
    stdout "Hello, World";
);

// Basic Math Tests

test!(int8_add => r#"
    export fn main(): ExitCode = ExitCode(getOrExit(add(i8(1), i8(2))));"#;
    status 3;
);
test!(int8_sub => r#"
    export fn main(): ExitCode = ExitCode(getOrExit(sub(i8(2), i8(1))));"#;
    status 1;
);
test!(int8_mul => r#"
    export fn main(): ExitCode = ExitCode(getOrExit(mul(i8(2), i8(1))));"#;
    status 2;
);
test!(int8_div => r#"
    export fn main(): ExitCode = ExitCode(getOrExit(div(i8(6), i8(2))));"#;
    status 3;
);
test!(int8_mod => r#"
    export fn main(): ExitCode = ExitCode(getOrExit(mod(i8(6), i8(4))));"#;
    status 2;
);
test!(int8_pow => r#"
    export fn main(): ExitCode = ExitCode(getOrExit(pow(i8(6), i8(2))));"#;
    status 36;
);
test!(int8_min => r#"
    export fn main() {
      print(min(i8(3), i8(5)));
    }"#;
    stdout "3\n";
);
test!(int8_max => r#"
    export fn main() {
      print(max(i8(3), i8(5)));
    }"#;
    stdout "5\n";
);

test!(int16_add => r#"
    from @std/app import start, print, exit
    on start {
      print(add(toInt16(1), toInt16(2)));
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int16_sub => r#"
    from @std/app import start, print, exit
    on start {
      print(sub(toInt16(2), toInt16(1)));
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(int16_mul => r#"
    from @std/app import start, print, exit
    on start {
      print(mul(toInt16(2), toInt16(1)));
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int16_div => r#"
    from @std/app import start, print, exit
    on start {
      print(div(toInt16(6), toInt16(2)));
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int16_mod => r#"
    from @std/app import start, print, exit
    on start {
      print(mod(toInt16(6), toInt16(4)));
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int16_pow => r#"
    from @std/app import start, print, exit
    on start {
      print(pow(toInt16(6), toInt16(2)));
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(int16_min => r#"
    from @std/app import start, print, exit
    on start {
      min(3.toInt16(), 5.toInt16()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int16_max => r#"
    from @std/app import start, print, exit
    on start {
      max(3.toInt16(), 5.toInt16()).print();
      emit exit 0;
    }"#;
    stdout "5\n";
);

test!(int32_add => r#"
    from @std/app import start, print, exit
    on start {
      add(1.toInt32(), 2.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int32_sub => r#"
    from @std/app import start, print, exit
    on start {
      sub(2.toInt32(), 1.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(int32_mul => r#"
    from @std/app import start, print, exit
    on start {
      mul(2.toInt32(), 1.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int32_div => r#"
    from @std/app import start, print, exit
    on start {
      div(6.toInt32(), 2.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int32_mod => r#"
    from @std/app import start, print, exit
    on start {
      mod(6.toInt32(), 4.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int32_pow => r#"
    from @std/app import start, print, exit
    on start {
      pow(6.toInt32(), 2.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(int32_min => r#"
    from @std/app import start, print, exit
    on start {
      min(3.toInt32(), 5.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int32_max => r#"
    from @std/app import start, print, exit
    on start {
      max(3.toInt32(), 5.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "5\n";
);

test!(int64_add => r#"
    from @std/app import start, print, exit
    on start {
      print(1 + 2);
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int64_sub => r#"
    from @std/app import start, print, exit
    on start {
      print(2 - 1);
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(int64_mul => r#"
    from @std/app import start, print, exit
    on start {
      print(2 * 1);
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int64_div => r#"
    from @std/app import start, print, exit
    on start {
      print(6 / 2);
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int64_mod => r#"
    from @std/app import start, print, exit
    on start {
      print(6 % 4);
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int64_pow => r#"
    from @std/app import start, print, exit
    on start {
      print(6 ** 2);
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(int64_min => r#"
    from @std/app import start, print, exit
    on start {
      min(3, 5).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int64_max => r#"
    from @std/app import start, print, exit
    on start {
      max(3.toInt64(), 5.toInt64()).print();
      emit exit 0;
    }"#;
    stdout "5\n";
);

test!(float32_add => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(1) + toFloat32(2));
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float32_sub => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(2) - toFloat32(1));
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(float32_mul => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(2) * toFloat32(1));
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(float32_div => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(6) / toFloat32(2));
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float32_sqrt => r#"
    from @std/app import start, print, exit
    on start {
      print(sqrt(toFloat32(36)));
      emit exit 0;
    }"#;
    stdout "6\n";
);
test!(float32_pow => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(6) ** toFloat32(2));
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(float32_min => r#"
    from @std/app import start, print, exit
    on start {
      min(3.toFloat32(), 5.toFloat32()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float32_max => r#"
    from @std/app import start, print, exit
    on start {
      max(3.toFloat32(), 5.toFloat32()).print();
      emit exit 0;
    }"#;
    stdout "5\n";
);

test!(float64_add => r#"
    from @std/app import start, print, exit
    on start {
      (1.0 + 2.0).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float64_sub => r#"
    from @std/app import start, print, exit
    on start {
      (2.0 - 1.0).print();
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(float64_mul => r#"
    from @std/app import start, print, exit
    on start {
      (2.0 * 1.0).print();
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(float64_div => r#"
    from @std/app import start, print, exit
    on start {
      (6.0 / 2.0).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float64_sqrt => r#"
    from @std/app import start, print, exit
    on start {
      sqrt(36.0).print();
      emit exit 0;
    }"#;
    stdout "6\n";
);
test!(float64_pow => r#"
    from @std/app import start, print, exit
    on start {
      (6.0 ** 2.0).print();
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(float64_min => r#"
    from @std/app import start, print, exit
    on start {
      min(3.toFloat64(), 5.toFloat64()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float64_max => r#"
    from @std/app import start, print, exit
    on start {
      max(3.toFloat64(), 5.toFloat64()).print();
      emit exit 0;
    }"#;
    stdout "5\n";
);

test!(grouping => r#"
    from @std/app import start, print, exit
    on start {
      print(2 / (3));
      print(3 / (1 + 2));
      emit exit 0;
    }"#;
    stdout "0\n1\n";
);

test!(string_min => r#"
    from @std/app import start, print, exit
    on start {
      min(3.toString(), 5.toString()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(string_max => r#"
    from @std/app import start, print, exit
    on start {
      max(3.toString(), 5.toString()).print();
      emit exit 0;
    }"#;
    stdout "5\n";
);

// Bitwise Math

test!(int8_bitwise => r#"
    from @std/app import start, print, exit

    prefix toInt8 as ~ precedence 10

    on start {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
      emit exit 0;
    }"#;
    stdout "0\n3\n6\n-1\n-1\n-4\n-7\n";
);
test!(int16_bitwise => r#"
    from @std/app import start, print, exit

    prefix toInt16 as ~ precedence 10

    on start {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
      emit exit 0;
    }"#;
    stdout "0\n3\n6\n-1\n-1\n-4\n-7\n";
);
test!(int32_bitwise => r#"
    from @std/app import start, print, exit

    prefix toInt32 as ~ precedence 10

    on start {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
      emit exit 0;
    }"#;
    stdout "0\n3\n6\n-1\n-1\n-4\n-7\n";
);
test!(int64_bitwise => r#"
    from @std/app import start, print, exit

    on start {
      print(1 & 2);
      print(1 | 3);
      print(5 ^ 3);
      print(!0);
      print(1 !& 2);
      print(1 !| 2);
      print(5 !^ 3);
      emit exit 0;
    }"#;
    stdout "0\n3\n6\n-1\n-1\n-4\n-7\n";
);

// Boolean Logic

test!(boolean_logic => r#"
    from @std/app import start, print, exit

    on start {
      print(true);
      print(false);
      print(toBool(1));
      print(toBool(0));
      print(toBool(15));
      print(toBool(-1));
      print(toBool(0.0));
      print(toBool(1.2));
      print(toBool(''));
      print(toBool('hi'));

      print(true && true);
      print(and(true, false));
      print(false & true);
      print(false.and(false));

      print(true || true);
      print(or(true, false));
      print(false | true);
      print(false.or(false));

      print(true ^ true);
      print(xor(true, false));
      print(false ^ true);
      print(false.xor(false));

      print(!true);
      print(not(false));

      print(true !& true);
      print(nand(true, false));
      print(false !& true);
      false.nand(false).print();

      print(true !| true);
      print(nor(true, false));
      print(false !| true);
      false.nor(false).print();

      print(true !^ true);
      print(xnor(true, false));
      print(false !^ true);
      false.xnor(false).print();

      emit exit 0;
    }"#;
    stdout r#"true
false
true
false
true
true
false
true
false
false
true
false
false
false
true
true
true
false
false
true
true
false
false
true
false
true
true
true
false
false
false
true
true
false
false
true
"#;
);

// String Manipulation

test!(string_ops => r#"
    from @std/app import start, print, exit

    on start {
      concat('Hello, ', 'World!').print();
      print('Hello, ' + 'World!');

      repeat('hi ', 5).print();
      print('hi ' * 5);

      matches('foobar', 'fo.*').print();
      print('foobar' ~ 'fo.*');

      index('foobar', 'ba').print();
      print('foobar' @ 'ba');

      length('foobar').print();
      print(#'foobar');

      trim('   hi   ').print();
      print(\`'   hi   ');

      split('Hello, World!', ', ')[0].print();
      print(('Hello, World!' / ', ')[1]);

      const res = split('Hello, World!', ', ');
      res[0].print();

      const res2 = 'Hello, World!' / ', ';
      print(res2[1]);

      emit exit 0;
    }"#;
    stdout r#"Hello, World!
Hello, World!
hi hi hi hi hi 
hi hi hi hi hi 
true
true
3
3
6
6
hi
hi
Hello
World!
Hello
World!
"#;
);
test!(string_global_local_equality => r#"
    from @std/app import start, print, exit

    on start {
      const foo = 'foo';
      print(foo.trim() == foo);
      emit exit 0;
    }"#;
    stdout "true\n";
);
test!(string_char_array => r#"
    from @std/app import start, print, exit

    on start {
      const fooCharArray = 'foo'.toCharArray();
      print(#fooCharArray);
      print(fooCharArray[0]);
      print(fooCharArray[1]);
      print(fooCharArray[2]);

      emit exit 0;
    }"#;
    stdout r#"3
f
o
o
"#;
);
/* Pending
test!(string_templating => r#"
    from @std/app import start, print, exit

    on start {
      template('\${greet}, \${name}!', new Map<string, string> {
        'greet': 'Hello'
        'name': 'World'
      }).print()
      print('\${greet}, \${name}!' % new Map<string, string> {
        'greet': 'Good-bye'
        'name': 'World'
      })

      emit exit 0
    }"#;
    stdout "Hello, World!\nGood-bye, World!\n";
);
*/

// Comparators

test!(equality => r#"
    from @std/app import start, print, exit

    on start {
      print(toInt8(0) == toInt8(0));
      print(toInt8(1).eq(toInt8(0)));

      print(toInt16(0) == toInt16(0));
      print(toInt16(1).eq(toInt16(0)));

      print(toInt32(0) == toInt32(0));
      print(toInt32(1).eq(toInt32(0)));

      print(0 == 0);
      print(1.eq(0));

      print(toFloat32(0.0) == toFloat32(0.0));
      print(toFloat32(1.2).eq(toFloat32(0.0)));

      print(0.0 == 0.0);
      print(1.2.eq(0.0));

      print(true == true);
      print(true.eq(false));

      print('hello' == 'hello');
      print('hello'.eq('world'));

      emit exit 0;
    }"#;
    stdout r#"true
false
true
false
true
false
true
false
true
false
true
false
true
false
true
false
"#;
);
test!(not_equals => r#"
    from @std/app import start, print, exit

    on start {
      print(toInt8(0) != toInt8(0));
      print(toInt8(1).neq(toInt8(0)));

      print(toInt16(0) != toInt16(0));
      print(toInt16(1).neq(toInt16(0)));

      print(toInt32(0) != toInt32(0));
      print(toInt32(1).neq(toInt32(0)));

      print(0 != 0);
      print(1.neq(0));

      print(toFloat32(0.0) != toFloat32(0.0));
      print(toFloat32(1.2).neq(toFloat32(0.0)));

      print(0.0 != 0.0);
      print(1.2.neq(0.0));

      print(true != true);
      print(true.neq(false));

      print('hello' != 'hello');
      print('hello'.neq('world'));

      emit exit 0;
    }"#;
    stdout r#"false
true
false
true
false
true
false
true
false
true
false
true
false
true
false
true
"#;
);
test!(less_than => r#"
    from @std/app import start, print, exit

    on start {
      print(toInt8(0) < toInt8(1));
      print(toInt8(1).lt(toInt8(0)));

      print(toInt16(0) < toInt16(1));
      print(toInt16(1).lt(toInt16(0)));

      print(toInt32(0) < toInt32(1));
      print(toInt32(1).lt(toInt32(0)));

      print(0 < 1);
      print(1.lt(0));

      print(toFloat32(0.0) < toFloat32(1.0));
      print(toFloat32(1.2).lt(toFloat32(0.0)));

      print(0.0 < 1.0);
      print(1.2.lt(0.0));

      print('hello' < 'hello');
      print('hello'.lt('world'));

      emit exit 0;
    }"#;
    stdout r#"true
false
true
false
true
false
true
false
true
false
true
false
false
true
"#;
);
test!(less_than_or_equal => r#"
    from @std/app import start, print, exit

    on start {
      print(toInt8(0) <= toInt8(1));
      print(toInt8(1).lte(toInt8(0)));

      print(toInt16(0) <= toInt16(1));
      print(toInt16(1).lte(toInt16(0)));

      print(toInt32(0) <= toInt32(1));
      print(toInt32(1).lte(toInt32(0)));

      print(0 <= 1);
      print(1.lte(0));

      print(toFloat32(0.0) <= toFloat32(1.0));
      print(toFloat32(1.2).lte(toFloat32(0.0)));

      print(0.0 <= 1.0);
      print(1.2.lte(0.0));

      print('hello' <= 'hello');
      print('hello'.lte('world'));

      emit exit 0;
    }"#;
    stdout r#"true
false
true
false
true
false
true
false
true
false
true
false
true
true
"#;
);
test!(greater_than => r#"
    from @std/app import start, print, exit

    on start {
      print(toInt8(0) > toInt8(1));
      print(toInt8(1).gt(toInt8(0)));

      print(toInt16(0) > toInt16(1));
      print(toInt16(1).gt(toInt16(0)));

      print(toInt32(0) > toInt32(1));
      print(toInt32(1).gt(toInt32(0)));

      print(0 > 1);
      print(1.gt(0));

      print(toFloat32(0.0) > toFloat32(1.0));
      print(toFloat32(1.2).gt(toFloat32(0.0)));

      print(0.0 > 1.0);
      print(1.2.gt(0.0));

      print('hello' > 'hello');
      print('hello'.gt('world'));

      emit exit 0;
    }"#;
    stdout r#"false
true
false
true
false
true
false
true
false
true
false
true
false
false
"#;
);
test!(greater_than_or_equal => r#"
    from @std/app import start, print, exit

    on start {
      print(toInt8(0) >= toInt8(1));
      print(toInt8(1).gte(toInt8(0)));

      print(toInt16(0) >= toInt16(1));
      print(toInt16(1).gte(toInt16(0)));

      print(toInt32(0) >= toInt32(1));
      print(toInt32(1).gte(toInt32(0)));

      print(0 >= 1);
      print(1.gte(0));

      print(toFloat32(0.0) >= toFloat32(1.0));
      print(toFloat32(1.2).gte(toFloat32(0.0)));

      print(0.0 >= 1.0);
      print(1.2.gte(0.0));

      print('hello' >= 'hello');
      print('hello'.gte('world'));

      emit exit 0;
    }"#;
    stdout r#"false
true
false
true
false
true
false
true
false
true
false
true
true
false
"#;
);
test!(type_coercion_aliases => r#"
    from @std/app import start, print, exit

    on start {
      print(toInt(0) == toInt64(0));
      print(toFloat(0.0) == toFloat(0.0));

      emit exit 0;
    }"#;
    stdout "true\ntrue\n";
);

// Functions and Custom Operators

test!(functions_and_custom_operators => r#"
    from @std/app import start, print, exit

    fn foo() {
      print('foo');
    }

    fn bar(str: string, a: int64, b: int64): string {
      return str * a + b.toString();
    }

    fn baz(pre: string, body: string): void {
      print(pre + bar(body, 1, 2));
    }

    // 'int' is an alias for 'int64'
    fn double(a: int) = a * 2;

    prefix double as ## precedence 10

    /**
     * It should be possible to write 'doublesum' as:
     *
     * fn doublesum(a: int64, b: int64) = ##a + ##b
     *
     * but the function definitions are all parsed before the first operator mapping is done.
     */
    fn doublesum(a: int64, b: int64) = a.double() + b.double();

    infix doublesum as #+# precedence 11

    on start fn (): void {
      foo();
      'to bar'.bar(2, 3).print();
      '>> '.baz('text here');
      4.double().print();
      print(##3);
      4.doublesum(1).print();
      print(2 #+# 3);
      emit exit 0;
    }"#;
    stdout r#"foo
to barto bar3
>> text here2
8
6
10
10
"#;
);

// Conditionals

test!(basic_conditionals => r#"
    from @std/app import start, print, exit

    fn bar() {
      print('bar!');
    }

    fn baz() {
      print('baz!');
    }

    on start {
      if 1 == 0 {
        print('What!?');
      } else {
        print('Math is sane...');
      }

      if 1 == 0 {
        print('Not this again...');
      } else if 1 == 2 {
        print('Still wrong...');
      } else {
        print('Math is still sane, for now...');
      }

      const foo: bool = true == true;
      if foo bar else baz

      const isTrue = true == true;
      cond(isTrue, fn {
        print(\"It's true!\");
      });
      cond(!isTrue, fn {
        print('This should not have run');
      });

      emit exit 0;
    }"#;
    stdout r#"Math is sane...
Math is still sane, for now...
bar!
It's true!
"#;
);
test!(nested_conditionals => r#"
    from @std/app import start, print, exit

    on start {
      if true {
        print(1);
        if 1 == 2 {
          print('What?');
        } else {
          print(2);
          if 2 == 1 {
            print('Uhh...');
          } else if 2 == 2 {
            print(3);
          } else {
            print('Nope');
          }
        }
      } else {
        print('Hmm');
      }
      emit exit 0;
    }"#;
    stdout "1\n2\n3\n";
);
test!(early_return => r#"
    from @std/app import start, print, exit

    fn nearOrFar(distance: float64): string {
      if distance < 5.0 {
        return 'Near!';
      } else {
        return 'Far!';
      }
    }

    on start {
      print(nearOrFar(3.14));
      print(nearOrFar(6.28));

      emit exit 0;
    }"#;
    stdout "Near!\nFar!\n";
);
/* Dropping the ternary operators since either they behave consistently with other operators and
 * are therefore unexpected for end users, or they are inconsistent and a whole lot of pain is
 * needed to support them. */
test!(conditional_let_assignment => r#"
    from @std/app import start, print, exit

    on start {
      let a = 0;
      let b = 1;
      let c = 2;

      if true {
        a = b;
      } else {
        a = c;
      }
      print(a);
      emit exit 0;
    }"#;
    stdout "1\n";
);

// Object Literals

test!(object_literal_compiler_checks => r#"
    from @std/app import start, print, exit

    type Foo {
      bar: string,
      baz: bool,
    }

    on start {
      const foo = new Foo {
        bay: 1.23,
      };
      emit exit 0;
    }"#;
    stderr r#"Foo object literal improperly defined
Missing fields: bar, baz
Extra fields: bay
new Foo {
            bay: 1.23,
          } on line 2:24
"#;
);
test!(array_literals => r#"
    from @std/app import start, print, exit

    on start {
      const test3 = new Array<int64> [ 1, 2, 4, 8, 16, 32, 64 ];
      print(test3[0]);
      print(test3[1]);
      print(test3[2]);

      emit exit 0;
    }"#;
    stdout "1\n2\n4\n";
);
test!(object_literals => r#"
    from @std/app import start, print, exit

    type MyType {
      foo: string,
      bar: bool,
    }

    on start {
      const test = new MyType {
        foo: 'foo!',
        bar: true,
      };
      print(test.foo);
      print(test.bar);

      emit exit 0;
    }"#;
    stdout "foo!\ntrue\n";
);
test!(object_and_array_reassignment => r#"
    from @std/app import start, print, exit

    type Foo {
      bar: bool
    }

    on start {
      let test = new Array<int64> [ 1, 2, 3 ];
      print(test[0]);
      test.set(0, 0);
      print(test[0]);

      let test2 = new Array<Foo> [
        new Foo {
          bar: true
        },
        new Foo {
          bar: false
        }
      ];
      let test3 = test2[0] || new Foo {
        bar: false
      };
      print(test3.bar);
      test3.bar = false;
      test2.set(0, test3); // TODO: is the a better way to do nested updates?
      const test4 = test2[0] || new Foo {
        bar: true
      };
      print(test4.bar);

      emit exit 0;
    }"#;
    stdout "1\n0\ntrue\nfalse\n";
);
/* Pending
test!(map_support => r#"
    from @std/app import start, print, exit

    on start {
      const test5 = new Map<bool, int64> {
        true: 1
        false: 0
      }

      print(test5[true])
      print(test5[false])

      let test6 = new Map<string, string> {
        'foo': 'bar'
      }
      test6['foo'] = 'baz'
      print(test6['foo'])

      emit exit 0
    }"#;
    stdout "1\n0\nbaz\n";
);
*/

// Arrays

test!(array_accessor_and_length => r#"
    from @std/app import start, print, exit

    on start {
      print('Testing...');
      const test = '1,2,3'.split(',');
      print(test.length());
      print(test[0]);
      print(test[1]);
      print(test[2]);
      emit exit 0;
    }"#;
    stdout r#"Testing...
3
1
2
3
"#;
);

test!(array_literal_syntax => r#"
    from @std/app import start, print, exit

    on start {
      print('Testing...');
      const test = new Array<int64> [ 1, 2, 3 ];
      print(test[0]);
      print(test[1]);
      print(test[2]);
      const test2 = [ 4, 5, 6 ];
      print(test2[0]);
      print(test2[1]);
      print(test2[2]);
      emit exit 0;
    }"#;
    stdout r#"Testing...
1
2
3
4
5
6
"#;
);
test!(array_mutable_push_pop => r#"
    from @std/app import start, print, exit

    on start {
      print('Testing...');
      let test = new Array<int64> [];
      test.push(1);
      test.push(2);
      test.push(3);
      print(test[0]);
      print(test[1]);
      print(test[2]);
      print(test.pop());
      print(test.pop());
      print(test.pop());
      print(test.pop()); // Should print error message
      emit exit 0;
    }"#;
    stdout r#"Testing...
1
2
3
3
2
1
cannot pop empty array
"#;
);
test!(array_length_index_has_join => r#"
    from @std/app import start, print, exit

    on start {
      const test = new Array<int64> [ 1, 1, 2, 3, 5, 8 ];
      const test2 = new Array<string> [ 'Hello', 'World!' ];
      print('has test');
      print(test.has(3));
      print(test.has(4));

      print('length test');
      test.length().print();
      print(#test);

      print('index test');
      test.index(5).print();
      print(test2 @ 'Hello');

      print('join test');
      test2.join(', ').print();

      emit exit 0;
    }"#;
    stdout r#"has test
true
false
length test
6
6
index test
4
0
join test
Hello, World!
"#;
);
/* Without the ternary syntax, there is no ternary abuse possible. But a syntax kinda like this
 * doesn't seem so bad? (Eg `1:2:3` produces an array of [1, 2, 3]. It almost feels like a
 * replacement for the array literal syntax. */
test!(array_map => r#"
    from @std/app import start, print, exit

    on start {
      const count = [1, 2, 3, 4, 5]; // Ah, ah, ahh!
      const byTwos = count.map(fn (n: int64): Result<int64> = n * 2);
      count.map(fn (n: int64) = toString(n)).join(', ').print();
      byTwos.map(fn (n: Result<int64>) = toString(n)).join(', ').print();
      emit exit 0;
    }"#;
    stdout "1, 2, 3, 4, 5\n2, 4, 6, 8, 10\n";
);
test!(array_repeat_and_map_lin => r#"
    from @std/app import start, print, exit

    on start {
      const arr = [1, 2, 3] * 3;
      const out = arr.mapLin(fn (x: int64): string = x.toString()).join(', ');
      print(out);
      emit exit 0;
    }"#;
    stdout "1, 2, 3, 1, 2, 3, 1, 2, 3\n";
);
test!(array_each_and_find => r#"
    from @std/app import start, print, exit

    on start {
      const test = [ 1, 1, 2, 3, 5, 8 ];
      test.find(fn (val: int64): bool = val % 2 == 1).getOr(0).print();
      test.each(fn (val: int64) = print('=' * val));
      emit exit 0;
    }"#;
    stdout r#"1
=
=
==
===
=====
========
"#;
);
test!(array_every_some_del => r#"
    from @std/app import start, print, exit

    fn isOdd (val: int64): bool = val % 2 == 1;

    on start {
      const test = [ 1, 1, 2, 3, 5, 8 ];
      test.every(isOdd).print();
      test.some(isOdd).print();
      print(test.length());
      print(test.delete(1));
      print(test.delete(4));
      print(test.delete(10));
      emit exit 0;
    }"#;
    stdout r#"false
true
6
1
8
cannot remove idx 10 from array with length 4
"#;
);
test!(array_reduce_filter_concat => r#"
    from @std/app import start, print, exit

    on start {
      const test = [ 1, 1, 2, 3, 5, 8 ];
      const test2 = [ 4, 5, 6 ];
      print('reduce test');
      test.reduce(fn (a: int, b: int): int = a + b || 0).print();
      test.reduce(min).print();
      test.reduce(max).print();

      print('filter test');
      test.filter(fn (val: int64): bool {
        return val % 2 == 1;
      }).map(fn (val: int64): string {
        return toString(val);
      }).join(', ').print();

      print('concat test');
      test.concat(test2).map(fn (val: int64): string {
        return toString(val);
      }).join(', ').print();
      (test + test2).map(fn (val: int64): string {
        return toString(val);
      }).join(', ').print();

      print('reduce as filter and concat test');
      // TODO: Lots of improvements needed for closures passed directly to opcodes. This one-liner is ridiculous
      test.reduce(fn (acc: string, i: int): string = ((acc == '') && (i % 2 == 1)) ? i.toString() : (i % 2 == 1 ? (acc + ', ' + i.toString()) : acc), '').print();
      // TODO: Even more ridiculous when you want to allow parallelism
      test.reducePar(fn (acc: string, i: int): string = ((acc == '') && (i % 2 == 1)) ? i.toString() : (i % 2 == 1 ? (acc + ', ' + i.toString()) : acc), fn (acc: string, cur: string): string = ((acc != '') && (cur != '')) ? (acc + ', ' + cur) : (acc != '' ? acc : cur), '').print();

      emit exit 0;
    }"#;
    stdout r#"reduce test
20
1
8
filter test
1, 1, 3, 5
concat test
1, 1, 2, 3, 5, 8, 4, 5, 6
1, 1, 2, 3, 5, 8, 4, 5, 6
reduce as filter and concat test
1, 1, 3, 5
1, 1, 3, 5
"#;
);
test!(array_custom_types => r#"
    from @std/app import start, print, exit

    type Foo {
      foo: string,
      bar: bool
    }

    on start {
      const five = [1, 2, 3, 4, 5];
      five.map(fn (n: int64): Foo {
        return new Foo {
          foo: n.toString(),
          bar: n % 2 == 0,
        };
      }).filter(fn (f: Foo): bool = f.bar).map(fn (f: Foo): string = f.foo).join(', ').print();
      emit exit 0;
    }"#;
    stdout "2, 4\n";
);

// Hashing
// TODO: I have no idea how I'm going to make this work in pure Rust, but damnit I'm gonna try.
// This was super useful for a whole host of things.

test!(to_hash => r#"
    from @std/app import start, print, exit

    on start {
      print(toHash(1));
      print(toHash(3.14159));
      print(toHash(true));
      print(toHash('false'));
      print(toHash([1, 2, 5, 3]));
      emit exit 0;
    }"#;
    stdout r#"-1058942856030168491
-5016367128657347516
-1058942856030168491
6288867289231076425
-1521185239552941064
"#;
);
test!(basic_hashmap => r#"
    from @std/app import start, print, exit

    on start {
      const test = newHashMap('foo', 1);
      test.set('bar', 2);
      test.set('baz', 99);
      print(test.keyVal().map(fn (n: KeyVal<string, int64>): string {
        return 'key: ' + n.key + \"\\nval: \" + toString(n.val);
      }).join(\"\\n\"));
      print(test.keys().join(', '));
      print(test.vals().map(fn (n: int64): string = n.toString()).join(', '));
      print(test.length());
      print(test.get('foo'));
      emit exit 0;
    }"#;
    stdout r#"key: foo
val: 1
key: bar
val: 2
key: baz
val: 99
foo, bar, baz
1, 2, 99
3
1
"#;
);
test!(keyval_to_hashmap => r#"
    from @std/app import start, print, exit

    fn kv(k: any, v: anythingElse) = new KeyVal<any, anythingElse> {
      key: k,
      val: v
    }

    on start {
      const kva = [ kv(1, 'foo'), kv(2, 'bar'), kv(3, 'baz') ];
      const hm = kva.toHashMap();
      print(hm.keyVal().map(fn (n: KeyVal<int64, string>): string {
        return 'key: ' + toString(n.key) + \"\\nval: \" + n.val;
      }).join(\"\\n\"));
      print(hm.get(1));
      emit exit 0;
    }"#;
    stdout r#"key: 1
val: foo
key: 2
val: bar
key: 3
val: baz
foo
"#;
);
test!(hashmap_double_set => r#"
    from @std/app import start, print, exit

    on start {
      let test = newHashMap('foo', 'bar');
      test.get('foo').print();
      test.set('foo', 'baz');
      print(test.get('foo'));
      emit exit 0;
    }"#;
    stdout "bar\nbaz\n";
);
/* Pending
test!(hashmap_ops => r#"
    from @std/app import start, print, exit

    on start {
      const test = new Map<string, int64> {
        'foo': 1
        'bar': 2
        'baz': 99
      }

      print('keyVal test')
      test.keyVal().each(fn (n: KeyVal<string, int64>) {
        print('key: ' + n.key)
        print('val: ' + n.value.toString())
      })

      print('keys test')
      test.keys().each(print)

      print('values test')
      test.values().each(print)

      print('length test')
      test.length().print()
      print(#test)

      emit exit 0
    }"#;
    stdout r#"keyVal test
key: bar
val: 2
key: foo
val: 1
key: baz
val: 99
keys test
bar
foo
baz
values test
2
1
99
length test
3
3
"#;
);
*/

// Generics

test!(generics => r#"
    from @std/app import start, print, exit

    type box<V> {
      set: bool,
      val: V
    }

    on start fn {
      let int8Box = new box<int8> {
        val: 8.toInt8(),
        set: true
      };
      print(int8Box.val);
      print(int8Box.set);

      let stringBox = new box<string> {
        val: 'hello, generics!',
        set: true
      };
      print(stringBox.val);
      print(stringBox.set);

      const stringBoxBox = new box<box<string>> {
        val: new box<string> {
          val: 'hello, nested generics!',
          set: true
        },
        set: true
      };
      stringBoxBox.set.print();
      stringBoxBox.val.set.print();
      print(stringBoxBox.val.val);

      emit exit 0;
    }"#;
    stdout r#"8
true
hello, generics!
true
true
true
hello, nested generics!
"#;
);
test!(invalid_generics => r#"
    from @std/app import start, print, exit

    type box<V> {
      set: bool,
      val: V
    }

    on start fn {
      let stringBox = new box<string> {
        set: true,
        val: 'str'
      };
      stringBox.val = 8;

      emit exit 0;
    }"#;
    stderr "stringBox.val is of type string but assigned a value of type int64\n"
);

// Interfaces

test!(basic_interfaces => r#"
    from @std/app import start, print, exit

    interface Stringifiable {
      toString(Stringifiable): string
    }

    fn quoteAndPrint(toQuote: Stringifiable) {
      print(\"'\" + toString(toQuote) + \"'\");
    }

    on start {
      quoteAndPrint('Hello, World');
      quoteAndPrint(5);
      emit exit 0;
    }"#;
    stdout "'Hello, World!'\n'5'\n";
);

/* TODO: Add support for generating multiple source files for a test. Just copying over the whole
 * original test for now because the exact structure isn't yet clear
 *
  Describe "import behavior"
    before() {
      sourceToFile datetime.ln "
        from @std/app import print

        export type Year {
          year: int32
        }

        export type YearMonth {
          year: int32,
          month: int8
        }

        export type Date {
          year: int32,
          month: int8,
          day: int8
        }

        export type Hour {
          hour: int8
        }

        export type HourMinute {
          hour: int8,
          minute: int8
        }

        export type Time {
          hour: int8,
          minute: int8,
          second: float64
        }

        export type DateTime {
          date: Date,
          time: Time,
          timezone: HourMinute
        }

        export fn makeYear(year: int32): Year {
          return new Year {
            year: year
          };
        }

        export fn makeYear(year: int64): Year {
          return new Year {
            year: toInt32(year)
          };
        }

        export fn makeYearMonth(year: int32, month: int8): YearMonth {
          return new YearMonth {
            year: year,
            month: month
          };
        }

        export fn makeYearMonth(y: Year, month: int64): YearMonth {
          return new YearMonth {
            year: y.year,
            month: toInt8(month),
          };
        }

        export fn makeDate(year: int32, month: int8, day: int8): Date {
          return new Date {
            year: year,
            month: month,
            day: day,
          };
        }

        export fn makeDate(ym: YearMonth, day: int64): Date {
          return new Date {
            year: ym.year,
            month: ym.month,
            day: toInt8(day)
          };
        }

        export fn makeHour(hour: int8): Hour {
          return new Hour {
            hour: hour
          };
        }

        export fn makeHourMinute(hour: int8, minute: int8): HourMinute {
          return new HourMinute {
            hour: hour,
            minute: minute
          };
        }

        export fn makeHourMinute(hour: int64, minute: int64): HourMinute {
          return new HourMinute {
            hour: toInt8(hour),
            minute: toInt8(minute)
          };
        }

        export fn makeHourMinute(h: Hour, minute: int8): HourMinute {
          return new HourMinute {
            hour: h.hour,
            minute: minute
          };
        }

        export fn makeTime(hour: int8, minute: int8, second: float64): Time {
          return new Time {
            hour: hour,
            minute: minute,
            second: second
          };
        }

        export fn makeTime(hm: HourMinute, second: float64): Time {
          return new Time {
            hour: hm.hour,
            minute: hm.minute,
            second: second
          };
        }

        export fn makeTime(hm: HourMinute, second: int64): Time {
          return new Time {
            hour: hm.hour,
            minute: hm.minute,
            second: toFloat64(second)
          };
        }

        export fn makeTime(hm: Array<int64>, second: int64): Time {
          return new Time {
            hour: hm[0].toInt8(),
            minute: hm[1].toInt8(),
            second: second.toFloat64()
          };
        }

        export fn makeDateTime(date: Date, time: Time, timezone: HourMinute): DateTime {
          return new DateTime {
            date: date,
            time: time,
            timezone: timezone
          };
        }

        export fn makeDateTime(date: Date, time: Time): DateTime {
          return new DateTime {
            date: date,
            time: time,
            timezone: 00:00,
          };
        }

        export fn makeDateTimeTimezone(dt: DateTime, timezone: HourMinute): DateTime {
          return new DateTime {
            date: dt.date,
            time: dt.time,
            timezone: timezone
          };
        }

        export fn makeDateTimeTimezone(dt: DateTime, timezone: Array<int64>): DateTime {
          return new DateTime {
            date: dt.date,
            time: dt.time,
            timezone: new HourMinute {
              hour: timezone[0].toInt8(),
              minute: timezone[1].toInt8(),
            }
          };
        }

        export fn makeDateTimeTimezoneRev(dt: DateTime, timezone: HourMinute): DateTime {
          return new DateTime {
            date: dt.date,
            time: dt.time,
            timezone: new HourMinute {
              hour: timezone.hour.snegate(),
              minute: timezone.minute
            }
          };
        }

        export fn makeDateTimeTimezoneRev(dt: DateTime, timezone: Array<int64>): DateTime {
          return new Datetime {
            date: dt.date,
            time: dt.time,
            timezone: new HourMinute {
              hour: toInt8(timezone[0]).snegate(),
              minute: toInt8(timezone[1])
            }
          };
        }

        export fn print(dt: DateTime) {
          // TODO: Work on formatting stuff
          const timezoneOffsetSymbol = dt.timezone.hour < toInt8(0) ? \"-\" : \"+\";
          let str = (new Array<string> [
            toString(dt.date.year), \"-\", toString(dt.date.month), \"-\", toString(dt.date.day), \"@\",
            toString(dt.time.hour), \":\", toString(dt.time.minute), \":\", toString(dt.time.second),
            timezoneOffsetSymbol, sabs(dt.timezone.hour).toString(), \":\", toString(dt.timezone.minute)
          ]).join('');
          print(str);
        }

        export prefix makeYear as # precedence 2
        export infix makeYearMonth as - precedence 2
        export infix makeDate as - precedence 2
        export infix makeHourMinute as : precedence 7
        export infix makeTime as : precedence 7
        export infix makeDateTime as @ precedence 2
        export infix makeDateTimeTimezone as + precedence 2
        export infix makeDateTimeTimezoneRev as - precedence 2

        export interface datetime {
          # int64: Year,
          Year - int64: YearMonth,
          YearMonth - int64: Date,
          int64 : int64: HourMinute,
          HourMinute : int64: Time,
          Date @ Time: DateTime,
          DateTime + HourMinute: DateTime,
          DateTime - HourMinute: DateTime,
          print(DateTime): void,
        }
      "

      sourceToAll "
        from @std/app import start, print, exit
        from ./datetime import datetime

        on start {
          const dt = #2020 - 07 - 02@12:07:30 - 08:00;
          dt.print();
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanFile datetime.ln
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The output should eq "2020-7-2@12:7:30-8:0"
    End

    It "runs agc"
      When run test_agc
      The output should eq "2020-7-2@12:7:30-8:0"
    End
  End
*/

// Maybe, Result, and Either

test!(maybe => r#"
    from @std/app import start, print, exit

    fn fiver(val: float64) {
      if val.toInt64() == 5 {
        return some(5);
      } else {
        return none();
      }
    }

    on start {
      const maybe5 = fiver(5.5);
      if maybe5.isSome() {
        print(maybe5.getOr(0));
      } else {
        print('what?');
      }

      const maybeNot5 = fiver(4.4);
      if maybeNot5.isNone() {
        print('Correctly received nothing!');
      } else {
        print('uhhh');
      }

      if maybe5.isSome() {
        print(maybe5 || 0);
      } else {
        print('what?');
      }

      if maybeNot5.isNone() {
        print('Correctly received nothing!');
      } else {
        print('uhhh');
      }

      maybe5.toString().print();
      maybeNot5.toString().print();

      emit exit 0;
    }"#;
    stdout r#"5
Correctly received nothing!
5
Correctly received nothing!
5
none
"#;
);
test!(result => r#"
    from @std/app import start, print, exit

    fn reciprocal(val: float64) {
      if val == 0.0 {
        return err('Divide by zero error!');
      } else {
        return 1.0 / val;
      }
    }

    on start {
      const oneFifth = reciprocal(5.0);
      if oneFifth.isOk() {
        print(oneFifth.getOr(0.0));
      } else {
        print('what?');
      }

      const oneZeroth = reciprocal(0.0);
      if oneZeroth.isErr() {
        const error = oneZeroth.getErr(noerr());
        print(error);
      } else {
        print('uhhh');
      }

      if oneFifth.isOk() {
        print(oneFifth || 0.0);
      } else {
        print('what?');
      }

      if oneZeroth.isErr() {
        print(oneZeroth || 1.2345);
      } else {
        print('uhhh');
      }

      oneFifth.toString().print();
      oneZeroth.toString().print();

      const res = ok('foo');
      print(res.getErr('there is no error'));

      emit exit 0;
    }"#;
    stdout r#"0.2
Divide by zero error!
0.2
1.2345
0.2
Divide by zero error!
there is no error
"#;
);
test!(either => r#"
    from @std/app import start, print, exit

    on start {
      const strOrNum = getMainOrAlt(true);
      if strOrNum.isMain() {
        print(strOrNum.getMainOr(''));
      } else {
        print('what?');
      }

      const strOrNum2 = getMainOrAlt(false);
      if strOrNum2.isAlt() {
        print(strOrNum2.getAltOr(0));
      } else {
        print('uhhh');
      }

      strOrNum.toString().print();
      strOrNum2.toString().print();

      emit exit 0;
    }

    fn getMainOrAlt(isMain: bool) {
      if isMain {
        return main('string');
      } else {
        return alt(2);
      }
    }"#;
    stdout r#"string
2
string
2
"#;
);

// Types

test!(user_types_and_generics => r#"
    from @std/app import start, print, exit

    type foo<A, B> {
      bar: A,
      baz: B
    }

    type foo2 = foo<int64, float64>

    on start fn {
      let a = new foo<string, int64> {
        bar: 'bar',
        baz: 0
      };
      let b = new foo<int64, bool> {
        bar: 0,
        baz: true
      };
      let c = new foo2 {
        bar: 0,
        baz: 1.23
      };
      let d = new foo<int64, float64> {
        bar: 1,
        baz: 3.14
      };
      print(a.bar);
      print(b.bar);
      print(c.bar);
      print(d.bar);

      emit exit 0;
    }"#;
    stdout "bar\n0\n0\n1\n";
);
/* Pending multi-file support
 *
  Describe "using non-imported type returned by imported function"
    before() {
      sourceToTemp "
        from @std/app import start, exit
        from @std/http import fetch, Request

        on start {
          arghFn('{\"test\":\"test\"}');
          emit exit 0;
        }

        fn arghFn(arghStr: string) {
          fetch(new Request {
              method: 'POST',
              url: 'https://reqbin.com/echo/post/json',
              headers: newHashMap('Content-Length', arghStr.length().toString()),
              body: arghStr,
            });
        }
      "
      sourceToFile test_server.js "
        const http = require('http');

        http.createServer((req, res) => {
          console.log('received');
          res.end('Hello, world!');
        }).listen(8088);
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    afterEach() {
      kill $PID1
      wait $PID1 2>/dev/null
      # kill $PID2
      # wait $PID2 2>/dev/null
      return 0
    }
    After afterEach

    It "runs js"
      Pending unimported-types-returned-by-imported-functions
      node test_$$/test_server.js 1>test_$$/test_server.js.out 2>/dev/null &
      PID1=$!
      # node test_$$/temp.js 1>/dev/null &
      # PID2=$!
      sleep 1
      When run cat test_$$/test_server.js.out
      The output should eq "received"
    End

    It "runs agc"
      Pending unimported-types-returned-by-imported-functions
      node test_$$/test_server.js 1>test_$$/test_server.agc.out 2>/dev/null &
      PID1=$!
      # alan run test_$$/temp.agc 1>/dev/null 2>/dev/null  &
      # PID2=$!
      sleep 1
      When run cat test_$$/test_server.agc.out
      The output should eq "received"
    End
  End
*/

// Custom Events

test!(custom_event_loop => r#"
    from @std/app import start, print, exit

    event loop: int64

    on loop fn looper(val: int64) {
      print(val);
      if val >= 10 {
        emit exit 0;
      } else {
        emit loop val + 1 || 0;
      }
    }

    on start {
      emit loop 0;
    }"#;
    stdout r#"0
1
2
3
4
5
6
7
8
9
10
"#;
);
test!(user_defined_type_event => r#"
    from @std/app import start, print, exit

    type Thing {
      foo: int64,
      bar: string
    }

    event thing: Thing

    on thing fn (t: Thing) {
      print(t.foo);
      print(t.bar);
      emit exit 0;
    }

    on start {
      emit thing new Thing {
        foo: 1,
        bar: 'baz'
      };
    }"#;
    stdout "1\nbaz\n";
);
test!(multiple_event_handlers => r#"
    from @std/app import start, print, exit

    event aString: string

    on aString fn(str: string) {
      print('hey I got a string! ' + str);
    }

    on aString fn(str: string) {
      print('I also got a string! ' + str);
    }

    on aString fn(ignore: string) {
      wait(100);
      emit exit 0;
    }

    on start {
      emit aString 'hi';
    }"#;
    stdout "hey I got a string! hi\nI also got a string! hi\n"; // TODO: The order is not guaranteed, support that
);

// Closures

test!(closure_creation_and_usage => r#"
    from @std/app import start, print, exit

    fn closure(): function {
      let num = 0;
      return fn (): int64 {
        num = num + 1 || 0;
        return num;
      };
    }

    on start fn (): void {
      const counter1 = closure();
      const counter2 = closure();
      print(counter1());
      print(counter1());
      print(counter2());
      emit exit 0;
    }"#;
    stdout "1\n2\n1\n";
);
test!(closure_by_name => r#"
    from @std/app import start, print, exit

    fn double(x: int64): int64 = x * 2 || 0;

    on start {
      const numbers = [1, 2, 3, 4, 5];
      numbers.map(double).map(toString).join(', ').print();
      emit exit 0;
    }"#;
    stdout "2, 4, 6, 8, 10\n";
);
test!(inlined_closure_with_arg => r#"
    from @std/app import start, print, exit

    on start {
      const arghFn = fn(argh: string) {
        print(argh);
      };
      arghFn('argh');
      emit exit 0;
    }"#;
    stdout "argh\n";
);

// Compiler Errors

test!(cross_type_comparisons => r#"
    from @std/app import start, print, exit

    on start {
      print(true == 1);
      emit exit 0;
    }"#;
    stderr r#"Cannot resolve operators with remaining statement
true == 1
<bool> == <int64>
"#;
);
test!(unreachable_code => r#"
    from @std/app import start, print, exit

    fn unreachable() {
      return 'blah';
      print('unreachable!');
    }

    on start {
      unreachable();
      emit exit 0;
    }"#;
    stderr r#"Unreachable code in function 'unreachable' after:
return 'blah'; on line 4:12
"#;
);
test!(recursive_functions => r#"
    from @std/app import start, print, exit

    fn fibonacci(n: int64) {
      if n < 2 {
        return 1;
      } else {
        return fibonacci(n - 1 || 0) + fibonacci(n - 2 || 0);
      }
    }

    on start {
      print(fibonacci(0));
      print(fibonacci(1));
      print(fibonacci(2));
      print(fibonacci(3));
      print(fibonacci(4));
      emit exit 0;
    }"#;
    stderr "Recursive callstack detected: fibonacci -> fibonacci. Aborting.\n";
);
test!(undefined_function_call => r#"
    from @std/app import start, print, exit

    on start {
      print(i64str(5)); // Illegal direct opcode usage
      emit exit 0;
    }"#;
    stderr "i64str is not a function but used as one.\ni64str on line 4:18\n";
);
test!(totally_broken_statement => r#"
    import @std/app

    on app.start {
      app.oops
    }"#;
    stderr "TODO";
);
/* Pending
  Describe "Importing unexported values"
    before() {
      sourceToFile piece.ln "
        type Piece {
          owner: bool
        }
      "
      sourceToTemp "
        from @std/app import start, print, exit
        from ./piece import Piece

        on start {
          const piece = new Piece {
            owner: false
          };
          print('Hello World');
          if piece.owner == true {
            print('OK');
          } else {
            print('False');
          }
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanFile piece.ln
      cleanTemp
    }
    AfterAll after

    It "doesn't work"
      When run alan compile test_$$/temp.ln test_$$/temp.amm
      The status should not eq "0"
      The error should eq "Piece is not a type
new Piece {
            owner: false
          } on line 2:26"
    End
  End
*/

// Module-level constants

test!(module_level_constant => r#"
    import @std/app

    const helloWorld = 'Hello, World!';

    on app.start {
      app.print(helloWorld);
      emit app.exit 0;
    }"#;
    stdout "Hello, World!\n";
);
test!(module_level_constant_from_function_call => r#"
    from @std/app import start, print, exit

    const three = add(1, 2);

    fn fiver() = 5;

    const five = fiver();

    on start {
      print(three);
      print(five);
      emit exit 0;
    }"#;
    stdout "3\n5\n";
);

// @std/trig

test!(std_trig => r#"
    from @std/app import start, print, exit
    import @std/trig
    from @std/trig import e, pi, tau
    // shouldn't be necessary, but compiler issue makes it so

    on start {
      'Logarithms and e^x'.print();
      print(trig.exp(e));
      print(trig.ln(e));
      print(trig.log(e));

      'Basic Trig functions'.print();
      print(trig.sin(tau / 6.0));
      print(trig.cos(tau / 6.0));
      print(trig.tan(tau / 6.0));
      print(trig.sec(tau / 6.0));
      print(trig.csc(tau / 6.0));
      print(trig.cot(tau / 6.0));

      'Inverse Trig functions'.print();
      print(trig.arcsine(0.0));
      print(trig.arccosine(1.0));
      print(trig.arctangent(0.0));
      print(trig.arcsecant(tau / 6.0));
      print(trig.arccosecant(tau / 6.0));
      print(trig.arccotangent(tau / 6.0));

      'Historic Trig functions (useful for navigation and as a teaching aid: https://en.wikipedia.org/wiki/File:Circle-trig6.svg )'.print();
      print(trig.versine(pi / 3.0));
      print(trig.vercosine(pi / 3.0));
      print(trig.coversine(pi / 3.0));
      print(trig.covercosine(pi / 3.0));
      print(trig.haversine(pi / 3.0));
      print(trig.havercosine(pi / 3.0));
      print(trig.hacoversine(pi / 3.0));
      print(trig.hacovercosine(pi / 3.0));
      print(trig.exsecant(pi / 3.0));
      print(trig.excosecant(pi / 3.0));
      print(trig.chord(pi / 3.0));

      'Historic Inverse Trig functions'.print();
      print(trig.aver(0.0));
      print(trig.avcs(0.5));
      print(trig.acvs(1.0));
      print(trig.acvc(1.0));
      print(trig.ahav(0.5));
      print(trig.ahvc(0.5));
      print(trig.ahcv(0.5));
      print(trig.ahcc(0.5));
      print(trig.aexs(0.5));
      print(trig.aexc(0.5));
      print(trig.acrd(0.5));

      'Hyperbolic Trig functions'.print();
      print(trig.sinh(tau / 6.0));
      print(trig.cosh(tau / 6.0));
      print(trig.tanh(tau / 6.0));
      print(trig.sech(tau / 6.0));
      print(trig.csch(tau / 6.0));
      print(trig.coth(tau / 6.0));

      'Inverse Hyperbolic Trig functions'.print();
      print(trig.hyperbolicArcsine(tau / 6.0));
      print(trig.hyperbolicArccosine(tau / 6.0));
      print(trig.hyperbolicArctangent(tau / 6.0));
      print(trig.hyperbolicArcsecant(0.5));
      print(trig.hyperbolicArccosecant(tau / 6.0));
      print(trig.hyperbolicArccotangent(tau / 6.0));

      emit exit 0;
    }"#;
    stdout r#"Logarithms and e^x
15.154262241479259
1
0.4342944819032518
Basic Trig functions
0.8660254037844386
0.5000000000000001
1.7320508075688767
1.9999999999999996
1.1547005383792517
0.577350269189626
Inverse Trig functions
0
0
0
0.3013736097452911
1.2694227170496055
0.7623475341648746
Historic Trig functions (useful for navigation and as a teaching aid: https://en.wikipedia.org/wiki/File:Circle-trig6.svg )
0.4999999999999999
1.5
0.1339745962155614
1.8660254037844386
0.24999999999999994
0.75
0.0669872981077807
0.9330127018922193
0.9999999999999996
0.15470053837925168
0.9999999999999999
Historic Inverse Trig functions
0
2.0943951023931957
0
0
1.5707963267948966
1.5707963267948966
0
0
0.8410686705679303
0.7297276562269663
0.5053605102841573
Hyperbolic Trig functions
1.2493670505239751
1.600286857702386
0.7807144353592677
0.6248879662960872
0.8004052928885931
1.2808780710450447
Inverse Hyperbolic Trig functions
0.9143566553928857
0.3060421086132653
1.8849425394276085
1.3169578969248166
0.8491423010640059
1.8849425394276085
"#;
);

/* TODO: Convert the @std/dep (maybe, dep management will likely change quite a bit with the new
 * import syntax) and @std/http tests some time in the future. For now they are included below
 * as-is:
 *
Describe "@std/deps"
  Describe "package dependency add"
    before() {
      sourceToAll "
        from @std/deps import Package, install, add, commit, dependency, using, block, fullBlock

        on install fn (package: Package) = package
          .using(['@std/app', '@std/cmd'])
          .dependency('https://github.com/alantech/hellodep.git')
            .add()
          .block('@std/tcp')
          .fullBlock('@std/httpcommon')
          .commit()
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    after_each() {
      rm -r ./dependencies
    }
    After after_each

    has_dependencies() {
      test -d "./dependencies"
    }

    has_alantech() {
      test -d "./dependencies/alantech"
    }

    has_hellodep() {
      test -d "./dependencies/alantech/hellodep"
    }

    has_index() {
      test -f "./dependencies/alantech/hellodep/index.ln"
    }

    has_nested_dependencies() {
      test -d "./dependencies/alantech/hellodep/dependencies"
    }

    has_nested_alantech() {
      test -d "./dependencies/alantech/hellodep/dependencies/alantech"
    }

    has_nested_hellodep() {
      test -d "./dependencies/alantech/hellodep/dependencies/alantech/nestedhellodep"
    }

    has_nested_index() {
      test -f "./dependencies/alantech/hellodep/dependencies/alantech/nestedhellodep/index.ln"
    }

    has_modules() {
      test -d "./dependencies/modules"
    }

    has_std() {
      test -d "./dependencies/modules/std"
    }

    has_blacklisted_module() {
      test -d "./dependencies/modules/std/tcpserver"
    }

    not_has_cmd() {
      if [ -d ./dependencies/modules/std/cmd ]; then
        return 1
      fi
      return 0
    }

    has_pkg_block() {
      test -d "./dependencies/modules/std/tcp"
    }

    has_pkg_full_block_applied() {
      test -d "./dependencies/alantech/hellodep/modules/std/httpcommon" && grep -R -q "export const mock = true" "./dependencies/alantech/hellodep/modules/std/httpcommon/index.ln"
    }

    run_js() {
      node test_$$/temp.js | head -1
    }

    run_agc() {
      alan run test_$$/temp.agc | head -1
    }

    It "runs js"
      When run run_js
      The output should eq "Cloning into './dependencies/alantech/hellodep'..."
      Assert has_dependencies
      Assert has_alantech
      Assert has_hellodep
      Assert has_index
      Assert has_nested_dependencies
      Assert has_nested_alantech
      Assert has_nested_hellodep
      Assert has_nested_index
      Assert has_modules
      Assert has_std
      Assert has_blacklisted_module
      Assert not_has_cmd
      Assert has_pkg_block
      Assert has_pkg_full_block_applied
    End

    It "runs agc"
      When run run_agc
      The output should eq "Cloning into './dependencies/alantech/hellodep'..."
      Assert has_dependencies
      Assert has_alantech
      Assert has_hellodep
      Assert has_index
      Assert has_nested_dependencies
      Assert has_nested_alantech
      Assert has_nested_hellodep
      Assert has_nested_index
      Assert has_modules
      Assert has_std
      Assert has_blacklisted_module
      Assert not_has_cmd
      Assert has_pkg_block
      Assert has_pkg_full_block_applied
    End
  End
End

Describe "@std/http"
  Describe "basic get"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import get

        on start {
          print(get('https://raw.githubusercontent.com/alantech/hellodep/aea1ce817a423d00107577a430a046993e4e6cad/index.ln'));
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The output should eq "export const comeGetMe = \"You got me!\""
    End

    It "runs agc"
      When run test_agc
      The output should eq "export const comeGetMe = \"You got me!\""
    End
  End

Describe "basic post"
  before() {
    # All my homies hate CORS...
    node -e "const http = require('http'); http.createServer((req, res) => { const headers = { 'Access-Control-Allow-Origin': '*','Access-Control-Allow-Methods': 'OPTIONS, POST, GET, PUT','Access-Control-Max-Age': 2592000, 'Access-Control-Allow-Headers': '*', }; if (req.method === 'OPTIONS') { res.writeHead(204, headers); res.end(); return; } res.writeHead(200, headers); req.pipe(res); req.on('end', () => res.end()); }).listen(8765)" 1>/dev/null 2>/dev/null &
    ECHO_PID=$!
    disown $ECHO_PID
    sourceToAll "
      from @std/app import start, print, exit
      from @std/http import post

      on start {
        print(post('http://localhost:8765', '{\"test\":\"test\"}'));
        emit exit 0;
      }
    "
  }
  BeforeAll before

  after() {
    kill -9 $ECHO_PID 1>/dev/null 2>/dev/null || true
    cleanTemp
  }
  AfterAll after

  It "runs js"
    When run test_js
    The output should eq "{\"test\":\"test\"}"
  End

  It "runs agc"
    When run test_agc
    The output should eq "{\"test\":\"test\"}"
  End
End

  Describe "fetch directly"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import fetch, Request, Response

        on start {
          const res = fetch(new Request {
            method: 'GET',
            url: 'https://raw.githubusercontent.com/alantech/hellodep/aea1ce817a423d00107577a430a046993e4e6cad/index.ln',
            headers: newHashMap('User-Agent', 'Alanlang'),
            body: '',
          });
          print(res.isOk());
          const r = res.getOrExit();
          print(r.status);
          print(r.headers.length());
          print(r.body);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    # The number of headers returned in the two runtimes is slightly different. Node includes the
    # "connection: close" header and Hyper.rs does not
    FETCHJSOUTPUT="true
200
25
export const comeGetMe = \"You got me!\""

    FETCHAGCOUTPUT="true
200
23
export const comeGetMe = \"You got me!\""

    It "runs js"
      When run test_js
      The output should eq "$FETCHJSOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$FETCHAGCOUTPUT"
    End
  End

  Describe "Hello World webserver"
    before() {
      sourceToAll "
        from @std/app import start, exit
        from @std/httpserver import connection, body, send, Connection

        on connection fn (conn: Connection) {
          const req = conn.req;
          const res = conn.res;
          set(res.headers, 'Content-Type', 'text/plain');
          if req.method == 'GET' {
            res.body('Hello, World!').send();
          } else {
            res.body('Hello, Failure!').send();
          }
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    afterEach() {
      kill $PID
      wait $PID 2>/dev/null
      return 0
    }
    After afterEach

    It "runs js"
      node test_$$/temp.js 1>/dev/null 2>/dev/null &
      PID=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End

    It "runs agc"
      alan run test_$$/temp.agc 1>/dev/null 2>/dev/null &
      PID=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End
  End

  Describe "importing http get doesn't break hashmap get"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/http import get

        on start {
          const str = get('https://raw.githubusercontent.com/alantech/hellodep/aea1ce817a423d00107577a430a046993e4e6cad/index.ln').getOr('');
          const kv = str.split(' = ');
          const key = kv[0] || 'bad';
          const val = kv[1] || 'bad';
          const hm = newHashMap(key, val);
          hm.get(key).getOr('failed').print();
          hm.get('something else').getOr('correct').print();
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    GETGETOUTPUT="\"You got me!\"

correct"

    It "runs js"
      When run test_js
      The output should eq "${GETGETOUTPUT}"
    End

    It "runs agc"
      When run test_agc
      The output should eq "${GETGETOUTPUT}"
    End
  End

  Describe "Double-send in a single connection doesn't crash"
    before() {
      sourceToAll "
        from @std/app import print, exit
        from @std/httpserver import connection, Connection, body, send

        on connection fn (conn: Connection) {
          const res = conn.res;
          const firstMessage = res.body('First Message').send();
          print(firstMessage);
          const secondMessage = res.body('Second Message').send();
          print(secondMessage);
          wait(1000);
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      node test_$$/temp.js 1>./out.txt 2>/dev/null &
      sleep 1
      When run curl -s localhost:8000
      The output should eq "First Message"
    End

    It "response from js"
      When run cat ./out.txt
      The output should eq "HTTP server listening on port 8000
ok
connection not found"
      rm out.txt
    End

    It "runs agc"
      sleep 2
      alan run test_$$/temp.agc 1>./out.txt 2>/dev/null &
      sleep 1
      When run curl -s localhost:8000
      The output should eq "First Message"
    End

    It "response from agc"
      When run cat ./out.txt
      The output should eq "HTTP server listening on port 8000
ok
cannot call send twice for the same connection"
      rm out.txt
    End
  End

End
*/

// Clone

test!(clone => r#"
    from @std/app import start, print, exit

    on start {
      let a = 3;
      let b = a.clone();
      a = 4;
      print(a);
      print(b);
      let c = [1, 2, 3];
      let d = c.clone();
      d.set(0, 2);
      c.map(fn (val: int): string = val.toString()).join(', ').print();
      d.map(fn (val: int): string = val.toString()).join(', ').print();
      emit exit 0;
    }"#;
    stdout "4\n3\n1, 2, 3\n2, 2, 3\n";
);

// Runtime Error

test!(get_or_exit => r#"
    from @std/app import start, print, exit

    on start {
      const xs = [0, 1, 2, 5];
      const x1 = xs[1].getOrExit();
      print(x1);
      const x2 = xs[2].getOrExit();
      print(x2);
      const x5 = xs[5].getOrExit();
      print(x5);

      emit exit 0;
    }"#;
    status 1;
);

/* It's not known *if* @std/datastore will be restored or what changes there will be needed with
 * the new focus, so just copying the tests for it directly to keep or drop eventually
 *

Describe "@std/datastore"
  Describe "distributed kv"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/datastore import namespace, has, set, del, getOr

        on start {
          const ns = namespace('foo');
          print(ns.has('bar'));
          ns.set('bar', 'baz');
          print(ns.has('bar'));
          print(ns.getOr('bar', ''));
          ns.del('bar');
          print(ns.has('bar'));
          print(ns.getOr('bar', ''));

          ns.set('inc', 0);
          emit waitAndInc 100;
          emit waitAndInc 200;
          emit waitAndInc 300;
        }

        event waitAndInc: int64

        on waitAndInc fn (ms: int64) {
          wait(ms);
          let i = namespace('foo').getOr('inc', 0);
          i = i + 1 || 0;
          print(i);
          namespace('foo').set('inc', i);
          if i == 3 {
            emit exit 0;
          }
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    DSOUTPUT="false
true
baz
false

1
2
3"

    It "runs js"
      When run test_js
      The output should eq "$DSOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$DSOUTPUT"
    End
  End

  Describe "distributed compute"
    before() {
      sourceToAll "
        from @std/app import start, print, exit
        from @std/datastore import namespace, set, ref, mut, with, run, mutOnly, closure, getOr

        on start {
          // Initial setup
          const ns = namespace('foo');
          ns.set('foo', 'bar');

          // Basic remote execution
          const baz = ns.ref('foo').run(fn (foo: string) = foo.length());
          print(baz);

          // Closure-based remote execution
          let bar = 'bar';
          const bay = ns.ref('foo').closure(fn (foo: string): int64 {
            bar = 'foobar: ' + foo + bar;
            return foo.length();
          });
          print(bay);
          print(bar);

          // Constrained-closure that only gets the 'with' variable
          const bax = ns.ref('foo').with(bar).run(fn (foo: string, bar: string): int64 = #foo +. #bar);
          print(bax);

          // Mutable closure
          const baw = ns.mut('foo').run(fn (foo: string): int64 {
            foo = foo + 'bar';
            return foo.length();
          });
          print(baw);

          // Mutable closure that affects the foo variable
          const bav = ns.mut('foo').closure(fn (foo: string): int64 {
            foo = foo + 'bar';
            bar = bar * foo.length();
            return bar.length();
          });
          print(bav);
          print(bar);

          // Constrained mutable closure that affects the foo variable
          const bau = ns.mut('foo').with(bar).run(fn (foo: string, bar: string): int64 {
            foo = foo * #bar;
            return foo.length();
          });
          print(bau);

          // 'Pure' function that only does mutation
          ns.mut('foo').mutOnly(fn (foo: string) {
            foo = foo + foo;
          });
          print(ns.getOr('foo', 'not found'));

          // Constrained 'pure' function that only does mutation
          ns.mut('foo').with(bar).mutOnly(fn (foo: string, bar: string) {
            foo = foo + bar;
          });
          print(ns.getOr('foo', 'not found'));

          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    DCOUTPUT="3
3
foobar: barbar
17
6
126
foobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbar
1134
barbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbar
barbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbarfoobar: barbar"

    It "runs js"
      When run test_js
      The output should eq "$DCOUTPUT"
    End

    It "runs agc"
      When run test_agc
      The output should eq "$DCOUTPUT"
    End
  End

End
*/

// @std/seq

test!(seq_and_next => r#"
    from @std/app import start, print, exit
    from @std/seq import seq, next

    on start {
      let s = seq(2);
      print(s.next());
      print(s.next());
      print(s.next());
      emit exit 0;
    }"#;
    stdout "0\n1\nerror: sequence out-of-bounds\n";
);
test!(seq_each => r#"
    from @std/app import start, print, exit
    from @std/seq import seq, each

    on start {
      let s = seq(3);
      s.each(fn (i: int64) = print(i));
      emit exit 0;
    }"#;
    stdout "0\n1\n2\n";
);
test!(seq_while => r#"
    from @std/app import start, print, exit
    from @std/seq import seq, while

    on start {
      let s = seq(100);
      let sum = 0;
      s.while(fn = sum < 10, fn {
        sum = sum + 1 || 0;
      });
      print(sum);
      emit exit 0;
    }"#;
    stdout "10\n";
);
test!(seq_do_while => r#"
    from @std/app import start, print, exit
    from @std/seq import seq, doWhile

    on start {
      let s = seq(100);
      let sum = 0;
      // TODO: Get automatic type inference working on anonymous multi-line functions
      s.doWhile(fn (): bool {
        sum = sum + 1 || 0;
        return sum < 10;
      });
      print(sum);
      emit exit 0;
    }"#;
    stdout "10\n";
);
test!(seq_recurse => r#"
    from @std/app import start, print, exit
    from @std/seq import seq, Self, recurse

    on start {
      print(seq(100).recurse(fn fibonacci(self: Self, i: int64): Result<int64> {
        if i < 2 {
          return ok(1);
        } else {
          const prev = self.recurse(i - 1 || 0);
          const prevPrev = self.recurse(i - 2 || 0);
          if prev.isErr() {
            return prev;
          }
          if prevPrev.isErr() {
            return prevPrev;
          }
          // TODO: Get type inference inside of recurse working so we don't need to unwrap these
          return (prev || 0) + (prevPrev || 0);
        }
      }, 8));
      emit exit 0;
    }"#;
    stdout "34\n";
);
test!(seq_no_op_one_liner_regression_test => r#"
    import @std/app
    from @std/seq import seq, Self, recurse

    fn doNothing(x: int) : int = x;

    fn doNothingRec(x: int) : int = seq(x).recurse(fn (self: Self, x: int) : Result<int> {
        return ok(x);
    }, x) || 0;

    on app.start {
        const x = 5;
        app.print(doNothing(x)); // 5
        app.print(doNothingRec(x)); // 5

        const xs = [1, 2, 3];
        app.print(xs.map(doNothing).map(toString).join(' ')); // 1 2 3
        app.print(xs.map(doNothingRec).map(toString).join(' ')); // 1 2 3

        emit app.exit 0;
    }"#;
    stdout "5\n5\n1 2 3\n1 2 3\n"; // TODO: Do we keep a regression test for a prior iteration?
);
test!(seq_recurse_decrement_regression_test => r#"
    import @std/app
    from @std/seq import seq, Self, recurse

    fn triangularRec(x: int) : int = seq(x + 1 || 0).recurse(fn (self: Self, x: int) : Result<int> {
      if x == 0 {
        return ok(x);
      } else {
        // TODO: Get type inference inside of recurse working so we don't need to unwrap these
        return x + (self.recurse(x - 1 || 0) || 0);
      }
    }, x) || 0

    on app.start {
      const xs = [1, 2, 3];
      app.print(xs.map(triangularRec).map(toString).join(' ')); // 1 3 6

      emit app.exit 0;
    }"#;
    stdout "1 3 6\n"; // TODO: Same concern, do regression tests matter for a different codebase?
);

// Tree

test!(tree_construction_and_access => r#"
    from @std/app import start, print, exit

    on start {
      const myTree = newTree('foo');
      const barNode = myTree.addChild('bar');
      const bazNode = myTree.addChild('baz');
      const bayNode = barNode.addChild('bay');

      print(myTree.getRootNode() || 'wrong');
      print(bayNode.getParent() || 'wrong');
      print(myTree.getChildren().map(fn (c: Node<string>): string = c || 'wrong').join(', '));

      emit exit 0;
    }"#;
    stdout "foo\nbar\nbar, baz\n";
);
test!(tree_user_defined_types => r#"
    from @std/app import start, print, exit

    type Foo {
      foo: string,
      bar: bool,
    }

    on start {
      const myTree = newTree(new Foo {
        foo: 'myFoo',
        bar: false,
      });
      const wrongFoo = new Foo {
        foo: 'wrongFoo',
        bar: false,
      };
      const myFoo = myTree.getRootNode() || wrongFoo;
      print(myFoo.foo);
      emit exit 0;
    }"#;
    stdout "myFoo\n";
);
test!(tree_every_find_some_reduce_prune => r#"
    from @std/app import start, print, exit

    on start {
      const myTree = newTree('foo');
      const barNode = myTree.addChild('bar');
      const bazNode = myTree.addChild('baz');
      const bayNode = barNode.addChild('bay');

      print(myTree.every(fn (c: Node<string>): bool = (c || 'wrong').length() == 3));
      print(myTree.some(fn (c: Node<string>): bool = (c || 'wrong').length() == 1));
      print(myTree.find(fn (c: Node<string>): bool = (c || 'wrong') == 'bay').getOr('wrong'));
      print(myTree.find(fn (c: Node<string>): bool = (c || 'wrong') == 'asf').getOr('wrong'));

      print(myTree.length());
      myTree.getChildren().eachLin(fn (c: Node<string>) {
        const n = c || 'wrong';
        if n == 'bar' {
          c.prune();
        }
      });
      print(myTree.getChildren().map(fn (c: Node<string>): string = c || 'wrong').join(', '));
      print(myTree.length());

      myTree.reduce(fn (acc: int, i: Node<string>): int = (i || 'wrong').length() + acc || 0, 0).print();
      emit exit 0;
    }"#;
    stdout r#"true
false
bay
wrong
4
baz
2
6
"#;
);
test!(subtree_and_nested_tree_construction => r#"
    from @std/app import start, print, exit

    on start {
      const bigNestedTree = newTree('foo')
        .addChild('bar')
        .getTree()
        .addChild(newTree('baz')
          .addChild('quux')
          .getTree()
        ).getTree();

      const mySubtree = bigNestedTree
        .getRootNode()
        .getChildren()[1]
        .getOr(newTree('what').getRootNode())
        .toSubtree();

      print(bigNestedTree.getRootNode() || 'wrong');
      print(mySubtree.getRootNode() || 'wrong');

      emit exit 0;
    }"#;
    stdout "foo\nbaz\n";
);

// Error printing

test!(eprint => r#"
    from @std/app import start, eprint, exit
    on start {
      eprint('This is an error');
      emit exit 0;
    }"#;
    stderr "This is an error\n";
);
test!(stderr_event => r#"
    from @std/app import start, stderr, exit
    on start {
      emit stderr 'This is an error';
      wait(10);
      emit exit 0;
    }"#;
    stderr "This is an error";
);

// @std/cmd

test!(cmd_exec => r#"
    import @std/app
    import @std/cmd

    on app.start {
      const executionResult: cmd.ExecRes = cmd.exec('echo 1');
      app.print(executionResult.stdout);
      emit app.exit 0;
    }"#;
    stdout "1\n";
);
test!(cmd_sequential => r#"
    from @std/app import start, print, exit
    from @std/cmd import exec

    on start {
      exec('touch test.txt');
      exec('echo foo >> test.txt');
      exec('echo bar >> test.txt');
      exec('cat test.txt').stdout.print();
      exec('rm test.txt');

      emit exit 0;
    }"#;
    stdout "foobar\n";
);

/* TODO: Module import testing once the test macros are improved

Describe "Module imports"
  Describe "can import with trailing whitespace"
    before() {
      sourceToFile piece.ln "
        export type Piece {
          owner: bool,
        }
      "
      sourceToAll "
        from @std/app import start, print, exit
        // Intentionally put an extra space after the import
        from ./piece import Piece

        on start {
          const piece = new Piece {
            owner: false,
          };
          print('Hello, World!');
          if piece.owner == true {
            print('OK');
          } else {
            print('False');
          }
          emit exit 0;
        }
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    It "runs js"
      When run test_js
      The output should eq "Hello, World!
False"
    End

    It "runs agc"
      When run test_agc
      The output should eq "Hello, World!
False"
    End
  End
End
*/

// JSON

test!(json_construction_printing => r#"
    from @std/app import start, print, exit
    from @std/json import JSON, toJSON, toString, JSONBase, JSONNode, IsObject, Null

    on start {
      1.0.toJSON().print();
      true.toJSON().print();
      'Hello, JSON!'.toJSON().print();
      [1.0, 2.0, 5.0].toJSON().print();
      toJSON().print();

      emit exit 0;
    }"#;
    stdout r#"1
true
"Hello, JSON!"
[1, 2, 5]
null
"#;
);
test!(json_complex_construction => r#"
    from @std/app import start, print, exit
    from @std/json import JSON, toString, JSONBase, JSONNode, IsObject, Null, newJSONObject, newJSONArray, addKeyVal, push

    on start {
      newJSONObject()
        .addKeyVal('mixed', 'values')
        .addKeyVal('work', true)
        .addKeyVal('even', newJSONArray()
          .push(4.0)
          .push('arrays'))
        .print();

      emit exit 0;
    }"#;
    stdout r#"{"mixed": "values", "work": true, "even": [4, "arrays"]}""#;
);

/* TODO: Support the tcp server tests

Describe "@std/tcp"
  Describe "webserver tunnel test"
    before() {
      sourceToTemp "
        from @std/tcpserver import tcpConn
        from @std/tcp import TcpChannel, connect, addContext, ready, chunk, TcpContext, read, write, tcpClose, close

        on tcpConn fn (channel: TcpChannel) {
          const tunnel = connect('localhost', 8088);
          channel.addContext(tunnel);
          tunnel.addContext(channel);
          channel.ready();
          tunnel.ready();
        }

        on chunk fn (ctx: TcpContext<TcpChannel>) {
          ctx.context.write(ctx.channel.read());
        }

        on tcpClose fn (ctx: TcpContext<TcpChannel>) {
          ctx.context.close();
        }
      "
      tempToAmm
      tempToJs
      sourceToFile test_server.js "
        const http = require('http')

        http.createServer((req, res) => res.end('Hello, World!')).listen(8088)
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    afterEach() {
      kill $PID1
      wait $PID1 2>/dev/null
      kill $PID2
      wait $PID2 2>/dev/null
      return 0
    }
    After afterEach

    It "runs js"
      node test_$$/test_server.js 1>/dev/null 2>/dev/null &
      PID1=$!
      node test_$$/temp.js 1>/dev/null 2>/dev/null &
      PID2=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End
  End

  Describe "webserver tunnel function test"
    before() {
      sourceToAll "
        from @std/tcpserver import tunnel
        from @std/app import start, print

        on start {
          let connected = tunnel(8088);
          print(connected ? 'Tunneling to 8088' : 'Failed to establish a tunnel');
        }
      "
      sourceToFile test_server.js "
        const http = require('http')

        http.createServer((req, res) => res.end('Hello, World!')).listen(8088)
      "
    }
    BeforeAll before

    after() {
      cleanTemp
    }
    AfterAll after

    afterEach() {
      kill $PID1
      wait $PID1 2>/dev/null
      kill $PID2
      wait $PID2 2>/dev/null
      return 0
    }
    After afterEach

    It "runs js"
      node test_$$/test_server.js 1>/dev/null 2>/dev/null &
      PID1=$!
      node test_$$/temp.js 1>/dev/null 2>/dev/null &
      PID2=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End

    It "runs agc"
      node test_$$/test_server.js 1>/dev/null 2>/dev/null &
      PID1=$!
      alan run test_$$/temp.agc 1>/dev/null 2>/dev/null &
      PID2=$!
      sleep 1
      When run curl -s localhost:8000
      The output should eq "Hello, World!"
    End
  End
End
*/

// Saturating Math

test!(int8_sadd => r#"
    from @std/app import start, exit
    on start { emit exit sadd(toInt8(1), toInt8(2)); }"#;
    status 3;
);
test!(int8_ssub => r#"
    from @std/app import start, exit
    on start { emit exit ssub(toInt8(2), toInt8(1)); }"#;
    status 1;
);
test!(int8_smul => r#"
    from @std/app import start, exit
    on start { emit exit smul(toInt8(2), toInt8(1)); }"#;
    status 2;
);
test!(int8_sdiv => r#"
    from @std/app import start, exit
    on start { emit exit sdiv(toInt8(6), toInt8(0)); }"#;
    status 127;
);
test!(int8_spow => r#"
    from @std/app import start, exit
    on start { emit exit spow(toInt8(6), toInt8(2)); }"#;
    status 36;
);
test!(int16_sadd => r#"
    from @std/app import start, print, exit
    on start {
      print(sadd(toInt16(1), toInt16(2)));
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int16_ssub => r#"
    from @std/app import start, print, exit
    on start {
      print(ssub(toInt16(2), toInt16(1)));
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(int16_smul => r#"
    from @std/app import start, print, exit
    on start {
      print(smul(toInt16(2), toInt16(1)));
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int16_sdiv => r#"
    from @std/app import start, print, exit
    on start {
      print(sdiv(toInt16(6), toInt16(2)));
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int16_spow => r#"
    from @std/app import start, print, exit
    on start {
      print(spow(toInt16(6), toInt16(2)));
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(int32_sadd => r#"
    from @std/app import start, print, exit
    on start {
      sadd(1.toInt32(), 2.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int32_ssub => r#"
    from @std/app import start, print, exit
    on start {
      ssub(2.toInt32(), 1.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(int32_smul => r#"
    from @std/app import start, print, exit
    on start {
      smul(2.toInt32(), 1.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int32_sdiv => r#"
    from @std/app import start, print, exit
    on start {
      sdiv(6.toInt32(), 2.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int32_spow => r#"
    from @std/app import start, print, exit
    on start {
      spow(6.toInt32(), 2.toInt32()).print();
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(int64_sadd => r#"
    from @std/app import start, print, exit
    on start {
      print(1 +. 2);
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int64_ssub => r#"
    from @std/app import start, print, exit
    on start {
      print(2 -. 1);
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(int64_smul => r#"
    from @std/app import start, print, exit
    on start {
      print(2 *. 1);
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(int64_sdiv => r#"
    from @std/app import start, print, exit
    on start {
      print(6 /. 2);
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(int64_spow => r#"
    from @std/app import start, print, exit
    on start {
      print(6 **. 2);
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(float32_sadd => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(1) +. toFloat32(2));
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float32_ssub => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(2) -. toFloat32(1));
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(float32_smul => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(2) *. toFloat32(1));
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(float32_sdiv => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(6) /. toFloat32(2));
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float32_spow => r#"
    from @std/app import start, print, exit
    on start {
      print(toFloat32(6) **. toFloat32(2));
      emit exit 0;
    }"#;
    stdout "36\n";
);
test!(float64_sadd => r#"
    from @std/app import start, print, exit
    on start {
      (1.0 +. 2.0).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float64_ssub => r#"
    from @std/app import start, print, exit
    on start {
      (2.0 -. 1.0).print();
      emit exit 0;
    }"#;
    stdout "1\n";
);
test!(float64_smul => r#"
    from @std/app import start, print, exit
    on start {
      (2.0 *. 1.0).print();
      emit exit 0;
    }"#;
    stdout "2\n";
);
test!(float64_sdiv => r#"
    from @std/app import start, print, exit
    on start {
      (6.0 /. 2.0).print();
      emit exit 0;
    }"#;
    stdout "3\n";
);
test!(float64_spow => r#"
    from @std/app import start, print, exit
    on start {
      (6.0 **. 2.0).print();
      emit exit 0;
    }"#;
    stdout "36\n";
);
