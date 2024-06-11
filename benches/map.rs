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
        let executable = format!("{}", stringify!($name));
        remove_file(&sourcefile)?;
        remove_file(&executable)?;
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build!(map_1 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 1).map(double); }
    "#);
    build!(map_10 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 10).map(double); }
    "#);
    build!(map_100 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 100).map(double); }
    "#);
    build!(map_1_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 1_000).map(double); }
    "#);
    build!(map_10_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 10_000).map(double); }
    "#);
    build!(map_100_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 100_000).map(double); }
    "#);
    build!(map_1_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 1_000_000).map(double); }
    "#);
    build!(map_10_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 10_000_000).map(double); }
    "#);
    build!(map_100_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 100_000_000).map(double); }
    "#);
    build!(parmap_1 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 1).parmap(double); }
    "#);
    build!(parmap_10 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 10).parmap(double); }
    "#);
    build!(parmap_100 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 100).parmap(double); }
    "#);
    build!(parmap_1_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 1_000).parmap(double); }
    "#);
    build!(parmap_10_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 10_000).parmap(double); }
    "#);
    build!(parmap_100_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 100_000).parmap(double); }
    "#);
    build!(parmap_1_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 1_000_000).parmap(double); }
    "#);
    build!(parmap_10_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 10_000_000).parmap(double); }
    "#);
    build!(parmap_100_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { filled(2, 100_000_000).parmap(double); }
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
    build!(gpgpu_1_000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 1_000));
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
    build!(gpgpu_10_000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 10_000));
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
    build!(gpgpu_100_000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 100_000));
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
    build!(gpgpu_1_000_000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 1_000_000));
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
    build!(gpgpu_10_000_000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 10_000_000));
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
    build!(gpgpu_100_000_000 => r#"
        export fn main {
          let g = GPU();
          let b = g.createBuffer(storageBuffer(), filled(2.i32(), 100_000_000));
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
    clean!(map_1_000);
    clean!(map_10_000);
    clean!(map_100_000);
    clean!(map_1_000_000);
    clean!(map_10_000_000);
    clean!(map_100_000_000);
    clean!(parmap_1);
    clean!(parmap_10);
    clean!(parmap_100);
    clean!(parmap_1_000);
    clean!(parmap_10_000);
    clean!(parmap_100_000);
    clean!(parmap_1_000_000);
    clean!(parmap_10_000_000);
    clean!(parmap_100_000_000);
    clean!(gpgpu_1);
    clean!(gpgpu_10);
    clean!(gpgpu_100);
    clean!(gpgpu_1_000);
    clean!(gpgpu_10_000);
    clean!(gpgpu_100_000);
    clean!(gpgpu_1_000_000);
    clean!(gpgpu_10_000_000);
    clean!(gpgpu_100_000_000);
    Ok(())
}

run!(map_1);
run!(map_10);
run!(map_100);
run!(map_1_000);
run!(map_10_000);
run!(map_100_000);
run!(map_1_000_000);
run!(map_10_000_000);
run!(map_100_000_000);
run!(parmap_1);
run!(parmap_10);
run!(parmap_100);
run!(parmap_1_000);
run!(parmap_10_000);
run!(parmap_100_000);
run!(parmap_1_000_000);
run!(parmap_10_000_000);
run!(parmap_100_000_000);
run!(gpgpu_1);
run!(gpgpu_10);
run!(gpgpu_100);
run!(gpgpu_1_000);
run!(gpgpu_10_000);
run!(gpgpu_100_000);
run!(gpgpu_1_000_000);
run!(gpgpu_10_000_000);
run!(gpgpu_100_000_000);
