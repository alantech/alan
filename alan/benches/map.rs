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
        #[divan::bench(max_time = 10)]
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
    build!(t01_map_1 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 1).map(double); v[0].print; }
    "#);
    build!(t02_map_10 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 10).map(double); v[0].print; }
    "#);
    build!(t03_map_100 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 100).map(double); v[0].print; }
    "#);
    build!(t04_map_1_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 1_000).map(double); v[0].print; }
    "#);
    build!(t05_map_10_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 10_000).map(double); v[0].print; }
    "#);
    build!(t06_map_100_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 100_000).map(double); v[0].print; }
    "#);
    build!(t07_map_1_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 1_000_000).map(double); v[0].print; }
    "#);
    build!(t08_map_10_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 10_000_000).map(double); v[0].print; }
    "#);
    build!(t09_map_100_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 100_000_000).map(double); v[0].print; }
    "#);
    build!(t10_parmap_1 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 1).parmap(double); v[0].print; }
    "#);
    build!(t11_parmap_10 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 10).parmap(double); v[0].print; }
    "#);
    build!(t12_parmap_100 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 100).parmap(double); v[0].print; }
    "#);
    build!(t13_parmap_1_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 1_000).parmap(double); v[0].print; }
    "#);
    build!(t14_parmap_10_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 10_000).parmap(double); v[0].print; }
    "#);
    build!(t15_parmap_100_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 100_000).parmap(double); v[0].print; }
    "#);
    build!(t16_parmap_1_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 1_000_000).parmap(double); v[0].print; }
    "#);
    build!(t17_parmap_10_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 10_000_000).parmap(double); v[0].print; }
    "#);
    build!(t18_parmap_100_000_000 => r#"
        fn double(x: i64) -> i64 = x * 2;
        export fn main { let v = filled(2, 100_000_000).parmap(double); v[0].print; }
    "#);
    build!(t19_gpgpu_1 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 1))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    build!(t20_gpgpu_10 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 10))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    build!(t21_gpgpu_100 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 100))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    build!(t22_gpgpu_1_000 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 1_000))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    build!(t23_gpgpu_10_000 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 10_000))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    build!(t24_gpgpu_100_000 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 100_000))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    build!(t25_gpgpu_1_000_000 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 1_000_000))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    build!(t26_gpgpu_10_000_000 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 10_000_000))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    build!(t27_gpgpu_100_000_000 => r#"
        export fn main {
          let b = GBuffer(storageBuffer(), filled(2.i32(), 100_000_000))!!;
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
          plan.run;
          let v = b.read{i32};
          v[0].print;
        }
    "#);
    divan::main();
    clean!(t01_map_1);
    clean!(t02_map_10);
    clean!(t03_map_100);
    clean!(t04_map_1_000);
    clean!(t05_map_10_000);
    clean!(t06_map_100_000);
    clean!(t07_map_1_000_000);
    clean!(t08_map_10_000_000);
    clean!(t09_map_100_000_000);
    clean!(t10_parmap_1);
    clean!(t11_parmap_10);
    clean!(t12_parmap_100);
    clean!(t13_parmap_1_000);
    clean!(t14_parmap_10_000);
    clean!(t15_parmap_100_000);
    clean!(t16_parmap_1_000_000);
    clean!(t17_parmap_10_000_000);
    clean!(t18_parmap_100_000_000);
    clean!(t19_gpgpu_1);
    clean!(t20_gpgpu_10);
    clean!(t21_gpgpu_100);
    clean!(t22_gpgpu_1_000);
    clean!(t23_gpgpu_10_000);
    clean!(t24_gpgpu_100_000);
    clean!(t25_gpgpu_1_000_000);
    clean!(t26_gpgpu_10_000_000);
    clean!(t27_gpgpu_100_000_000);
    Ok(())
}

run!(t01_map_1);
run!(t02_map_10);
run!(t03_map_100);
run!(t04_map_1_000);
run!(t05_map_10_000);
run!(t06_map_100_000);
run!(t07_map_1_000_000);
run!(t08_map_10_000_000);
run!(t09_map_100_000_000);
run!(t10_parmap_1);
run!(t11_parmap_10);
run!(t12_parmap_100);
run!(t13_parmap_1_000);
run!(t14_parmap_10_000);
run!(t15_parmap_100_000);
run!(t16_parmap_1_000_000);
run!(t17_parmap_10_000_000);
run!(t18_parmap_100_000_000);
run!(t19_gpgpu_1);
run!(t20_gpgpu_10);
run!(t21_gpgpu_100);
run!(t22_gpgpu_1_000);
run!(t23_gpgpu_10_000);
run!(t24_gpgpu_100_000);
run!(t25_gpgpu_1_000_000);
run!(t26_gpgpu_10_000_000);
run!(t27_gpgpu_100_000_000);
