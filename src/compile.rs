// TODO: Figure out how to integrate `rustc` into the `alan` binary.
use std::fs::{remove_file, write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::lntors::lntors;

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

#[cfg(test)]
mod test_compile {
    #[test]
    fn hello_world() -> Result<(), Box<dyn std::error::Error>> {
        // Simple Hello, World app test
        // TODO: Switch to proposed fully-qualified function export instead of event-based start
        super::write("./hello.ln", r#"
            on start {
                print("Hello, World!");
            }
        "#)?;
        assert_eq!((), super::compile("./hello.ln".to_string())?);
        // Confirm the generated binary does what it should
        let hello_world = String::from_utf8(super::Command::new("./hello").output()?.stdout)?;
        // Drop the generated files first just in case
        super::remove_file("./hello.ln")?;
        super::remove_file("./hello")?;
        assert_eq!("Hello, World!\n", &hello_world);
        Ok(())
    }
}
