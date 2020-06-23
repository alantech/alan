use crate::vm::memory::HandlerMemory;

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

pub fn benchmark() {
  println!("Benchmark!");
}