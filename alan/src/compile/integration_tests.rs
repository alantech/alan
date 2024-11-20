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
                crate::program::Program::set_target_lang_rs();
                let filename = format!("{}.ln", stringify!($rule));
                match std::fs::write(&filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", filename, e).into());
                    }
                };
                let mut program = crate::program::Program::get_program();
                program.env.insert("ALAN_TARGET".to_string(), "test".to_string());
                crate::program::Program::return_program(program);
                match crate::compile::build(filename.to_string()) {
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
                $( $type!($test_val, true, &run); )+
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
                crate::program::Program::set_target_lang_rs();
                let filename = format!("{}.ln", stringify!($rule));
                match std::fs::write(&filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", filename, e).into());
                    }
                };
                let mut program = crate::program::Program::get_program();
                program.env.insert("ALAN_TARGET".to_string(), "test".to_string());
                crate::program::Program::return_program(program);
                match crate::compile::build(filename.to_string()) {
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
                $( $type!($test_val, true, &run); )+
                match std::fs::remove_file(&cmd) {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not remove the test binary {:?}", e)),
                }?;
                crate::program::Program::set_target_lang_js();
                let mut program = crate::program::Program::get_program();
                program.env.insert("ALAN_TARGET".to_string(), "test".to_string());
                crate::program::Program::return_program(program);
                match crate::compile::web(filename.to_string()) {
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
                $( $type!($test_val, false, &run); )+
                match std::fs::remove_file(&cmd) {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not remove the generated JS file {:?}", e)),
                }?;
                std::fs::remove_file(&filename)?;
                Ok(())
            }
        }
    };
    ( $rule:ident $entryfile:expr => $( $filename:expr => $code:expr),+ ; $( $type:ident $test_val:expr);+ $(;)? ) => {
        #[cfg(test)]
        mod $rule {
            #[test]
            fn $rule() -> Result<(), Box<dyn std::error::Error>> {
                crate::program::Program::set_target_lang_rs();
                $( match std::fs::write($filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", $filename, e).into());
                    }
                })+
                let mut program = crate::program::Program::get_program();
                program.env.insert("ALAN_TARGET".to_string(), "test".to_string());
                crate::program::Program::return_program(program);
                match crate::compile::build(format!("{}.ln", $entryfile)) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        $( std::fs::remove_file($filename)?; )+
                        return Err(format!("Failed to compile {:?}", e).into());
                    }
                };
                let cmd = if cfg!(windows) {
                    format!(".\\{}.exe", $entryfile)
                } else {
                    format!("./{}", $entryfile)
                };
                let run = match std::process::Command::new(cmd.clone()).output() {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not run the test binary {:?}", e)),
                }?;
                $( $type!($test_val, true, &run); )+
                match std::fs::remove_file(&cmd) {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not remove the test binary {:?}", e)),
                }?;
                crate::program::Program::set_target_lang_js();
                let mut program = crate::program::Program::get_program();
                program.env.insert("ALAN_TARGET".to_string(), "test".to_string());
                crate::program::Program::return_program(program);
                match crate::compile::web(format!("{}.ln", $entryfile)) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        $( std::fs::remove_file($filename)?; )+
                        return Err(format!("Failed to compile {:?}", e).into());
                    }
                };
                let cmd = if cfg!(windows) {
                    format!(".\\{}.js", $entryfile)
                } else {
                    format!("./{}.js", $entryfile)
                };
                let run = match std::process::Command::new("node").arg(cmd.to_string()).output() {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not run the test JS code {:?}", e)),
                }?;
                $( $type!($test_val, false, &run); )+
                match std::fs::remove_file(&cmd) {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not remove the generated JS file {:?}", e)),
                }?;
                $( std::fs::remove_file($filename)?; )+
                Ok(())
            }
        }
    };
}
macro_rules! test_gpgpu {
    ( $rule: ident => $code:expr; $( $type:ident $test_val:expr);+ $(;)? ) => {
        #[cfg(test)]
        mod $rule {
            #[test]
            fn $rule() -> Result<(), Box<dyn std::error::Error>> {
                crate::program::Program::set_target_lang_rs();
                let filename = format!("{}.ln", stringify!($rule));
                match std::fs::write(&filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", filename, e).into());
                    }
                };
                let mut program = crate::program::Program::get_program();
                program.env.insert("ALAN_TARGET".to_string(), "test".to_string());
                crate::program::Program::return_program(program);
                match crate::compile::build(filename.to_string()) {
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
                $( $type!($test_val, true, &run); )+
                match std::fs::remove_file(&cmd) {
                    Ok(a) => Ok(a),
                    Err(e) => Err(format!("Could not remove the test binary {:?}", e)),
                }?;
                // TODO: For now, Chromium only allows WebGPU on these two platforms (unless you're
                // willing to muck about with CLI arguments *and* config flags simultaneously to
                // enable it for Linux, which Playwright doesn't even support...
                // My playwright scripts only work on Linux and MacOS, though, so that reduces it
                // to just MacOS to test this on.
                // if cfg!(windows) || cfg!(macos) {
                if cfg!(target_os = "macos") {
                    crate::program::Program::set_target_lang_js();
                    let mut program = crate::program::Program::get_program();
                    program.env.insert("ALAN_TARGET".to_string(), "test".to_string());
                    crate::program::Program::return_program(program);
                    match crate::compile::web(filename.to_string()) {
                        Ok(_) => { /* Do nothing */ }
                        Err(e) => {
                            std::fs::remove_file(&filename)?;
                            return Err(format!("Failed to compile {:?}", e).into());
                        }
                    };
                    let jsfile = if cfg!(windows) {
                        format!(".\\{}.js", stringify!($rule))
                    } else {
                        format!("./{}.js", stringify!($rule))
                    };
                    // We need to create an HTML file that will run the generated code and a node
                    // script to fire up Playwright and grab the console.log output and shove it
                    // into stdout for the rest of the test suite to grab. Because the outermost
                    // directory of this repo is simultaneously a Rust and Node project, we're
                    // taking advantage of that to have the latter parts pre-written, but we can't
                    // do that for the HTML file because the script it loads is different for each
                    // test.
                    let htmlfile = if cfg!(windows) {
                        format!(".\\{}.html", stringify!($rule))
                    } else {
                        format!("./{}.html", stringify!($rule))
                    };
                    match std::fs::write(&htmlfile, format!("
                        <!doctype html>
                        <html>
                            <head>
                                <title>Testing {}</title>
                                <script src=\"{}\"></script>
                            </head>
                            <body></body>
                        </html>
                    ", stringify!($rule), jsfile)) {
                        Ok(_) => { /* Do nothing */ }
                        Err(e) => {
                            std::fs::remove_file(&filename)?;
                            return Err(format!("Failed to create temporary HTML file {:?}", e).into());
                        }
                    };
                    // We're assuming an HTTP server is already running
                    let run = match std::process::Command::new("bash")
                        .arg("-c")
                        .arg(format!("yarn -s chrome-console http://localhost:8080/alan/{}.html", stringify!($rule)))
                        .output() {
                            Ok(a) => Ok(a),
                            Err(e) => Err(format!("Could not run the test JS code {:?}", e)),
                        }?;
                    $( $type!($test_val, false, &run); )+
                    match std::fs::remove_file(&jsfile) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the generated JS file {:?}", e)),
                    }?;
                    match std::fs::remove_file(&htmlfile) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the generated HTML file {:?}", e)),
                    }?;
                }
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
                // Needs to run at least the Rust path so it properly fails on `main`
                let filename = format!("{}.ln", stringify!($rule));
                match std::fs::write(&filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", filename, e).into());
                    }
                };
                let mut program = crate::program::Program::get_program();
                program.env.insert("ALAN_TARGET".to_string(), "test".to_string());
                crate::program::Program::return_program(program);
                match crate::compile::build(filename.to_string()) {
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
                $( $type!($test_val, true, &run); )+
                std::fs::remove_file(&filename)?;
                std::fs::remove_file(&cmd)?;
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
    ( $test_val:expr, $in_rs:expr, $real_val:expr ) => {
        let std_out = if cfg!(windows) {
            String::from_utf8($real_val.stdout.clone())?.replace("\r\n", "\n")
        } else {
            String::from_utf8($real_val.stdout.clone())?
        };
        assert_eq!($test_val, &std_out);
    };
}
#[cfg(test)]
macro_rules! stdout_rs {
    ( $test_val:expr, $in_rs:expr, $real_val:expr ) => {
        if $in_rs {
            let std_out = if cfg!(windows) {
                String::from_utf8($real_val.stdout.clone())?.replace("\r\n", "\n")
            } else {
                String::from_utf8($real_val.stdout.clone())?
            };
            assert_eq!($test_val, &std_out);
        }
    };
}
#[cfg(test)]
macro_rules! stdout_js {
    ( $test_val:expr, $in_rs:expr, $real_val:expr ) => {
        if !$in_rs {
            let std_out = if cfg!(windows) {
                String::from_utf8($real_val.stdout.clone())?.replace("\r\n", "\n")
            } else {
                String::from_utf8($real_val.stdout.clone())?
            };
            assert_eq!($test_val, &std_out);
        }
    };
}
#[cfg(test)]
macro_rules! stdout_contains {
    ( $test_val:expr, $in_rs:expr, $real_val:expr ) => {
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
    ( $test_val:expr, $in_rs:expr, $real_val:expr ) => {
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
    ( $test_val:expr, $in_rs:expr, $real_val:expr ) => {
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

test_full!(normal_exit_code => r#"
    export fn main() -> ExitCode {
        return ExitCode(0);
    }"#;
    status 0;
);
test_full!(error_exit_code => r#"
    export fn main() = ExitCode(1);"#;
    status 1;
);
test_full!(non_global_memory_exit_code => r#"
    export fn main() {
      let x: i64 = 0;
      return x.ExitCode;
    }"#;
    status 0;
);

// TODO: There's no way to check equality of the `void` type, only printing allows this right now
test_full!(void_values => r#"
    export fn main {
        5.print;
        5.void.print;
        void().print; // TODO: `void.print` should work, too. Figure out why it isn't
    }"#;
    stdout "5\nvoid\nvoid\n";
);

// Printing Tests

test_full!(print_function => r#"
    export fn main() {
      print('Hello, World');
      return ExitCode(0);
    }"#;
    stdout "Hello, World\n";
    status 0;
);
test_full!(duration_print => r#"
    export fn main() -> void {
        const i = now();
        wait(100); // Increased from 10ms to 100ms because the node.js event loop seems less
                   // capable of guaranteeing staying below 20ms in the delay here.
        const d = i.elapsed;
        print(d);
    }"#;
    stdout_contains "0.1";
);
test_full!(print_compile_time_string => r#"
    type FooBar = Concat{"Foo", "Bar"};
    export fn main {
      {FooBar}().print;
    }"#;
    stdout "FooBar\n";
);
test_full!(stdout_and_stderr => r#"
    export fn main {
      let hello = 'Hello';
      let goodbye = 'Goodbye';
      let comma = ', ';
      let world = 'World';
      let end = '!\n';

      stdout(hello);
      stdout(comma);
      stdout(world);
      stdout(end);
      goodbye.stderr;
      comma.stderr;
      world.stderr;
      end.stderr;
    }"#;
    stdout "Hello, World!\n";
    stderr "Goodbye, World!\n";
);

// TODO: Unify the string output for these two so it can be tested more reliably
test_full!(string_parse => r#"
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
    stdout_rs "8\nError: invalid digit found in string\n16\nError: invalid digit found in string\n32\nError: invalid digit found in string\n64\nError: invalid digit found in string\n";
    stdout_js "8\nError: Not a Number\n16\nError: Not a Number\n32\nError: Not a Number\n64\nError: Cannot convert foo to a BigInt\n";
);

// GPGPU

test_gpgpu!(hello_gpu => r#"
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
test_gpgpu!(hello_gpu_new => r#"
    export fn main {
      let b = GBuffer(filled(2.i32, 4));
      let idx = gFor(4);
      let compute = b[idx].store(b[idx] * idx.gi32);
      compute.build.run;
      b.read{i32}.print;
    }"#;
    stdout "[0, 2, 4, 6]\n";
);

test_gpgpu!(hello_gpu_odd => r#"
    export fn main {
      let b = GBuffer(filled(2.i32, 4));
      let idx = gFor(4, 1);
      let compute = b[idx.i].store(b[idx.i] * idx.i.gi32 + 1);
      compute.build.run;
      b.read{i32}.print;
    }"#;
    stdout "[1, 3, 5, 7]\n";
);

test_gpgpu!(gpu_map => r#"
    export fn main {
        let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32]);
        let out = b.map(fn (val: gi32) = val + 2);
        out.read{i32}.print;
    }"#;
    stdout "[3, 4, 5, 6]\n";
);

test_gpgpu!(gpu_if => r#"
    export fn main {
        let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32]);
        let out = b.map(fn (val: gi32, i: gu32) = if(
            i % 2 == 0,
            val * i.gi32,
            val - i.gi32));
        out.read{i32}.print;
    }"#;
    stdout "[0, 1, 6, 1]\n";
);

test_gpgpu!(gpu_replace => r#"
    export fn main {
        let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32]);
        b.map(fn (val: gi32) = val + 2).read{i32}.print;
        b.replace([2.i32, 4.i32, 6.i32, 8.i32]);
        b.map(fn (val: gi32) = val / 2).read{i32}.print;
    }"#;
    stdout "[3, 4, 5, 6]\n[1, 2, 3, 4]\n";
);

test_gpgpu!(gpu_abs => r#"
    export fn main {
        let b = GBuffer([1.i32, -2.i32, -3.i32, 4.i32]);
        b.map(fn (val: gi32) = val.abs).read{i32}.print;
    }"#;
    stdout "[1, 2, 3, 4]\n";
);

test_gpgpu!(gpu_clz => r#"
    export fn main {
        let b = GBuffer([1.i32, -2.i32, -3.i32, 4.i32]);
        b.map(fn (val: gi32) = val.clz).read{i32}.print;
    }"#;
    stdout "[31, 0, 0, 29]\n";
);

// TODO: Fix u64 numeric constants to get u64 bitwise tests in the new test suite
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

test_full!(mutable_functions => r#"
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
test_full!(object_and_array_reassignment => r#"
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

test_full!(array_custom_types => r#"
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
test_full!(basic_dict => r#"
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
test_full!(keyval_array_to_dict => r#"
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
test_full!(dict_double_store => r#"
    export fn main {
      let test = Dict('foo', 'bar');
      test.get('foo').print;
      test.store('foo', 'baz');
      print(test.get('foo'));
    }"#;
    stdout "bar\nbaz\n";
);
test_full!(basic_set => r#"
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

test_full!(generics => r#"
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
test_full!(generic_functions => r#"
    fn empty{T}() = Array{T}(); // Pointless, but just for testing

    export fn main {
      let foo = empty{i64}();
      print(foo);
    }
"#;
    stdout "[]\n";
);
test_full!(generic_in_a_generic => r#"
    fn condition{T}(a: T, b: T) -> bool {
      return a == b;
    }

    fn batchCompare{T}(a: Array{T}, b: Array{T}, cond: (T, T) -> bool) {
      return a.map(fn (aVal: T) = b.some(fn (bVal: T) = cond(aVal, bVal)));
    }

    export fn main {
      let vals1 = [1, 9, 1];
      let vals2 = [1, 2, 3, 5, 7];

      batchCompare(vals1, vals2, condition).print;
    }"#;
    stdout_js "[ true, false, true ]\n";
    stdout_rs "[true, false, true]\n"; // TODO: Make these match
);
test_full!(first_arg_generic_fn => r#"
    fn batchCompare{T}(cond: (T, T) -> bool, a: Array{T}, b: Array{T}) {
      return a.map(fn (aVal: T) = b.some(fn (bVal: T) = cond(aVal, bVal)));
    }

    export fn main {
      let vals1 = [1, 9, 25];
      let vals2 = [1, 3, 5, 7, 9];

      batchCompare(eq, vals1, vals2).print;
    }"#;
    stdout_js "[ true, true, false ]\n";
    stdout_rs "[true, true, false]\n";
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

// TODO: Since tests are executed "in parallel", the files created by the tests can't match. This
// should be automatically scoped to separate test directories at some point when I can affect the
// PWD without using the thread-unsafe std::env for it. For now, these two tests that create
// multiple test files with manual naming just have to have different filenames.
test_full!(basic_type_import "type_foo" =>
    "type_bar.ln" => r#"
        export type Bar = "Bar";
    "#,
    "type_foo.ln" => r#"
        type Bar <-- "./type_bar.ln";

        export fn main {
            {Bar}().print;
        }
    "#;
    stdout "Bar\n";
);

test_full!(basic_fn_import "fn_foo" =>
    "fn_bar.ln" => r#"
        export fn bar = "Bar";
    "#,
    "fn_foo.ln" => r#"
        fn bar <-- "./fn_bar.ln";

        export fn main {
            bar().print;
        }
    "#;
    stdout "Bar\n";
);

// Maybe, Result, and Either

test_full!(maybe_exists => r#"
    export fn main {
        const maybe5 = Maybe{i64}(5);
        maybe5.exists.print;
        const intOrStr = {i64 | string}("It's a string!");
        intOrStr.i64.exists.print;
        intOrStr.string.exists.print;
    }"#;
    stdout "true\nfalse\ntrue\n";
);

test_full!(maybe => r#"
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
test_full!(fallible => r#"
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
test_full!(either => r#"
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

test_full!(user_types_and_generics => r#"
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

// Module-level constants

test_full!(module_level_constant => r#"
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

// Trigonometry

test_full!(cpu_trig => r#"
    export fn main {
      'Logarithms and e^x'.print;
      print(exp(e).string(4));
      print(ln(e).string(4));
      print(log10(e).string(4));
      print(log2(e).string(4));

      'Basic Trig functions'.print;
      print(sin(tau / 6.0).string(4));
      print(cos(tau / 6.0).string(4));
      print(tan(tau / 6.0).string(4));
      print(sec(tau / 6.0).string(4));
      print(csc(tau / 6.0).string(4));
      print(cot(tau / 6.0).string(4));

      'Inverse Trig functions'.print;
      asin(0.0).string(4).print;
      acos(1.0).string(4).print;
      atan(0.0).string(4).print;
      atan2(1.0, 2.0).string(4).print;
      print(asec(tau / 6.0).string(4));
      print(acsc(tau / 6.0).string(4));
      print(acot(tau / 6.0).string(4));

      'Hyperbolic Trig functions'.print;
      print(sinh(tau / 6.0).string(4));
      print(cosh(tau / 6.0).string(4));
      print(tanh(tau / 6.0).string(4));
      print(sech(tau / 6.0).string(4));
      print(csch(tau / 6.0).string(4));
      print(coth(tau / 6.0).string(4));

      'Inverse Hyperbolic Trig functions'.print;
      print(asinh(tau / 6.0).string(4));
      print(acosh(tau / 6.0).string(4));
      print(atanh(pi / 6.0).string(4));
      print(asech(0.5).string(4));
      print(acsch(tau / 6.0).string(4));
      print(acoth(tau / 6.0).string(4));
    }"#;
    stdout r#"Logarithms and e^x
15.1543
1.0000
0.4343
1.4427
Basic Trig functions
0.8660
0.5000
1.7321
2.0000
1.1547
0.5774
Inverse Trig functions
0.0000
0.0000
0.0000
0.4636
0.3014
1.2694
0.7623
Hyperbolic Trig functions
1.2494
1.6003
0.7807
0.6249
0.8004
1.2809
Inverse Hyperbolic Trig functions
0.9144
0.3060
0.5813
1.3170
0.8491
1.8849
"#;
);

test_gpgpu!(gpu_trig => r#"
    export fn main {
      'Logarithms and e^x'.print;
      // Contrived way to get the GPU to do this work, don't follow this pattern for real GPU usage
      GBuffer([e.f32]).map(fn (v: gf32) = exp(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([e.f32]).map(fn (v: gf32) = ln(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([e.f32]).map(fn (v: gf32) = log10(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([e.f32]).map(fn (v: gf32) = log2(v)).read{f32}[0].getOrExit.string(2).print;

      'Basic Trig functions'.print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = sin(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = cos(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = tan(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = sec(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = csc(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = cot(v)).read{f32}[0].getOrExit.string(2).print;

      'Inverse Trig functions'.print;
      GBuffer([0.0.f32]).map(fn (v: gf32) = asin(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([1.0.f32]).map(fn (v: gf32) = acos(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([0.0.f32]).map(fn (v: gf32) = atan(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([1.0.f32]).map(fn (v: gf32) = atan2(v, 2.0)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = asec(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = acsc(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = acot(v)).read{f32}[0].getOrExit.string(2).print;

      'Hyperbolic Trig functions'.print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = sinh(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = cosh(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = tanh(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = sech(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = csch(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = coth(v)).read{f32}[0].getOrExit.string(2).print;

      'Inverse Hyperbolic Trig functions'.print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = asinh(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = acosh(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([pi.f32 / 6.0.f32]).map(fn (v: gf32) = atanh(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([0.5.f32]).map(fn (v: gf32) = asech(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = acsch(v)).read{f32}[0].getOrExit.string(2).print;
      GBuffer([tau.f32 / 6.0.f32]).map(fn (v: gf32) = acoth(v)).read{f32}[0].getOrExit.string(2).print;
    }"#;
    stdout r#"Logarithms and e^x
15.15
1.00
0.43
1.44
Basic Trig functions
0.87
0.50
1.73
2.00
1.15
0.58
Inverse Trig functions
0.00
0.00
0.00
0.46
0.30
1.27
0.76
Hyperbolic Trig functions
1.25
1.60
0.78
0.62
0.80
1.28
Inverse Hyperbolic Trig functions
0.91
0.31
0.58
1.32
0.85
1.88
"#;
);

// Clone

test_full!(clone => r#"
    export fn main {
      let a = 3;
      let b = a.clone;
      a = 4;
      print(a);
      print(b);
      let c = [1, 2, 3];
      let d = c.clone;
      d[0] = 2;
      c.map(string).join(', ').print;
      d.map(string).join(', ').print;
    }"#;
    stdout "4\n3\n1, 2, 3\n2, 1, 2, 3\n";
);

// Runtime Error

test_full!(get_or_exit => r#"
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

test_full!(tree_construction_and_access => r#"
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

test_full!(eprint => r#"
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
