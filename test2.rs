// The functions above `main` are taken from Alan's current standard library. They're only used by
// the first part of the test

/// `i64toi32` casts an i64 to an i32.
fn i64toi32(i: &i64) -> i32 {
    *i as i32
}

/// `println` is a simple function that prints basically anything
fn println<A: std::fmt::Display>(a: &A) {
    println!("{}", a);
}

/// `filled` returns a filled Vec<V> of the provided value for the provided size
fn filled<V: std::clone::Clone>(i: &V, l: &i64) -> Vec<V> {
    vec![i.clone(); *l as usize]
}

/// `now` is a function that returns std::time::Instant for right now
fn now() -> std::time::Instant {
    std::time::Instant::now()
}

/// `elapsed` gets the duration since the instant was created TODO: Borrow these values instead
fn elapsed(i: &std::time::Instant) -> std::time::Duration {
    i.elapsed()
}

/// `print_duration` pretty-prints a duration value. TODO: Move this into Alan code and out of here
fn print_duration(d: &std::time::Duration) {
    println!("{}.{:0>9}", d.as_secs(), d.subsec_nanos()); // TODO: Figure out which subsec to use
}

fn main() {
    // This first section of the test (excluding the println macro call) was simply copied directly
    // from the compiler output of the following Alan program:
    //   export fn main {
    //     let t1 = now();
    //     let v2 = filled(2.i32(), 500000000);
    //     print("Array creation");
    //     t1.elapsed().print();
    //   }
    // It's the baseline performance that I intend to compare against to see if the structure of
    // the generated output Rust is having any impact on the performance of this operation.
    let mut t1 = now();
    let mut v2 = filled(&mut i64toi32(&mut 2), &mut 500000000);
    println(&mut "Alan Array creation time".to_string());
    print_duration(&mut elapsed(&mut t1));
    println!("{}", v2[2]); // To try to make absolutely sure that this isn't optimized away

    // This section is a simple rewriting of the above using idiomatic Rust with the same types
    // involved. This is to check if passing everything but mutable reference rather than passing
    // true ownership has any impact.
    let t2 = now();
    let v_rust = vec![2i32; 500_000_000]; // If you don't assign this to a variable, it will be
                                       // optimized away
    println!("Rust Vec creation time");
    print_duration(&t2.elapsed());
    println!("{}", v_rust[2]); // Making sure the vec is not optimized away

    // These tests were to see if fixed-length arrays would fare better, but they're stack
    // allocated and half a million i32s simply blows the stack. Supposedly `Box`ing it should make
    // it heap allocated, but that's not the result I got. Keeping these here just to note that
    // sometimes you don't want to stack allocate your data. ;)
    /*let t3 = now();
    let a_rust = [2; 500_000_000];
    println!("Rust Fixed Array creation time");
    print_duration(&t3.elapsed());
    println!("{}", a_rust[2]); // This causes the program to stack overflow
   
    let t3 = now();
    let b_rust = Box::new([2; 500_000_000]);
    println!("Rust Fixed Array heap creation time");
    print_duration(&t3.elapsed());
    println!("{}", b_rust[2]); // Found this trick on StackOverflow. It also stack overflows... */

    // The ordering of these two determines which is faster. I suspect that the `alloc` function is
    // just an alias for `alloc_zeroed` on Linux, and that the optimization has figured out that
    // the memory isn'g being changed so it can just re-use the allocation between runs. These
    // don't fully accomplish the task since they don't set the array to the desired values, but
    // it's interesting to see just how efficient allocating a zeroed array is on modern hardware
    // (identical to a "regular" allocation).
    unsafe {
        let t4 = now();
        let memory = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(2_000_000_000, 4));
        println!("Rust memory allocation time");
        print_duration(&t4.elapsed());
        println!("{}", *memory); // No idea what will happen here
    }

    unsafe {
        let t5 = now();
        let zeroed = std::alloc::alloc_zeroed(std::alloc::Layout::from_size_align_unchecked(2_000_000_000, 4));
        println!("Rust zeroed memory allocation time");
        print_duration(&t5.elapsed());
        println!("{}", *zeroed); // Should get a zero
    }

    // This one allocates a raw block of memory and sets all of the fields to the desired value,
    // but completely manually. It turns out to have the same performance as the first two
    // approaches, so it's probably what they do under-the-hood.
    unsafe {
        let t6 = now();
        let twoed: &mut [i32] = std::slice::from_raw_parts_mut(std::alloc::alloc_zeroed(std::alloc::Layout::from_size_align_unchecked(2_000_000_000, 4)) as *mut i32, 500_000_000);
        for i in 0..500_000_000 { twoed[i] = 2; }
        println!("Rust memory allocation set to 2 time");
        print_duration(&t6.elapsed());
        println!("{}", twoed[2]); // Should get a two
    }

    // This abomination was me wondering if I could get better perf by setting the block of memory
    // to the desired value 16 bytes at a time. Despite the reduced number of iterations, it's
    // still the same time, which implies the same bottleneck.
    unsafe {
        let t7 = now();
        let mem = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(2_000_000_000, 4));
        let twoed_set: &mut [i128] = std::slice::from_raw_parts_mut(mem as *mut i128, 125_000_000);
        let twoed_get: &[i32] = std::slice::from_raw_parts(mem as *const i32, 500_000_000);
        const four_twos: i128 = 2i128 + 2i128.pow(33) + 2i128.pow(65) + 2i128.pow(97); // Optimize plz
        for i in 0..125_000_000 { twoed_set[i] = four_twos; }
        println!("Rust memory allocation set to 2 four-at-a-time");
        print_duration(&t7.elapsed());
        println!("{}", twoed_get[2]); // Should get a two
    }

    unsafe {
        let t8 = now();
        let memory = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(2_000_000_000, 4));
        let first_twos: &mut [i32] = std::slice::from_raw_parts_mut(memory as *mut i32, 250_000_000);
        let second_twos: &mut [i32] = std::slice::from_raw_parts_mut((memory as *mut i32).offset(250_000_000), 250_000_000);
        let all_twos: &[i32] = std::slice::from_raw_parts(memory as *const i32, 500_000_000);
        let first_half = std::thread::spawn(move || {
            for i in 0..250_000_000 { first_twos[i] = 2; }
        });
        let second_half = std::thread::spawn(move || {
            for i in 0..250_000_000 { second_twos[i] = 2; }
        });
        let _ = first_half.join();
        let _ = second_half.join();
        println!("Rust memory allocation parallel thread set time");
        print_duration(&t8.elapsed());
        println!("{}", all_twos[2]);
    }

    unsafe {
        let t9 = now();
        let memory = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(2_000_000_000, 4));
        let first_twos: &mut [i32] = std::slice::from_raw_parts_mut(memory as *mut i32, 100_000_000);
        let second_twos: &mut [i32] = std::slice::from_raw_parts_mut((memory as *mut i32).offset(100_000_000), 100_000_000);
        let third_twos: &mut [i32] = std::slice::from_raw_parts_mut((memory as *mut i32).offset(200_000_000), 100_000_000);
        let fourth_twos: &mut [i32] = std::slice::from_raw_parts_mut((memory as *mut i32).offset(300_000_000), 100_000_000);
        let fifth_twos: &mut [i32] = std::slice::from_raw_parts_mut((memory as *mut i32).offset(400_000_000), 100_000_000);
        let all_twos: &[i32] = std::slice::from_raw_parts(memory as *const i32, 500_000_000);
        let first_fifth = std::thread::spawn(move || {
            for i in 0..100_000_000 { first_twos[i] = 2; }
        });
        let second_fifth = std::thread::spawn(move || {
            for i in 0..100_000_000 { second_twos[i] = 2; }
        });
        let third_fifth = std::thread::spawn(move || {
            for i in 0..100_000_000 { third_twos[i] = 2; }
        });
        let fourth_fifth = std::thread::spawn(move || {
            for i in 0..100_000_000 { fourth_twos[i] = 2; }
        });
        let fifth_fifth = std::thread::spawn(move || {
            for i in 0..100_000_000 { fifth_twos[i] = 2; }
        });
        let _ = first_fifth.join();
        let _ = second_fifth.join();
        let _ = third_fifth.join();
        let _ = fourth_fifth.join();
        let _ = fifth_fifth.join();
        println!("Rust memory allocation 5 parallel thread set time");
        print_duration(&t9.elapsed());
        println!("{}", all_twos[2]);
    }

    let t10 = now();
    let test_thread = std::thread::spawn(|| {
        println!("hi");
    });
    let _ = test_thread.join();
    println!("Rust thread fork-join time");
    print_duration(&t10.elapsed());
}