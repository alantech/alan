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
                match std::fs::write(&filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", filename, e).into());
                    }
                };
                match std::process::Command::new("cargo")
                    .env("ALAN_TARGET", "test")
                    .env_remove("ALAN_OUTPUT_LANG")
                    .arg("run")
                    .arg("--release")
                    .arg("--")
                    .arg("compile")
                    .arg(filename.clone())
                    .output() {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        std::fs::remove_file(&filename)?;
                        return Err(format!("Failed to compile {:?}", e).into());
                    }
                };
                let cmd = if cfg!(windows) {
                    format!(".\\{}.exe", stringify!($rule))
                } else {
                    format!("./{}", stringify!($rule))
                };
                let run = std::process::Command::new(cmd.clone()).output()?;
                $( $type!($test_val, &run); )+
                std::fs::remove_file(&filename)?;
                std::fs::remove_file(&cmd)?;
                Ok(())
            }
        }
    }
}
macro_rules! test_full {
    ( $rule: ident => $code:expr; $( $type:ident $test_val:expr);+ $(;)? ) => {
        #[cfg(test)]
        mod $rule {
            #[test]
            fn $rule() -> Result<(), Box<dyn std::error::Error>> {
                let filename = format!("{}.ln", stringify!($rule));
                match std::fs::write(&filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", filename, e).into());
                    }
                };
                match std::process::Command::new("cargo")
                    .env("ALAN_TARGET", "test")
                    .env_remove("ALAN_OUTPUT_LANG")
                    .arg("run")
                    .arg("--release")
                    .arg("--")
                    .arg("compile")
                    .arg(filename.clone())
                    .output() {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        std::fs::remove_file(&filename)?;
                        return Err(format!("Failed to compile {:?}", e).into());
                    }
                };
                let cmd = if cfg!(windows) {
                    format!(".\\{}.exe", stringify!($rule))
                } else {
                    format!("./{}", stringify!($rule))
                };
                let run = match std::process::Command::new(cmd.clone()).output() {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not run the test binary {:?}", e)),
                }?;
                $( $type!($test_val, &run); )+
                match std::fs::remove_file(&cmd) {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not remove the test binary {:?}", e)),
                }?;
                match std::process::Command::new("cargo")
                    .env("ALAN_TARGET", "test")
                    .env_remove("ALAN_OUTPUT_LANG")
                    .arg("run")
                    .arg("--release")
                    .arg("--")
                    .arg("bundle")
                    .arg(filename.clone())
                    .output() {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        std::fs::remove_file(&filename)?;
                        return Err(format!("Failed to compile {:?}", e).into());
                    }
                };
                let cmd = if cfg!(windows) {
                    format!(".\\{}.js", stringify!($rule))
                } else {
                    format!("./{}.js", stringify!($rule))
                };
                let run = match std::process::Command::new("node").arg(cmd.to_string()).output() {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not run the test JS code {:?}", e)),
                }?;
                $( $type!($test_val, &run); )+
                match std::fs::remove_file(&cmd) {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not remove the generated JS file {:?}", e)),
                }?;
                std::fs::remove_file(&filename)?;
                Ok(())
            }
        }
    }
}
macro_rules! test_ignore {
    ( $rule: ident => $code:expr; $( $type:ident $test_val:expr);+ $(;)? ) => {
        #[cfg(test)]
        mod $rule {
            #[test]
            #[ignore]
            fn $rule() -> Result<(), Box<dyn std::error::Error>> {
                Ok(())
            }
        }
    };
}
macro_rules! test_compile_error {
    ( $rule: ident => $code:expr; error $test_val:expr; ) => {
        #[cfg(test)]
        mod $rule {
            #[test]
            fn $rule() -> Result<(), Box<dyn std::error::Error>> {
                let filename = format!("{}.ln", stringify!($rule));
                std::fs::write(&filename, $code)?;
                let res = crate::compile::compile(filename.to_string());
                std::fs::remove_file(&filename)?;
                match res {
                    Ok(_) => Err("Unexpectedly succeeded!".into()),
                    Err(e) => Ok(assert_eq!(format!("{}", e), $test_val)),
                }
            }
        }
    };
    ( $rule: ident => $code:expr; ) => {
        #[cfg(test)]
        mod $rule {
            #[test]
            fn $rule() -> Result<(), Box<dyn std::error::Error>> {
                let filename = format!("{}.ln", stringify!($rule));
                std::fs::write(&filename, $code)?;
                let res = crate::compile::compile(filename.to_string());
                std::fs::remove_file(&filename)?;
                match res {
                    Ok(_) => Err("Unexpectedly succeeded!".into()),
                    Err(_) => Ok(()),
                }
            }
        }
    };
}
#[cfg(test)]
macro_rules! stdout {
    ( $test_val:expr, $real_val:expr ) => {
        let std_out = if cfg!(windows) {
            String::from_utf8($real_val.stdout.clone())?.replace("\r\n", "\n")
        } else {
            String::from_utf8($real_val.stdout.clone())?
        };
        assert_eq!($test_val, &std_out);
    };
}
#[cfg(test)]
macro_rules! stdout_contains {
    ( $test_val:expr, $real_val:expr ) => {
        let std_out = if cfg!(windows) {
            String::from_utf8($real_val.stdout.clone())?.replace("\r\n", "\n")
        } else {
            String::from_utf8($real_val.stdout.clone())?
        };
        assert_eq!(std_out.contains($test_val), true);
    };
}
#[cfg(test)]
macro_rules! stderr {
    ( $test_val:expr, $real_val:expr ) => {
        let std_err = if cfg!(windows) {
            String::from_utf8($real_val.stderr.clone())?.replace("\r\n", "\n")
        } else {
            String::from_utf8($real_val.stderr.clone())?
        };
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
test_full!(hello_world => r#"
    export fn main() -> () {
        print('Hello, World!');
    }"#;
    stdout "Hello, World!\n";
    status 0;
);
test_full!(multi_line_hello_world => r#"
export fn main = print(
"Hello,
World!");"#;
    stdout r#"Hello,
World!
"#;
    status 0;
);

// Exit Tests

test!(normal_exit_code => r#"
    export fn main() -> ExitCode {
        return ExitCode(0);
    }"#;
    status 0;
);
test!(error_exit_code => r#"
    export fn main() = ExitCode(1);"#;
    status 1;
);
test!(non_global_memory_exit_code => r#"
    export fn main() {
      let x: i64 = 0;
      return x.ExitCode;
    }"#;
    status 0;
);

// Unorganized Tests (TODO: Find a better grouping for these)
test_full!(passing_ints_to_function => r#"
    fn aNumber(num: i64) {
      print('I got a number! '.concat(num.string));
    }

    export fn main {
      aNumber(5);
    }"#;
    stdout "I got a number! 5\n";
    status 0;
);
test_full!(underscores_in_numbers => r#"
    export fn main = print(1_000_000 * 2);
"#;
    stdout "2000000\n";
    status 0;
);
test_full!(other_integer_syntaxes => r#"
    export fn main {
      print(0b10 == 2);
      print(0o10 == 8);
      print(0x10 == 16);
      print(0xF == 15);
    }
"#;
    stdout "true\ntrue\ntrue\ntrue\n";
);
test_full!(scientific_notation => r#"
    export fn main {
      print(15.0 == 1.5e1);
      print(-5.0 == -5e0);
      print(1e3 == 1000.0);
      print(1e-3 == 0.001);
    }
"#;
    stdout "true\ntrue\ntrue\ntrue\n";
);
test_full!(void_values => r#"
    export fn main {
        5.print;
        5.void.print;
        void().print; // TODO: `void.print` should work, too. Figure out why it isn't
    }"#;
    stdout "5\nvoid\nvoid\n";
);

// Printing Tests

// This one will replace the hello_world test above once the syntax is updated
test_full!(print_function => r#"
    export fn main() {
      print('Hello, World');
      return ExitCode(0);
    }"#;
    stdout "Hello, World\n";
    status 0;
);
test!(duration_print => r#"
    export fn main() -> void {
        const i = now();
        wait(10);
        const d = i.elapsed;
        print(d);
    }"#;
    stdout_contains "0.01";
);

// Basic Math Tests

test_full!(i8_add => r#"
    export fn main = ExitCode(add(i8(1), i8(2)));"#;
    status 3;
);
test_full!(i8_sub => r#"
    export fn main = ExitCode(sub(i8(2), i8(1)));"#;
    status 1;
);
test_full!(i8_mul => r#"
    export fn main = ExitCode(mul(i8(2), i8(1)));"#;
    status 2;
);
test_full!(i8_div => r#"
    export fn main = ExitCode(div(i8(6), i8(2)));"#;
    status 3;
);
test_full!(i8_mod => r#"
    export fn main = ExitCode(mod(i8(6), i8(4)));"#;
    status 2;
);
test_full!(i8_pow => r#"
    export fn main = ExitCode(pow(i8(6), i8(2)));"#;
    status 36;
);
test_full!(i8_min => r#"
    export fn main {
      print(min(i8(3), i8(5)));
    }"#;
    stdout "3\n";
);
test_full!(i8_max => r#"
    export fn main {
      print(max(i8(3), i8(5)));
    }"#;
    stdout "5\n";
);
test_full!(i8_neg => r#"
    export fn main = print(neg(i8(3)));"#;
    stdout "-3\n";
);

test_full!(i16_add => r#"
    export fn main {
      print(add(i16(1), i16(2)));
    }"#;
    stdout "3\n";
);
test_full!(i16_sub => r#"
    export fn main {
      print(sub(i16(2), i16(1)));
    }"#;
    stdout "1\n";
);
test_full!(i16_mul => r#"
    export fn main {
      print(mul(i16(2), i16(1)));
    }"#;
    stdout "2\n";
);
test_full!(i16_div => r#"
    export fn main {
      print(div(i16(6), i16(2)));
    }"#;
    stdout "3\n";
);
test_full!(i16_mod => r#"
    export fn main{
      print(mod(i16(6), i16(4)));
    }"#;
    stdout "2\n";
);
test_full!(i16_pow => r#"
    export fn main {
      print(pow(i16(6), i16(2)));
    }"#;
    stdout "36\n";
);
test_full!(i16_min => r#"
    export fn main {
      min(3.i16, 5.i16).print;
    }"#;
    stdout "3\n";
);
test_full!(i16_max => r#"
    export fn main {
      max(3.i16, 5.i16).print;
    }"#;
    stdout "5\n";
);
test_full!(i16_neg => r#"
    export fn main = print(-i16(3));"#;
    stdout "-3\n";
);

test_full!(i32_add => r#"
    export fn main {
      add(1.i32(), 2.i32()).print();
    }"#;
    stdout "3\n";
);
test_full!(i32_sub => r#"
    export fn main {
      sub(2.i32, 1.i32).print;
    }"#;
    stdout "1\n";
);
test_full!(i32_mul => r#"
    export fn main {
      (2.i32 * 1.i32).print;
    }"#;
    stdout "2\n";
);
test_full!(i32_div => r#"
    export fn main {
      (6.i32() / 2.i32()).print();
    }"#;
    stdout "3\n";
);
test_full!(i32_mod => r#"
    export fn main {
      mod(6.i32, 4.i32).print;
    }"#;
    stdout "2\n";
);
test_full!(i32_pow => r#"
    export fn main {
      pow(6.i32(), 2.i32()).print();
    }"#;
    stdout "36\n";
);
test_full!(i32_min => r#"
    export fn main {
      min(3.i32, 5.i32).print;
    }"#;
    stdout "3\n";
);
test_full!(i32_max => r#"
    export fn main {
      max(3.i32(), 5.i32()).print();
    }"#;
    stdout "5\n";
);
test_full!(i32_neg => r#"
    export fn main = print(- 3.i32);"#; // You wouldn't naturally write this, but should still work
    stdout "-3\n";
);

test_full!(i64_add => r#"
    export fn main = print(1 + 2);"#;
    stdout "3\n";
);
test_full!(i64_sub => r#"
    export fn main = print(2 - 1);"#;
    stdout "1\n";
);
test_full!(i64_mul => r#"
    export fn main = print(2 * 1);"#;
    stdout "2\n";
);
test_full!(i64_div => r#"
    export fn main = print(6 / 2);"#;
    stdout "3\n";
);
test_full!(i64_mod => r#"
    export fn main = print(6 % 4);"#;
    stdout "2\n";
);
test_full!(i64_pow => r#"
    export fn main = print(6 ** 2);"#;
    stdout "36\n";
);
test_full!(i64_min => r#"
    export fn main = min(3, 5).print;"#;
    stdout "3\n";
);
test_full!(i64_max => r#"
    export fn main = max(3.i64, 5.i64).print;"#;
    stdout "5\n";
);
test_full!(i64_neg => r#"
    export fn main = print(- 3);"#; // You wouldn't naturally write this, but should still work
    stdout "-3\n";
);

test_full!(u8_add => r#"
    export fn main() -> ExitCode = ExitCode(add(u8(1), u8(2)));"#;
    status 3;
);
test_full!(u8_sub => r#"
    export fn main() = ExitCode(sub(u8(2), u8(1)));"#;
    status 1;
);
test_full!(u8_mul => r#"
    export fn main() -> ExitCode = ExitCode(mul(u8(2), u8(1)));"#;
    status 2;
);
test_full!(u8_div => r#"
    export fn main() = ExitCode(div(u8(6), u8(2)));"#;
    status 3;
);
test_full!(u8_mod => r#"
    export fn main() -> ExitCode = ExitCode(mod(u8(6), u8(4)));"#;
    status 2;
);
test_full!(u8_pow => r#"
    export fn main() = ExitCode(pow(u8(6), u8(2)));"#;
    status 36;
);
test_full!(u8_min => r#"
    export fn main() {
      print(min(u8(3), u8(5)));
    }"#;
    stdout "3\n";
);
test_full!(u8_max => r#"
    export fn main() {
      print(max(u8(3), u8(5)));
    }"#;
    stdout "5\n";
);

test_full!(u16_add => r#"
    export fn main {
      print(add(u16(1), u16(2)));
    }"#;
    stdout "3\n";
);
test_full!(u16_sub => r#"
    export fn main {
      print(sub(u16(2), u16(1)));
    }"#;
    stdout "1\n";
);
test_full!(u16_mul => r#"
    export fn main {
      print(mul(u16(2), u16(1)));
    }"#;
    stdout "2\n";
);
test_full!(u16_div => r#"
    export fn main {
      print(div(u16(6), u16(2)));
    }"#;
    stdout "3\n";
);
test_full!(u16_mod => r#"
    export fn main{
      print(mod(u16(6), u16(4)));
    }"#;
    stdout "2\n";
);
test_full!(u16_pow => r#"
    export fn main {
      print(pow(u16(6), u16(2)));
    }"#;
    stdout "36\n";
);
test_full!(u16_min => r#"
    export fn main {
      min(3.u16, 5.u16).print;
    }"#;
    stdout "3\n";
);
test_full!(u16_max => r#"
    export fn main {
      max(3.u16, 5.u16).print;
    }"#;
    stdout "5\n";
);

test_full!(u32_add => r#"
    export fn main {
      add(1.u32(), 2.u32()).print();
    }"#;
    stdout "3\n";
);
test_full!(u32_sub => r#"
    export fn main {
      sub(2.u32, 1.u32).print;
    }"#;
    stdout "1\n";
);
test_full!(u32_mul => r#"
    export fn main {
      (2.u32 * 1.u32).print;
    }"#;
    stdout "2\n";
);
test_full!(u32_div => r#"
    export fn main {
      (6.u32() / 2.u32()).print();
    }"#;
    stdout "3\n";
);
test_full!(u32_mod => r#"
    export fn main {
      mod(6.u32, 4.u32).print;
    }"#;
    stdout "2\n";
);
test_full!(u32_pow => r#"
    export fn main {
      pow(6.u32(), 2.u32()).print();
    }"#;
    stdout "36\n";
);
test_full!(u32_min => r#"
    export fn main {
      min(3.u32, 5.u32).print;
    }"#;
    stdout "3\n";
);
test_full!(u32_max => r#"
    export fn main {
      max(3.u32(), 5.u32()).print();
    }"#;
    stdout "5\n";
);

test_full!(u64_add => r#"
    export fn main = print(1.u64 + 2.u64);"#;
    stdout "3\n";
);
test_full!(u64_sub => r#"
    export fn main = print(2.u64 - 1.u64);"#;
    stdout "1\n";
);
test_full!(u64_mul => r#"
    export fn main = print(2.u64 * 1.u64);"#;
    stdout "2\n";
);
test_full!(u64_div => r#"
    export fn main = print(6.u64 / 2.u64);"#;
    stdout "3\n";
);
test_full!(u64_mod => r#"
    export fn main = print(6.u64 % 4.u64);"#;
    stdout "2\n";
);
test_full!(u64_pow => r#"
    export fn main = print(6.u64 ** 2.u64);"#;
    stdout "36\n";
);
test_full!(u64_min => r#"
    export fn main = min(3.u64, 5.u64).print;"#;
    stdout "3\n";
);
test_full!(u64_max => r#"
    export fn main = max(3.u64, 5.u64).print;"#;
    stdout "5\n";
);

test_full!(f32_add => r#"
    export fn main {
      print(f32(1) + f32(2));
    }"#;
    stdout "3\n";
);
test_full!(f32_sub => r#"
    export fn main {
      print(f32(2) - f32(1));
    }"#;
    stdout "1\n";
);
test_full!(f32_mul => r#"
    export fn main {
      print(f32(2) * f32(1));
    }"#;
    stdout "2\n";
);
test_full!(f32_div => r#"
    export fn main {
      print(f32(6) / f32(2));
    }"#;
    stdout "3\n";
);
test_full!(f32_sqrt => r#"
    export fn main {
      print(sqrt(f32(36)));
    }"#;
    stdout "6\n";
);
test_full!(f32_pow => r#"
    export fn main {
      print(f32(6) ** f32(2));
    }"#;
    stdout "36\n";
);
test_full!(f32_min => r#"
    export fn main {
      min(3.f32, 5.f32).print;
    }"#;
    stdout "3\n";
);
test_full!(f32_max => r#"
    export fn main {
      max(3.f32, 5.f32).print;
    }"#;
    stdout "5\n";
);
test_full!(f32_neg => r#"
    export fn main = print(- 3.f32);"#; // You wouldn't naturally write this, but should still work
    stdout "-3\n";
);

test_full!(f64_add => r#"
    export fn main {
      (1.0 + 2.0).print;
    }"#;
    stdout "3\n";
);
test_full!(f64_sub => r#"
    export fn main {
      (2.0 - 1.0).print;
    }"#;
    stdout "1\n";
);
test_full!(f64_mul => r#"
    export fn main {
      (2.0 * 1.0).print;
    }"#;
    stdout "2\n";
);
test_full!(f64_div => r#"
    export fn main {
      (6.0 / 2.0).print;
    }"#;
    stdout "3\n";
);
test_full!(f64_sqrt => r#"
    export fn main {
      sqrt(36.0).print;
    }"#;
    stdout "6\n";
);
test_full!(f64_pow => r#"
    export fn main {
      (6.0 ** 2.0).print;
    }"#;
    stdout "36\n";
);
test_full!(f64_min => r#"
    export fn main {
      min(3.f64, 5.f64).print;
    }"#;
    stdout "3\n";
);
test_full!(f64_max => r#"
    export fn main {
      max(3.f64, 5.f64).print;
    }"#;
    stdout "5\n";
);
test_full!(f64_neg => r#"
    export fn main = print(- 3.f64);"#; // You wouldn't naturally write this, but should still work
    stdout "-3\n";
);

test_full!(grouping => r#"
    export fn main {
      print(2 / (3));
      print(3 / (1 + 2));
    }"#;
    stdout "0\n1\n";
);

test_full!(string_min => r#"
    export fn main {
      min(3.string, 5.string).print;
    }"#;
    stdout "3\n";
);
test_full!(string_max => r#"
    export fn main {
      max(3.string, 5.string).print;
    }"#;
    stdout "5\n";
);
test!(string_parse => r#"
    export fn main {
      "8".i8.print;
      "foo".i8.print;
      "16".i16.print;
      "foo".i16.print;
      "32".i32.print;
      "foo".i32.print;
      "64".i64.print;
      "foo".i64.print;
    }"#;
    stdout "8\nError: invalid digit found in string\n16\nError: invalid digit found in string\n32\nError: invalid digit found in string\n64\nError: invalid digit found in string\n";
);

// GPGPU

test!(hello_gpu => r#"
    export fn main {
      let b = GBuffer(filled(2.i32, 4));
      let plan = GPGPU("
        @group(0)
        @binding(0)
        var<storage, read_write> vals: array<i32>;

        @compute
        @workgroup_size(1)
        fn main(@builtin(global_invocation_id) id: vec3<u32>) {
          vals[id.x] = vals[id.x] * i32(id.x);
        }
      ", b);
      plan.run;
      b.read{i32}.print;
    }"#;
    stdout "[0, 2, 4, 6]\n";
);
test!(hello_gpu_new => r#"
    export fn main {
      let b = GBuffer(filled(2.i32, 4));
      let idx = gFor(4);
      let compute = b[idx].store(b[idx] * idx.gi32);
      compute.build.run;
      b.read{i32}.print;
    }"#;
    stdout "[0, 2, 4, 6]\n";
);

test!(hello_gpu_odd => r#"
    export fn main {
      let b = GBuffer(filled(2.i32, 4));
      let idx = gFor(4, 1);
      let compute = b[idx.i].store(b[idx.i] * idx.i.gi32 + 1);
      compute.build.run;
      b.read{i32}.print;
    }"#;
    stdout "[1, 3, 5, 7]\n";
);

test!(gpu_map => r#"
    export fn main {
        let b = GBuffer([1, 2, 3, 4]);
        let out = b.map(fn (val: gi32) = val + 2);
        out.read{i32}.print;
    }"#;
    stdout "[3, 4, 5, 6]\n";
);

test!(gpu_if => r#"
    export fn main {
        let b = GBuffer([1, 2, 3, 4]);
        let out = b.map(fn (val: gi32, i: gu32) = if(
            i % 2 == 0,
            val * i.gi32,
            val - i.gi32));
        out.read{i32}.print;
    }"#;
    stdout "[0, 1, 6, 1]\n";
);

// Bitwise Math

test_full!(i8_bitwise => r#"
    prefix i8 as ~ precedence 10

    export fn main {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
    }"#;
    stdout "0\n3\n6\n-1\n-1\n-4\n-7\n";
);
test_full!(i16_bitwise => r#"
    prefix i16 as ~ precedence 10

    export fn main {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
    }"#;
    stdout "0\n3\n6\n-1\n-1\n-4\n-7\n";
);
test_full!(i32_bitwise => r#"
    prefix i32 as ~ precedence 10

    export fn main {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
    }"#;
    stdout "0\n3\n6\n-1\n-1\n-4\n-7\n";
);
test_full!(i64_bitwise => r#"
    export fn main {
      print(1 & 2);
      print(1 | 3);
      print(5 ^ 3);
      print(!0);
      print(1 !& 2);
      print(1 !| 2);
      print(5 !^ 3);
    }"#;
    stdout "0\n3\n6\n-1\n-1\n-4\n-7\n";
);

test_full!(u8_bitwise => r#"
    prefix u8 as ~ precedence 10

    export fn main {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
    }"#;
    stdout "0\n3\n6\n255\n255\n252\n249\n";
);
test_full!(u16_bitwise => r#"
    prefix u16 as ~ precedence 10

    export fn main {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
    }"#;
    stdout "0\n3\n6\n65535\n65535\n65532\n65529\n";
);
test_full!(u32_bitwise => r#"
    prefix u32 as ~ precedence 10

    export fn main {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
    }"#;
    stdout "0\n3\n6\n4294967295\n4294967295\n4294967292\n4294967289\n";
);
test_full!(u64_bitwise => r#"
    prefix u64 as ~ precedence 10

    export fn main {
      print(~1 & ~2);
      print(~1 | ~3);
      print(~5 ^ ~3);
      print(! ~0);
      print(~1 !& ~2);
      print(~1 !| ~2);
      print(~5 !^ ~3);
    }"#;
    stdout "0\n3\n6\n18446744073709551615\n18446744073709551615\n18446744073709551612\n18446744073709551609\n";
);

// Boolean Logic

test!(boolean_logic => r#"
    export fn main {
      print(true);
      print(false);
      print(bool(1));
      print(bool(0));
      print(bool(15));
      print(bool(-1));
      print(bool(0.0));
      print(bool(1.2));
      print(bool(''));
      print(bool('hi'));

      print(true & true);
      print(and(true, false));
      print(false & true);
      print(false.and(false));

      print(true | true);
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
      false.nand(false).print;

      print(true !| true);
      print(nor(true, false));
      print(false !| true);
      false.nor(false).print;

      print(true !^ true);
      print(xnor(true, false));
      print(false !^ true);
      false.xnor(false).print;
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

test_full!(string_ops => r#"
    export fn main {
      concat('Hello, ', 'World!').print;

      repeat('hi ', 5).print;

      // TODO: Add regex support
      //matches('foobar', 'fo.*').print;
      //print('foobar' ~ 'fo.*');

      index('foobar', 'ba').print;

      len('foobar').print;
      print(#'foobar');

      trim('   hi   ').print;

      split('Hello, World!', ', ')[0].print;

      const res = "Hello, World!".split(', ');
      res[0].print;
    }"#;
    stdout r#"Hello, World!
hi hi hi hi hi 
3
6
6
hi
Hello
Hello
"#;
);
test_full!(string_const_vs_computed_equality => r#"
    export fn main {
      const foo = 'foo';
      print(foo.trim == foo);
    }"#;
    stdout "true\n";
);
test_full!(string_chars_direct => r#"
    export fn main {
        const foo = 'foo';
        print(#foo);
        print(foo[0]);
        print(foo[1]);
        print(foo[2]);
        print(foo[3]);
    }"#;
    stdout r#"3
f
o
o
Error: Index 3 is out-of-bounds for a string length of 3
"#;
);

/* Pending
test_ignore!(string_templating => r#"
    from @std/app import start, print, exit

    on start {
      template('\${greet}, \${name}!', new Map<string, string> {
        'greet': 'Hello'
        'name': 'World'
      }).print
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
    export fn main {
      print(i8(0) == i8(0));
      print(i8(1).eq(i8(0)));

      print(i16(0) == i16(0));
      print(i16(1).eq(i16(0)));

      print(i32(0) == i32(0));
      print(i32(1).eq(i32(0)));

      print(0 == 0);
      print(1.eq(0));

      print(u8(0) == u8(0));
      print(u8(1).eq(u8(0)));

      print(u16(0) == u16(0));
      print(u16(1).eq(u16(0)));

      print(u32(0) == u32(0));
      print(u32(1).eq(u32(0)));

      print(0.u64 == 0.u64);
      print(1.u64.eq(0.u64));

      print(f32(0.0) == f32(0.0));
      print(f32(1.2).eq(f32(0.0)));

      print(0.0 == 0.0);
      print(1.2.eq(0.0));

      print(true == true);
      print(true.eq(false));

      print('hello' == 'hello');
      print('hello'.eq('world'));
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
    export fn main {
      print(i8(0) != i8(0));
      print(i8(1).neq(i8(0)));

      print(i16(0) != i16(0));
      print(i16(1).neq(i16(0)));

      print(i32(0) != i32(0));
      print(i32(1).neq(i32(0)));

      print(0 != 0);
      print(1.neq(0));

      print(u8(0) != u8(0));
      print(u8(1).neq(u8(0)));

      print(u16(0) != u16(0));
      print(u16(1).neq(u16(0)));

      print(u32(0) != u32(0));
      print(u32(1).neq(u32(0)));

      print(0.u64 != 0.u64);
      print(1.u64.neq(0.u64));

      print(f32(0.0) != f32(0.0));
      print(f32(1.2).neq(f32(0.0)));

      print(0.0 != 0.0);
      print(1.2.neq(0.0));

      print(true != true);
      print(true.neq(false));

      print('hello' != 'hello');
      print('hello'.neq('world'));
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
    export fn main {
      print(i8(0) < i8(1));
      print(i8(1).lt(i8(0)));

      print(i16(0) < i16(1));
      print(i16(1).lt(i16(0)));

      print(i32(0) < i32(1));
      print(i32(1).lt(i32(0)));

      print(0 < 1);
      print(1.lt(0));

      print(u8(0) < u8(1));
      print(u8(1).lt(u8(0)));

      print(u16(0) < u16(1));
      print(u16(1).lt(u16(0)));

      print(u32(0) < u32(1));
      print(u32(1).lt(u32(0)));

      print(0.u64 < 1.u64);
      print(1.u64.lt(0.u64));

      print(f32(0.0) < f32(1.0));
      print(f32(1.2).lt(f32(0.0)));

      print(0.0 < 1.0);
      print(1.2.lt(0.0));

      print('hello' < 'hello');
      print('hello'.lt('world'));
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
true
false
true
false
false
true
"#;
);
test!(less_than_or_equal => r#"
    export fn main {
      print(i8(0) <= i8(1));
      print(i8(1).lte(i8(0)));

      print(i16(0) <= i16(1));
      print(i16(1).lte(i16(0)));

      print(i32(0) <= i32(1));
      print(i32(1).lte(i32(0)));

      print(0 <= 1);
      print(1.lte(0));

      print(u8(0) <= u8(1));
      print(u8(1).lte(u8(0)));

      print(u16(0) <= u16(1));
      print(u16(1).lte(u16(0)));

      print(u32(0) <= u32(1));
      print(u32(1).lte(u32(0)));

      print(0.u64 <= 1.u64);
      print(1.u64.lte(0.u64));

      print(f32(0.0) <= f32(1.0));
      print(f32(1.2).lte(f32(0.0)));

      print(0.0 <= 1.0);
      print(1.2.lte(0.0));

      print('hello' <= 'hello');
      print('hello'.lte('world'));
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
true
false
true
false
true
true
"#;
);
test!(greater_than => r#"
    export fn main {
      print(i8(0) > i8(1));
      print(i8(1).gt(i8(0)));

      print(i16(0) > i16(1));
      print(i16(1).gt(i16(0)));

      print(i32(0) > i32(1));
      print(i32(1).gt(i32(0)));

      print(0 > 1);
      print(1.gt(0));

      print(u8(0) > u8(1));
      print(u8(1).gt(u8(0)));

      print(u16(0) > u16(1));
      print(u16(1).gt(u16(0)));

      print(u32(0) > u32(1));
      print(u32(1).gt(u32(0)));

      print(0.u64 > 1.u64);
      print(1.u64.gt(0.u64));

      print(f32(0.0) > f32(1.0));
      print(f32(1.2).gt(f32(0.0)));

      print(0.0 > 1.0);
      print(1.2.gt(0.0));

      print('hello' > 'hello');
      print('hello'.gt('world'));
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
false
true
false
true
false
false
"#;
);
test!(greater_than_or_equal => r#"
    export fn main {
      print(i8(0) >= i8(1));
      print(i8(1).gte(i8(0)));

      print(i16(0) >= i16(1));
      print(i16(1).gte(i16(0)));

      print(i32(0) >= i32(1));
      print(i32(1).gte(i32(0)));

      print(0 >= 1);
      print(1.gte(0));

      print(u8(0) >= u8(1));
      print(u8(1).gte(u8(0)));

      print(u16(0) >= u16(1));
      print(u16(1).gte(u16(0)));

      print(u32(0) >= u32(1));
      print(u32(1).gte(u32(0)));

      print(0.u64 >= 1.u64);
      print(1.u64.gte(0.u64));

      print(f32(0.0) >= f32(1.0));
      print(f32(1.2).gte(f32(0.0)));

      print(0.0 >= 1.0);
      print(1.2.gte(0.0));

      print('hello' >= 'hello');
      print('hello'.gte('world'));
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
false
true
false
true
true
false
"#;
);

// Bitshifting/rotating
test!(bitshifting => r#"
    export fn main {
      print(1.i8 >> 1.i8);
      print(1.i8 << 1.i8);
      print(100.i8 >>> 2.i8);
      print(100.i8 <<< 2.i8);
      print(shr(1.i16, 1.i16));
      print(shl(1.i16, 1.i16));
      print(wrr(100.i16, 2.i16));
      print(wrl(100.i16, 2.i16));
      print(1.i32.shr(1.i32));
      print(1.i32.shl(1.i32));
      print(100.i32.wrr(2.i32));
      print(100.i32.wrl(2.i32));
      print(1 >> 1);
      print(1 << 1);
      print(100 >>> 2);
      print(100 <<< 2);
      print(1.u8 >> 1.u8);
      print(1.u8 << 1.u8);
      print(100.u8 >>> 2.u8);
      print(100.u8 <<< 2.u8);
      print(shr(1.u16, 1.u16));
      print(shl(1.u16, 1.u16));
      print(wrr(100.u16, 2.u16));
      print(wrl(100.u16, 2.u16));
      print(1.u32.shr(1.u32));
      print(1.u32.shl(1.u32));
      print(100.u32.wrr(2.u32));
      print(100.u32.wrl(2.u32));
      print(1.u64 >> 1.u64);
      print(1.u64 << 1.u64);
      print(100.u64 >>> 2.u64);
      print(100.u64 <<< 2.u64);
    }"#;
    stdout r#"0
2
25
-111
0
2
25
400
0
2
25
400
0
2
25
400
0
2
25
145
0
2
25
400
0
2
25
400
0
2
25
400
"#;
);

// Functions and Custom Operators

test_full!(basic_function_usage => r#"
    fn foo() = print('foo');

    fn bar(s: string) = s.concat("bar");

    export fn main {
      foo();
      'foo'.bar.print;
    }"#;
    stdout r#"foo
foobar
"#;
);

test_full!(functions_and_custom_operators => r#"
    fn foo() {
      print('foo');
    }

    fn bar(str: string, a: i64, b: i64) {
      return str.repeat(a).concat(b.string);
    }

    fn baz(pre: string, body: string) -> void {
      print(pre.concat(bar(body, 1, 2)));
    }

    fn double(a: i64) = a * 2;

    prefix double as ## precedence 10

    fn doublesum(a: i64, b: i64) = ##a + ##b

    infix doublesum as #+# precedence 11

    export fn main {
      foo();
      'to bar'.bar(2, 3).print;
      '>> '.baz('text here');
      4.double.print;
      print(##3);
      4.doublesum(1).print;
      print(2 #+# 3);
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

// TODO: Need to figure out how to get this working in JS-land
test!(mutable_functions => r#"
    fn addeq (a: Mut{i64}, b: i64) {
        a = a.clone() + b;
    }

    infix addeq as += precedence 0;

    export fn main {
        let five = 3;
        five.print;
        five += 2;
        five.print;
    }"#;
    stdout "3\n5\n";
);

// Conditionals

test_full!(if_fn => r#"
    export fn main {
        if(1 == 0, fn = print('What!?'), fn = print('Math is sane...'));
        if(1 == 2, fn = 'Uhh...').print;
        if(1 == 1, 'Correct!').print;
    }"#;
    stdout "Math is sane...\nvoid\nCorrect!\n";
);

test_ignore!(basic_conditionals => r#"
    fn bar() {
      print('bar!');
    }

    fn baz() {
      print('baz!');
    }

    export fn main {
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
    }"#;
    stdout r#"Math is sane...
Math is still sane, for now...
bar!
It's true!
"#;
);
test_ignore!(nested_conditionals => r#"
    export fn main {
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
    }"#;
    stdout "1\n2\n3\n";
);
test_ignore!(early_return => r#"
    fn nearOrFar(distance: float64) -> string {
      if distance < 5.0 {
        return 'Near!';
      } else {
        return 'Far!';
      }
    }

    export fn main {
      print(nearOrFar(3.14));
      print(nearOrFar(6.28));
    }"#;
    stdout "Near!\nFar!\n";
);
/* Dropping the ternary operators since either they behave consistently with other operators and
 * are therefore unexpected for end users, or they are inconsistent and a whole lot of pain is
 * needed to support them. */
test_ignore!(conditional_let_assignment => r#"
    export fn main {
      let a = 0;
      let b = 1;
      let c = 2;

      if true {
        a = b;
      } else {
        a = c;
      }
      print(a);
    }"#;
    stdout "1\n";
);

test_full!(conditional_compilation => r#"
    type{true} foo = string;
    type{false} foo = i64;

    const{true} var = "Hello, World!";
    const{false} var = 32;

    infix{true} add as + precedence 5;
    infix{false} add as + precedence 0;

    type infix{true} Add as + precedence 5;
    type infix{false} Add as + precedence 0;

    fn{true} bar = print(true);
    fn{false} bar = print(false);

    // type TestBuffer = Buffer{i64, 1 + 2 * 3};
    type TestBuffer = i64[1 + 2 * 3];

    export fn main {
      print(var.foo); // Should print "Hello, World!"
      print(1 + 2 * 3); // Should print 9 because + now has a higher precedence
      print(TestBuffer(0).len); // Should print 9 because the type operator + now has higher precedence
      bar(); // Should print "true"
    }"#;
    stdout "Hello, World!\n9\n9\ntrue\n";
);
test_full!(library_testing => r#"
    export fn add1(a: i64) -> i64 = a + 1;
    export postfix add1 as ++ precedence 5;

    export fn{Test} main {
      let a = 1;
      print(a++);
    }"#;
    stdout "2\n";
);

// Objects

test_compile_error!(object_constructor_compiler_checks => r#"
    type Foo =
      bar: string,
      baz: bool;

    export fn main {
      const foo = Foo(1.23);
    }"#;
    error "Could not find a function with a call signature of Foo(f64)";
);
test_full!(array_literals => r#"
    export fn main {
      const test3 = [ 1, 2, 4, 8, 16, 32, 64 ];
      print(test3[0]);
      print(test3[1]);
      print(test3[2]);
    }"#;
    stdout "1\n2\n4\n";
);
test_full!(object_literals => r#"
    type MyType =
      foo: string,
      bar: bool;

    export fn main {
      const test = MyType('foo!', true);
      print(test.foo);
      print(test.bar);
    }"#;
    stdout "foo!\ntrue\n";
);
test!(object_and_array_reassignment => r#"
    type Foo =
      bar: bool;

    export fn main {
      let test = [ 1, 2, 3 ];
      print(test[0]);
      test.store(0, 0);
      print(test[0]);
      test[0] = 2;
      print(test[0]);

      let test2 = [Foo(true), Foo(false)];
      let test3 = test2[0].getOr(Foo(false));
      print(test3.bar);
      test3.bar = false;
      test2[0] = test3; // TODO: is the a better way to do nested updates?
      const test4 = test2[0].getOr(Foo(true));
      print(test4.bar);
    }"#;
    stdout "1\n0\n2\ntrue\nfalse\n";
);

// Arrays

test!(array_accessor_and_length => r#"
    export fn main {
      print('Testing...');
      const test = '1,2,3'.split(',');
      print(test.len);
      print(test[0]);
      print(test[1]);
      print(test[2]);
    }"#;
    stdout r#"Testing...
3
1
2
3
"#;
);

test!(array_literal_syntax => r#"
    export fn main {
      print('Testing...');
      const test = Array{i64}(1, 2, 3);
      print(test[0]);
      print(test[1]);
      print(test[2]);
      const test2 = [ 4, 5, 6 ];
      print(test2[0]);
      print(test2[1]);
      print(test2[2]);
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
    export fn main {
      print('Testing...');
      let test = Array{i64}();
      test.push(1);
      test.push(2);
      test.push(3);
      print(test[0]);
      print(test[1]);
      print(test[2]);
      print(test.pop);
      print(test.pop);
      print(test.pop);
      print(test.pop); // Should print void
    }"#;
    stdout r#"Testing...
1
2
3
3
2
1
void
"#;
);
test!(array_has => r#"
    fn even(t: i64) = t % 2 == 0;
    fn odd(t: i64) = t % 2 == 1;
    export fn main {
        const test = [ 1, 1, 2, 3, 5, 8 ];
        test.has(3).print;
        test.has(4).print;
        test.has(even).print;
        test.has(odd).print;
    }"#;
    stdout "true\nfalse\ntrue\ntrue\n";
);
test!(array_map => r#"
    export fn main {
      const count = [1, 2, 3, 4, 5]; // Ah, ah, ahh!
      const byTwos = count.map(fn (n: i64) = n * 2);
      count.map(fn (n: i64) = string(n)).join(', ').print;
      byTwos.map(fn (n: i64) = string(n)).join(', ').print;
    }"#;
    stdout "1, 2, 3, 4, 5\n2, 4, 6, 8, 10\n";
);
test!(array_repeat => r#"
    export fn main {
      const arr = [1, 2, 3].repeat(3);
      const out = arr.map(fn (x: i64) = x.string).join(', ');
      print(out);
    }"#;
    stdout "1, 2, 3, 1, 2, 3, 1, 2, 3\n";
);
test!(array_find => r#"
    fn odd(x: i64) = x % 2 == 1;
    export fn main {
        const test = [ 1, 1, 2, 3, 5, 8 ];
        test.find(odd).getOr(0).print;
    }"#;
    stdout "1\n";
);
test!(array_every => r#"
    fn odd(x: i64) = x % 2 == 1;

    export fn main {
        const test = [ 1, 1, 2, 3, 5, 8 ];
        test.every(odd).print;
    }"#;
    stdout "false\n";
);
test!(array_some => r#"
    fn odd(x: i64) = x % 2 == 1;

    export fn main {
        const test = [ 1, 1, 2, 3, 5, 8 ];
        test.some(odd).print;
    }"#;
    stdout "true\n";
);
test!(array_index => r#"
    fn odd(x: i64) = x % 2 == 1;

    export fn main {
        const test = [ 1, 1, 2, 3, 5, 8 ];
        test.index(odd).print;
    }"#;
    stdout "0\n";
);
test!(array_concat => r#"
    export fn main {
        const test = [ 1, 1, 2, 3, 5, 8 ];
        const test2 = [ 4, 5, 6 ];
        test.concat(test2).map(string).join(', ').print;
    }"#;
    stdout "1, 1, 2, 3, 5, 8, 4, 5, 6\n";
);
test!(array_reduce_filter_concat => r#"
    export fn main {
      const test = [ 1, 1, 2, 3, 5, 8 ];
      const test2 = [ 4, 5, 6 ];
      print('reduce test');
      test.reduce(add).print;
      test.reduce(min).print;
      test.reduce(max).print;

      print('filter test');
      test.filter(fn isOdd(i: i64) = i % 2 == 1).map(string).join(', ').print;

      print('concat test');
      test.concat(test2).map(string).join(', ').print;
    }"#;
    stdout r#"reduce test
20
1
8
filter test
1, 1, 3, 5
concat test
1, 1, 2, 3, 5, 8, 4, 5, 6
"#;
);
test!(array_store_and_delete => r#"
    export fn main {
        const test = [ 1, 2, 5 ];
        test.store(2, 3);
        test[3] = 4;
        test.print;
        test.delete(4).print;
        test.print;
    }"#;
    stdout "[1, 2, 3, 4, 5]\n5\n[1, 2, 3, 4]\n";
);
test!(array_custom_types => r#"
    type Foo =
      foo: string,
      bar: bool;

    export fn main {
      const five = [1, 2, 3, 4, 5];
      five.map(fn (n: i64) {
        return Foo(n.string, n % 2 == 0);
      }).filter(fn (f: Foo) = f.bar).map(fn (f: Foo) = f.foo).join(', ').print;
    }"#;
    stdout "2, 4\n";
);
// Buffers
test!(buffer_map => r#"
    fn double(x: i64) = x * 2;
    export fn main {
        const b = Buffer{i64, 3}(1, 2, 3);
        b.print;
        b.len.print;
        b.map(double).print;
        b.map(add).print;
    }"#;
    stdout "[1, 2, 3]\n3\n[2, 4, 6]\n[1, 3, 5]\n";
);
test!(buffer_join => r#"
    export fn main {
        const b = {string[2]}("Hello", "World!");
        b.join(", ").print;
    }"#;
    stdout "Hello, World!\n";
);
test!(buffer_reduce => r#"
    fn concat(s: string, i: i64) = s.concat(i.string);
    export fn main {
        const b = {i64[5]}(1, 2, 3, 4, 5);
        b.reduce(add).print;
        b.reduce("0", concat).print;
    }"#;
    stdout "15\n012345\n";
);
test!(buffer_has => r#"
    fn even(t: i64) = t % 2 == 0;
    fn odd(t: i64) = t % 2 == 1;
    export fn main {
        const test = {i64[6]}(1, 1, 2, 3, 5, 8);
        test.has(3).print;
        test.has(4).print;
        test.has(even).print;
        test.has(odd).print;
    }"#;
    stdout "true\nfalse\ntrue\ntrue\n";
);
test!(buffer_find => r#"
    fn odd(x: i64) = x % 2 == 1;
    export fn main {
        const test = {i64[6]}(1, 1, 2, 3, 5, 8);
        test.find(odd).getOr(0).print;
    }"#;
    stdout "1\n";
);
test!(buffer_every => r#"
    fn odd(x: i64) = x % 2 == 1;

    export fn main {
        const test = {i64[6]}(1, 1, 2, 3, 5, 8);
        test.every(odd).print;
    }"#;
    stdout "false\n";
);
test!(buffer_concat => r#"
    export fn main {
        const test = {i64[6]}(1, 1, 2, 3, 5, 8);
        const test2 = {i64[3]}(4, 5, 6);
        test.concat(test2).map(string).join(', ').print;
    }"#;
    stdout "1, 1, 2, 3, 5, 8, 4, 5, 6\n";
);
test!(buffer_repeat => r#"
    export fn main {
      const buf = {i64[3]}(1, 2, 3).repeat(3);
      const out = buf.map(string).join(', ');
      print(out);
    }"#;
    stdout "1, 2, 3, 1, 2, 3, 1, 2, 3\n";
);
test!(buffer_store => r#"
    export fn main {
        let buf = {i64[3]}(1, 2, 5);
        print(buf);
        buf.store(2, 3).print;
        print(buf);
        buf[2] = 4;
        print(buf);
    }"#;
    stdout "[1, 2, 5]\n5\n[1, 2, 3]\n[1, 2, 4]\n";
);

// Hashing
test!(hash => r#"
    export fn main {
      print(hash(1));
      print(hash(3.14159));
      print(hash(true));
      print(hash('false'));
      print(hash([1, 2, 5, 3]));
    }"#;
    stdout r#"1742378985846435984
-3655443395552619065
4952851536318644461
-3294960077868127759
-8231513229892369092
"#;
);
test!(basic_dict => r#"
    export fn main {
      let test = Dict('foo', 1);
      // Equivalent to:
      // let test = Dict{string, i64}();
      // test.store('foo', 1);
      test.store('bar', 2);
      test['baz'] = 99;
      print(test.Array.map(fn (n: (string, i64)) {
        return 'key: '.concat(n.0).concat("\nval: ").concat(string(n.1));
      }).join("\n"));
      print(test.keys.join(', '));
      print(test.vals.map(string).join(', '));
      print(test.len);
      print(test.get('foo'));
      test['bar'].print;
      let test2 = Dict('foo', 3);
      test2['bay'] = 4;
      test.concat(test2).Array.map(fn (n: (string, i64)) {
        return 'key: '.concat(n.0).concat("\nval: ").concat(n.1.string);
      }).join("\n").print;
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
2
key: foo
val: 3
key: bar
val: 2
key: baz
val: 99
key: bay
val: 4
"#;
);
test!(keyval_array_to_dict => r#"
    export fn main {
      // TODO: Improve this with anonymous tuple support
      // const kva = [ (1, 'foo'), (2, 'bar'), (3, 'baz') ];
      const kva = [ {(i64, string)}(1, 'foo'), {(i64, string)}(2, 'bar'), {(i64, string)}(3, 'baz') ];
      const hm = Dict(kva);
      print(hm.Array.map(fn (n: (i64, string)) {
        return 'key: '.concat(string(n.0)).concat("\nval: ").concat(n.1);
      }).join("\n"));
      print(hm.get(1));
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
test!(dict_double_store => r#"
    export fn main {
      let test = Dict('foo', 'bar');
      test.get('foo').print;
      test.store('foo', 'baz');
      print(test.get('foo'));
    }"#;
    stdout "bar\nbaz\n";
);
test!(basic_set => r#"
    export fn main {
        let test = Set(0);
        test.len.print;
        test.has(0).print;
        test.has(1).print;
        test.store(1);
        test.len.print;
        let test2 = Set([1, 2]);
        test.union(test2).len.print;
        test.intersect(test2).Array.print;
        test.difference(test2).Array.print;
        test.symmetricDifference(test2).len.print;
        test.product(test2).len.print;
    }"#;
    stdout "1\ntrue\nfalse\n2\n3\n[1]\n[0]\n2\n4\n";
);

// Generics

test!(generics => r#"
    type box{V} =
      val: V,
      set: bool;

    export fn main {
      let i8Box = box{i8}(8.i8, true);
      print(i8Box.val);
      print(i8Box.set);

      let stringBox = box{string}('hello, generics!', true);
      print(stringBox.val);
      print(stringBox.set);

      const stringBoxBox = box{box{string}}(
        box{string}('hello, nested generics!', true),
        true
      );
      stringBoxBox.set.print;
      stringBoxBox.val.set.print;
      print(stringBoxBox.val.val);
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
test!(generic_functions => r#"
    fn empty{T}() = Array{T}(); // Pointless, but just for testing

    export fn main {
      let foo = empty{i64}();
      print(foo);
    }
"#;
    stdout "[]\n";
);
test_compile_error!(invalid_generics => r#"
    type box{V} =
      set: bool,
      val: V;

    export fn main {
      let stringBox = box{string}(true, 'str');
      stringBox.val = 8;
    }"#;
    error "Could not find a function with a call signature of store(string, i64)";
); // TODO: Make a better error message

// Interfaces

test_ignore!(basic_interfaces => r#"
    interface Stringifiable {
      string(Stringifiable) -> string
    }

    fn quoteAndPrint(toQuote: Stringifiable) {
      print(\"'\" + string(toQuote) + \"'\");
    }

    export fn main {
      quoteAndPrint('Hello, World');
      quoteAndPrint(5);
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

        export fn makeYear(year: int32) -> Year {
          return new Year {
            year: year
          };
        }

        export fn makeYear(year: int64) -> Year {
          return new Year {
            year: toInt32(year)
          };
        }

        export fn makeYearMonth(year: int32, month: int8) -> YearMonth {
          return new YearMonth {
            year: year,
            month: month
          };
        }

        export fn makeYearMonth(y: Year, month: int64) -> YearMonth {
          return new YearMonth {
            year: y.year,
            month: toInt8(month),
          };
        }

        export fn makeDate(year: int32, month: int8, day: int8) -> Date {
          return new Date {
            year: year,
            month: month,
            day: day,
          };
        }

        export fn makeDate(ym: YearMonth, day: int64) -> Date {
          return new Date {
            year: ym.year,
            month: ym.month,
            day: toInt8(day)
          };
        }

        export fn makeHour(hour: int8) -> Hour {
          return new Hour {
            hour: hour
          };
        }

        export fn makeHourMinute(hour: int8, minute: int8) -> HourMinute {
          return new HourMinute {
            hour: hour,
            minute: minute
          };
        }

        export fn makeHourMinute(hour: int64, minute: int64) -> HourMinute {
          return new HourMinute {
            hour: toInt8(hour),
            minute: toInt8(minute)
          };
        }

        export fn makeHourMinute(h: Hour, minute: int8) -> HourMinute {
          return new HourMinute {
            hour: h.hour,
            minute: minute
          };
        }

        export fn makeTime(hour: int8, minute: int8, second: float64) -> Time {
          return new Time {
            hour: hour,
            minute: minute,
            second: second
          };
        }

        export fn makeTime(hm: HourMinute, second: float64) -> Time {
          return new Time {
            hour: hm.hour,
            minute: hm.minute,
            second: second
          };
        }

        export fn makeTime(hm: HourMinute, second: int64) -> Time {
          return new Time {
            hour: hm.hour,
            minute: hm.minute,
            second: toFloat64(second)
          };
        }

        export fn makeTime(hm: Array{int64}, second: int64) -> Time {
          return new Time {
            hour: hm[0].i8,
            minute: hm[1].i8,
            second: second.f64
          };
        }

        export fn makeDateTime(date: Date, time: Time, timezone: HourMinute) -> DateTime {
          return new DateTime {
            date: date,
            time: time,
            timezone: timezone
          };
        }

        export fn makeDateTime(date: Date, time: Time) -> DateTime {
          return new DateTime {
            date: date,
            time: time,
            timezone: 00:00,
          };
        }

        export fn makeDateTimeTimezone(dt: DateTime, timezone: HourMinute) -> DateTime {
          return new DateTime {
            date: dt.date,
            time: dt.time,
            timezone: timezone
          };
        }

        export fn makeDateTimeTimezone(dt: DateTime, timezone: Array{int64}) -> DateTime {
          return new DateTime {
            date: dt.date,
            time: dt.time,
            timezone: new HourMinute {
              hour: timezone[0].i8,
              minute: timezone[1].i8,
            }
          };
        }

        export fn makeDateTimeTimezoneRev(dt: DateTime, timezone: HourMinute) -> DateTime {
          return new DateTime {
            date: dt.date,
            time: dt.time,
            timezone: new HourMinute {
              hour: timezone.hour.snegate,
              minute: timezone.minute
            }
          };
        }

        export fn makeDateTimeTimezoneRev(dt: DateTime, timezone: Array{int64}) -> DateTime {
          return new Datetime {
            date: dt.date,
            time: dt.time,
            timezone: new HourMinute {
              hour: toInt8(timezone[0]).snegate,
              minute: toInt8(timezone[1])
            }
          };
        }

        export fn print(dt: DateTime) {
          // TODO: Work on formatting stuff
          const timezoneOffsetSymbol = dt.timezone.hour < toInt8(0) ? \"-\" : \"+\";
          let str = (new Array{string} [
            string(dt.date.year), \"-\", string(dt.date.month), \"-\", string(dt.date.day), \"@\",
            string(dt.time.hour), \":\", string(dt.time.minute), \":\", string(dt.time.second),
            timezoneOffsetSymbol, sabs(dt.timezone.hour).string, \":\", string(dt.timezone.minute)
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
          print(DateTime) -> void,
        }
      "

      sourceToAll "
        from @std/app import start, print, exit
        from ./datetime import datetime

        on start {
          const dt = #2020 - 07 - 02@12:07:30 - 08:00;
          dt.print;
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

test!(maybe_exists => r#"
    export fn main {
        const maybe5 = Maybe{i64}(5);
        maybe5.exists.print;
        const intOrStr = {i64 | string}("It's a string!");
        intOrStr.i64.exists.print;
        intOrStr.string.exists.print;
    }"#;
    stdout "true\nfalse\ntrue\n";
);

test!(maybe => r#"
    // TODO: Rewrite these conditionals with conditional syntax once implemented
    fn fiver(val: f64) = if(val.i64 == 5, fn = {i64?}(5), fn = {i64?}());

    export fn main {
      const maybe5 = fiver(5.5);
      if(maybe5.exists,
        fn = maybe5.getOr(0).print,
        fn = 'what?'.print);

      const maybeNot5 = fiver(4.4);
      if(!maybeNot5.exists,
        fn = 'Correctly received nothing!'.print,
        fn = 'uhhh'.print);

      maybe5.print;
      maybeNot5.print;
    }"#;
    stdout r#"5
Correctly received nothing!
5
void
"#;
);
test!(fallible => r#"
    // TODO: Rewrite these conditionals with conditional syntax once implemented
    fn reciprocal(val: f64) = if(val == 0.0, fn {
      return Error{f64}('Divide by zero error!');
    }, fn {
      return Fallible{f64}(1.0 / val);
    });

    export fn main {
      const oneFifth = reciprocal(5.0);
      if(oneFifth.f64.exists,
        fn = print(oneFifth.getOr(0.0)),
        fn = print('what?'));

      const oneZeroth = reciprocal(0.0);
      if(oneZeroth.Error.exists,
        fn = print(oneZeroth.Error.getOr(Error('No error'))),
        fn = print('uhhh'));

      oneFifth.print;
      oneZeroth.print;

      const res = Fallible{string}('foo');
      print(res.Error.getOr(Error('there is no error')));
    }"#;
    stdout r#"0.2
Error: Divide by zero error!
0.2
Error: Divide by zero error!
Error: there is no error
"#;
);
test!(either => r#"
    type strOrI64 = string | i64;
    export fn main {
      const someStr = strOrI64('string');
      print(someStr.string);
      print(someStr.i64);
      print(someStr.getOr(0));
      print(someStr.getOr('text'));

      const someI64 = strOrI64(3);
      print(someI64.string);
      print(someI64.i64);
      print(someI64.getOr(0));
      print(someI64.getOr('text'));

      let either = strOrI64(3);
      either.string.print;
      either.i64.print;
      either = 'text';
      either.string.print;
      either.i64.print;
    }"#;
    stdout r#"string
void
0
string
void
3
3
text
void
3
text
void
"#;
);

// Types

test!(user_types_and_generics => r#"
    type foo{A, B} =
      bar: A,
      baz: B;

    type foo2 = foo{i64, f64};

    type tagged{A, B} =
      tag: A,
      value: B;

    type taggedInt = tagged{"integer", i64};

    export fn main {
      let a = foo{string, i64}('bar', 0);
      let b = foo{i64, bool}(0, true);
      let c = foo2(0, 1.23);
      let d = foo{i64, f64}(1, 3.14);
      let e = {i64?}(2);
      let f = taggedInt(5);
      print(a.bar);
      print(b.bar);
      print(c.bar);
      print(d.bar);
      print(e.i64);
      print(f.tag);
      print(f.value);
    }"#;
    stdout "bar\n0\n0\n1\n2\ninteger\n5\n";
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
              headers: newHashMap('Content-Length', arghStr.length.string),
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

// Closures

test_ignore!(closure_creation_and_usage => r#"
    fn closure() -> (() -> i64) {
      let num = 0;
      return fn () -> i64 {
        num = num + 1;
        return num;
      };
    }

    export fn main {
      const counter1 = closure();
      const counter2 = closure();
      print(counter1());
      print(counter1());
      print(counter2());
    }"#;
    stdout "1\n2\n1\n";
);
test!(closure_by_name => r#"
    fn double(x: i64) = x * 2;

    export fn main {
      const numbers = [1, 2, 3, 4, 5];
      numbers.map(double).map(string).join(', ').print;
    }"#;
    stdout "2, 4, 6, 8, 10\n";
);
test_ignore!(inlined_closure_with_arg => r#"
    export fn main {
      const arghFn = fn(argh: string) {
        print(argh);
      };
      arghFn('argh');
    }"#;
    stdout "argh\n";
);

// Compiler Errors

test_compile_error!(cross_type_comparisons => r#"
    export fn main {
      print(true == 1);
    }"#;
    error "Could not find a function with a call signature of eq(bool, i64)";
);
test_ignore!(unreachable_code => r#"
    fn unreachable() {
      return 'blah';
      print('unreachable!');
    }

    export fn main {
      unreachable();
    }"#;
    stderr r#"Unreachable code in function 'unreachable' after:
return 'blah'; on line 4:12
"#;
);
test_ignore!(recursive_functions => r#"
    fn fibonacci(n: int64) {
      if n < 2 {
        return 1;
      } else {
        return fibonacci(n - 1 || 0) + fibonacci(n - 2 || 0);
      }
    }

    export fn main {
      print(fibonacci(0));
      print(fibonacci(1));
      print(fibonacci(2));
      print(fibonacci(3));
      print(fibonacci(4));
    }"#;
    stderr "Recursive callstack detected: fibonacci -> fibonacci. Aborting.\n";
);
test_compile_error!(undefined_function_call => r#"
    export fn main {
      print(i64str(5)); // Illegal direct opcode usage
    }"#;
    error "Could not find a function with a call signature of i64str(i64)";
);
test_ignore!(totally_broken_statement => r#"
    on app.start {
      app.oops
    }"#;
    stderr "what";
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
    const helloWorld = 'Hello, World!';

    export fn main {
      print(helloWorld);
    }"#;
    stdout "Hello, World!\n";
);
test_ignore!(module_level_constant_from_function_call => r#"
    const three = add(1, 2);

    fn fiver() = 5;

    const five = fiver();

    export fn main {
      print(three);
      print(five);
    }"#;
    stdout "3\n5\n";
);

// @std/trig

test_ignore!(std_trig => r#"
    import @std/trig
    from @std/trig import e, pi, tau
    // shouldn't be necessary, but compiler issue makes it so

    export fn main {
      'Logarithms and e^x'.print;
      print(trig.exp(e));
      print(trig.ln(e));
      print(trig.log(e));

      'Basic Trig functions'.print;
      print(trig.sin(tau / 6.0));
      print(trig.cos(tau / 6.0));
      print(trig.tan(tau / 6.0));
      print(trig.sec(tau / 6.0));
      print(trig.csc(tau / 6.0));
      print(trig.cot(tau / 6.0));

      'Inverse Trig functions'.print;
      print(trig.arcsine(0.0));
      print(trig.arccosine(1.0));
      print(trig.arctangent(0.0));
      print(trig.arcsecant(tau / 6.0));
      print(trig.arccosecant(tau / 6.0));
      print(trig.arccotangent(tau / 6.0));

      'Historic Trig functions (useful for navigation and as a teaching aid: https://en.wikipedia.org/wiki/File:Circle-trig6.svg )'.print;
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

      'Historic Inverse Trig functions'.print;
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

      'Hyperbolic Trig functions'.print;
      print(trig.sinh(tau / 6.0));
      print(trig.cosh(tau / 6.0));
      print(trig.tanh(tau / 6.0));
      print(trig.sech(tau / 6.0));
      print(trig.csch(tau / 6.0));
      print(trig.coth(tau / 6.0));

      'Inverse Hyperbolic Trig functions'.print;
      print(trig.hyperbolicArcsine(tau / 6.0));
      print(trig.hyperbolicArccosine(tau / 6.0));
      print(trig.hyperbolicArctangent(tau / 6.0));
      print(trig.hyperbolicArcsecant(0.5));
      print(trig.hyperbolicArccosecant(tau / 6.0));
      print(trig.hyperbolicArccotangent(tau / 6.0));
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
            .add
          .block('@std/tcp')
          .fullBlock('@std/httpcommon')
          .commit
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
          print(res.isOk);
          const r = res.getOrExit;
          print(r.status);
          print(r.headers.length);
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
            res.body('Hello, World!').send;
          } else {
            res.body('Hello, Failure!').send;
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
          hm.get(key).getOr('failed').print;
          hm.get('something else').getOr('correct').print;
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
          const firstMessage = res.body('First Message').send;
          print(firstMessage);
          const secondMessage = res.body('Second Message').send;
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
    // TODO: Implement re-assignment statements
    export fn main {
      let a = 3;
      let b = a.clone;
      // a = 4;
      print(a);
      print(b);
      let c = [1, 2, 3];
      let d = c.clone;
      // d[0] = 2;
      c.map(string).join(', ').print;
      d.map(string).join(', ').print;
    }"#;
    // stdout "4\n3\n1, 2, 3\n2, 2, 3\n";
    stdout "3\n3\n1, 2, 3\n1, 2, 3\n";
);

// Runtime Error

test!(get_or_exit => r#"
    export fn main {
      const xs = [0, 1, 2, 5];
      const x1 = xs[1].getOrExit;
      print(x1);
      const x2 = xs[2].getOrExit;
      print(x2);
      const x5 = xs[5].getOrExit;
      print(x5);
    }"#;
    status 101;
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
          const baz = ns.ref('foo').run(fn (foo: string) = foo.length);
          print(baz);

          // Closure-based remote execution
          let bar = 'bar';
          const bay = ns.ref('foo').closure(fn (foo: string) -> int64 {
            bar = 'foobar: ' + foo + bar;
            return foo.length;
          });
          print(bay);
          print(bar);

          // Constrained-closure that only gets the 'with' variable
          const bax = ns.ref('foo').with(bar).run(fn (foo: string, bar: string) -> int64 = #foo +. #bar);
          print(bax);

          // Mutable closure
          const baw = ns.mut('foo').run(fn (foo: string) -> int64 {
            foo = foo + 'bar';
            return foo.length;
          });
          print(baw);

          // Mutable closure that affects the foo variable
          const bav = ns.mut('foo').closure(fn (foo: string) -> int64 {
            foo = foo + 'bar';
            bar = bar * foo.length;
            return bar.length;
          });
          print(bav);
          print(bar);

          // Constrained mutable closure that affects the foo variable
          const bau = ns.mut('foo').with(bar).run(fn (foo: string, bar: string) -> int64 {
            foo = foo * #bar;
            return foo.length;
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

test_ignore!(seq_and_next => r#"
    from @std/seq import seq, next

    export fn main {
      let s = seq(2);
      print(s.next);
      print(s.next);
      print(s.next);
    }"#;
    stdout "0\n1\nerror: sequence out-of-bounds\n";
);
test_ignore!(seq_each => r#"
    from @std/seq import seq, each

    export fn main {
      let s = seq(3);
      s.each(fn (i: int64) = print(i));
    }"#;
    stdout "0\n1\n2\n";
);
test_ignore!(seq_while => r#"
    from @std/seq import seq, while

    export fn main {
      let s = seq(100);
      let sum = 0;
      s.while(fn = sum < 10, fn {
        sum = sum + 1 || 0;
      });
      print(sum);
    }"#;
    stdout "10\n";
);
test_ignore!(seq_do_while => r#"
    from @std/seq import seq, doWhile

    export fn main {
      let s = seq(100);
      let sum = 0;
      // TODO: Get automatic type inference working on anonymous multi-line functions
      s.doWhile(fn () -> bool {
        sum = sum + 1 || 0;
        return sum < 10;
      });
      print(sum);
    }"#;
    stdout "10\n";
);
test_ignore!(seq_recurse => r#"
    from @std/seq import seq, Self, recurse

    export fn main {
      print(seq(100).recurse(fn fibonacci(self: Self, i: int64) -> Result{int64} {
        if i < 2 {
          return ok(1);
        } else {
          const prev = self.recurse(i - 1 || 0);
          const prevPrev = self.recurse(i - 2 || 0);
          if prev.isErr {
            return prev;
          }
          if prevPrev.isErr {
            return prevPrev;
          }
          // TODO: Get type inference inside of recurse working so we don't need to unwrap these
          return (prev || 0) + (prevPrev || 0);
        }
      }, 8));
    }"#;
    stdout "34\n";
);
test_ignore!(seq_no_op_one_liner_regression_test => r#"
    from @std/seq import seq, Self, recurse

    fn doNothing(x: int) : int = x;

    fn doNothingRec(x: int) : int = seq(x).recurse(fn (self: Self, x: int) : Result{int} {
        return ok(x);
    }, x) || 0;

    export fn main {
        const x = 5;
        print(doNothing(x)); // 5
        print(doNothingRec(x)); // 5

        const xs = [1, 2, 3];
        print(xs.map(doNothing).map(string).join(' ')); // 1 2 3
        print(xs.map(doNothingRec).map(string).join(' ')); // 1 2 3
    }"#;
    stdout "5\n5\n1 2 3\n1 2 3\n"; // TODO: Do we keep a regression test for a prior iteration?
);
test_ignore!(seq_recurse_decrement_regression_test => r#"
    from @std/seq import seq, Self, recurse

    fn triangularRec(x: int) : int = seq(x + 1 || 0).recurse(fn (self: Self, x: int) : Result{int} {
      if x == 0 {
        return ok(x);
      } else {
        // TODO: Get type inference inside of recurse working so we don't need to unwrap these
        return x + (self.recurse(x - 1 || 0) || 0);
      }
    }, x) || 0

    export fn main {
      const xs = [1, 2, 3];
      print(xs.map(triangularRec).map(string).join(' ')); // 1 3 6
    }"#;
    stdout "1 3 6\n"; // TODO: Same concern, do regression tests matter for a different codebase?
);

// Tree

test!(tree_construction_and_access => r#"
    export fn main {
      let myTree = Tree('foo');
      const barNode = myTree.addChild('bar');
      const bazNode = myTree.addChild('baz');
      const bayNode = barNode.addChild('bay');

      let secondTree = Tree('second');
      const secondNode = secondTree.addChild('node');

      bayNode.addChild(secondTree);

      print(myTree.rootNode.getOr('wrong'));
      // TODO: Need to dig in deeper in the codegen portion of the compiler
      //print(bayNode.parent.getOrExit.getOr('wrong'));
      //print(myTree.children.map(fn (c: Node{string}) -> string = c.getOr('wrong')).join(', '));
    }"#;
    //stdout "foo\nbar\nbar, baz\n";
    stdout "foo\n";
);
test_ignore!(tree_user_defined_types => r#"
    type Foo {
      foo: string,
      bar: bool,
    }

    export fn main {
      const myTree = newTree(new Foo {
        foo: 'myFoo',
        bar: false,
      });
      const wrongFoo = new Foo {
        foo: 'wrongFoo',
        bar: false,
      };
      const myFoo = myTree.getRootNode || wrongFoo;
      print(myFoo.foo);
    }"#;
    stdout "myFoo\n";
);
test_ignore!(tree_every_find_has_reduce_prune => r#"
    export fn main {
      const myTree = newTree('foo');
      const barNode = myTree.addChild('bar');
      const bazNode = myTree.addChild('baz');
      const bayNode = barNode.addChild('bay');

      print(myTree.every(fn (c: Node{string}) -> bool = (c || 'wrong').length == 3));
      print(myTree.has(fn (c: Node{string}) -> bool = (c || 'wrong').length == 1));
      print(myTree.find(fn (c: Node{string}) -> bool = (c || 'wrong') == 'bay').getOr('wrong'));
      print(myTree.find(fn (c: Node{string}) -> bool = (c || 'wrong') == 'asf').getOr('wrong'));

      print(myTree.length);
      myTree.getChildren.eachLin(fn (c: Node{string}) {
        const n = c || 'wrong';
        if n == 'bar' {
          c.prune;
        }
      });
      print(myTree.getChildren.map(fn (c: Node{string}) -> string = c || 'wrong').join(', '));
      print(myTree.length);

      myTree.reduce(fn (acc: int, i: Node{string}) -> int = (i || 'wrong').length + acc || 0, 0).print;
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
test_ignore!(subtree_and_nested_tree_construction => r#"
    export fn main {
      const bigNestedTree = newTree('foo')
        .addChild('bar')
        .getTree
        .addChild(newTree('baz')
          .addChild('quux')
          .getTree
        ).getTree;

      const mySubtree = bigNestedTree
        .getRootNode
        .getChildren[1]
        .getOr(newTree('what').getRootNode)
        .toSubtree;

      print(bigNestedTree.getRootNode || 'wrong');
      print(mySubtree.getRootNode || 'wrong');
    }"#;
    stdout "foo\nbaz\n";
);

// Error printing

test!(eprint => r#"
    export fn main {
      eprint('This is an error');
    }"#;
    stderr "This is an error\n";
);

// @std/cmd

test_ignore!(cmd_exec => r#"
    import @std/cmd

    export fn main {
      const executionResult: cmd.ExecRes = cmd.exec('echo 1');
      print(executionResult.stdout);
    }"#;
    stdout "1\n";
);
test_ignore!(cmd_sequential => r#"
    from @std/cmd import exec

    export fn main {
      exec('touch test.txt');
      exec('echo foo >> test.txt');
      exec('echo bar >> test.txt');
      exec('cat test.txt').stdout.print;
      exec('rm test.txt');
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

test_ignore!(json_construction_printing => r#"
    from @std/json import JSON, toJSON, string, JSONBase, JSONNode, IsObject, Null

    export fn main {
      1.0.toJSON.print;
      true.toJSON.print;
      'Hello, JSON!'.toJSON.print;
      [1.0, 2.0, 5.0].toJSON.print;
      toJSON.print;
    }"#;
    stdout r#"1
true
"Hello, JSON!"
[1, 2, 5]
null
"#;
);
test_ignore!(json_complex_construction => r#"
    from @std/json import JSON, string, JSONBase, JSONNode, IsObject, Null, newJSONObject, newJSONArray, addKeyVal, push

    export fn main {
      newJSONObject()
        .addKeyVal('mixed', 'values')
        .addKeyVal('work', true)
        .addKeyVal('even', newJSONArray()
          .push(4.0)
          .push('arrays'))
        .print;
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
          channel.ready;
          tunnel.ready;
        }

        on chunk fn (ctx: TcpContext{TcpChannel}) {
          ctx.context.write(ctx.channel.read);
        }

        on tcpClose fn (ctx: TcpContext{TcpChannel}) {
          ctx.context.close;
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
