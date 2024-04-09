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
    build!(gpgpu_1 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 1));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
    "#);
    build!(gpgpu_10 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 10));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
    "#);
    build!(gpgpu_100 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 100));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
    "#);
    build!(gpgpu_1000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 1000));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
    "#);
    build!(gpgpu_10000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 10000));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
    "#);
    build!(gpgpu_100000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 100000));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
    "#);
    build!(gpgpu_1000000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 1000000));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
    "#);
    build!(gpgpu_10000000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 10000000));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
    "#);
    build!(gpgpu_100000000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 100000000));
          let plan = GPGPU("
            @group(0)
            @binding(0)
            var<storage, read_write> vals: array<i32>;

            @compute
            @workgroup_size(1)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
              let i = id.x + 65535 * id.y + 4294836225 * id.z;
              let l = arrayLength(&vals);
              if i > l { return; }
              vals[id.x + 65535 * id.y + 4294836225 * id.z] = vals[id.x + 65535 * id.y + 4294836225 * id.z] * 2;
            }
          ", b);
          g.run(plan);
          g.read(b);
        }
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
    clean!(gpgpu_1);
    clean!(gpgpu_10);
    clean!(gpgpu_100);
    clean!(gpgpu_1000);
    clean!(gpgpu_10000);
    clean!(gpgpu_100000);
    clean!(gpgpu_1000000);
    clean!(gpgpu_10000000);
    clean!(gpgpu_100000000);
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
run!(gpgpu_1);
run!(gpgpu_10);
run!(gpgpu_100);
run!(gpgpu_1000);
run!(gpgpu_10000);
run!(gpgpu_100000);
run!(gpgpu_1000000);
run!(gpgpu_10000000);
run!(gpgpu_100000000);
