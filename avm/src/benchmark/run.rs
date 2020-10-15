use std::collections::HashMap;
use std::time::{Duration, Instant};

use rand::Rng;
use rand::thread_rng;
use rayon::prelude::*; // Ugh, the Rayon documentation makes it difficult to not do this...

use crate::vm::memory::HandlerMemory;
use crate::vm::program::{PROGRAM, Program};

/// This benchmark is meant to determine which form of array parallelization should be used in the
/// project. Two approaches exist but there are unknown restrictions within the Rust runtime that
/// will make one of the two better scaling than the other. We could dig deep through the libraries
/// and Rust source code to determine this, but it will be much faster to determine this
/// empirically.
///
/// The benchmark will evaluate parallelization on 3 axes: the parallelization strategy, the size
/// of the array of data to compute in parallel, and the amount of shared memory needed in the
/// compute loop.
///
/// To make it more realistic, we're using the HandlerMemory data structure that the runtime uses
/// for representing the data and storing any intermediate results, and are simulating the opcodes
/// with functions that store all intermediate state into that data structure.
///
/// Of the three axes, the parallelization strategy is discrete and wraps around the other two
/// axes, by being the implementation of how the work is distributed and marshalled. One strategy
/// is the per-thread-copy strategy, where the array is divided equally among `n` threads (where
/// `n` is the number of CPU cores minus one) and the shared memory is copied to each thread along
/// with the relevant fragment of memory, then the main thread marshalls the array data back and
/// inserts it into the HandlerMemory structure as expected at the end. Another strategy is the
/// true-shared-memory strategy, where the shared memory is referenced by all parallel executions
/// and the Rayon library's par_iter is used to perform the parallel execution. The final strategy
/// is a simple "null" strategy where no parallelization is applied at all.
///
/// The array size axis can be easily scaled to as many sample sizes as desired, anywhere from 1 to
/// (almost) infinity. For the purposes of the benchmark, though, we'll sample at 100 elements,
/// 10,000 elements, and 1,000,000 elements. If we can't see any differences we can increase the
/// scale to 100,000,000 elements and 10,000,000,000 elements or higher, but sticking to the 100x
/// factor will make it easier to understand the data without having to actually generate a plot.
///
/// The amount of shared memory needed can only be manipulated by changing the underlying
/// computation to perform, so this will also necessarily be discrete, with no-shared-data,
/// little-shared-data, and lots-of-shared-data implementations. The no-shared-data algorithm is
/// simply a `square` function that multiplies the input by itself. The little-shared-data is a `y
/// = mx + b` calculator with shared `m` and `b` to mutate the input `x`. The lots-of-shared-data
/// implementation is a 1D E-Field calculator that takes a fixed array of "charges" and computes
/// the apparent voltage at that point in the array and stores it, so lots of reads will be
/// necessary for it.
///
/// The benchmark program will run all combinations of these axes (24 cases by default) 30 times
/// each and print stats on the mean and stddev of the running time. The loading of the initial
/// data into the HandlerMemory will not count in the runtime, but the final saving of the
/// resulting data *will*. This is to simulate the total wall clock time of an actual `map` opcode
/// runtime.

/// These functions are not quite like opcodes, there should be a sequence of them with a
/// constraint of two input arguments per opcode, but for the benchmark, we'll have a truly
/// arbitrary number of arguments to make the benchmark code simpler.
fn square(args: &Vec<i64>, hand_mem: &mut HandlerMemory) {
  let a = hand_mem.read_fixed(args[0]);
  let out = a * a;
  hand_mem.write_fixed(args[1], out);
}

fn mx_plus_b(args: &Vec<i64>, hand_mem: &mut HandlerMemory) {
  let x = hand_mem.read_fixed(args[0]);
  let m = hand_mem.read_fixed(args[1]);
  let b = hand_mem.read_fixed(args[2]);
  let intermediate = m * x;
  hand_mem.write_fixed(args[3], intermediate);
  let out = intermediate + b;
  hand_mem.write_fixed(args[4], out);
}

fn e_field(args: &Vec<i64>, hand_mem: &mut HandlerMemory) {
  let arr = hand_mem.read_fractal(args[1]);
  let len = arr.len();
  let mut out = 0.0f64;
  // This vector is to avoid issues with borrowing `hand_mem` twice at the same time even though it
  // would be safe
  let i = hand_mem.read_fixed(args[0]);
  let d = args[2];
  let s = args[3];
  let v = args[4];
  let c = args[5];
  for n in 0..len {
    // All of the intermediate values are stored into the HandlerMemory to mimic how this function
    // would be translated from alan to the runtime execution
    //hand_mem.copy_from(args[1], args[6], n);
    //let scalar = hand_mem.read_fixed(args[6]) as f64;
    let scalar = hand_mem.read_fractal(args[1])[n].1 as f64;
    //let scalar = arr.read_fixed(n) as f64;
    let distance = (i - n as i64) as f64;
    if distance != 0.0 {
      hand_mem.write_fixed(d, i64::from_ne_bytes(distance.to_ne_bytes()));
      let sqdistance = distance * distance;
      hand_mem.write_fixed(s, i64::from_ne_bytes(sqdistance.to_ne_bytes()));
      let invsqdistance = 1f64 / sqdistance;
      hand_mem.write_fixed(v, i64::from_ne_bytes(invsqdistance.to_ne_bytes()));
      let scaled = invsqdistance * scalar;
      hand_mem.write_fixed(c, i64::from_ne_bytes(scaled.to_ne_bytes()));
      out = out + scaled;
    }
  }
  hand_mem.write_fixed(args[6], i64::from_ne_bytes(out.to_ne_bytes()));
}

fn gen_rand_array(size: i64) -> Vec<i64> {
  let mut rng = thread_rng();
  let mut out: Vec<i64> = Vec::new();
  for _ in 0..size {
    out.push((100000.0f64 * rng.gen::<f64>()) as i64);
  }
  return out;
}

/// The three sequential tests, with the array sizes passed in and the return is the time in ns it
/// took to run
fn lin_square(size: i64) -> Duration {
  let mut mem = HandlerMemory::new(None, 2);
  let data = gen_rand_array(size);
  let mut output: Vec<i64> = Vec::with_capacity(size as usize);
  let start = Instant::now();
  let args = vec![0, 1];
  for input in data {
    mem.write_fixed(0, input);
    square(&args, &mut mem);
    output.push(mem.read_fixed(1));
  }
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

fn lin_mx_plus_b(size: i64) -> Duration {
  let mut mem = HandlerMemory::new(None, 5);
  mem.write_fixed(1, 2);
  mem.write_fixed(2, 3);
  let data = gen_rand_array(size);
  let mut output: Vec<i64> = Vec::with_capacity(size as usize);
  let start = Instant::now();
  let args = vec![0, 1, 2, 3, 4];
  for input in data {
    mem.write_fixed(0, input);
    mx_plus_b(&args, &mut mem);
    output.push(mem.read_fixed(4));
  }
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

fn lin_e_field(size: i64) -> Duration {
  let mut mem = HandlerMemory::new(None, 7);
  let data = gen_rand_array(size);
  mem.write_fractal(1, &Vec::new());
  for input in data {
    mem.push_fixed(1, input);
  }
  let mut output: Vec<i64> = Vec::with_capacity(size as usize);
  let start = Instant::now();
  let args = vec![0, 1, 2, 3, 4, 5, 6];
  for i in 0..size {
    mem.write_fixed(0, i);
    e_field(&args, &mut mem);
    output.push(mem.read_fixed(6));
  }
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

fn coarse_square(size: i64) -> Duration {
  let mut full_output: Vec<i64> = Vec::new();
  let cpu_threads = (num_cpus::get() - 1) as i64;
  let mut data_chunks: Vec<Vec<i64>> = Vec::new();
  for i in 0..cpu_threads {
    let mut inner_size = size / cpu_threads;
    if size % cpu_threads > 0 && i == cpu_threads - 1 {
      inner_size = inner_size + (size % cpu_threads);
    }
    data_chunks.push(gen_rand_array(inner_size));
  }
  let start = Instant::now();
  let result_chunks: Vec<Vec<i64>> = data_chunks.par_iter().map(|chunk| {
    let mut output: Vec<i64> = Vec::new();
    let mut mem = HandlerMemory::new(None, 2);
    let args = vec![0, 1];
    for input in chunk {
      mem.write_fixed(0, *input);
      square(&args, &mut mem);
      output.push(mem.read_fixed(1));
    }
    return output;
  }).collect();
  for chunk in result_chunks {
    for i in 0..chunk.len() {
      full_output.push(chunk[i]);
    }
  }
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

fn coarse_mx_plus_b(size: i64) -> Duration {
  let mut full_output: Vec<i64> = Vec::new();
  let cpu_threads = (num_cpus::get() - 1) as i64;
  let mut data_chunks: Vec<Vec<i64>> = Vec::new();
  for i in 0..cpu_threads {
    let mut inner_size = size / cpu_threads;
    if size % cpu_threads > 0 && i == cpu_threads - 1 {
      inner_size = inner_size + (size % cpu_threads);
    }
    data_chunks.push(gen_rand_array(inner_size));
  }
  let mut full_mem = HandlerMemory::new(None, 5);
  full_mem.write_fixed(1, 2);
  full_mem.write_fixed(2, 3);
  let start = Instant::now();
  let result_chunks: Vec<Vec<i64>> = data_chunks.par_iter().map(|chunk| {
    let mut mem = full_mem.clone();
    let mut output: Vec<i64> = Vec::new();
    let args = vec![0, 1, 2, 3, 4];
    for input in chunk {
      mem.write_fixed(0, *input);
      mx_plus_b(&args, &mut mem);
      output.push(mem.read_fixed(4));
    }
    return output;
  }).collect();
  for chunk in result_chunks {
    for i in 0..chunk.len() {
      full_output.push(chunk[i]);
    }
  }
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

fn coarse_e_field(size: i64) -> Duration {
  let mut full_output: Vec<i64> = Vec::new();
  let cpu_threads = (num_cpus::get() - 1) as i64;
  let data = gen_rand_array(size);
  let mut real_mem = HandlerMemory::new(None, 7);
  real_mem.write_fractal(1, &Vec::new());
  for input in &data {
    real_mem.push_fixed(1, *input);
  }
  let start = Instant::now();
  let mut mems: Vec<HandlerMemory> = Vec::new();
  let mut inner_sizes: Vec<usize> = Vec::new();
  for i in 0..cpu_threads {
    let mem = real_mem.clone();
    mems.push(mem);
    let mut inner_size = size / cpu_threads;
    if size % cpu_threads > 0 && i == cpu_threads - 1 {
      inner_size = inner_size + (size % cpu_threads);
    }
    inner_sizes.push(inner_size as usize);
  }
  let result_chunks: Vec<Vec<i64>> = mems.par_iter_mut().enumerate().map(|(i, mut mem)| {
    let inner_size = inner_sizes[i];
    let mut output = Vec::with_capacity(inner_size);
    let offset = i * (size / cpu_threads) as usize;
    let args = vec![0, 1, 2, 3, 4, 5, 6];
    for j in offset..(offset+inner_size) {
      let addr = j;
      mem.write_fixed(0, addr as i64);
      e_field(&args, &mut mem);
      output.push(mem.read_fixed(6));
    }
    return output.to_vec();
  }).collect();
  for mut chunk in result_chunks {
    full_output.append(&mut chunk);
  }
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

fn fine_square(size: i64) -> Duration {
  let data = gen_rand_array(size);
  let start = Instant::now();
  let _output: Vec<i64> = data.par_iter().map(|value| {
    let mut mem = HandlerMemory::new(None, 2);
    let args = vec![0, 1];
    mem.write_fixed(0, *value);
    square(&args, &mut mem);
    return mem.read_fixed(1);
  }).collect();
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

fn fine_mx_plus_b(size: i64) -> Duration {
  let mut real_mem = HandlerMemory::new(None, 5);
  real_mem.write_fixed(1, 2);
  real_mem.write_fixed(2, 3);
  let data = gen_rand_array(size);
  let start = Instant::now();
  let _output: Vec<i64> = data.par_iter().map(|value| {
    let mut mem = real_mem.clone();
    let args = vec![0, 1, 2, 3, 4];
    mem.write_fixed(0, *value);
    mx_plus_b(&args, &mut mem);
    return mem.read_fixed(4);
  }).collect();
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

fn fine_e_field(size: i64) -> Duration {
  let mut real_mem = HandlerMemory::new(None, 7);
  let data = gen_rand_array(size);
  real_mem.write_fractal(1, &Vec::new());
  for input in data {
    real_mem.push_fixed(1, input);
  }
  let start = Instant::now();
  let _output: Vec<i64> = (0..size).into_par_iter().map(|i| {
    let mut mem = real_mem.clone();
    let args = vec![0, 1, 2, 3, 4, 5, 6];
    let addr = i;
    mem.write_fixed(0, addr);
    e_field(&args, &mut mem);
    return mem.read_fixed(6);
  }).collect();
  let end = Instant::now();
  return end.saturating_duration_since(start);
}

pub fn benchmark() {
  // Initialize the global PROGRAM value
  let init = PROGRAM.set(Program {
    event_handlers: HashMap::new(),
    event_pls: HashMap::new(),
    gmem: Vec::new(),
  });
  if init.is_err() {
    eprintln!("Failed to load bytecode");
    std::process::exit(1);
  }
  // Initialize the global Rayon threadpool
  let cpu_threads = num_cpus::get() - 1;
  rayon::ThreadPoolBuilder::new().num_threads(cpu_threads).build_global().unwrap();
  // Start benchmarking!
  println!("Benchmark! Parallel tests using {} threads", cpu_threads);
  println!("Quick test sequential squares 100-element array: {:?}", lin_square(100));
  println!("Quick test sequential squares 10,000-element array: {:?}", lin_square(10000));
  println!("Quick test sequential squares 1,000,000-element array: {:?}", lin_square(1000000));
  println!("Quick test sequential mx+b 100-element array: {:?}", lin_mx_plus_b(100));
  println!("Quick test sequential mx+b 10,000-element array: {:?}", lin_mx_plus_b(10000));
  println!("Quick test sequential mx+b 1,000,000-element array: {:?}", lin_mx_plus_b(1000000));
  println!("Quick test sequential e-field 100-element array: {:?}", lin_e_field(100));
  println!("Quick test sequential e-field 10,000-element array: {:?}", lin_e_field(10000));
  // println!("Quick test sequential e-field 1,000,000-element array: {:?}", lin_e_field(1000000));
  println!("Quick test coarse parallel squares 100-element array: {:?}", coarse_square(100));
  println!("Quick test coarse parallel squares 10,000-element array: {:?}", coarse_square(10000));
  println!("Quick test coarse parallel squares 1,000,000-element array: {:?}", coarse_square(1000000));
  println!("Quick test coarse parallel mx+b 100-element array: {:?}", coarse_mx_plus_b(100));
  println!("Quick test coarse parallel mx+b 10,000-element array: {:?}", coarse_mx_plus_b(10000));
  println!("Quick test coarse parallel mx+b 1,000,000-element array: {:?}", coarse_mx_plus_b(1000000));
  println!("Quick test coarse parallel e-field 100-element array: {:?}", coarse_e_field(100));
  println!("Quick test coarse parallel e-field 10,000-element array: {:?}", coarse_e_field(10000));
  // println!("Quick test coarse parallel e-field 1,000,000-element array: {:?}", coarse_e_field(1000000));
  println!("Quick test fine parallel squares 100-element array: {:?}", fine_square(100));
  println!("Quick test fine parallel squares 10,000-element array: {:?}", fine_square(10000));
  println!("Quick test fine parallel squares 1,000,000-element array: {:?}", fine_square(1000000));
  println!("Quick test fine parallel mx+b 100-element array: {:?}", fine_mx_plus_b(100));
  println!("Quick test fine parallel mx+b 10,000-element array: {:?}", fine_mx_plus_b(10000));
  println!("Quick test fine parallel mx+b 1,000,000-element array: {:?}", fine_mx_plus_b(1000000));
  println!("Quick test fine parallel e-field 100-element array: {:?}", fine_e_field(100));
  println!("Quick test fine parallel e-field 10,000-element array: {:?}", fine_e_field(10000));
  // println!("Quick test fine parallel e-field 1,000,000-element array: {:?}", fine_e_field(1000000));
}