// TODO: Figure out how to integrate `rustc` into the `alan` binary.
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
    }
}
#[cfg(test)]
macro_rules! stderr {
    ( $test_val:expr, $real_val:expr ) => {
        let std_err = String::from_utf8($real_val.stderr.clone())?;
        assert_eq!($test_val, &std_err);
    }
}
#[cfg(test)]
macro_rules! status {
    ( $test_val:expr, $real_val:expr ) => {
        let status = $real_val.status.code().unwrap();
        assert_eq!($test_val, status);
    }
}

// The only test that works for now
test!(hello_world => r#"
    on start {
        print("Hello, World!");
    }"#;
    stdout "Hello, World!\n";
    status 0;
);

// Event Tests

test!(normal_exit_code => r#"
    from @std/app import start, exit

    on start { emit exit 0; }"#;
    status 0;
);
test!(error_exit_code => r#"
    from @std/app import start, exit

    on start { emit exit 1; }"#;
    status 1;
);
test!(non_global_memory_exit_code => r#"
    import @std/app

    on app.start {
      let x: int64 = 0;
      emit app.exit x;
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
    from @std/app import start, print, exit
    on start {
      print('Hello, World');
      emit exit 0;
    }"#;
    stdout "Hello, World\n";
);
test!(stdout_event => r#"
    from @std/app import start, stdout, exit
    on start {
      emit stdout 'Hello, World';
      wait(10);
      emit exit 0;
    }"#;
    stdout "Hello, World";
);

// Basic Math Tests

test!(int8_add => r#"
    from @std/app import start, exit
    on start { emit exit add(toInt8(1), toInt8(2)).getOrExit(); }"#;
    status 3;
);
test!(int8_sub => r#"
    from @std/app import start, exit
    on start { emit exit sub(toInt8(2), toInt8(1)).getOrExit(); }"#;
    status 1;
);
test!(int8_mul => r#"
    from @std/app import start, exit
    on start { emit exit mul(toInt8(2), toInt8(1)).getOrExit(); }"#;
    status 2;
);
test!(int8_div => r#"
    from @std/app import start, exit
    on start { emit exit div(toInt8(6), toInt8(2)).getOrExit(); }"#;
    status 3;
);
test!(int8_mod => r#"
    from @std/app import start, exit
    on start { emit exit mod(toInt8(6), toInt8(4)); }"#;
    status 2;
);
test!(int8_pow => r#"
    from @std/app import start, exit
    on start { emit exit pow(toInt8(6), toInt8(2)).getOrExit(); }"#;
    status 36;
);
test!(int8_min => r#"
    from @std/app import start, print, exit
    on start {
      min(3.toInt8(), 5.toInt8()).print();
      emit exit 0;
    }"#;
    status 3;
);
test!(int8_max => r#"
    from @std/app import start, print, exit
    on start {
      max(3.toInt8(), 5.toInt8()).print();
      emit exit 0;
    }"#;
    status 5;
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
