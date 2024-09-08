use std::fs::{remove_file, write};
use std::process::{Command, Output};

use alan::compile::compile;

macro_rules! build {
    ( $name:ident => $code:expr ) => {
        let filename = format!("{}.ln", stringify!($name));
        write(&filename, $code)?;
        compile(filename.to_string())?;
    };
}

macro_rules! run {
    ( $name:ident ) => {
        #[divan::bench(max_time = 60)]
        fn $name() -> Result<Output, std::io::Error> {
            Command::new(format!("./{}", stringify!($name))).output()
        }
    };
}

macro_rules! clean {
    ( $name:ident ) => {
        let sourcefile = format!("{}.ln", stringify!($name));
        let executable = if cfg!(windows) {
            format!("{}.exe", stringify!($name))
        } else {
            format!("{}", stringify!($name))
        };
        remove_file(&sourcefile)?;
        remove_file(&executable)?;
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build!(t01_fill_100 => r#"
        export fn main { let v = filled(5, 100); v[0].print; }
    "#);
    build!(t02_fill_100_000 => r#"
        export fn main { let v = filled(5, 100_000); v[0].print; }
    "#);
    build!(t03_fill_100_000_000 => r#"
        export fn main { let v = filled(5, 100_000_000); v[0].print; }
    "#);
    divan::main();
    clean!(t01_fill_100);
    clean!(t02_fill_100_000);
    clean!(t03_fill_100_000_000);
    Ok(())
}

run!(t01_fill_100);
run!(t02_fill_100_000);
run!(t03_fill_100_000_000);

#[divan::bench(max_time = 60)]
fn t04_vec_100() -> Result<(), std::io::Error> {
    let v = vec![5; 100];
    write("/dev/null", format!("{}", v[0]))
}

#[divan::bench(max_time = 60)]
fn t05_vec_100_000() -> Result<(), std::io::Error> {
    let v = vec![5; 100_000];
    write("/dev/null", format!("{}", v[0]))
}

#[divan::bench(max_time = 60)]
fn t06_vec_100_000_000() -> Result<(), std::io::Error> {
    let v = vec![5; 100_000_000];
    write("/dev/null", format!("{}", v[0]))
}
