use std::collections::HashMap;
use std::time::Instant;

use byteorder::{ByteOrder, LittleEndian};
use rand::Rng;
use rand::thread_rng;

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
  let a = LittleEndian::read_i64(hand_mem.read(args[0], 8));
  let out = a * a;
  hand_mem.write(args[1], 8, &out.to_le_bytes());
}

fn mx_plus_b(args: &Vec<i64>, hand_mem: &mut HandlerMemory) {
  let x = LittleEndian::read_i64(hand_mem.read(args[0], 8));
  let m = LittleEndian::read_i64(hand_mem.read(args[1], 8));
  let b = LittleEndian::read_i64(hand_mem.read(args[2], 8));
  let intermediate = m * x;
  hand_mem.write(args[3], 8, &intermediate.to_le_bytes());
  let out = intermediate + b;
  hand_mem.write(args[4], 8, &out.to_le_bytes());
}

fn e_field(args: &Vec<i64>, hand_mem: &mut HandlerMemory) {
  let i = LittleEndian::read_i64(hand_mem.read(args[0], 8));
  let arr = hand_mem.get_fractal(args[1]);
  let len = arr.len_as_arr() as i64;
  let mut out = 0.0f64;
  // This vector is to avoid issues with borrowing `hand_mem` twice at the same time even though it
  // would be safe
  let mut charges: Vec<i64> = Vec::new();
  for n in 0..len {
    let charge = LittleEndian::read_i64(arr.read(n * 8, 8));
    charges.push(charge);
  }
  for n in 0..len {
    // All of the intermediate values are stored into the HandlerMemory to mimic how this function
    // would be translated from alan to the runtime execution
    let distance = i - n;
    hand_mem.write(args[2], 8, &distance.to_le_bytes());
    let sqdistance = distance * distance;
    hand_mem.write(args[3], 8, &sqdistance.to_le_bytes());
    let invsqdistance = 1f64 / (sqdistance as f64);
    hand_mem.write(args[4], 8, &invsqdistance.to_le_bytes());
    let scaled = invsqdistance * (charges[n as usize] as f64);
    hand_mem.write(args[5], 8, &scaled.to_le_bytes());
    out = out + scaled;
  }
  hand_mem.write(args[6], 8, &out.to_le_bytes());
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
fn lin_square(size: i64) -> u128 {
  let mut mem = HandlerMemory::new(None, 16);
  let data = gen_rand_array(size);
  let mut output: Vec<i64> = Vec::new();
  let start = Instant::now();
  let args = vec![0, 8];
  for input in data {
    mem.write(0, 8, &input.to_le_bytes());
    square(&args, &mut mem);
    output.push(LittleEndian::read_i64(mem.read(8, 8)));
  }
  let end = Instant::now();
  return end.saturating_duration_since(start).as_nanos();
}

fn lin_mx_plus_b(size: i64) -> u128 {
  let mut mem = HandlerMemory::new(None, 40);
  mem.write(8, 8, &2i64.to_le_bytes());
  mem.write(16, 8, &3i64.to_le_bytes());
  let data = gen_rand_array(size);
  let mut output: Vec<i64> = Vec::new();
  let start = Instant::now();
  let args = vec![0, 8, 16, 24, 32];
  for input in data {
    mem.write(0, 8, &input.to_le_bytes());
    mx_plus_b(&args, &mut mem);
    output.push(LittleEndian::read_i64(mem.read(32, 8)));
  }
  let end = Instant::now();
  return end.saturating_duration_since(start).as_nanos();
}

pub fn benchmark() {
  // Initialize the global PROGRAM value
  PROGRAM.set(Program {
    event_handlers: HashMap::new(),
    event_pls: HashMap::new(),
    gmem: Vec::new(),
  });
  println!("Benchmark!");
  println!("Quick test sequential squares 100-element array: {}ns", lin_square(100));
  println!("Quick test sequential squares 10,000-element array: {}ns", lin_square(10000));
  println!("Quick test sequential squares 1,000,000-element array: {}ns", lin_square(1000000));
  println!("Quick test sequential mx+b 100-element array: {}ns", lin_mx_plus_b(100));
  println!("Quick test sequential mx+b 10,000-element array: {}ns", lin_mx_plus_b(10000));
  println!("Quick test sequential mx+b 1,000,000-element array: {}ns", lin_mx_plus_b(1000000));
}