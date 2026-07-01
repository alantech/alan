use std::env::current_dir;
use std::fs::{create_dir_all, remove_file, write, File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use dirs::config_dir;
use fs2::FileExt;

use alan_compiler::lntojs::lntojs;
use alan_compiler::lntors::lntors;
use alan_compiler::program::Program;

mod integration_tests;

/// Defense-in-depth backstop for unexpected panics during compilation (e.g. path parsing bugs).
/// This does *not* recover from stack overflow in the parser; that is handled by the parse-depth
/// cap in `alan_compiler::parse`.
fn catch_compile_panics<F, T>(f: F) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce() -> Result<T, Box<dyn std::error::Error>> + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => result,
        Err(_) => Err("Internal compiler error: unexpected panic during compilation".into()),
    }
}

/// Acquire an exclusive lock on a file with timeout and retry logic
fn acquire_file_lock(
    lockfile_path: &PathBuf,
    timeout_secs: u64,
) -> Result<File, Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);
    let sleep_time = std::time::Duration::from_millis(50);

    loop {
        // Try to open the file for exclusive access. NOTE: we must NOT truncate here -- this
        // lockfile doubles as the store for the "last dependency update" timestamp, and truncating
        // on every open would wipe that timestamp, making `should_rebuild_deps` always true and
        // forcing a (often pointless) `cargo update` on every single invocation.
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lockfile_path)
        {
            Ok(file) => {
                // Try to acquire an exclusive lock
                match file.lock_exclusive() {
                    Ok(_) => {
                        // Successfully acquired the lock
                        return Ok(file);
                    }
                    Err(_) => {
                        // Lock failed, check if we've exceeded timeout
                        if start_time.elapsed() > timeout {
                            return Err("Could not acquire lock within timeout period".into());
                        }
                        // Sleep briefly before retrying
                        std::thread::sleep(sleep_time);
                    }
                }
            }
            Err(_) => {
                // File open failed, check if we've exceeded timeout
                if start_time.elapsed() > timeout {
                    return Err("Could not open lockfile within timeout period".into());
                }
                // Sleep briefly before retrying
                std::thread::sleep(sleep_time);
            }
        }
    }
}

fn write_fast_linker_config(project_dir: &Path, find_cmd: &str, bindir: &Option<PathBuf>) {
    let dot_cargo = project_dir.join(".cargo");
    let config_path = dot_cargo.join("config.toml");
    // Only treat a linker as available if `which`/`where` actually *found* it. Note that
    // `output()` returns `Ok` as long as the lookup command itself ran -- even when it reports
    // "not found" via a non-zero exit code -- so we must inspect the exit status, not just `is_ok`.
    // Otherwise we'd force `-fuse-ld=mold` on machines without mold and break linking entirely.
    let is_available = |name: &str| {
        Command::new(find_cmd)
            .arg(name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    };
    let linker = if is_available("mold") {
        Some("mold")
    } else if is_available("lld") {
        Some("lld")
    } else {
        None
    };
    let linker = match linker {
        Some(l) => l,
        None => {
            // No faster linker present: fall back to the default linker. Crucially, remove any
            // config we (or an older, buggy version) may have left behind that forces a linker
            // which isn't installed here -- otherwise every build would fail to link.
            let _ = std::fs::remove_file(&config_path);
            return;
        }
    };
    // Fast path: if the existing config already forces the linker we found, it's correct, so we're
    // done. This matters because it lets us skip the (relatively expensive) `rustc -vV` host-triple
    // lookup on every build -- we only fall through to regenerating the config when it's missing or
    // now points at a different linker (which is rare). The cheap `which` checks above are all we
    // pay in the steady state.
    let desired_flag = format!("-Clink-arg=-fuse-ld={linker}");
    if let Ok(existing) = std::fs::read_to_string(&config_path) {
        if existing.contains(&desired_flag) {
            return;
        }
    }
    // The config is missing or stale: (re)generate it. Resolve the host triple for the
    // `[target.<triple>]` table, using the toolchain's `rustc` directly (via `bindir`) when we have
    // it so we skip the rustup shim's overhead.
    let rustc_path = bindir
        .as_ref()
        .map(|d| d.join("rustc"))
        .unwrap_or_else(|| PathBuf::from("rustc"));
    let host = match (|| -> Option<String> {
        let output = Command::new(&rustc_path).arg("-vV").output().ok()?;
        let stdout = String::from_utf8(output.stdout).ok()?;
        stdout
            .lines()
            .find(|l| l.starts_with("host:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .map(|s| s.to_string())
    })() {
        Some(h) => h,
        None => return,
    };
    if std::fs::create_dir_all(&dot_cargo).is_err() {
        return;
    }
    let _ = std::fs::write(
        &config_path,
        format!("[target.{host}]\nrustflags = [\"{desired_flag}\"]\n"),
    );
}

/// Resolve the directory containing the *real* toolchain `cargo`/`rustc` binaries, bypassing the
/// `rustup` proxy shims. When Rust is installed via rustup (the recommended way), the `cargo` and
/// `rustc` found on `PATH` are thin shim binaries that, on *every* invocation, parse rustup's TOML
/// config to select a toolchain and then re-exec the real binary. For a fast `alan` run that just
/// builds a tiny program, this shim overhead (a TOML parse plus an extra process exec) is a
/// meaningful slice of wall-clock time, and it's paid once for `cargo` and again for every `rustc`
/// invocation. Resolving the real toolchain `bin` directory once lets us invoke those binaries
/// directly and skip the shims entirely.
///
/// Returns `None` when rustup is not in use (e.g. a distro-packaged toolchain), in which case the
/// plain `cargo`/`rustc` on `PATH` are already the real binaries and need no special handling.
///
/// The resolution itself costs ~40ms (it's a full `rustup which cargo` invocation), so we cache
/// the result in `cache_dir` keyed on the things that determine which toolchain rustup would pick:
/// the mtime of rustup's `settings.toml` (which rustup rewrites on `rustup default`/install/
/// uninstall) and the `RUSTUP_TOOLCHAIN` override env var. A cache hit is two cheap `stat`s instead
/// of spawning rustup, and a toolchain swap (`rustup default ...`) invalidates the cache
/// automatically because it bumps `settings.toml`'s mtime.
fn rustup_bindir(project_dir: &Path, cache_dir: &Path) -> Option<PathBuf> {
    // Escape hatch: allow opting out of the bypass in case it misbehaves in an exotic toolchain
    // setup.
    if std::env::var_os("ALAN_DISABLE_RUSTUP_BYPASS").is_some() {
        return None;
    }
    // Build the cache key from rustup's toolchain-selection inputs.
    let settings_mtime = rustup_settings_mtime();
    let toolchain_env = std::env::var("RUSTUP_TOOLCHAIN").unwrap_or_default();
    let key = format!(
        "{}\n{}",
        settings_mtime
            .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        toolchain_env,
    );
    let cache_path = cache_dir.join(".toolchain_cache");
    // Only trust the cache when we have a stable key (i.e. we could read settings.toml's mtime).
    if settings_mtime.is_some() {
        if let Ok(contents) = std::fs::read_to_string(&cache_path) {
            if let Some((cached_key, cached_bindir)) = contents.split_once("\n::\n") {
                if cached_key == key {
                    let bindir = PathBuf::from(cached_bindir.trim());
                    // Guard against a cached toolchain that has since been uninstalled.
                    if bindir.join("cargo").exists() {
                        return Some(bindir);
                    }
                }
            }
        }
    }
    // Cache miss (or no settings.toml): resolve via rustup. We run it in `project_dir` so any
    // `rust-toolchain.toml` override resolution matches the directory where cargo will actually
    // run.
    let output = Command::new("rustup")
        .current_dir(project_dir)
        .arg("which")
        .arg("cargo")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let cargo_path = PathBuf::from(String::from_utf8(output.stdout).ok()?.trim());
    let bindir = cargo_path.parent()?.to_path_buf();
    // Best-effort cache write (skipped when we lack a stable key).
    if settings_mtime.is_some() {
        let _ = std::fs::write(&cache_path, format!("{}\n::\n{}", key, bindir.display()));
    }
    Some(bindir)
}

/// Returns the modification time of rustup's `settings.toml`, which rustup rewrites whenever the
/// active toolchain changes (`rustup default`, toolchain install/uninstall). Used as a cheap
/// cache-invalidation signal for `rustup_bindir`.
fn rustup_settings_mtime() -> Option<std::time::SystemTime> {
    let rustup_home = match std::env::var_os("RUSTUP_HOME") {
        Some(h) => PathBuf::from(h),
        None => dirs::home_dir()?.join(".rustup"),
    };
    std::fs::metadata(rustup_home.join("settings.toml"))
        .ok()?
        .modified()
        .ok()
}

/// Construct a `cargo` `Command` that bypasses the rustup shims when possible (see
/// `rustup_bindir`). `bindir` should be the cached result of a single `rustup_bindir()` call for
/// this build so the resolution cost is paid at most once.
fn cargo_command(bindir: &Option<PathBuf>, project_dir: &Path) -> Command {
    let mut cmd = match bindir {
        Some(dir) => {
            let mut cmd = Command::new(dir.join("cargo"));
            // Point `cargo` at the real `rustc` so it doesn't reach for the rustc shim (which it
            // would otherwise find via `PATH`), and prepend the toolchain `bin` dir to `PATH` so
            // any other toolchain tools resolve directly too.
            cmd.env("RUSTC", dir.join("rustc"));
            if let Ok(path) = std::env::var("PATH") {
                let mut paths = vec![dir.clone()];
                paths.extend(std::env::split_paths(&path));
                if let Ok(joined) = std::env::join_paths(paths) {
                    cmd.env("PATH", joined);
                }
            }
            cmd
        }
        None => Command::new("cargo"),
    };
    cmd.current_dir(project_dir);
    cmd
}

/// The `build` function creates a temporary directory that is a Cargo project primarily consisting
/// of a single source file, plus a Cargo.toml file including the 3rd party dependencies in the
/// standard library and user source code.
pub fn build(source_file: String, profile: &str) -> Result<String, Box<dyn std::error::Error>> {
    catch_compile_panics(|| build_inner(source_file, profile))
}

fn build_inner(source_file: String, profile: &str) -> Result<String, Box<dyn std::error::Error>> {
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
    let binary_path = {
        let mut r = project_dir.clone();
        r.push("target");
        r.push(profile);
        r
    };
    let cargo_str = r#"[package]
name = "alan_generated_bin"
edition = "2021"

# To continue supporting earlier versions of Rust
[patch.crates-io]
bytemuck_derive = { git = "https://github.com/Lokathor/bytemuck", tag = "bytemuck_derive-v1.8.1" }

[profile.interp]
inherits = "dev"
opt-level = 0
debug = false
codegen-units = 256

[dependencies]"#;
    let cargo_path = {
        let mut c = project_dir.clone();
        c.push("Cargo.toml");
        c
    };

    // Create the config directory if it doesn't exist
    if !alan_config.exists() {
        create_dir_all(alan_config.clone())?;
    }

    // Resolve the real toolchain binaries (cached in the config dir) so every `cargo`/`rustc`
    // invocation below can skip the rustup proxy shims.
    let bindir = rustup_bindir(&project_dir, &alan_config);

    // Acquire the lock with a 3-minute timeout
    let mut lockfile = acquire_file_lock(&lockfile_path, 180)?;

    // Check if this is the first time or if we need to rebuild dependencies
    let first_time = !project_dir.exists();
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
                fs2::FileExt::unlock(&lockfile)?;
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
                fs2::FileExt::unlock(&lockfile)?;
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
                fs2::FileExt::unlock(&lockfile)?;
                Err(e)
            }
        }?;
        let (rs_str, deps) = match lntors(hello_path.to_string_lossy().to_string()) {
            Ok(a) => Ok(a),
            Err(e) => {
                fs2::FileExt::unlock(&lockfile)?;
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
                            let parts = v.split("#").collect::<Vec<&str>>();
                            if parts.len() == 2 {
                                format!(
                                    "{} = {{ git = \"{}\", branch = \"{}\" }}",
                                    k, parts[0], parts[1]
                                )
                            } else {
                                // We'll assume there's only one part, since the alternative
                                // wouldn't parse properly. If it blows up, it's on them.
                                format!("{k} = {{ git = \"{v}\" }}")
                            }
                        } else {
                            format!("{k} = \"{v}\"")
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        ) {
            Ok(a) => Ok(a),
            Err(e) => {
                fs2::FileExt::unlock(&lockfile)?;
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
                fs2::FileExt::unlock(&lockfile)?;
                Err(e)
            }
        }?;
        match remove_file(hello_path) {
            Ok(a) => Ok(a),
            Err(e) => {
                fs2::FileExt::unlock(&lockfile)?;
                Err(e)
            }
        }?;
        let mut first_build = cargo_command(&bindir, &project_dir);
        first_build.arg("build");
        if profile == "release" {
            first_build.arg("--release");
        } else {
            first_build.arg("--profile").arg(profile);
        }
        match first_build
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(a) => Ok(a),
            Err(e) => {
                fs2::FileExt::unlock(&lockfile)?;
                Err(e)
            }
        }?;
        if profile != "interp" {
            let mut interp_warmup = cargo_command(&bindir, &project_dir);
            interp_warmup
                .arg("build")
                .arg("--profile")
                .arg("interp")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            match interp_warmup.output() {
                Ok(a) => Ok(a),
                Err(e) => {
                    fs2::FileExt::unlock(&lockfile)?;
                    Err(e)
                }
            }?;
        }
    }
    write_fast_linker_config(&project_dir, find_process, &bindir);
    // We need to remove the prior binary, if it exists, to prevent a prior successful compilation
    // from accidentally being treated as the output of an unsuccessful compilation.
    if binary_path.exists() {
        match Command::new("rm")
            .current_dir(binary_path.clone())
            .arg("-f")
            .arg("alan_generated_bin")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(a) => Ok(a),
            Err(e) => {
                fs2::FileExt::unlock(&lockfile)?;
                Err(e)
            }
        }?;
    }
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
            fs2::FileExt::unlock(&lockfile)?;
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
                    let parts = v.split("#").collect::<Vec<&str>>();
                    if parts.len() == 2 {
                        format!(
                            "{} = {{ git = \"{}\", branch = \"{}\" }}",
                            k, parts[0], parts[1]
                        )
                    } else {
                        // We'll assume there's only one part, since the alternative
                        // wouldn't parse properly. If it blows up, it's on them.
                        format!("{k} = {{ git = \"{v}\" }}")
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        ),
    ) {
        Ok(a) => Ok(a),
        Err(e) => {
            fs2::FileExt::unlock(&lockfile)?;
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
            fs2::FileExt::unlock(&lockfile)?;
            Err(e)
        }
    }?;
    // Update the cargo lockfile, if necessary
    if should_rebuild_deps {
        match cargo_command(&bindir, &project_dir)
            .arg("update")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(a) => Ok(a),
            Err(e) => {
                fs2::FileExt::unlock(&lockfile)?;
                Err(e)
            }
        }?;
        // Update the lockfile timestamp
        lockfile.set_len(0)?;
        lockfile.seek(std::io::SeekFrom::Start(0))?;
        write!(
            lockfile,
            "{}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        )?;
    }
    // Build the executable
    let mut build_cmd = cargo_command(&bindir, &project_dir);
    build_cmd.arg("build");
    if profile == "release" {
        build_cmd.arg("--release");
    } else {
        build_cmd.arg("--profile").arg(profile);
    }
    match build_cmd
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
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!("{e}"))
        }
    }?;
    // Copy the binary from the temp directory to the current directory
    let project_name_path = PathBuf::from(source_file);
    let project_name_str = match project_name_path.file_stem() {
        None => panic!("Somehow can't parse the source file name as a path?"),
        Some(n) => n.to_string_lossy().to_string(),
    };
    match Command::new("cp")
        .current_dir(binary_path)
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
            fs2::FileExt::unlock(&lockfile)?;
            Err(e)
        }
    }?;
    // Drop the lockfile
    fs2::FileExt::unlock(&lockfile)?;
    Ok(project_name_str)
}

/// The `compile` function is a thin wrapper on top of `build` that builds an executable in release
/// mode and exits, printing the time it took to run on success.
pub fn compile(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    catch_compile_panics(|| compile_inner(source_file))
}

fn compile_inner(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    Program::set_target_lang_rs();
    Program::set_compile_env("ALAN_TARGET", "release");
    build(source_file, "release")?;
    println!("Done! Took {:.2}sec", start_time.elapsed().as_secs_f32());
    Ok(())
}

/// The `interp` function builds the source file with minimal optimizations (like an interpreter),
/// runs it, and deletes the binary.
pub fn interp(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    catch_compile_panics(|| interp_inner(source_file))
}

fn interp_inner(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    Program::set_target_lang_rs();
    Program::set_compile_env("ALAN_TARGET", "interp");
    let binary = build(source_file, "interp")?;
    let mut run = Command::new(format!("./{binary}"))
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

/// The `test` function is a thin wrapper on top of `compile` that compiles the specified file in
/// test mode, then immediately invokes it, and deletes the binary when done.
pub fn test(source_file: String, js: bool) -> Result<(), Box<dyn std::error::Error>> {
    catch_compile_panics(|| test_inner(source_file, js))
}

fn test_inner(source_file: String, js: bool) -> Result<(), Box<dyn std::error::Error>> {
    if js {
        Program::set_target_lang_js();
    } else {
        Program::set_target_lang_rs();
    }
    Program::set_compile_env("ALAN_TARGET", "test");
    if js {
        let jsfile = web(source_file)?;
        let mut run = Command::new("node")
            .current_dir(current_dir()?)
            .arg(format!("{jsfile}.js"))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        let ecode = run.wait()?;
        Command::new("rm")
            .current_dir(current_dir()?)
            .arg(format!("{jsfile}.js"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        if !ecode.success() {
            std::process::exit(ecode.code().unwrap());
        }
    } else {
        let binary = build(source_file, "release")?;
        let mut run = Command::new(format!("./{binary}"))
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
    }
    Ok(())
}

/// The `web` function creates a temporary directory that is an NPM project, primarily consisting
/// of a single source file, plus a package.json file including third party dependencies in the
/// standard library and user source code.
pub fn web(source_file: String) -> Result<String, Box<dyn std::error::Error>> {
    catch_compile_panics(|| web_inner(source_file))
}

fn web_inner(source_file: String) -> Result<String, Box<dyn std::error::Error>> {
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
    let has_yarn =
        matches!(Command::new(find_process).arg("yarn").output(), Ok(a) if !a.stdout.is_empty());
    let has_pnpm =
        matches!(Command::new(find_process).arg("pnpm").output(), Ok(a) if !a.stdout.is_empty());
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
        l.push(".web_lockfile");
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
    // Create the config directory if it doesn't exist
    if !alan_config.exists() {
        create_dir_all(alan_config.clone())?;
    }

    // Acquire the lock with a 3-minute timeout
    let lockfile = acquire_file_lock(&lockfile_path, 180)?;

    // Check if this is the first time or if we need to rebuild dependencies
    let first_time = !project_dir.exists();
    if first_time {
        // First time initialization of the alan config directory
        match create_dir_all(project_dir.clone()) {
            Ok(a) => Ok(a),
            Err(e) => {
                fs2::FileExt::unlock(&lockfile)?;
                Err(format!("Could not create the project directory {e:?}"))
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
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!("Could not delete the prior bundle.js file {e:?}"))
        }
    }?;
    // Generate the js code to bundle
    let (js_str, deps) = match lntojs(source_file.clone()) {
        Ok(s) => Ok(s),
        Err(e) => {
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!("Could not generate the Javascript code {e:?}"))
        }
    }?;
    // Always write the `package.json` file, in case the cache is out-of-date from a prior version of
    // the Alan compiler is still present.
    match write(
        package_json_path.clone(),
        format!(
            "{{\n  \"name\": \"alan_generated_bundle\",\n  \"main\": \"index.js\",\n  \"dependencies\": {{\n    {}\n  }},\n  \"devDependencies\": {{\n    \"rollup\": \"4.x\",\n    \"@rollup/plugin-node-resolve\": \"15.x\",\n \"@rollup/plugin-terser\": \"^1.0.0\"\n  }}\n}}",
            deps.iter()
                .map(|(k, v)| {
                    format!("    \"{k}\": \"{v}\"")
                })
                .collect::<Vec<String>>()
                .join(",\n")
        ),
    ) {
        Ok(a) => Ok(a),
        Err(e) => {
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!("Could not create the package.json file {e:?}"))
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
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!(
                "Could not save the generated Javascript to disk {e:?}"
            ))
        }
    }?;
    // Clean stale lockfiles from other package managers; keep node_modules for caching
    match Command::new("rm")
        .current_dir(project_dir.clone())
        .arg("-f")
        .arg("package-lock.json")
        .arg("yarn.lock")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
    {
        Ok(a) => Ok(a),
        Err(e) => {
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!("Could not clear package-lock.json {e:?}"))
        }
    }?;
    match Command::new(if cfg!(windows) {
        "npm.cmd"
    } else if has_pnpm {
        "pnpm"
    } else if has_yarn {
        "yarn"
    } else {
        "npm"
    })
    .current_dir(project_dir.clone())
    .arg("install")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .output()
    {
        Ok(a) => Ok(a),
        Err(e) => {
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!("Could not run npm install {e:?}"))
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
            .arg("-p")
            .arg("@rollup/plugin-terser")
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
            .arg("-p")
            .arg("@rollup/plugin-terser")
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
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!("Could not generate the bundle.js file {e:?}"))
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
            fs2::FileExt::unlock(&lockfile)?;
            Err(format!(
                "Could not copy the bundled Javascript to the PWD {e:?}"
            ))
        }
    }?;
    // Drop the lockfile
    fs2::FileExt::unlock(&lockfile)?;
    Ok(project_name_str)
}

/// The `bundle` function is a thin wrapper on top of `web` that builds an executable in release
/// mode and exits, printing the time it took to run on success.
pub fn bundle(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    catch_compile_panics(|| bundle_inner(source_file))
}

fn bundle_inner(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    Program::set_target_lang_js();
    Program::set_compile_env("ALAN_TARGET", "release");
    web(source_file)?;
    println!("Done! Took {:.2}sec", start_time.elapsed().as_secs_f32());
    Ok(())
}

/// The `to_rs` function is an thin wrapper on top of `lntors` that shoves the output into a `.rs`
/// file.
pub fn to_rs(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    catch_compile_panics(|| to_rs_inner(source_file))
}

fn to_rs_inner(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    Program::set_target_lang_rs();
    Program::set_compile_env("ALAN_TARGET", "release");
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
                    let parts = v.split("#").collect::<Vec<&str>>();
                    if parts.len() == 2 {
                        format!(
                            "{} = {{ git = \"{}\", branch = \"{}\" }}",
                            k, parts[0], parts[1]
                        )
                    } else {
                        // We'll assume there's only one part, since the alternative
                        // wouldn't parse properly. If it blows up, it's on them.
                        format!("{k} = {{ git = \"{v}\" }}")
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
    catch_compile_panics(|| to_js_inner(source_file))
}

fn to_js_inner(source_file: String) -> Result<(), Box<dyn std::error::Error>> {
    Program::set_target_lang_js();
    Program::set_compile_env("ALAN_TARGET", "release");
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
                    format!("    \"{k}\": \"{v}\"")
                })
                .collect::<Vec<String>>()
                .join(",\n")
        );
        write("package.json", pkg_str)?;
    }
    Ok(())
}
