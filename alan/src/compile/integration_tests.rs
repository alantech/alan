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
                let rs_filename = filename.clone();
                let js_filename = filename.clone();
                let rs_handle = std::thread::spawn(move || -> Result<(), String> {
                    alan_compiler::program::Program::set_target_lang_rs();
                    match crate::compile::build(rs_filename, "release") {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Failed to compile to Rust {:?}", e)),
                    }
                });
                let js_handle = std::thread::spawn(move || -> Result<(), String> {
                    alan_compiler::program::Program::set_target_lang_js();
                    match crate::compile::web(js_filename) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Failed to compile to Javascript {:?}", e)),
                    }
                });
                let (rs_ok, rs_err) = match rs_handle.join() {
                    Ok(Ok(())) => (true, None),
                    Ok(Err(e)) => (false, Some(e)),
                    Err(e) => (false, Some(format!("Rust compilation thread panicked: {:?}", e))),
                };
                let (js_ok, js_err) = match js_handle.join() {
                    Ok(Ok(())) => (true, None),
                    Ok(Err(e)) => (false, Some(e)),
                    Err(e) => (false, Some(format!("Javascript compilation thread panicked: {:?}", e))),
                };
                if rs_ok {
                    let rs_cmd = if cfg!(windows) {
                        format!(".\\{}.exe", stringify!($rule))
                    } else {
                        format!("./{}", stringify!($rule))
                    };
                    let rs_run = match std::process::Command::new(rs_cmd.clone()).output() {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not run the test binary {:?}", e)),
                    }?;
                    $( $type!($test_val, true, &rs_run); )+
                    match std::fs::remove_file(&rs_cmd) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the test binary {:?}", e)),
                    }?;
                }
                if js_ok {
                    let js_cmd = if cfg!(windows) {
                        format!(".\\{}.js", stringify!($rule))
                    } else {
                        format!("./{}.js", stringify!($rule))
                    };
                    let js_run = match std::process::Command::new("node").arg(js_cmd.to_string()).output() {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not run the test JS code {:?}", e)),
                    }?;
                    $( $type!($test_val, false, &js_run); )+
                    match std::fs::remove_file(&js_cmd) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the generated JS file {:?}", e)),
                    }?;
                }
                std::fs::remove_file(&filename)?;
                if let Some(e) = rs_err {
                    return Err(e.into());
                }
                if let Some(e) = js_err {
                    return Err(e.into());
                }
                Ok(())
            }
        }
    };
    ( $rule:ident $entryfile:expr => $( $filename:expr => $code:expr),+ ; $( $type:ident $test_val:expr);+ $(;)? ) => {
        #[cfg(test)]
        mod $rule {
            #[test]
            fn $rule() -> Result<(), Box<dyn std::error::Error>> {
                $( match std::fs::write($filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", $filename, e).into());
                    }
                })+
                let rs_entry = format!("{}.ln", $entryfile);
                let js_entry = rs_entry.clone();
                let rs_handle = std::thread::spawn(move || -> Result<(), String> {
                    alan_compiler::program::Program::set_target_lang_rs();
                    match crate::compile::build(rs_entry, "release") {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Failed to compile to Rust {:?}", e)),
                    }
                });
                let js_handle = std::thread::spawn(move || -> Result<(), String> {
                    alan_compiler::program::Program::set_target_lang_js();
                    match crate::compile::web(js_entry) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Failed to compile to Javascript {:?}", e)),
                    }
                });
                let (rs_ok, rs_err) = match rs_handle.join() {
                    Ok(Ok(())) => (true, None),
                    Ok(Err(e)) => (false, Some(e)),
                    Err(e) => (false, Some(format!("Rust compilation thread panicked: {:?}", e))),
                };
                let (js_ok, js_err) = match js_handle.join() {
                    Ok(Ok(())) => (true, None),
                    Ok(Err(e)) => (false, Some(e)),
                    Err(e) => (false, Some(format!("Javascript compilation thread panicked: {:?}", e))),
                };
                if rs_ok {
                    let rs_cmd = if cfg!(windows) {
                        format!(".\\{}.exe", $entryfile)
                    } else {
                        format!("./{}", $entryfile)
                    };
                    let rs_run = match std::process::Command::new(rs_cmd.clone()).output() {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not run the test binary {:?}", e)),
                    }?;
                    $( $type!($test_val, true, &rs_run); )+
                    match std::fs::remove_file(&rs_cmd) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the test binary {:?}", e)),
                    }?;
                }
                if js_ok {
                    let js_cmd = if cfg!(windows) {
                        format!(".\\{}.js", $entryfile)
                    } else {
                        format!("./{}.js", $entryfile)
                    };
                    let js_run = match std::process::Command::new("node").arg(js_cmd.to_string()).output() {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not run the test JS code {:?}", e)),
                    }?;
                    $( $type!($test_val, false, &js_run); )+
                    match std::fs::remove_file(&js_cmd) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the generated JS file {:?}", e)),
                    }?;
                }
                $( std::fs::remove_file($filename)?; )+
                if let Some(e) = rs_err {
                    return Err(e.into());
                }
                if let Some(e) = js_err {
                    return Err(e.into());
                }
                Ok(())
            }
        }
    };
}
/// Like [`test!`] but sets `ALAN_TARGET=test` for compile-time `Test` type resolution.
macro_rules! test_with_alan_target {
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
                let rs_filename = filename.clone();
                let js_filename = filename.clone();
                let rs_handle = std::thread::spawn(move || -> Result<(), String> {
                    alan_compiler::program::Program::set_target_lang_rs();
                    alan_compiler::program::Program::set_compile_env("ALAN_TARGET", "test");
                    match crate::compile::build(rs_filename, "release") {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Failed to compile to Rust {:?}", e)),
                    }
                });
                let js_handle = std::thread::spawn(move || -> Result<(), String> {
                    alan_compiler::program::Program::set_target_lang_js();
                    alan_compiler::program::Program::set_compile_env("ALAN_TARGET", "test");
                    match crate::compile::web(js_filename) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Failed to compile to Javascript {:?}", e)),
                    }
                });
                let (rs_ok, rs_err) = match rs_handle.join() {
                    Ok(Ok(())) => (true, None),
                    Ok(Err(e)) => (false, Some(e)),
                    Err(e) => (false, Some(format!("Rust compilation thread panicked: {:?}", e))),
                };
                let (js_ok, js_err) = match js_handle.join() {
                    Ok(Ok(())) => (true, None),
                    Ok(Err(e)) => (false, Some(e)),
                    Err(e) => (false, Some(format!("Javascript compilation thread panicked: {:?}", e))),
                };
                if rs_ok {
                    let rs_cmd = if cfg!(windows) {
                        format!(".\\{}.exe", stringify!($rule))
                    } else {
                        format!("./{}", stringify!($rule))
                    };
                    let rs_run = match std::process::Command::new(rs_cmd.clone()).output() {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not run the test binary {:?}", e)),
                    }?;
                    $( $type!($test_val, true, &rs_run); )+
                    match std::fs::remove_file(&rs_cmd) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the test binary {:?}", e)),
                    }?;
                }
                if js_ok {
                    let js_cmd = if cfg!(windows) {
                        format!(".\\{}.js", stringify!($rule))
                    } else {
                        format!("./{}.js", stringify!($rule))
                    };
                    let js_run = match std::process::Command::new("node").arg(js_cmd.to_string()).output() {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not run the test JS code {:?}", e)),
                    }?;
                    $( $type!($test_val, false, &js_run); )+
                    match std::fs::remove_file(&js_cmd) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the generated JS file {:?}", e)),
                    }?;
                }
                std::fs::remove_file(&filename)?;
                if let Some(e) = rs_err {
                    return Err(e.into());
                }
                if let Some(e) = js_err {
                    return Err(e.into());
                }
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
                let filename = format!("{}.ln", stringify!($rule));
                match std::fs::write(&filename, $code) {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        return Err(format!("Unable to write {} to disk. {:?}", filename, e).into());
                    }
                };
                let rs_filename = filename.clone();
                let rs_handle = std::thread::spawn(move || -> Result<(), String> {
                    alan_compiler::program::Program::set_target_lang_rs();
                    match crate::compile::build(rs_filename, "release") {
                        Ok(_) => Ok(()),
                        Err(e) => Err(format!("Failed to compile to Rust {:?}", e)),
                    }
                });
                let (js_ok, js_err) = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
                    let js_filename = filename.clone();
                    let js_handle = std::thread::spawn(move || -> Result<(), String> {
                        alan_compiler::program::Program::set_target_lang_js();
                        match crate::compile::web(js_filename) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(format!("Failed to compile to Javascript {:?}", e)),
                        }
                    });
                    match js_handle.join() {
                        Ok(Ok(())) => (true, None),
                        Ok(Err(e)) => (false, Some(e)),
                        Err(e) => (false, Some(format!("Javascript compilation thread panicked: {:?}", e))),
                    }
                } else {
                    (true, None)
                };
                let (rs_ok, rs_err) = match rs_handle.join() {
                    Ok(Ok(())) => (true, None),
                    Ok(Err(e)) => (false, Some(e)),
                    Err(e) => (false, Some(format!("Rust compilation thread panicked: {:?}", e))),
                };
                if rs_ok {
                    let rs_cmd = if cfg!(windows) {
                        format!(".\\{}.exe", stringify!($rule))
                    } else {
                        format!("./{}", stringify!($rule))
                    };
                    let rs_run = match std::process::Command::new(rs_cmd.clone()).output() {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not run the test binary {:?}", e)),
                    }?;

                    $( $type!($test_val, true, &rs_run); )+
                    match std::fs::remove_file(&rs_cmd) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(format!("Could not remove the test binary {:?}", e)),
                    }?;
                }
                // TODO: For now, Chromium only allows WebGPU on these two platforms (unless you're
                // willing to muck about with CLI arguments *and* config flags simultaneously to
                // enable it for Linux, which Playwright doesn't even support...
                // My playwright scripts only work on Linux and MacOS, though, so that reduces it
                // to just MacOS to test this on.
                // if cfg!(windows) || cfg!(macos) {
                if cfg!(all(target_os = "macos", target_arch = "aarch64")) && js_ok {
                    // We need to create an HTML file that will run the generated code and a node
                    // script to fire up Playwright and grab the console.log output and shove it
                    // into stdout for the rest of the test suite to grab. Because the outermost
                    // directory of this repo is simultaneously a Rust and Node project, we're
                    // taking advantage of that to have the latter parts pre-written, but we can't
                    // do that for the HTML file because the script it loads is different for each
                    // test.
                    let jsfile = if cfg!(windows) {
                        format!(".\\{}.js", stringify!($rule))
                    } else {
                        format!("./{}.js", stringify!($rule))
                    };
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
                        .arg(format!("pnpm --silent run chrome-console http://localhost:8080/alan/{}.html", stringify!($rule)))
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
                if let Some(e) = rs_err {
                    return Err(e.into());
                }
                if let Some(e) = js_err {
                    return Err(e.into());
                }
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
                match crate::compile::build(filename.to_string(), "release") {
                    Ok(_) => { /* Do nothing */ }
                    Err(e) => {
                        std::fs::remove_file(&filename)?;
                        return Err(format!("Failed to compile to Rust {:?}", e).into());
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
        let std_err = if cfg!(windows) {
            String::from_utf8($real_val.stderr.clone())?.replace("\r\n", "\n")
        } else {
            String::from_utf8($real_val.stderr.clone())?
        };
        let err_info = if std_err.is_empty() {
            "".to_string()
        } else {
            format!(" (stderr: {})", std_err)
        };
        assert_eq!(
            $test_val,
            &std_out,
            "{}{}",
            if $in_rs {
                format!("Rust: {} == {}", $test_val, &std_out)
            } else {
                format!("JS: {} === {}", $test_val, &std_out)
            },
            err_info
        );
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
            let std_err = if cfg!(windows) {
                String::from_utf8($real_val.stderr.clone())?.replace("\r\n", "\n")
            } else {
                String::from_utf8($real_val.stderr.clone())?
            };
            let err_info = if std_err.is_empty() {
                "".to_string()
            } else {
                format!(" (stderr: {})", std_err)
            };
            assert_eq!(
                $test_val, &std_out,
                "Rust: {} == {}{}",
                $test_val, &std_out, err_info
            );
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
            let std_err = if cfg!(windows) {
                String::from_utf8($real_val.stderr.clone())?.replace("\r\n", "\n")
            } else {
                String::from_utf8($real_val.stderr.clone())?
            };
            let err_info = if std_err.is_empty() {
                "".to_string()
            } else {
                format!(" (stderr: {})", std_err)
            };
            assert_eq!(
                $test_val, &std_out,
                "JS: {} == {}{}",
                $test_val, &std_out, err_info
            );
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
        let std_err = if cfg!(windows) {
            String::from_utf8($real_val.stderr.clone())?.replace("\r\n", "\n")
        } else {
            String::from_utf8($real_val.stderr.clone())?
        };
        let err_info = if std_err.is_empty() {
            "".to_string()
        } else {
            format!(" (stderr: {})", std_err)
        };
        assert_eq!(
            std_out.contains($test_val),
            true,
            "{}{}",
            if $in_rs {
                format!("Rust: {} contained in {}", $test_val, &std_out)
            } else {
                format!("JS: {} contained in {}", $test_val, &std_out)
            },
            err_info
        );
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
        assert_eq!(
            $test_val,
            &std_err,
            "{}",
            if $in_rs {
                format!("Rust: {} == {}", $test_val, &std_err)
            } else {
                format!("JS: {} == {}", $test_val, &std_err)
            }
        );
    };
}
#[cfg(test)]
macro_rules! status {
    ( $test_val:expr, $in_rs:expr, $real_val:expr ) => {
        let status = $real_val.status.code().unwrap();
        assert_eq!(
            $test_val,
            status,
            "{}",
            if $in_rs {
                format!("Rust: {} == {}", $test_val, status)
            } else {
                format!("JS: {} == {}", $test_val, status)
            }
        );
    };
}

// The gold standard test. If you can't do this, are you even a language at all? :P
test!(hello_world => r#"export fn main() -> () {
  print('Hello, World!');
}
"#;
    stdout "Hello, World!\n";
    status 0;
);
test!(multi_line_hello_world => r#"export fn main =
  print("Hello,
World!");
"#;
    stdout r#"Hello,
World!
"#;
    status 0;
);

// Exit Tests

test!(normal_exit_code => r#"export fn main() -> ExitCode {
  return ExitCode(0);
}
"#;
    status 0;
);
test!(error_exit_code => r#"export fn main() = ExitCode(1);
"#;
    status 1;
);
test!(non_global_memory_exit_code => r#"export fn main() {
  let x: i64 = 0;
  return x.ExitCode;
}
"#;
    status 0;
);

// TODO: There's no way to check equality of the `void` type, only printing allows this right now
test!(void_values => r#"export fn main {
  5.print;
  5.void.print;
  void().print;
  void.print;
}
"#;
    stdout "5\nvoid\nvoid\nvoid\n";
);

// Printing Tests

test!(print_function => r#"export fn main() {
  print('Hello, World');
  return ExitCode(0);
}
"#;
    stdout "Hello, World\n";
    status 0;
);
// Numeric constant type selection (issue #215): a numeric literal is typed as an `AnyOf` of the
// numeric types that can hold its value and is narrowed by context (annotation/accessor) or, when
// unconstrained, collapses to the last candidate in FUI order (Floats, Unsigned, Ints, ascending
// bit width). See `docs/int-float-constant-selection-plan.md`.
test!(numeric_const_u64_annotation => r#"export fn main {
  let big: u64 = 18446744073709551615;
  print(big);
}
"#;
    stdout "18446744073709551615\n";
    status 0;
);
test!(numeric_const_u64_default => r#"export fn main {
  // No annotation: above i64::MAX, so the only viable integer type is u64, which (being last in
  // FUI order among the survivors) becomes the default.
  let big = 18446744073709551615;
  print(big);
}
"#;
    stdout "18446744073709551615\n";
    status 0;
);
test!(numeric_const_u64_cast => r#"export fn main {
  // The original issue #215 repro: casting a literal larger than i64::MAX used to overflow the
  // intermediate i64. The literal now resolves directly to u64.
  let big = 18446744073709551615.u64;
  print(big);
}
"#;
    stdout "18446744073709551615\n";
    status 0;
);
test!(numeric_const_small_unsigned => r#"export fn main {
  let x: u8 = 200;
  print(x);
}
"#;
    stdout "200\n";
    status 0;
);
test!(numeric_const_int_literal_as_float => r#"export fn main {
  // An integer-form literal narrowed to a float type renders as a float literal.
  let x: f32 = 5;
  print(x);
}
"#;
    stdout "5\n";
    status 0;
);
test!(numeric_const_implicit_param => r#"fn foo(x: u8) = print(x);
export fn main {
  // A bare literal argument is narrowed to the parameter's concrete type (u8) -- no annotation
  // or cast needed.
  foo(200);
}
"#;
    stdout "200\n";
    status 0;
);
test!(numeric_const_implicit_param_pruned => r#"fn foo(x: u8) = print(x);
fn foo(x: i32) = print(x);
export fn main {
  // 300 does not fit u8, so only the i32 overload is viable and is selected.
  foo(300);
}
"#;
    stdout "300\n";
    status 0;
);
test!(duration_print => r#"export fn main() {
  const i = now();
  wait(110); // Increased from 10ms to 110ms because the node.js event loop seems less
  // capable of guaranteeing staying below 20ms in the delay here. Adding an extra
  // 10ms in case it accidentally waits 90-something ms instead.
  const d = i.elapsed;
  print(d);
}
"#;
    stdout_contains "0.1";
);
test!(print_compile_time_string => r#"type FooBar = Concat{"Foo", "Bar"};
export fn main {
  {FooBar}().print;
}
"#;
    stdout "FooBar\n";
);
test!(stdout_and_stderr => r#"export fn main {
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
}
"#;
    stdout "Hello, World!\n";
    stderr "Goodbye, World!\n";
);

// TODO: Unify the string output for these two so it can be tested more reliably
test!(string_parse => r#"export fn main {
  "8".i8.print;
  "foo".i8.print;
  "16".i16.print;
  "foo".i16.print;
  "32".i32.print;
  "foo".i32.print;
  "64".i64.print;
  "foo".i64.print;
}
"#;
    stdout_rs "8\nError: invalid digit found in string\n16\nError: invalid digit found in string\n32\nError: invalid digit found in string\n64\nError: invalid digit found in string\n";
    stdout_js "8\nError: Not a Number\n16\nError: Not a Number\n32\nError: Not a Number\n64\nError: Cannot convert foo to a BigInt\n";
);

// GPGPU

test_gpgpu!(hello_gpu => r#"export fn main {
  let b = GBuffer(filled(2.i32, 4))!!;
  let plan = GPGPU(
    "
        @group(0)
        @binding(0)
        var<storage, read_write> vals: array<i32>;

        @compute
        @workgroup_size(1)
        fn main(@builtin(global_invocation_id) id: vec3<u32>) {
          vals[id.x] = vals[id.x] * i32(id.x);
        }
      ",
    b,
    {i64[3]}(1, 1, 1)
  );
  plan.run;
  b.read.print;
}
"#;
    stdout "[0, 2, 4, 6]\n";
);
test_gpgpu!(hello_gpu_new => r#"export fn main {
  let b = GBuffer(filled(2.i32, 4))!!;
  let idx = gFor(4);
  let compute = b[idx].store(b[idx] * idx.gi32);
  compute.build.run;
  b.read.print;
}
"#;
    stdout "[0, 2, 4, 6]\n";
);
test_gpgpu!(list_of_gpu_tasks => r#"export fn main {
  let b1 = GBuffer(filled(2.i32, 8))!!;
  let b2 = GBuffer(filled(5.i32, 4))!!;
  let i1 = gFor(8);
  let i2 = gFor(4);
  let c1 = b1[i1].store(b1[i1] * i1.gi32);
  let c2 = b2[i2].store(b1[i2] + i2.gi32);
  [c1.build, c2.build].run; // Execution order determined here
  b1.read.print;
  b2.read.print;
}
"#;
    stdout "[0, 2, 4, 6, 8, 10, 12, 14]\n[0, 3, 6, 9]\n";
);

test_gpgpu!(hello_gpu_odd => r#"export fn main {
  let b = GBuffer(filled(2.i32, 4))!!;
  let idx = gFor(4, 1);
  let compute = b[idx.i].store(b[idx.i] * idx.i.gi32 + 1);
  compute.build.run;
  b.read.print;
}
"#;
    stdout "[1, 3, 5, 7]\n";
);

test_gpgpu!(gpu_map => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  let out = b.map(fn(val: gi32) = val + 2);
  out.read.print;
}
"#;
    stdout "[3, 4, 5, 6]\n";
);

test_gpgpu!(gpu_if => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  let out = b.map(fn(val: gi32, i: gu32) = if(i % 2 == 0, val * i.gi32, val - i.gi32));
  out.read.print;
}
"#;
    stdout "[0, 1, 6, 1]\n";
);

// Closure-form GPU `if`: deferred arms emit a real `if (c) { ... } else { ... }` wgsl block
// (vs. the value form above, which lowers to a branchless `select`). Both arms derive from a
// shared upstream value (`doubled`); the rewrite splits each arm's statements into the shared
// prefix (hoisted once, before the block) and the branch-local remainder (emitted inside the
// matching brace). `shaderOf` mirrors `map`'s lowering but returns the generated wgsl so we can
// assert statement placement; the `map` call then confirms the shader actually compiles and runs.
// For input [1,2,3,4]: doubled = [2,4,6,8]; `val > 2` selects [-, -, +, +] -> [1, 3, 7, 9].
test_gpgpu!(gpu_if_block => r#"fn shaderOf{G, G2}(
  gb: GBuffer{G},
  f: Prop{WgpuTypeMap, String{G}} -> Prop{WgpuTypeMap, String{G2}}
) {
  let idx = gFor(gb.cpulen);
  let val = gb[idx];
  let out = GBuffer{G2}(gb.cpulen)!!;
  let compute = out[idx].store(f(val));
  return compute.build.shader;
}
export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  b.shaderOf(fn(val: gi32) {
    let doubled = val * 2.gi32;
    return if(val > 2.gi32, fn = doubled + 1.gi32, fn = doubled - 1.gi32);
  }).print;
  b.map(fn(val: gi32) {
    let doubled = val * 2.gi32;
    return if(val > 2.gi32, fn = doubled + 1.gi32, fn = doubled - 1.gi32);
  }).read.print;
}
"#;
    // Shared upstream (`doubled`) hoisted to top level (2-space indent), declared once:
    stdout_contains "  var mul_i32_";
    // A real branch, not a `select`:
    stdout_contains "  if (";
    stdout_contains "  } else {";
    // Each arm's local var lands inside its brace (4-space indent):
    stdout_contains "    var add_i32_";
    stdout_contains "    var sub_i32_";
    // The result `var` is declared before the block (top level):
    stdout_contains "  var if_i32_";
    // ...and the shader compiles and runs with correct per-branch results:
    stdout_contains "[1, 3, 7, 9]";
);

// The same guarded GPU branch written with block (`if cond { ... } else { ... }`) syntax instead of
// the functional `if(cond, fn = ..., fn = ...)` form. The Phase 1 conditional lowering routes the
// `gbool` condition through the same closure-form `if`, so it must produce the identical real
// `if`/`else` wgsl block (not a `select`) and the identical result -- this guards against the block
// syntax silently regressing to the value form or failing to dispatch for `gbool`.
test_gpgpu!(gpu_if_block_syntax => r#"fn shaderOf{G, G2}(
  gb: GBuffer{G},
  f: Prop{WgpuTypeMap, String{G}} -> Prop{WgpuTypeMap, String{G2}}
) {
  let idx = gFor(gb.cpulen);
  let val = gb[idx];
  let out = GBuffer{G2}(gb.cpulen)!!;
  let compute = out[idx].store(f(val));
  return compute.build.shader;
}
export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  b.shaderOf(fn(val: gi32) {
    let doubled = val * 2.gi32;
    if val > 2.gi32 {
      return doubled + 1.gi32;
    } else {
      return doubled - 1.gi32;
    }
  }).print;
  b.map(fn(val: gi32) {
    let doubled = val * 2.gi32;
    if val > 2.gi32 {
      return doubled + 1.gi32;
    } else {
      return doubled - 1.gi32;
    }
  }).read.print;
}
"#;
    // Shared upstream (`doubled`) hoisted to top level (2-space indent), declared once:
    stdout_contains "  var mul_i32_";
    // A real branch, not a `select`:
    stdout_contains "  if (";
    stdout_contains "  } else {";
    // Each arm's local var lands inside its brace (4-space indent):
    stdout_contains "    var add_i32_";
    stdout_contains "    var sub_i32_";
    // The result `var` is declared before the block (top level):
    stdout_contains "  var if_i32_";
    // ...and the shader compiles and runs with correct per-branch results:
    stdout_contains "[1, 3, 7, 9]";
);

// Imperative conditional *mutation* of an outer-scope variable inside a GPU shader. Unlike the
// value-producing `if`s above (whose arms `return`), here the arms reassign `out`, with the block
// tail (`return out`) following the conditional. The conditional-assignment lowering rewrites this
// to a shadowing `let out = if(...)`, so the GPU closure-`if` emits a real `var out; if (c) { out =
// ...; }` block and the tail reads the mutated variable -- rather than folding the tail (returning
// `gi32[]`) into both arms, which the GPU `if` cannot combine. The `else if` chain exercises the
// nested case (and the hoisting of the `@builtin(global_invocation_id)` metadata out of a single
// arm). For input [1,2,3,4]: 1 -> default 0, 2 -> 50 (== 2), 3 & 4 -> 100 (> 2).
test_gpgpu!(gpu_if_block_mutation => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  b.map(fn(val: gi32) {
    let out = 0.gi32;
    if val > 2.gi32 {
      out = 100.gi32;
    } else if val == 2.gi32 {
      out = 50.gi32;
    }
    return out;
  }).read.print;
}
"#;
    stdout "[0, 50, 100, 100]\n";
);

// Multiple distinct variables mutated per branch. Each gets its own per-variable phi
// (`let lo = if(...); let hi = if(...)`), so the GPU `if` lowering handles them independently
// rather than folding the tail (which would silently drop the plain reassignments). `hi`'s arm
// also reads `lo`, exercising that bindings are emitted in assignment order so the cross-reference
// sees the updated value. For [1,2,3,4]: <=2 -> lo=30,hi=lo+10=40 -> 70; >2 -> lo=10,hi=20 -> 30.
test_gpgpu!(gpu_if_block_multivar => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  b.map(fn(val: gi32) {
    let lo = 0.gi32;
    let hi = 0.gi32;
    if val > 2.gi32 {
      lo = 10.gi32;
      hi = 20.gi32;
    } else {
      lo = 30.gi32;
      hi = lo + 10.gi32;
    }
    return lo + hi;
  }).read.print;
}
"#;
    stdout "[70, 70, 30, 30]\n";
);

// A branch that reassigns the same variable more than once. The per-variable phi can only bind a
// variable once, so the conditional lowering first SSA-ifies the branch -- composing the sequential
// assignments into a single expression (`acc = acc + 10; acc = acc + 100` -> `(acc + 10) + 100`) --
// before the phi rewrite. For [1,2,3,4], `val > 2` (3,4) -> (0+10)+100 = 110, else 0.
test_gpgpu!(gpu_if_block_compose => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  b.map(fn(val: gi32) {
    let acc = 0.gi32;
    if val > 2.gi32 {
      acc = acc + 10.gi32;
      acc = acc + 100.gi32;
    }
    return acc;
  }).read.print;
}
"#;
    stdout "[0, 0, 110, 110]\n";
);

// A branch with a branch-local `let`. The SSA-ifier inlines the local into the outer-variable
// assignment (`let bump = val + 1; acc = bump * 2` -> `acc = (val + 1) * 2`), so the local never
// escapes and the outer `acc` gets a single composed phi binding. For [1,2,3,4], `val > 2` (3,4) ->
// (3+1)*2 = 8 and (4+1)*2 = 10, else 0.
test_gpgpu!(gpu_if_block_local_let => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  b.map(fn(val: gi32) {
    let acc = 0.gi32;
    if val > 2.gi32 {
      let bump = val + 1.gi32;
      acc = bump * 2.gi32;
    }
    return acc;
  }).read.print;
}
"#;
    stdout "[0, 0, 8, 10]\n";
);

// Straight-line (no conditional) reassignment of a scalar GPU builder. `acc` starts as a plain
// value (`0.gi32`) with no assignable WGSL location, so the first `acc = ...` *promotes* it on
// demand to a mutable `var m_<uuid>: i32` initialized to its prior value, then assigns into it; the
// second reuses that same `var` (the `@lvalue` marker makes the store idempotent). The reassignment
// works through the `Mut`-form GPU `store` (which mutates the builder in place), with the Rust
// backend's argument-hoisting handling the `store(&mut acc, acc + ...)` self-reference. `shaderOf`
// asserts the structure; `map` confirms it runs: for [1,2,3,4], acc = (0 + val) * 2 -> [2,4,6,8].
test_gpgpu!(gpu_straightline_mutation => r#"fn shaderOf{G, G2}(
  gb: GBuffer{G},
  f: Prop{WgpuTypeMap, String{G}} -> Prop{WgpuTypeMap, String{G2}}
) {
  let idx = gFor(gb.cpulen);
  let val = gb[idx];
  let out = GBuffer{G2}(gb.cpulen)!!;
  let compute = out[idx].store(f(val));
  return compute.build.shader;
}
export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  b.shaderOf(fn(val: gi32) {
    let acc = 0.gi32;
    acc = acc + val;
    acc = acc * 2.gi32;
    return acc;
  }).print;
  b.map(fn(val: gi32) {
    let acc = 0.gi32;
    acc = acc + val;
    acc = acc * 2.gi32;
    return acc;
  }).read.print;
}
"#;
    // The scalar is promoted once to a typed mutable `var`, initialized to its prior value (`0`):
    stdout_contains "  var m_";
    stdout_contains ": i32 = 0";
    // ...and the shader compiles and runs with the reassignments applied in order:
    stdout_contains "[2, 4, 6, 8]";
);

test_gpgpu!(gpu_replace => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  b.map(fn(val: gi32) = val + 2).read.print;
  b.replace([2.i32, 4.i32, 6.i32, 8.i32]);
  b.map(fn(val: gi32) = val / 2).read.print;
}
"#;
    stdout "[3, 4, 5, 6]\n[1, 2, 3, 4]\n";
);

test_gpgpu!(gpu_abs => r#"export fn main {
  let b = GBuffer([1.i32, -2.i32, -3.i32, 4.i32])!!;
  b.map(fn(val: gi32) = val.abs).read.print;
}
"#;
    stdout "[1, 2, 3, 4]\n";
);

test_gpgpu!(gpu_clz => r#"export fn main {
  let b = GBuffer([1.i32, -2.i32, -3.i32, 4.i32])!!;
  // Don't need the generic on the `read` call, but leaving it to show it still works
  b.map(fn(val: gi32) = val.clz).read{i32}.print;
}
"#;
    stdout "[31, 0, 0, 29]\n";
);

test_gpgpu!(gpu_ones => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, -1.i32])!!;
  b.map(fn(val: gi32) = val.ones).read.print;
}
"#;
    stdout "[1, 1, 2, 32]\n";
);

test_gpgpu!(gpu_ctz => r#"export fn main {
  let b = GBuffer([0.i32, 1.i32, 2.i32, -2_147_483_648.i32])!!;
  b.map(fn(val: gi32) = val.ctz).read.print;
}
"#;
    stdout "[32, 0, 1, 31]\n";
);

test_gpgpu!(gpu_cross => r#"// TODO: A nicer test involving `map`
export fn main {
  let b = GBuffer(filled(0.f32, 2))!!;
  let idx = gFor(2);
  let compute = b[idx].store(if(
    idx == 0,
    gvec3f(1.0, 0.0, 0.0) >< gvec3f(0.0, 1.0, 0.0),
    gvec3f(0.0, 1.0, 0.0) >< gvec3f(1.0, 0.0, 0.0)
  ).z);
  compute.build.run;
  b.read.print;
}
"#;
    stdout "[1, -1]\n";
);

test_gpgpu!(gpu_transpose => r#"export fn main {
  let b = GBuffer([1.i32, 2.i32, 3.i32, 4.i32])!!;
  let m = gmat2x2f(b[0], b[1], b[2], b[3]).transpose;
  let idx = gFor(1);
  [idx, b[0].store((m * gmat2x2f(1.0, 0.0, 0.0, 0.0) * gvec2f(1.0, 0.0)).x.gi32), b[1].store((m * gmat2x2f(
    1.0,
    0.0,
    0.0,
    0.0
  ) * gvec2f(1.0, 0.0)).y.gi32), b[2].store((m * gmat2x2f(0.0, 1.0, 0.0, 0.0) * gvec2f(1.0, 0.0)).x.gi32), b[3].store((m * gmat2x2f(
    0.0,
    1.0,
    0.0,
    0.0
  ) * gvec2f(1.0, 0.0)).y.gi32)].build.run;
  b.read.print;
}
"#;
    stdout "[1, 3, 2, 4]\n";
);

test_gpgpu!(gpu_reversebits => r#"export fn main {
  let b = GBuffer([0.i32, 1.i32, 2.i32, (-2_147_483_648).i32])!!;
  b.map(fn(val: gi32) = val.reverseBits).read.print;
}
"#;
    stdout "[0, -2147483648, 1073741824, 1]\n";
);

test_gpgpu!(gpu_extractbits => r#"export fn main {
  let b = GBuffer([0.u32, 1.u32, 2.u32, 5.u32])!!;
  b.map(fn(val: gu32) = val.extractBits(1, 2)).read.print;
}
"#;
    stdout "[0, 0, 1, 2]\n";
);

test_gpgpu!(gpu_insertbits => r#"export fn main {
  let b = GBuffer([0.u32, 31.u32])!!;
  b.map(fn(val: gu32) = val.insertBits(1, 2, 3)).read.print;
}
"#;
    stdout "[4, 7]\n";
);

test_gpgpu!(gpu_round => r#"export fn main {
  let b = GBuffer([1.5.f32, 1.75.f32, 2.5.f32, 2.75.f32, (-1.5).f32, (-1.75).f32, (-2.5).f32, (-2.75).f32])!!;
  b.map(fn(val: gf32) = val.round).read.print;
}
"#;
    stdout "[2, 2, 2, 3, -2, -2, -2, -3]\n";
);

test_gpgpu!(gpu_magnitude => r#"export fn main {
  let b = GBuffer([2.5.f32, -2.5.f32, 2.5.f32, -2.5.f32])!!;
  b.map(fn(val: gf32) = val.magnitude).read.print;
  let id = gFor(1);
  let out = GBuffer{f32}(1)!!;
  let compute = out[id].store(gvec4f(b[0], b[1], b[2], b[3]).magnitude);
  compute.build.run;
  out.read.print;
}
"#;
    stdout "[2.5, 2.5, 2.5, 2.5]\n[5]\n";
);

test_gpgpu!(gpu_normalize => r#"export fn main {
  let b = GBuffer([3.0.f32, 4.0.f32])!!;
  let id = gFor(1);
  let out = GBuffer{f32}(2)!!;
  let normal = gvec2f(b[0], b[1]).normalize;
  [out[id].store(normal.x), out[id + 1].store(normal.y)].build.run;
  out.read.map(fn(v: f32) = v.string(1)).join(', ').print;
}
"#;
    stdout "0.6, 0.8\n";
);

test_gpgpu!(gpu_saturate => r#"export fn main {
  let b = GBuffer([(-0.5).f32, 0.0.f32, 0.5.f32, 1.0.f32, 1.5.f32])!!;
  b.map(fn(val: gf32) = val.saturate).read.print;
}
"#;
    stdout "[0, 0, 0.5, 1, 1]\n";
);

test_gpgpu!(gpu_dot => r#"export fn main {
  let b = GBuffer([3.0.f32, 4.0.f32])!!;
  let id = gFor(1);
  let out = GBuffer{f32}(1)!!;
  let vec = gvec2f(b[0], b[1]);
  out[id].store(vec *. vec).build.run;
  out.read.map(fn(v: f32) = v.string(1)).join(', ').print;
}
"#;
    stdout "25.0\n";
);

test_gpgpu!(gpu_inverse_sqrt => r#"export fn main {
  let b = GBuffer([4.0.f32, 25.0.f32])!!;
  b.map(fn(val: gf32) = val.inverseSqrt).read.map(fn(v: f32) = v.string(1)).print;
}
"#;
    stdout "[0.5, 0.2]\n";
);

test_gpgpu!(gpu_fma => r#"export fn main {
  let b = GBuffer([2.0.f32, 3.0.f32, 4.0.f32])!!;
  let id = gFor(1);
  let out = GBuffer{f32}(1)!!;
  out[id].store(fma(b[0], b[1], b[2])).build.run;
  (out.read[0]!!).string(1).print;
}
"#;
    stdout "10.0\n";
);

test_gpgpu!(gpu_fract => r#"export fn main {
  let b = GBuffer([1.0.f32, 3.14.f32])!!;
  b.map(fn(val: gf32) = val.fract).read.map(fn(v: f32) = v.string(2)).join(", ").print;
}
"#;
    stdout "0.00, 0.14\n";
);

test_gpgpu!(gpu_determinant => r#"export fn main {
  let b = GBuffer([1.0.f32, 2.0.f32, 3.0.f32, 4.0.f32])!!;
  let id = gFor(1);
  let out = GBuffer{f32}(1)!!;
  out[id].store(gmat2x2f(b[0], b[1], b[2], b[3]).determinant).build.run;
  (out.read[0]!!).string(1).print;
}
"#;
    stdout "-2.0\n";
);

test_gpgpu!(gpu_storage_barrier => r#"export fn{Lin || Win} main {
  // On Linux and Windows, you can use `storageBarrier` to act as a synchronization point
  // across multiple threads running the same shader, which can let you do some work, then wait
  // to do more work that each thread may depend on the prior output of multiple threads to do
  let id = gFor(3, 3);
  let temp = GBuffer{f32}(9)!!;
  let out = GBuffer{f32}(9)!!;
  let compute = [temp[id.x + 3 * id.y] = (id.x.gf32 + id.y.gf32), storageBarrier(), out[id.x + 3 * id.y] =
  ((if(id.x > 0, temp[id.x - 1 + 3 * id.y], 0.0.gf32) +
  temp[id.x + 3 * id.y] +
  if(id.x < 2, temp[id.x + 1 + 3 * id.y], 0.0.gf32)) / 3.0)].build.run;
  out.read.map(fn(v: f32) = v.string(2)).join(", ").print;
}
export fn{Mac || Js} main {
  // It's never safe to use `storageBarrier` on a Mac because the new ARM Macs break it. This
  // version "simulates" the barrier by instead breaking the single shader into two separate
  // shaders that are run sequentially. This theoretically has a higher synchronization cost so
  // it's not ideal, but I haven't done any benchmarking to see how much of an impact it has.
  // Because it's not safe to use `storageBarrier` on a Mac it is also never safe to do so when
  // in a browser context because you don't know what platform will be underneath. I'd argue
  // that this means `storageBarrier` should never have been included in the WebGPU spec, but
  // here we are.
  let id = gFor(3, 3);
  let temp = GBuffer{f32}(9)!!;
  let out = GBuffer{f32}(9)!!;
  let compute = [build(temp[id.x + 3 * id.y] = (id.x.gf32 + id.y.gf32)), build(out[id.x + 3 * id.y] =
  ((if(id.x > 0, temp[id.x - 1 + 3 * id.y], 0.0.gf32) +
  temp[id.x + 3 * id.y] +
  if(id.x < 2, temp[id.x + 1 + 3 * id.y], 0.0.gf32)) / 3.0))];
  compute.run;
  out.read.map(fn(v: f32) = v.string(2)).join(", ").print;
}
"#;
    stdout "0.33, 1.00, 1.00, 1.00, 2.00, 1.67, 1.67, 3.00, 2.33\n";
);

test!(map_i32_function_value_sync_dispatch => r#"fn conv(idx: Buffer{i64, 3}) -> Buffer{i32, 3} {
  return idx.map(i32);
}

export fn main {
  let out = conv({i64[3]}(1, 2, 3));
  out.len.string.print;
}
"#;
    stdout "3\n";
);

// Functions and Custom Operators

test!(basic_function_usage => r#"fn foo() = print('foo');

fn bar(s: string) = s.concat("bar");

export fn main {
  foo();
  'foo'.bar.print;
}
"#;
    stdout r#"foo
foobar
"#;
);

test!(functions_and_custom_operators => r#"fn foo() {
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
}
"#;
    stdout r#"foo
to barto bar3
>> text here2
8
6
10
10
"#;
);

test!(mutable_functions => r#"fn addeq (a: Mut{i64}, b: i64) {
  a = a.clone() + b;
}

infix addeq as += precedence 0;

export fn main {
  let five = 3;
  five.print;
  five += 2;
  five.print;
}
"#;
    stdout "3\n5\n";
);

// A self-referential reassignment (`x = x + 1`) where the right-hand side reads the same variable
// being reassigned. This lowers to `store(&mut x, x + 1)`; rendered as a single nested call the
// explicit `&mut x` would overlap the rhs's `&x` read and fail to compile (E0502) on the Rust
// backend. The codegen hoists the by-value rhs into a `let` temporary evaluated before the `&mut`
// borrow, so it compiles and runs (and matches the JS backend). Guards that the value used is the
// pre-update one: 5 -> 5+1 -> 6*2 -> 12.
test!(self_referential_reassignment => r#"export fn main {
  let x = 5;
  x = x + 1;
  x = x * 2;
  x.print;
}
"#;
    stdout "12\n";
);

test!(complex_cloning => r#"type struct = b: bool, d: Dict{string, Set{i64}};

export fn main {
  let s = Set([1, 2, 3]);
  s.len.print;
  s.clone.len.print;
  let d = Dict('foo', s);
  d.len.print;
  d.clone.len.print;
  let b = struct(true, d);
  b.d.len.print;
  b.clone.d.len.print;
}
"#;
    stdout "3\n3\n1\n1\n1\n1\n";
);

// Conditionals

// An `if`/`else` where both branches return, but with a trailing tail after the conditional. The
// early-return branch exits; the fall-through path runs the tail.
test!(conditional_early_return_with_tail => r#"fn classify(n: i64) -> string {
  if n < 0 {
    return 'negative';
  }
  return 'nonneg';
}
export fn main {
  print(classify(0 - 3));
  print(classify(5));
}
"#;
    stdout "negative\nnonneg\n";
);
// `else if` chains normalize into nested conditionals.
test!(conditional_else_if_chain => r#"fn name(n: i64) -> string {
  if n == 1 {
    return 'one';
  } else if n == 2 {
    return 'two';
  } else {
    return 'other';
  }
}
export fn main {
  print(name(1));
  print(name(2));
  print(name(3));
}
"#;
    stdout "one\ntwo\nother\n";
);
// A nested conditional with partial (non-exhaustive) returns: the tail (`print('end')`) is
// duplicated onto every fall-through path, so it runs whenever a branch does not early-return.
test!(conditional_nested_partial_returns => r#"fn check(n: i64) {
  if n > 0 {
    print('positive');
    if n > 10 {
      print('big');
      return;
    }
  }
  print('end');
}
export fn main {
  check(50);
  check(5);
  check(0 - 1);
}
"#;
    stdout "positive\nbig\npositive\nend\nend\n";
);
// A void, side-effect-only `if` with no `else`, followed by more statements.
test!(conditional_void_no_else => r#"export fn main {
  let x = 5;
  if x == 5 {
    print('five');
  }
  print('after');
}
"#;
    stdout "five\nafter\n";
);
// Conditional mutation where a branch reassigns one variable multiple times and uses a branch-local
// `let`. The lowering SSA-ifies the branch (composing `acc = acc + 10; acc = acc + 100` and inlining
// `let bump = ...`) into one phi binding per outer variable, on both backends. For n=3 the branch
// runs: acc = (0+10)+100 = 110, then doubled via the local to 220; n=1 skips it (acc stays 0).
test!(conditional_compose_and_local_let => r#"fn run(n: i64) -> i64 {
  let acc = 0;
  if n > 2 {
    acc = acc + 10;
    acc = acc + 100;
    let bump = acc;
    acc = bump * 2;
  }
  return acc;
}
export fn main {
  print(run(3));
  print(run(1));
}
"#;
    stdout "220\n0\n";
);
// A void conditional whose taken branch awaits (`wait` is async on the JS backend). The enclosing
// function must be colored correctly so the awaited work completes before the following statement.
test!(conditional_void_async_branch => r#"export fn main {
  let x = 1;
  if x == 1 {
    let t = wait(10);
    print('waited');
  } else {
    print('skipped');
  }
  print('done');
}
"#;
    stdout "waited\ndone\n";
);
// A user-defined `if` overload that forwards its branch closures *directly* to the underlying `if`
// cfn (`if(cond, t, f)`, not re-wrapped as `fn() = t()`). This is the idiomatic way to opt into
// "DWIM" truthiness for a new condition type -- here `i64` (0 is false) and `string` (empty is
// false). The branch closures arrive at the cfn as closure-typed *values* rather than literal
// closures, so codegen must *call* them (`{ t() }` in Rust, `return t()` in JS) rather than emit
// the bare reference. Covers value-producing (`describe`) and void (`if name`) positions.
test!(conditional_forwarded_branch_closures => r#"fn if{T}(c: i64, t: () -> T, f: () -> T) = if(c != 0, t, f);
fn if{T}(c: string, t: () -> T, f: () -> T) = if(c.len > 0, t, f);
fn describe(n: i64) -> string {
  if n {
    return 'nonzero';
  }
  return 'zero';
}
export fn main {
  print(describe(0));
  print(describe(42));
  let name = '';
  if name {
    print('named');
  } else {
    print('anonymous');
  }
}
"#;
    stdout "zero\nnonzero\nanonymous\n";
);
// A *value-producing* `if` function call used as a discarded statement (its result is thrown
// away), followed by more statements. The branch closures' terminal value-`return`s must NOT
// return from the enclosing function -- otherwise `mid`/`end` would never run. This is the shape
// the built-in test harness's `it` uses (`if(cond, fn = arr.pop, fn = Maybe{string}(""))`), which
// previously miscompiled on the JS backend into an early return.
test!(conditional_discarded_value_if_statement => r#"fn run(n: i64) -> string {
  let acc = ['start'];
  if(n > 0, fn = acc.push('pos'), fn = acc.push('neg'));
  acc.push('end');
  return acc.join(' ');
}
export fn main {
  print(run(1));
  print(run(0 - 1));
}
"#;
    stdout "start pos end\nstart neg end\n";
);
// Both branches return *and* there is a trailing statement after the conditional: the tail is
// unreachable, which is a compile error.
test_compile_error!(conditional_both_arms_return_with_tail => r#"fn f(n: i64) -> string {
  if n > 0 {
    return 'pos';
  } else {
    return 'neg';
  }
  return 'unreachable';
}
export fn main {
  print(f(1));
}
"#;
    error "Unreachable statements after a conditional in which both branches return";
);
// JS-specific codegen check: a pure (non-awaiting) value conditional compiles to a *synchronous*
// IIFE -- a plain `(() => { ... })()` with no `await` and no `async` wrapper. This is the win the
// sync-function coloring enables.
#[cfg(test)]
mod conditional_js_sync_iife {
    #[test]
    fn conditional_js_sync_iife() -> Result<(), Box<dyn std::error::Error>> {
        alan_compiler::program::Program::set_target_lang_js();
        let filename = "conditional_js_sync_iife.ln".to_string();
        std::fs::write(
            &filename,
            r#"export fn main {
  let r = if(true, fn() = 'a', fn() = 'b');
  print(r);
}
"#,
        )?;
        let res = alan_compiler::lntojs::lntojs(filename.clone());
        std::fs::remove_file(&filename)?;
        let (js, _deps) = res?;
        assert!(
            js.contains("(() => { if ("),
            "expected a synchronous arrow IIFE for a pure value conditional, got:\n{js}"
        );
        assert!(
            !js.contains("await (async () =>"),
            "a pure conditional should not produce an awaited async IIFE, got:\n{js}"
        );
        Ok(())
    }
}
// JS-specific (Promise transparency, problem 1): a *pure-Alan* function declared to return a
// concrete type (`f64`) whose body returns an awaited native value (`wait` -> `Promise{f64}` on
// JS) type-checks -- Alan auto-awaits, so `Promise{f64}` is transparent to the declared `f64` --
// and is correctly colored `async` with the `await` emitted. (`wait` is synchronous on the Rust
// backend, so this awaited-return shape is JS-only and can't be a cross-backend `test!`.)
#[cfg(test)]
mod conditional_js_async_plain_return {
    #[test]
    fn conditional_js_async_plain_return() -> Result<(), Box<dyn std::error::Error>> {
        alan_compiler::program::Program::set_target_lang_js();
        let filename = "conditional_js_async_plain_return.ln".to_string();
        std::fs::write(
            &filename,
            r#"fn fetchVal(b: bool) -> f64 {
  return wait(10);
}
export fn main {
  print(fetchVal(true).string);
}
"#,
        )?;
        // Compiling at all is the regression check: this previously failed with "specified to
        // return f64 but actually returns Promise{f64}". (The single-`return` function is inlined
        // into `main`, so the await/async land there rather than in a separate `fetchVal`.)
        let res = alan_compiler::lntojs::lntojs(filename.clone());
        std::fs::remove_file(&filename)?;
        let (js, _deps) = res?;
        assert!(
            js.contains("async function main"),
            "the awaiting call should color the enclosing function async, got:\n{js}"
        );
        assert!(
            js.contains("await"),
            "the awaited native call should be emitted with `await`, got:\n{js}"
        );
        Ok(())
    }
}
// JS-specific (Promise transparency, problem 2): a value conditional whose branches return awaited
// native values resolves to the `if{T}` cfn (with `T` Promise-transparent), emits a native
// return-position `if/else` with `await` in both branches, and colors the function `async`.
#[cfg(test)]
mod conditional_js_async_return_branches {
    #[test]
    fn conditional_js_async_return_branches() -> Result<(), Box<dyn std::error::Error>> {
        alan_compiler::program::Program::set_target_lang_js();
        let filename = "conditional_js_async_return_branches.ln".to_string();
        std::fs::write(
            &filename,
            r#"fn pick(b: bool) -> f64 {
  if b {
    return wait(10);
  } else {
    return wait(20);
  }
}
export fn main {
  print(pick(true).string);
}
"#,
        )?;
        let res = alan_compiler::lntojs::lntojs(filename.clone());
        std::fs::remove_file(&filename)?;
        let (js, _deps) = res?;
        assert!(
            js.contains("async function pick"),
            "an awaiting conditional function should be colored async, got:\n{js}"
        );
        assert!(
            js.contains("if (") && js.contains("} else {"),
            "expected a native return-position if/else, got:\n{js}"
        );
        assert!(
            js.matches("await").count() >= 2,
            "expected both awaiting branches to emit `await`, got:\n{js}"
        );
        Ok(())
    }
}

test!(conditional_compilation => r#"type{true} foo = string;
type{false} foo = i64;

const {true}var = "Hello, World!";
const {false}var = 32;

infix{true} add as + precedence 7;
infix{false} add as + precedence 0;

type infix{true} Add as + precedence 7;
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
}
"#;
    stdout "Hello, World!\n9\n9\ntrue\n";
);
test_with_alan_target!(library_testing => r#"export fn add1(a: i64) -> i64 = a + 1;
export postfix add1 as ++ precedence 5;

export fn{Test} main {
  let a = 1;
  print(a++);
}
"#;
    stdout "2\n";
);

test!(compile_time_buffer_size => r#"// BUFSIZE is set by the `.cargo/config.toml` file
export fn main {
  {Buffer{i64, Int{Env{"BUFSIZE"}}}}(0).print;
}
"#;
    stdout_contains "0,";
);

test!(extend_type => r#"type Foo = bar: "bar";
type Foo = Unwrap{Foo}, baz: "baz";

export fn main {
  {String{Foo}}().print;
}
"#;
    stdout "Tuple{Field{bar, \"bar\"}, Field{baz, \"baz\"}}\n";
);

test!(what_type => r#"fn whatType{T} = {String{T}}().print;

export fn main {
  whatType{1}();
  whatType{ExitCode}();
  whatType{Tuple{ExitCode, "ExitCode"}}();
}
"#;
    stdout_rs "1\nBinds{\"std::process::ExitCode\"}\nTuple{Binds{\"std::process::ExitCode\"}, \"ExitCode\"}\n";
    stdout_js "1\nBinds{\"Number\"}\nTuple{Binds{\"Number\"}, \"ExitCode\"}\n";
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
test!(object_literals => r#"type MyType =
  foo: string,
  bar: bool;

export fn main {
  const test = MyType('foo!', true);
  print(test.foo);
  print(test.bar);
}
"#;
    stdout "foo!\ntrue\n";
);
test!(object_and_array_reassignment => r#"type Foo =
  bar: bool;

export fn main {
  let test = [1, 2, 3];
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
}
"#;
    stdout "1\n0\n2\ntrue\nfalse\n";
);

test!(array_custom_types => r#"type Foo =
  foo: string,
  bar: bool;

export fn main {
  const five = [1, 2, 3, 4, 5];
  five.map(fn(n: i64) {
    return Foo(n.string, n % 2 == 0);
  }).filter(fn(f: Foo) = f.bar).map(fn(f: Foo) = f.foo).join(', ').print;
}
"#;
    stdout "2, 4\n";
);

// Generics

test!(generics => r#"type box{V} =
  val: V,
  set: bool;

export fn main {
  let i8Box = box{i8}(8.i8, true);
  print(i8Box.val);
  print(i8Box.set);

  let stringBox = box{string}('hello, generics!', true);
  print(stringBox.val);
  print(stringBox.set);

  const stringBoxBox = box{box{string}}(box{string}('hello, nested generics!', true), true);
  stringBoxBox.set.print;
  stringBoxBox.val.set.print;
  print(stringBoxBox.val.val);
}
"#;
    stdout r#"8
true
hello, generics!
true
true
true
hello, nested generics!
"#;
);
test!(generic_functions => r#"fn empty{T}() = Array{T}(); // Pointless, but just for testing
export fn main {
  let foo = empty{i64}();
  print(foo);
}
"#;
    stdout "[]\n";
);
test!(generic_in_a_generic => r#"fn condition{T}(a: T, b: T) -> bool {
  return a == b;
}

fn batchCompare{T}(a: Array{T}, b: Array{T}, cond: (T, T) -> bool) {
  return a.map(fn(aVal: T) = b.some(fn(bVal: T) = cond(aVal, bVal)));
}

export fn main {
  let vals1 = [1, 9, 1];
  let vals2 = [1, 2, 3, 5, 7];

  batchCompare(vals1, vals2, condition).print;
}
"#;
    stdout_js "[ true, false, true ]\n";
    stdout_rs "[true, false, true]\n"; // TODO: Make these match
);
test!(first_arg_generic_fn => r#"fn batchCompare{T}(cond: (T, T) -> bool, a: Array{T}, b: Array{T}) {
  return a.map(fn(aVal: T) = b.some(fn(bVal: T) = cond(aVal, bVal)));
}

export fn main {
  let vals1 = [1, 9, 25];
  let vals2 = [1, 3, 5, 7, 9];

  batchCompare(eq, vals1, vals2).print;
}
"#;
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
test!(basic_type_import "type_foo" =>
    "type_bar.ln" => r#"export type Bar = "Bar";
"#,
    "type_foo.ln" => r#"type Bar <-- "./type_bar.ln";

export fn main {
  {Bar}().print;
}
"#;
    stdout "Bar\n";
);

test!(basic_fn_import "fn_foo" =>
    "fn_bar.ln" => r#"export fn bar = "Bar";
"#,
    "fn_foo.ln" => r#"fn bar <-- "./fn_bar.ln";

export fn main {
  bar().print;
}
"#;
    stdout "Bar\n";
);

test!(file_reading "file_reading" =>
    "test_file.txt" => "Hello, World!",
    "file_reading.ln" => r#"type File <-- '@std/fs';
fn string <-- '@std/fs'; // TODO: Should this be auto-imported?
export fn main {
  File('./test_file.txt').string.print;
}
"#;
    stdout "Hello, World!\n";
);

// Maybe, Result, and Either

test!(maybe => r#"// TODO: Rewrite these conditionals with conditional syntax once implemented
fn fiver(val: f64) = if(val.i64 == 5, fn = {i64?}(5), fn = {i64?}());

export fn main {
  const maybe5 = fiver(5.5);
  if(maybe5.exists, fn {
    maybe5.getOr(0).print;
  }, fn {
    'what?'.print;
  });

  const maybeNot5 = fiver(4.4);
  if(!maybeNot5.exists, fn {
    'Correctly received nothing!'.print;
  }, fn {
    'uhhh'.print;
  });

  maybe5.print;
  maybeNot5.print;
}
"#;
    stdout r#"5
Correctly received nothing!
5
void
"#;
);
test!(fallible => r#"// TODO: Rewrite these conditionals with conditional syntax once implemented
fn reciprocal(val: f64) =
  if(val == 0.0, fn {
    return Error{f64}('Divide by zero error!');
  }, fn {
    return Fallible{f64}(1.0 / val);
  });

export fn main {
  const oneFifth = reciprocal(5.0);
  if(oneFifth.f64.exists, fn {
    print(oneFifth.getOr(0.0));
  }, fn {
    print('what?');
  });

  const oneZeroth = reciprocal(0.0);
  if(oneZeroth.Error.exists, fn {
    print(oneZeroth.Error.getOr(Error('No error')));
  }, fn {
    print('uhhh');
  });

  oneFifth.print;
  oneZeroth.print;

  const res = Fallible{string}('foo');
  print(res.Error.getOr(Error('there is no error')));
}
"#;
    stdout r#"0.2
Error: Divide by zero error!
0.2
Error: Divide by zero error!
Error: there is no error
"#;
);

// Types

test!(user_types_and_generics => r#"type foo{A, B} =
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
}
"#;
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
test_compile_error!(totally_broken_statement => r#"
    on app.start {
      app.oops
    }"#;
);

// Malformed-input regression tests: parse errors must return cleanly, never abort.

#[cfg(test)]
mod malformed_input {
    fn write_and_compile(code: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("{name}.ln");
        std::fs::write(&filename, code)?;
        let res = crate::compile::compile(filename.clone());
        std::fs::remove_file(&filename)?;
        match res {
            Ok(_) => Err("Unexpectedly succeeded!".into()),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn deep_paren_nesting() -> Result<(), Box<dyn std::error::Error>> {
        let depth = 1000;
        let code = format!(
            "export fn main {{ let x = {}5{}; }}",
            "(".repeat(depth),
            ")".repeat(depth)
        );
        write_and_compile(&code, "deep_paren_nesting")
    }

    #[test]
    fn deep_array_nesting() -> Result<(), Box<dyn std::error::Error>> {
        let depth = 1000;
        let code = format!(
            "export fn main {{ let x = {}5{}; }}",
            "[".repeat(depth),
            "]".repeat(depth)
        );
        write_and_compile(&code, "deep_array_nesting")
    }

    #[test]
    fn deep_type_generic_nesting() -> Result<(), Box<dyn std::error::Error>> {
        let depth = 1000;
        let mut inner = "int64".to_string();
        for _ in 0..depth {
            inner = format!("Foo{{{inner}}}");
        }
        let code = format!("type Bar = {inner};\nexport fn main {{}}");
        write_and_compile(&code, "deep_type_generic_nesting")
    }

    #[test]
    fn truncated_input() -> Result<(), Box<dyn std::error::Error>> {
        write_and_compile("export fn main { print(", "truncated_input")
    }

    #[test]
    fn garbage_input() -> Result<(), Box<dyn std::error::Error>> {
        write_and_compile("@@@ not alan @@@", "garbage_input")
    }
}

// Module-level constants

test!(module_level_constant => r#"const helloWorld = 'Hello, World!';

export fn main {
  print(helloWorld);
}
"#;
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

test_gpgpu!(gpu_trig => r#"export fn main {
  'Logarithms and e^x'.print;
  // Contrived way to get the GPU to do this work, don't follow this pattern for real GPU usage
  GBuffer([e.f32]).getOrExit.map(fn(v: gf32) = exp(v)).read[0].getOrExit.string(2).print;
  GBuffer([e.f32]).getOrExit.map(fn(v: gf32) = ln(v)).read[0].getOrExit.string(2).print;
  GBuffer([e.f32]).getOrExit.map(fn(v: gf32) = log10(v)).read[0].getOrExit.string(2).print;
  GBuffer([e.f32]).getOrExit.map(fn(v: gf32) = log2(v)).read[0].getOrExit.string(2).print;

  'Basic Trig functions'.print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = sin(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = cos(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = tan(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = sec(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = csc(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = cot(v)).read[0].getOrExit.string(2).print;

  'Inverse Trig functions'.print;
  GBuffer([0.0.f32]).getOrExit.map(fn(v: gf32) = asin(v)).read[0].getOrExit.string(2).print;
  GBuffer([1.0.f32]).getOrExit.map(fn(v: gf32) = acos(v)).read[0].getOrExit.string(2).print;
  GBuffer([0.0.f32]).getOrExit.map(fn(v: gf32) = atan(v)).read[0].getOrExit.string(2).print;
  GBuffer([1.0.f32]).getOrExit.map(fn(v: gf32) = atan2(v, 2.0)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = asec(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = acsc(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = acot(v)).read[0].getOrExit.string(2).print;

  'Hyperbolic Trig functions'.print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = sinh(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = cosh(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = tanh(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = sech(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = csch(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = coth(v)).read[0].getOrExit.string(2).print;

  'Inverse Hyperbolic Trig functions'.print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = asinh(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = acosh(v)).read[0].getOrExit.string(2).print;
  GBuffer([pi.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = atanh(v)).read[0].getOrExit.string(2).print;
  GBuffer([0.5.f32]).getOrExit.map(fn(v: gf32) = asech(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = acsch(v)).read[0].getOrExit.string(2).print;
  GBuffer([tau.f32 / 6.0.f32]).getOrExit.map(fn(v: gf32) = acoth(v)).read[0].getOrExit.string(2).print;
}
"#;
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

// Runtime Error

test!(get_or_exit => r#"export fn main {
  const xs = [0, 1, 2, 5];
  const x1 = xs[1].getOrExit;
  print(x1);
  const x2 = xs[2].getOrExit;
  print(x2);
  const x5 = xs[5].getOrExit;
  print(x5);
}
"#;
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
test!(seq_while => r#"fn while <-- '@std/seq';

export fn main {
  let sum = 0;
  while(fn = sum < 10, fn {
    // sum = sum + 1;
    let s2 = sum.clone; // Need to fix Rust codegen to do this right
    sum = s2 + 1;
  });
  print(sum);
}
"#;
    stdout "10\n";
);
test!(seq_iter => r#"fn iter <-- '@std/seq';

export fn main {
  let sum = 0;
  fn(i: i64) {
    let s2 = sum.clone; // TODO: Fix rust codegen
    sum = s2 + i * i;
  }.iter(10);
  print(sum);
  let arr = fn(i: i64) {
    return i * i;
  }.iter(10);
  arr.map(string).join(', ').print;
}
"#;
    stdout "285\n0, 1, 4, 9, 16, 25, 36, 49, 64, 81\n";
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

test!(tree_user_defined_types => r#"
    type Foo =
      foo: string,
      bar: bool;

    export fn main {
      const myTree = Tree(Foo('myFoo', false));
      const myFoo = myTree.rootNode ?? Foo('wrongFoo', false);
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

test!(eprint => r#"export fn main {
  eprint('This is an error');
}
"#;
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

// Either deduplication

test!(either_dedup_explicit => r#"type DupEither = i64 | string | i64;
type NormalEither = i64 | string;
fn takeEither(x: NormalEither) = x.print;
export fn main {
  takeEither(42.DupEither);
}
"#;
  stdout "42\n";
);

// Rest{T} wrapper type tests for parent constructors

test!(rest_tuple_parent_constructor => r#"type MyTuple = a: i64, b: string, c: bool;
type MyFiltered = Rest{MyTuple};
export fn main {
  let v: MyTuple = MyTuple(42, "hello", true);
  let res: MyFiltered = MyFiltered(v);
  res.b.print;
  res.c.print;
}
"#;
  stdout "hello\ntrue\n";
);

test!(rest_tuple_parent_constructor_recursive => r#"type Grandparent = a: i64, b: string, c: bool, d: f64;
type Parent = Rest{Grandparent};
type Child = Rest{Parent};
export fn main {
  let gp: Grandparent = Grandparent(42, "hello", true, 3.14);
  let p: Parent = Parent(gp);
  let c: Child = Child(p);
  c.c.print;
  c.d.print;
}
"#;
  stdout "true\n3.14\n";
);

test!(rest_either_parent_constructor => r#"type MyEither = i64 | string | bool;
type MyFiltered = Rest{MyEither};
export fn main {
  let v: MyEither = "hello".MyEither;
  let res: Maybe{MyFiltered} = MyFiltered(v);
  if(res.exists, fn {
    res.getOrExit.print;
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "hello\n";
);

test!(rest_either_parent_constructor_none => r#"type MyEither = i64 | string | bool;
type MyFiltered = Rest{MyEither};
export fn main {
  let v: MyEither = 42.MyEither;
  let res: Maybe{MyFiltered} = MyFiltered(v);
  if(res.exists, fn {
    res.getOrExit.print;
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "void\n";
);

test!(rest_either_parent_constructor_recursive => r#"type Grandparent = i64 | string | bool | f64;
type Parent = Rest{Grandparent};
type Child = Rest{Parent};
export fn main {
  let gp: Grandparent = true.Grandparent;
  let p: Maybe{Parent} = Parent(gp);
  if(p.exists, fn {
    let c: Maybe{Child} = Child(p.getOrExit);
    if(c.exists, fn {
      c.getOrExit.print;
    }, fn {
      'void'.print;
    });
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "true\n";
);

test!(rest_either_parent_constructor_recursive_none => r#"type Grandparent = i64 | string | bool | f64;
type Parent = Rest{Grandparent};
type Child = Rest{Parent};
export fn main {
  let gp: Grandparent = "hello".Grandparent;
  let p: Maybe{Parent} = Parent(gp);
  if(p.exists, fn {
    let c: Maybe{Child} = Child(p.getOrExit);
    if(c.exists, fn {
      c.getOrExit.print;
    }, fn {
      'void'.print;
    });
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "void\n";
);

test!(either_dedup_single_unwrap => r#"type DupeI64 = i64 | i64;
export fn main {
  let a: i64 = 3;
  let b: DupeI64 = 5;
  print(a.add(b));
}
"#;
    stdout "8\n";
);

// Exclude constructor tests

test!(exclude_tuple_by_index => r#"type MyTuple = a: i64, b: string, c: bool;
type MyFiltered = Exclude{MyTuple, 1};
export fn main {
  let v: MyTuple = MyTuple(42, "hello", true);
  let res: MyFiltered = MyFiltered(v.a, v.c);
  res.a.print;
  res.c.print;
}
"#;
  stdout "42\ntrue\n";
);

test!(exclude_tuple_by_index_first => r#"type MyTuple2 = a: i64, b: string, c: bool;
type MyFiltered = Exclude{MyTuple2, 0};
export fn main {
  let v: MyTuple2 = MyTuple2(42, "hello", true);
  let res: MyFiltered = MyFiltered(v.b, v.c);
  res.b.print;
  res.c.print;
}
"#;
  stdout "hello\ntrue\n";
);

test!(exclude_tuple_parent_constructor => r#"type MyTuple = a: i64, b: string, c: bool;
type MyFiltered = Exclude{MyTuple, 1};
export fn main {
  let v: MyTuple = MyTuple(42, "hello", true);
  let res: MyFiltered = MyFiltered(v);
  res.a.print;
  res.c.print;
}
"#;
  stdout "42\ntrue\n";
);

test!(exclude_tuple_parent_constructor_recursive => r#"type Grandparent = a: i64, b: string, c: bool, d: f64;
type Parent = Exclude{Grandparent, 3};
type Child = Exclude{Parent, 1};
export fn main {
  let gp: Grandparent = Grandparent(42, "hello", true, 3.14);
  let p: Parent = Parent(gp);
  let c: Child = Child(p);
  c.a.print;
  c.c.print;
}
"#;
  stdout "42\ntrue\n";
);

// Exclude Either tests

test!(exclude_either_by_index => r#"type MyEither = i64 | string | bool;
type MyFiltered = Exclude{MyEither, 1};
export fn main {
  let v: MyEither = 42.MyEither;
  let res: MyFiltered = 42.MyFiltered;
  res.print;
}
"#;
  stdout "42\n";
);

test!(exclude_either_by_index_first => r#"type MyEither2 = i64 | string | bool;
type MyFiltered2 = Exclude{MyEither2, 0};
export fn main {
  let v: MyEither2 = true.MyEither2;
  let res: MyFiltered2 = true.MyFiltered2;
  res.print;
}
"#;
  stdout "true\n";
);

test!(exclude_either_parent_constructor => r#"type MyEither = i64 | string | bool;
type MyFiltered = Exclude{MyEither, 1};
export fn main {
  let v: MyEither = 42.MyEither;
  let res: Maybe{MyFiltered} = MyFiltered(v);
  if(res.exists, fn {
    res.getOrExit.print;
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "42\n";
);

test!(exclude_either_parent_constructor_none => r#"type MyEither = i64 | string | bool;
type MyFiltered = Exclude{MyEither, 1};
export fn main {
  let v: MyEither = "hello".MyEither;
  let res: Maybe{MyFiltered} = MyFiltered(v);
  if(res.exists, fn {
    res.getOrExit.print;
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "void\n";
);

test!(exclude_either_parent_constructor_recursive => r#"type Grandparent = i64 | string | bool | f64;
type Parent = Exclude{Grandparent, 3};
type Child = Exclude{Parent, 1};
export fn main {
  let gp: Grandparent = 42.Grandparent;
  let p: Maybe{Parent} = Parent(gp);
  if(p.exists, fn {
    let c: Maybe{Child} = Child(p.getOrExit);
    if(c.exists, fn {
      c.getOrExit.print;
    }, fn {
      'void'.print;
    });
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "42\n";
);

// Transitive parent constructor tests: constructing Child directly from Grandparent

test!(exclude_either_parent_constructor_direct_from_grandparent => r#"type Grandparent = i64 | string | bool | f64;
type Parent = Exclude{Grandparent, 3};
type Child = Exclude{Parent, 1};
export fn main {
  let gp: Grandparent = 42.Grandparent;
  let c: Maybe{Child} = Child(gp);
  if(c.exists, fn {
    c.getOrExit.print;
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "42\n";
);

test!(exclude_either_parent_constructor_direct_none => r#"type Grandparent = i64 | string | bool | f64;
type Parent = Exclude{Grandparent, 3};
type Child = Exclude{Parent, 1};
export fn main {
  let gp: Grandparent = "hello".Grandparent;
  let c: Maybe{Child} = Child(gp);
  if(c.exists, fn {
    c.getOrExit.print;
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "void\n";
);

test!(exclude_tuple_parent_constructor_direct_from_grandparent => r#"type Grandparent = a: i64, b: string, c: bool, d: f64;
type Parent = Exclude{Grandparent, 3};
type Child = Exclude{Parent, 1};
export fn main {
  let gp: Grandparent = Grandparent(42, "hello", true, 3.14);
  let c: Child = Child(gp);
  c.a.print;
  c.c.print;
}
"#;
  stdout "42\ntrue\n";
);

// Single-element and void reduction tests

test!(exclude_tuple_single_element_remains_tuple => r#"type MyTuple = a: i64, b: string;
type MySingle = Exclude{MyTuple, 1};
export fn main {
  let v: MyTuple = MyTuple(42, "hello");
  let res: MySingle = MySingle(v);
  res.a.print;
}
"#;
  stdout "42\n";
);

test!(exclude_either_single_element_remains_either => r#"type MyEither = i64 | string | bool;
type MySingle = Exclude{MyEither, 1};
export fn main {
  let v: MyEither = 42.MyEither;
  let res: Maybe{MySingle} = MySingle(v);
  if(res.exists, fn {
    res.getOrExit.i64.print;
  }, fn {
    'void'.print;
  });
}
"#;
  stdout "42\n";
);

test!(exclude_tuple_to_void => r#"type MyTuple = a: i64, b: string;
type MyPartial = Exclude{MyTuple, 1};
type MyVoid = Exclude{MyPartial, 0};
export fn main {
  let v: MyTuple = MyTuple(42, "hello");
  let p: MyPartial = MyPartial(v);
  let res: MyVoid = MyVoid(p);
  res.print;
}
"#;
  stdout "void\n";
);

test!(exclude_either_to_void => r#"type MyEither = i64 | string;
type MySingle = Exclude{MyEither, 1};
type MyVoid = Exclude{MySingle, 0};
export fn main {
  let v: MyEither = 42.MyEither;
  let s: Maybe{MySingle} = MySingle(v);
  let res: MyVoid = MyVoid(s.getOrExit);
  res.print;
}
"#;
  stdout "void\n";
);

// -.(dash-dot) "subtract property" Exclude operator syntax tests

test!(exclude_tuple_dashdot_chained => r#"type MyTuple = a: i64, b: string, c: bool;
type MySingle = MyTuple-.a-.c;
export fn main {
  let v: MyTuple = MyTuple(42, "hello", true);
  let res: MySingle = MySingle(v);
  res.b.print;
}
"#;
  stdout "hello\n";
);

// Len{Void} returns 0 tests (Void created by excluding all fields from a tuple)

test!(len_void_returns_zero => r#"type SingleField = a: i64;
export fn main {
  if({Len{Exclude{SingleField, 0}}}() == 0, fn {
    'zero'.print;
  }, fn {
    'not zero'.print;
  });
}
"#;
  stdout "zero\n";
);

test!(len_void_numeric_output => r#"type SingleField = a: i64;
export fn main {
  print({Len{Exclude{SingleField, 0}}}());
}
"#;
  stdout "0\n";
);

test!(len_void_vs_types => r#"type SingleField = a: i64;
export fn main {
  print({Len{Exclude{SingleField, 0}}}());
  print({Len{i64}}());
}
"#;
  stdout "0\n1\n";
);

// Eq and Neq type comparison tests

test!(eq_same_types_returns_true => r#"export fn main {
  if({Eq{i64, i64}}(), fn {
    'true'.print;
  }, fn {
    'false'.print;
  });
}
"#;
  stdout "true\n";
);

test!(eq_different_types_returns_false => r#"export fn main {
  if({Eq{i64, string}}(), fn {
    'true'.print;
  }, fn {
    'false'.print;
  });
}
"#;
  stdout "false\n";
);

test!(neq_different_types_returns_true => r#"export fn main {
  if({Neq{i64, string}}(), fn {
    'true'.print;
  }, fn {
    'false'.print;
  });
}
"#;
  stdout "true\n";
);

test!(neq_same_types_returns_false => r#"export fn main {
  if({Neq{string, string}}(), fn {
    'true'.print;
  }, fn {
    'false'.print;
  });
}
"#;
  stdout "false\n";
);

test!(eq_generic_type_check => r#"export fn main {
  if({Eq{i64, i64}}(), fn {
    'is i64'.print;
  }, fn {
    'not i64'.print;
  });
  if({Eq{string, i64}}(), fn {
    'is i64'.print;
  }, fn {
    'not i64'.print;
  });
  if({Eq{bool, i64}}(), fn {
    'is i64'.print;
  }, fn {
    'not i64'.print;
  });
}
"#;
  stdout "is i64\nnot i64\nnot i64\n";
);

test!(neq_generic_type_guard => r#"type SingleField = a: i64;
type MyVoid = Exclude{SingleField, 0};
fn isNotVoid{T}() -> bool = {Neq{T, Exclude{SingleField, 0}}}();

export fn main {
  if(isNotVoid{i64}(), fn {
    'i64 is not void'.print;
  }, fn {
    'unexpected'.print;
  });
  if(isNotVoid{MyVoid}(), fn {
    'unexpected'.print;
  }, fn {
    'void is void'.print;
  });
}
"#;
  stdout "i64 is not void\nvoid is void\n";
);

test!(eq_tuple_types => r#"type MyTuple = i64, string;
type SameTuple = i64, string;
type DiffTuple = string, i64;
export fn main {
  if({Eq{MyTuple, SameTuple}}(), fn {
    'same'.print;
  }, fn {
    'different'.print;
  });
  if({Eq{MyTuple, DiffTuple}}(), fn {
    'same'.print;
  }, fn {
    'different'.print;
  });
}
"#;
  stdout "same\ndifferent\n";
);

test!(anonymous_tuple_construction => r#"export fn main {
  let testTuple = (1, "test");
  testTuple.0.print;
  testTuple.1.print;
}
"#;
  stdout "1\ntest\n";
);

test!(void_literal_construction => r#"export fn main {
  let v = ();
  print(v);
}
"#;
  stdout "void\n";
);

test!(len_and_eq_combined => r#"type SingleField = a: i64;
fn emptyType{T}() -> bool = {Len{T}}() == 0;

export fn main {
  if(emptyType{Exclude{SingleField, 0}}(), fn {
    'Void is empty'.print;
  }, fn {
    'Void not empty'.print;
  });
  if(emptyType{i64}(), fn {
    'i64 is empty'.print;
  }, fn {
    'i64 not empty'.print;
  });
}
"#;
  stdout "Void is empty\ni64 not empty\n";
);

test!(shared_basic => r#"export fn main {
  let shared = {Shared{i64}}(42);

  let a = shared;
  let b = shared;

  a.store(100);
  b.print;
}
"#;
  stdout "100\n";
);

test!(shared_clone => r#"export fn main {
  let original = {Shared{i64}}(42);
  let copy = original.clone;

  original.store(100);
  copy.print;
}
"#;
  stdout "42\n";
);
