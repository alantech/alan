use std::env::{current_dir, set_var};
use std::fs::{create_dir_all, remove_file, write, File};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use dirs::config_dir;
use fs2::FileExt;

use crate::lntors::lntors;

mod integration_tests;

/// The `build` function creates a temporary directory that is a Cargo project primarily consisting
/// of a single source file, plus a Cargo.toml file including the 3rd party dependencies in the
/// standard library. While this *should* be some configurable thing in the standard library code
/// instead, the contents of the Cargo.toml are just hardwired in here for now.
pub fn build(source_file: String) -> Result<String, Box<dyn std::error::Error>> {
    let find_process = if cfg!(windows) { "where" } else { "which" };
    // Fail if rustc is not present
    match Command::new(find_process).arg("rustc").output() {
        Ok(a) => Ok(a),
        Err(_) => {
            Err("rustc not found. Please make sure you have rust installed before using Alan!")
        }
    }?;
    // Also fail if cargo is not present
    match Command::new(find_process).arg("cargo").output() {
        Ok(a) => Ok(a),
        Err(_) => {
            Err("cargo not found. Please make sure you have rust installed before using Alan!")
        }
    }?;
    // Because all Alan programs use the same Rust dependencies (for now), we can cut down a *lot*
    // of build time by re-using the `./target/release/build` and `./target/release/deps` directory
    // in subsequent builds. Since it takes over 30 seconds to make a release build on my laptop
    // there needs to be a multi-step process to detect if there's a concurrent build happening
    // that we should wait for. First we need to look for a `{CONFIG}/alan` directory. If it's not
    // there, make one, build a guaranteed Hello, World app within it, then use it in the regular
    // build flow. If it *is* there *and* the `hello_world` application is present, we need to see
    // if another Alan compile is concurrently running, if so, we sleep wait until it is gone
    // (either the lockfile is deleted or the process ID in the lockfile is no longer running and
    // then we delete it and continue. Then we continue with the regular build flow.
    let config_dir = match config_dir() {
        Some(c) => Ok(c),
        None => Err("Somehow no configuration directory exists on this operating system"),
    }?;
    let alan_config = {
        // All this because `push` is not chainable :/
        let mut a = config_dir.clone();
        a.push("alan");
        a
    };
    let lockfile_path = {
        let mut l = alan_config.clone();
        l.push(".lockfile");
        l
    };
    let project_dir = {
        let mut p = alan_config.clone();
        p.push("alan_generated_bin");
        p
    };
    let release_path = {
        let mut r = project_dir.clone();
        r.push("target");
        r.push("release");
        r
    };
    let cargo_str = r#"[package]
name = "alan_generated_bin"
edition = "2021"

[dependencies]
flume = "0.11.0"
futures = "0.3.30"
wgpu = "0.20.1""#;
    let cargo_path = {
        let mut c = project_dir.clone();
        c.push("Cargo.toml");
        c
    };
    let first_time = !alan_config.exists() || !lockfile_path.exists();
    if first_time {
        create_dir_all(alan_config.clone())?;
        write(
            lockfile_path.clone(),
            format!(
                "{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
            )
            .as_bytes(),
        )?;
    }
    let mut lockfile = File::open(lockfile_path.as_path())?;
    if cfg!(windows) {
        let timeout = std::time::Duration::from_secs(180);
        let sleep_time = std::time::Duration::from_millis(100);
        let mut now = std::time::Instant::now();
        let expiry = now + timeout;
        let mut locked = false;
        while now < expiry {
            match lockfile.lock_exclusive() {
                Err(_) => std::thread::sleep(sleep_time),
                Ok(_) => {
                    locked = true;
                    break;
                }
            }
            now = std::time::Instant::now();
        }
        if !locked {
            return Err("Could not lock the lockfile".into());
        }
    } else {
        lockfile.lock_exclusive()?;
    }
    let should_rebuild_deps = {
        let mut b = Vec::new();
        lockfile.read_to_end(&mut b)?;
        let t1 = match String::from_utf8(b) {
            Ok(s) => s.parse::<u64>().unwrap_or(0),
            Err(_) => 0,
        };
        let t2 = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        t2 > t1 + 24 * 60 * 60
    };
    if first_time {
        // First time initialization of the alan config directory
        match create_dir_all(project_dir.clone()) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        match write(cargo_path.clone(), cargo_str) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        let src_path = {
            let mut s = project_dir.clone();
            s.push("src");
            s
        };
        match create_dir_all(src_path.clone()) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        let hello_ln = "export fn main = print('Hello, World!');";
        let hello_path = {
            let mut l = src_path.clone();
            l.push("hello.ln");
            l
        };
        match write(hello_path.clone(), hello_ln) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        let rs_str = match lntors(hello_path.to_string_lossy().to_string()) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        let main_path = {
            let mut m = src_path.clone();
            m.push("main.rs");
            m
        };
        match write(main_path, rs_str) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        match remove_file(hello_path) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        match Command::new("cargo")
            .current_dir(project_dir.clone())
            .arg("build")
            .arg("--release")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
    }
    // Always write the `Cargo.toml` file, in case the cache is out-of-date from a prior version of
    // the Alan compiler is still present.
    match write(cargo_path, cargo_str) {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(e)
        }
    }?;
    // We need to remove the prior binary, if it exists, to prevent a prior successful compilation
    // from accidentally being treated as the output of an unsuccessful compilation.
    match Command::new("rm")
        .current_dir(release_path.clone())
        .arg("-f")
        .arg("alan_generated_bin")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(e)
        }
    }?;
    // Once we're here, the base hello world app we use as a build cache definitely exists, so
    // let's get to work! We can't use the `?` operator directly here, because we need to make sure
    // we remove the lockfile on any failure.
    let src_dir = {
        let mut s = project_dir.clone();
        s.push("src");
        s
    };
    // Generate the rust code to compile
    let rs_str = match lntors(source_file.clone()) {
        Ok(s) => Ok(s),
        Err(e) => {
            lockfile.unlock()?;
            Err(e)
        }
    }?;
    // Shove it into a temp file for rustc
    let rs_path = {
        let mut r = src_dir.clone();
        r.push("main.rs");
        r
    };
    match write(rs_path, rs_str) {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(e)
        }
    }?;
    // Update the cargo lockfile, if necessary
    if should_rebuild_deps {
        match Command::new("cargo")
            .current_dir(project_dir.clone())
            .arg("update")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        if cfg!(windows) {
            lockfile.unlock()?;
        } // Why is this necessary?
        write(
            lockfile_path.clone(),
            format!(
                "{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
            )
            .as_bytes(),
        )?;
        if cfg!(windows) {
            lockfile.lock_exclusive()?;
        }
    }
    // Build the executable
    match Command::new("cargo")
        .current_dir(project_dir.clone())
        .arg("build")
        .arg("--release")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(o) => match o {
            o if !o.status.success() => {
                eprintln!("Compilation failed after successful translation to Rust. Likely something is wrong with the bindings.");
                eprintln!("{}", String::from_utf8(o.stdout).unwrap());
                eprintln!("{}", String::from_utf8(o.stderr).unwrap());
                Err("Rust compilation error".to_string())
            }
            _ => Ok(o),
        },
        Err(e) => {
            lockfile.unlock()?;
            Err(format!("{}", e))
        }
    }?;
    // Copy the binary from the temp directory to the current directory
    let project_name_path = PathBuf::from(source_file);
    let project_name_str = match project_name_path.file_stem() {
        None => panic!("Somehow can't parse the source file name as a path?"),
        Some(n) => n.to_string_lossy().to_string(),
    };
    match Command::new("cp")
        .current_dir(release_path)
        .arg("alan_generated_bin")
        .arg(format!(
            "{}/{}",
            current_dir()?.to_string_lossy(),
            project_name_str
        ))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(e)
        }
    }?;
    // Drop the lockfile
    lockfile.unlock()?;
    Ok(project_name_str)
}

/// The `compile` function is a thin wrapper on top of `build` that builds an executable in release
/// mode and exits, printing the time it took to run on success.
pub fn compile(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    set_var("ALAN_TARGET", "release");
    build(source_file)?;
    println!("Done! Took {:.2}sec", start_time.elapsed().as_secs_f32());
    Ok(())
}

/// The `test` function is a thin wrapper on top of `compile` that compiles the specified file in
/// test mode, then immediately invokes it, and deletes the binary when done.
pub fn test(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    set_var("ALAN_TARGET", "test");
    let binary = build(source_file)?;
    let mut run = Command::new(format!("./{}", binary))
        .current_dir(current_dir()?)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let ecode = run.wait()?;
    Command::new("rm")
        .current_dir(current_dir()?)
        .arg(binary)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !ecode.success() {
        std::process::exit(ecode.code().unwrap());
    }
    Ok(())
}

/// The `to_rs` function is an thin wrapper on top of `lntors` that shoves the output into a `.rs`
/// file.
pub fn to_rs(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    set_var("ALAN_TARGET", "release");
    // Generate the rust code to compile
    let rs_str = lntors(source_file.clone())?;
    // Shove it into a temp file for rustc
    let out_file = match PathBuf::from(source_file).file_stem() {
        Some(pb) => format!("{}.rs", pb.to_string_lossy()),
        None => {
            return Err("Invalid path".into());
        }
    };
    write(out_file, rs_str)?;
    Ok(())
}
