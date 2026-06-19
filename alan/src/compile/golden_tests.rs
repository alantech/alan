/// Golden-output tests for lntors and lntojs.
///
/// These tests compile a representative .ln source file (test.ln) through both code generators
/// and compare the output against checked-in snapshots. Run with `UPDATE_GOLDEN=1` to regenerate
/// the snapshots after intentional code-generation changes.
///
/// NOTE: the code generators use deep recursion which overflows the default 2 MiB thread stack
/// in debug mode. Run these tests with `RUST_MIN_STACK=33554432 cargo test -p alan golden`.

#[cfg(test)]
mod golden {
    use std::path::Path;

    use alan_compiler::lntojs::lntojs;
    use alan_compiler::lntors::lntors;
    use alan_compiler::program::Program;

    const GOLDEN_DIR: &str = "testdata/golden";
    const INPUT_FILE: &str = "test.ln";

    /// Get the current program, insert ALAN_TARGET=test into its env, and return it.
    /// Must be called after `set_target_lang_rs()` or `set_target_lang_js()` so the
    /// correct thread_local cell is accessed.
    fn seed_test_env() {
        let mut program = Program::get_program();
        program
            .env
            .insert("ALAN_TARGET".to_string(), "test".to_string());
        Program::return_program(program);
    }

    /// Assert that `actual` matches the golden file at `{GOLDEN_DIR}/{name}.golden`.
    /// If `UPDATE_GOLDEN` env var is set, overwrites the golden file instead.
    fn assert_golden(name: &str, actual: &str) {
        let golden_path = format!("{GOLDEN_DIR}/{name}.golden");
        let golden = Path::new(&golden_path);

        if std::env::var("UPDATE_GOLDEN").is_ok() {
            std::fs::create_dir_all(GOLDEN_DIR)
                .expect("Failed to create golden directory");
            std::fs::write(golden, actual).expect("Failed to write golden file");
            return;
        }

        let expected = std::fs::read_to_string(golden).unwrap_or_else(|_| {
            panic!(
                "Golden file `{golden_path}` not found.\n\
                 Run with `UPDATE_GOLDEN=1 cargo test -p alan golden` to generate it."
            )
        });

        if expected != actual {
            let exp_lines: Vec<&str> = expected.lines().collect();
            let act_lines: Vec<&str> = actual.lines().collect();
            for (i, (a, b)) in exp_lines.iter().zip(act_lines.iter()).enumerate() {
                if a != b {
                    eprintln!("First difference at line {}:", i + 1);
                    eprintln!("  expected: {a}");
                    eprintln!("  actual:   {b}");
                    break;
                }
            }
            if exp_lines.len() != act_lines.len() {
                eprintln!(
                    "Line count differs: expected {}, actual {}",
                    exp_lines.len(),
                    act_lines.len()
                );
            }
            panic!(
                "Golden output mismatch for `{name}`.\n\
                 Run with `UPDATE_GOLDEN=1 cargo test -p alan golden` to accept the changes."
            );
        }
    }

    #[test]
    fn golden_rs_output() {
        Program::set_target_lang_rs();
        seed_test_env();
        let (rs_output, _deps) = lntors(INPUT_FILE.to_string())
            .expect("lntors failed to generate Rust code from test.ln");
        assert_golden("lntors_test", &rs_output);
    }

    #[test]
    fn golden_js_output() {
        Program::set_target_lang_js();
        seed_test_env();
        let (js_output, _deps) = lntojs(INPUT_FILE.to_string())
            .expect("lntojs failed to generate JS code from test.ln");
        assert_golden("lntojs_test", &js_output);
    }
}
