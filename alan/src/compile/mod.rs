use std::env::{current_dir, set_var, var};
use std::fs::{create_dir_all, remove_file, write, File};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use dirs::config_dir;
use fs2::FileExt;

use crate::lntojs::lntojs;
use crate::lntors::lntors;

mod integration_tests;

/// The `build` function creates a temporary directory that is a Cargo project primarily consisting
/// of a single source file, plus a Cargo.toml file including the 3rd party dependencies in the
/// standard library and user source code.
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

[dependencies]"#;
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
        let (rs_str, deps) = match lntors(hello_path.to_string_lossy().to_string()) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(e)
            }
        }?;
        match write(
            cargo_path.clone(),
            format!(
                "{}\n{}",
                cargo_str,
                deps.iter()
                    .map(|(k, v)| {
                        if v.starts_with("http") {
                            format!("{} = {{ git = \"{}\" }}", k, v)
                        } else {
                            format!("{} = \"{}\"", k, v)
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        ) {
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
    let (rs_str, deps) = match lntors(source_file.clone()) {
        Ok(s) => Ok(s),
        Err(e) => {
            lockfile.unlock()?;
            Err(e)
        }
    }?;
    // Always write the `Cargo.toml` file, in case the cache is out-of-date from a prior version of
    // the Alan compiler is still present.
    match write(
        cargo_path.clone(),
        format!(
            "{}\n{}",
            cargo_str,
            deps.iter()
                .map(|(k, v)| {
                    if v.starts_with("http") {
                        format!("{} = {{ git = \"{}\" }}", k, v)
                    } else {
                        format!("{} = \"{}\"", k, v)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        ),
    ) {
        Ok(a) => Ok(a),
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
    if var("ALAN_TARGET").is_err() {
        set_var("ALAN_TARGET", "release");
    }
    set_var("ALAN_OUTPUT_LANG", "rs");
    build(source_file)?;
    println!("Done! Took {:.2}sec", start_time.elapsed().as_secs_f32());
    Ok(())
}

/// The `test` function is a thin wrapper on top of `compile` that compiles the specified file in
/// test mode, then immediately invokes it, and deletes the binary when done.
pub fn test(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    set_var("ALAN_TARGET", "test");
    set_var("ALAN_OUTPUT_LANG", "rs");
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

/// The `web` function creates a temporary directory that is an NPM project, primarily consisting
/// of a single source file, plus a package.json file including third party dependencies in the
/// standard library and user source code.
pub fn web(source_file: String) -> Result<String, Box<dyn std::error::Error>> {
    let find_process = if cfg!(windows) { "where" } else { "which" };
    // Fail if node is not present
    match Command::new(find_process).arg("node").output() {
        Ok(a) => Ok(a),
        Err(_) => {
            Err("node not found. Please make sure you have node.js installed before using Alan!")
        }
    }?;
    // Also fail if npm is not present
    match Command::new(find_process).arg("npm").output() {
        Ok(a) => Ok(a),
        Err(_) => {
            Err("npm not found. Please make sure you have node.js installed before using Alan!")
        }
    }?;
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
        p.push("alan_generated_bundle");
        p
    };
    let package_json_path = {
        let mut c = project_dir.clone();
        c.push("package.json");
        c
    };
    let first_time = !alan_config.exists() || !lockfile_path.exists();
    if first_time {
        create_dir_all(alan_config.clone())?;
        match write(
            lockfile_path.clone(),
            format!(
                "{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
            )
            .as_bytes(),
        ) {
            Ok(a) => Ok(a),
            Err(e) => Err(format!("Failed to write to lockfile: {:?}", e)),
        }?;
    }
    let lockfile = File::open(lockfile_path.as_path())?;
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
        match lockfile.lock_exclusive() {
            Ok(a) => Ok(a),
            Err(e) => Err(format!("Could not acquire the lock {:?}", e)),
        }?;
    }
    if first_time || !project_dir.exists() {
        // First time initialization of the alan config directory
        match create_dir_all(project_dir.clone()) {
            Ok(a) => Ok(a),
            Err(e) => {
                lockfile.unlock()?;
                Err(format!("Could not create the project directory {:?}", e))
            }
        }?;
    }
    // We need to remove the prior bundle, if it exists, to prevent a prior successful compilation
    // from accidentally being treated as the output of an unsuccessful compilation.
    match Command::new("rm")
        .current_dir(project_dir.clone())
        .arg("bundle.js")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(format!("Could not delete the prior bundle.js file {:?}", e))
        }
    }?;
    // Generate the js code to bundle
    let (js_str, deps) = match lntojs(source_file.clone()) {
        Ok(s) => Ok(s),
        Err(e) => {
            lockfile.unlock()?;
            Err(format!("Could not generate the Javascript code {:?}", e))
        }
    }?;
    // Always write the `package.json` file, in case the cache is out-of-date from a prior version of
    // the Alan compiler is still present.
    match write(
        package_json_path.clone(),
        format!(
            "{{\n  \"name\": \"alan_generated_bundle\",\n  \"main\": \"index.js\",\n  \"dependencies\": {{\n    {}\n  }},\n  \"devDependencies\": {{\n    \"rollup\": \"4.x\",\n    \"@rollup/plugin-node-resolve\": \"15.x\"\n  }}\n}}",
            deps.iter()
                .map(|(k, v)| {
                    format!("    \"{}\": \"{}\"", k, v)
                })
                .collect::<Vec<String>>()
                .join(",\n")
        ),
    ) {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(format!("Could not create the package.json file {:?}", e))
        }
    }?;
    // Shove it into a temp file for rustc
    let js_path = {
        let mut r = project_dir.clone();
        r.push("index.js");
        r
    };
    match write(js_path, js_str) {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(format!(
                "Could not save the generated Javascript to disk {:?}",
                e
            ))
        }
    }?;
    // Update the npm lockfile, if necessary
    match Command::new("rm")
        .current_dir(project_dir.clone())
        .arg("-r")
        .arg("node_modules/")
        .arg("package-lock.json")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(format!("Could not clear package-lock.json {:?}", e))
        }
    }?;
    match Command::new(if cfg!(windows) { "npm.cmd" } else { "npm" })
        .current_dir(project_dir.clone())
        .arg("install")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(a) => Ok(a),
        Err(e) => {
            lockfile.unlock()?;
            Err(format!("Could not run npm install {:?}", e))
        }
    }?;
    // Build the bundle
    match if cfg!(windows) {
        Command::new("node")
            .current_dir(project_dir.clone())
            .arg("./node_modules/rollup/dist/bin/rollup")
            .arg("index.js")
            .arg("--format")
            .arg("iife")
            .arg("--name")
            .arg("alanGeneratedBundle")
            .arg("-p")
            .arg("@rollup/plugin-node-resolve")
            .arg("--file")
            .arg("bundle.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    } else {
        Command::new("./node_modules/.bin/rollup")
            .current_dir(project_dir.clone())
            .arg("index.js")
            .arg("--format")
            .arg("iife")
            .arg("--name")
            .arg("alanGeneratedBundle")
            .arg("-p")
            .arg("@rollup/plugin-node-resolve")
            .arg("--file")
            .arg("bundle.js")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    } {
        Ok(o) => match o {
            o if !o.status.success() => {
                eprintln!("Compilation failed after successful translation to Javascript. Likely something is wrong with the bindings.");
                eprintln!("{}", String::from_utf8(o.stdout).unwrap());
                eprintln!("{}", String::from_utf8(o.stderr).unwrap());
                Err("Javascript compilation error".to_string())
            }
            _ => Ok(o),
        },
        Err(e) => {
            lockfile.unlock()?;
            Err(format!("Could not generate the bundle.js file {:?}", e))
        }
    }?;
    // Copy the bundle from the temp directory to the current directory
    let project_name_path = PathBuf::from(source_file);
    let project_name_str = match project_name_path.file_stem() {
        None => panic!("Somehow can't parse the source file name as a path?"),
        Some(n) => n.to_string_lossy().to_string(),
    };
    match Command::new("cp")
        .current_dir(project_dir)
        .arg("bundle.js")
        .arg(format!(
            "{}/{}.js",
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
            Err(format!(
                "Could not copy the bundled Javascript to the PWD {:?}",
                e
            ))
        }
    }?;
    // Drop the lockfile
    lockfile.unlock()?;
    Ok(project_name_str)
}

/// The `bundle` function is a thin wrapper on top of `web` that builds an executable in release
/// mode and exits, printing the time it took to run on success.
pub fn bundle(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    if var("ALAN_TARGET").is_err() {
        set_var("ALAN_TARGET", "release");
    }
    set_var("ALAN_OUTPUT_LANG", "js");
    web(source_file)?;
    println!("Done! Took {:.2}sec", start_time.elapsed().as_secs_f32());
    Ok(())
}

/// The `to_rs` function is an thin wrapper on top of `lntors` that shoves the output into a `.rs`
/// file.
pub fn to_rs(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    if var("ALAN_TARGET").is_err() {
        set_var("ALAN_TARGET", "release");
    }
    set_var("ALAN_OUTPUT_LANG", "rs");
    // Generate the rust code to compile
    let (rs_str, deps) = lntors(source_file.clone())?;
    // Shove it into a temp file for rustc
    let out_file = match PathBuf::from(source_file.clone()).file_stem() {
        Some(pb) => format!("{}.rs", pb.to_string_lossy()),
        None => {
            return Err("Invalid path".into());
        }
    };
    write(out_file, rs_str)?;
    if !deps.is_empty() {
        let cargo_str = format!(
            "[package]\nname = \"{}\"\nedition = \"2021\"\n\n[dependencies]\n{}",
            PathBuf::from(source_file)
                .file_stem()
                .unwrap()
                .to_string_lossy(),
            deps.iter()
                .map(|(k, v)| {
                    if v.starts_with("http") {
                        format!("{} = {{ git = \"{}\" }}", k, v)
                    } else {
                        format!("{} = \"{}\"", k, v)
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        );
        write("Cargo.toml", cargo_str)?;
    }
    Ok(())
}

/// The `to_js` function is an thin wrapper on top of `lntojs` that shoves the output into a `.js`
/// file.
pub fn to_js(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    if var("ALAN_TARGET").is_err() {
        set_var("ALAN_TARGET", "release");
    }
    set_var("ALAN_OUTPUT_LANG", "js");
    // Generate the rust code to compile
    let (js_str, deps) = lntojs(source_file.clone())?;
    // Shove it into a temp file for rustc
    let out_file = match PathBuf::from(source_file.clone()).file_stem() {
        Some(pb) => format!("{}.js", pb.to_string_lossy()),
        None => {
            return Err("Invalid path".into());
        }
    };
    write(out_file, js_str)?;
    if !deps.is_empty() {
        let pkg_str = format!(
            "{{\n  \"name\": \"{}\",\n  \"main\": \"{}.js\",\n  \"dependencies\": {{\n    {}\n  }}\n}}",
            PathBuf::from(source_file.clone())
                .file_stem()
                .unwrap()
                .to_string_lossy(),
            PathBuf::from(source_file)
                .file_stem()
                .unwrap()
                .to_string_lossy(),
            deps.iter()
                .map(|(k, v)| {
                    format!("    \"{}\": \"{}\"", k, v)
                })
                .collect::<Vec<String>>()
                .join(",\n")
        );
        write("package.json", pkg_str)?;
    }
    Ok(())
}