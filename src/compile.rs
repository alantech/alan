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
