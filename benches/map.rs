use std::fs::{remove_file, write};
use std::process::{Command, Output};

use alan::compile::compile;

macro_rules! build {
    ( $name:ident => $code:expr ) => {
        let filename = format!("{}.ln", stringify!($name));
        write(&filename, $code)?;
        compile(filename.to_string())?;
    }
}

macro_rules! run {
    ( $name:ident ) => {
        #[divan::bench(max_time = 60)]
        fn $name() -> Result<Output, std::io::Error> {
            Command::new(format!("./{}", stringify!($name))).output()
        }
    }
}

macro_rules! clean {
    ( $name:ident ) => {
        let sourcefile = format!("{}.ln", stringify!($name));
        let executable = format!("{}", stringify!($name));
        remove_file(&sourcefile)?;
        remove_file(&executable)?;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build!(map_1 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 1).map(double); }
    "#);
    build!(map_10 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 10).map(double); }
    "#);
    build!(map_100 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 100).map(double); }
    "#);
    build!(map_1000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 1000).map(double); }
    "#);
    build!(map_10000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 10000).map(double); }
    "#);
    build!(map_100000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 100000).map(double); }
    "#);
    build!(map_1000000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 1000000).map(double); }
    "#);
    build!(map_10000000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 10000000).map(double); }
    "#);
    build!(map_100000000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 100000000).map(double); }
    "#);
    build!(parmap_1 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 1).parmap(double); }
    "#);
    build!(parmap_10 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 10).parmap(double); }
    "#);
    build!(parmap_100 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 100).parmap(double); }
    "#);
    build!(parmap_1000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 1000).parmap(double); }
    "#);
    build!(parmap_10000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 10000).parmap(double); }
    "#);
    build!(parmap_100000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 100000).parmap(double); }
    "#);
    build!(parmap_1000000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 1000000).parmap(double); }
    "#);
    build!(parmap_10000000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 10000000).parmap(double); }
    "#);
    build!(parmap_100000000 => r#"
        fn double(x: i64): Result<i64> = x * 2;
        export fn main { filled(2, 100000000).parmap(double); }
    "#);
    divan::main();
    clean!(map_1);
    clean!(map_10);
    clean!(map_100);
    clean!(map_1000);
    clean!(map_10000);
    clean!(map_100000);
    clean!(map_1000000);
    clean!(map_10000000);
    clean!(map_100000000);
    clean!(parmap_1);
    clean!(parmap_10);
    clean!(parmap_100);
    clean!(parmap_1000);
    clean!(parmap_10000);
    clean!(parmap_100000);
    clean!(parmap_1000000);
    clean!(parmap_10000000);
    clean!(parmap_100000000);
    Ok(())
}

run!(map_1);
run!(map_10);
run!(map_100);
run!(map_1000);
run!(map_10000);
run!(map_100000);
run!(map_1000000);
run!(map_10000000);
run!(map_100000000);
run!(parmap_1);
run!(parmap_10);
run!(parmap_100);
run!(parmap_1000);
run!(parmap_10000);
run!(parmap_100000);
run!(parmap_1000000);
run!(parmap_10000000);
run!(parmap_100000000);
