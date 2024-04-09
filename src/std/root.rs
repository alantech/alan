/// Rust functions that the root scope binds.

/// The `AlanError` type is a *cloneable* error that all errors are implemented as within Alan, to
/// simplify error handling. In the future it will have a stack trace based on the Alan source
/// code, but for now only a simple error message is provided.
#[derive(Clone, Debug)]
struct AlanError {
    message: String,
}

impl std::fmt::Display for AlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl std::error::Error for AlanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<&str> for AlanError {
    fn from(s: &str) -> AlanError {
        AlanError {
            message: s.to_string(),
        }
    }
}

/// `alan_ok` is a wrapper function that takes a reference to a value, clones it, and returns it as
/// a Result-wrapped value. Hopefully this weird function will die soon.
fn alan_ok<A: std::clone::Clone>(val: &A) -> Result<A, AlanError> {
    Ok(val.clone())
}

/// `to_exit_code_i8` converts a 64-bit integer into an exit code, for convenience since `i64` is the
/// default integer type in Alan.
fn to_exit_code_i8(i: &i8) -> std::process::ExitCode {
    (*i as u8).into()
}

/// `i64toi8` casts an i64 to an i8.
fn i64toi8(i: &i64) -> i8 {
    *i as i8
}

/// `Result_i8` is a type alias for Result<i8, AlanError>
type Result_i8 = Result<i8, AlanError>;

/// `get_or_i8` unwraps a Result_i8 with the default value if it is an error
fn get_or_i8(r: &Result_i8, default: &i8) -> i8 {
    match r {
        Ok(v) => *v,
        Err(_) => *default,
    }
}

/// `addi8` safely adds two i8s together, returning a Result-wrapped i8 (or an error on overflow)
fn addi8(a: &i8, b: &i8) -> Result_i8 {
    match a.checked_add(*b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `addi8_result` safely adds two Result_i8s together, returning a Result-wrapped i8 (or an error on overflow)
fn addi8_result(a: &Result_i8, b: &Result_i8) -> Result_i8 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_add(*b) {
                Some(c) => Ok(c),
                None => Err("Overflow".into()),
            }
        }
    }
}

/// `subi8` safely subtracts two i8s, returning a Result-wrapped i8 (or an error on underflow)
fn subi8(a: &i8, b: &i8) -> Result_i8 {
    match a.checked_sub(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `subi8_result` safely subtracts two i8s, returning a Result-wrapped i8 (or an error on underflow)
fn subi8_result(a: &Result_i8, b: &Result_i8) -> Result_i8 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_sub(*b) {
                Some(c) => Ok(c),
                None => Err("Underflow".into()),
            }
        }
    }
}

/// `muli8` safely multiplies two i8s, returning a Result-wrapped i8 (or an error on under/overflow)
fn muli8(a: &i8, b: &i8) -> Result_i8 {
    match a.checked_mul(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `muli8_result` safely multiplies two Result_i8s, returning a Result-wrapped i8 (or an error on under/overflow)
fn muli8_result(a: &Result_i8, b: &Result_i8) -> Result_i8 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_mul(*b) {
                Some(c) => Ok(c),
                None => Err("Underflow or Overflow".into()),
            }
        }
    }
}

/// `divi8` safely divides two i8s, returning a Result-wrapped i8 (or an error on divide-by-zero)
fn divi8(a: &i8, b: &i8) -> Result_i8 {
    match a.checked_div(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `divi8_result` safely divides two Result_i8s, returning a Result-wrapped i8 (or an error on divide-by-zero)
fn divi8_result(a: &Result_i8, b: &Result_i8) -> Result_i8 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_div(*b) {
                Some(c) => Ok(c),
                None => Err("Divide-by-zero".into()),
            }
        }
    }
}

/// `modi8` safely divides two i8s, returning a Result-wrapped remainder in i8 (or an error on divide-by-zero)
fn modi8(a: &i8, b: &i8) -> Result_i8 {
    match a.checked_rem(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi8_result` safely divides two Result_i8s, returning a Result-wrapped remainder in i8 (or an error on divide-by-zero)
fn modi8_result(a: &Result_i8, b: &Result_i8) -> Result_i8 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_rem(*b) {
                Some(c) => Ok(c),
                None => Err("Divide-by-zero".into()),
            }
        }
    }
}

/// `powi8` safely raises the first i8 to the second i8, returning a Result-wrapped i8 (or an error on under/overflow)
fn powi8(a: &i8, b: &i8) -> Result_i8 {
    // TODO: Support b being negative correctly
    match a.checked_pow(*b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `powi8_result` safely raises the first Result_i8 to the second Result_i8, returning a Result-wrapped i8 (or an error on under/overflow)
fn powi8_result(a: &Result_i8, b: &Result_i8) -> Result_i8 {
    // TODO: Support b being negative correctly
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_pow(*b as u32) {
                Some(c) => Ok(c),
                None => Err("Underflow or Overflow".into()),
            }
        }
    }
}

/// `mini8` returns the smaller of the two i8 values
fn mini8(a: &i8, b: &i8) -> i8 {
    if a < b { *a } else { *b }
}

/// `mini8_result` returns the smaller of the two Result_i8 values
fn mini8_result(a: &Result_i8, b: &Result_i8) -> Result_i8 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a < b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `maxi8` returns the larger of the two i8 values
fn maxi8(a: &i8, b: &i8) -> i8 {
    if a > b { *a } else { *b }
}

/// `maxi8_result` returns the larger of the two Result_i8 values
fn maxi8_result(a: &Result_i8, b: &Result_i8) -> Result_i8 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a > b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `i64toi16` casts an i64 to an i16.
fn i64toi16(i: &i64) -> i16 {
    *i as i16
}

/// `Result_i16` is a type alias for Result<i16, AlanError>
type Result_i16 = Result<i16, AlanError>;

/// `addi16` safely adds two i16s together, returning a Result-wrapped i16 (or an error on overflow)
fn addi16(a: &i16, b: &i16) -> Result_i16 {
    match a.checked_add(*b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `subi16` safely subtracts two i16s, returning a Result-wrapped i16 (or an error on underflow)
fn subi16(a: &i16, b: &i16) -> Result_i16 {
    match a.checked_sub(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `muli16` safely multiplies two i16s, returning a Result-wrapped i16 (or an error on under/overflow)
fn muli16(a: &i16, b: &i16) -> Result_i16 {
    match a.checked_mul(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `divi16` safely divides two i16s, returning a Result-wrapped i16 (or an error on divide-by-zero)
fn divi16(a: &i16, b: &i16) -> Result_i16 {
    match a.checked_div(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi16` safely divides two i16s, returning a Result-wrapped remainder in i16 (or an error on divide-by-zero)
fn modi16(a: &i16, b: &i16) -> Result_i16 {
    match a.checked_rem(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `powi16` safely raises the first i16 to the second i16, returning a Result-wrapped i16 (or an error on under/overflow)
fn powi16(a: &i16, b: &i16) -> Result_i16 {
    // TODO: Support b being negative correctly
    match a.checked_pow(*b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `mini16` returns the smaller of the two i16 values
fn mini16(a: &i16, b: &i16) -> i16 {
    if a < b { *a } else { *b }
}

/// `maxi16` returns the larger of the two i16 values
fn maxi16(a: &i16, b: &i16) -> i16 {
    if a > b { *a } else { *b }
}

/// `Result_i32` is a type alias for Result<i32, AlanError>
type Result_i32 = Result<i32, AlanError>;

/// `i64toi32` casts an i64 to an i32.
fn i64toi32(i: &i64) -> i32 {
    *i as i32
}

/// `addi32` safely adds two i32s together, returning a Result-wrapped i32 (or an error on overflow)
fn addi32(a: &i32, b: &i32) -> Result_i32 {
    match a.checked_add(*b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `subi32` safely subtracts two i32s, returning a Result-wrapped i32 (or an error on underflow)
fn subi32(a: &i32, b: &i32) -> Result_i32 {
    match a.checked_sub(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `muli32` safely multiplies two i32s, returning a Result-wrapped i32 (or an error on under/overflow)
fn muli32(a: &i32, b: &i32) -> Result_i32 {
    match a.checked_mul(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `divi32` safely divides two i32s, returning a Result-wrapped i32 (or an error on divide-by-zero)
fn divi32(a: &i32, b: &i32) -> Result_i32 {
    match a.checked_div(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi32` safely divides two i32s, returning a Result-wrapped remainder in i32 (or an error on divide-by-zero)
fn modi32(a: &i32, b: &i32) -> Result_i32 {
    match a.checked_rem(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `powi32` safely raises the first i32 to the second i32, returning a Result-wrapped i32 (or an error on under/overflow)
fn powi32(a: &i32, b: &i32) -> Result_i32 {
    // TODO: Support b being negative correctly
    match a.checked_pow(*b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `mini32` returns the smaller of the two i32 values
fn mini32(a: &i32, b: &i32) -> i32 {
    if a < b { *a } else { *b }
}

/// `maxi32` returns the larger of the two i32 values
fn maxi32(a: &i32, b: &i32) -> i32 {
    if a > b { *a } else { *b }
}

/// `Result_i64` is a type alias for Result<i64, AlanError>
type Result_i64 = Result<i64, AlanError>;

/// `get_or_i64` unwraps a Result_i64 with the default value if it is an error
fn get_or_i64(r: &Result_i64, default: &i64) -> i64 {
    match r {
        Ok(v) => *v,
        Err(_) => *default,
    }
}

/// `addi64` safely adds two i64s together, returning a Result-wrapped i64 (or an error on overflow)
fn addi64(a: &i64, b: &i64) -> Result_i64 {
    match a.checked_add(*b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `addi64_result` safely adds two Result_i64s together, returning a Result-wrapped i64 (or an error on overflow)
fn addi64_result(a: &Result_i64, b: &Result_i64) -> Result_i64 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_add(*b) {
                Some(c) => Ok(c),
                None => Err("Overflow".into()),
            }
        }
    }
}

/// `subi64` safely subtracts two i64s, returning a Result-wrapped i64 (or an error on underflow)
fn subi64(a: &i64, b: &i64) -> Result_i64 {
    match a.checked_sub(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `subi64_result` safely subtracts two Result_i64s, returning a Result-wrapped i64 (or an error on underflow)
fn subi64_result(a: &Result_i64, b: &Result_i64) -> Result_i64 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_sub(*b) {
                Some(c) => Ok(c),
                None => Err("Underflow".into()),
            }
        }
    }
}

/// `muli64` safely multiplies two i64s, returning a Result-wrapped i64 (or an error on under/overflow)
fn muli64(a: &i64, b: &i64) -> Result_i64 {
    match a.checked_mul(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `muli64_result` safely multiplies two Result_i64s, returning a Result-wrapped i64 (or an error on under/overflow)
fn muli64_result(a: &Result_i64, b: &Result_i64) -> Result_i64 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_mul(*b) {
                Some(c) => Ok(c),
                None => Err("Underflow or Overflow".into()),
            }
        }
    }
}

/// `divi64` safely divides two i64s, returning a Result-wrapped i64 (or an error on divide-by-zero)
fn divi64(a: &i64, b: &i64) -> Result_i64 {
    match a.checked_div(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `divi64_result` safely divides two Resul_i64s, returning a Result-wrapped i64 (or an error on divide-by-zero)
fn divi64_result(a: &Result_i64, b: &Result_i64) -> Result_i64 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_div(*b) {
                Some(c) => Ok(c),
                None => Err("Divide-by-zero".into()),
            }
        }
    }
}

/// `modi64` safely divides two i64s, returning a Result-wrapped remainder in i64 (or an error on divide-by-zero)
fn modi64(a: &i64, b: &i64) -> Result_i64 {
    match a.checked_rem(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi64_result` safely divides two Result_i64s, returning a Result-wrapped remainder in i64 (or an error on divide-by-zero)
fn modi64_result(a: &Result_i64, b: &Result_i64) -> Result_i64 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_rem(*b) {
                Some(c) => Ok(c),
                None => Err("Divide-by-zero".into()),
            }
        }
    }
}

/// `powi64` safely raises the first i64 to the second i64, returning a Result-wrapped i64 (or an error on under/overflow)
fn powi64(a: &i64, b: &i64) -> Result_i64 {
    // TODO: Support b being negative correctly
    match a.checked_pow(*b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `powi64_result` safely raises the first Result_i64 to the second Result_i64, returning a Result-wrapped i64 (or an error on under/overflow)
fn powi64_result(a: &Result_i64, b: &Result_i64) -> Result_i64 {
    // TODO: Support b being negative correctly
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.checked_pow(*b as u32) {
                Some(c) => Ok(c),
                None => Err("Underflow or Overflow".into()),
            }
        }
    }
}

/// `mini64` returns the smaller of the two i64 values
fn mini64(a: &i64, b: &i64) -> i64 {
    if a < b { *a } else { *b }
}

/// `mini64_result` returns the smaller of the two Result_i64 values
fn mini64_result(a: &Result_i64, b: &Result_i64) -> Result_i64 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a < b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `maxi64` returns the larger of the two i64 values
fn maxi64(a: &i64, b: &i64) -> i64 {
    if a > b { *a } else { *b }
}

/// `maxi64_result` returns the larger of the two Result_i64 values
fn maxi64_result(a: &Result_i64, b: &Result_i64) -> Result_i64 {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a > b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `get_or_exit` is basically an alias to `unwrap`, but as a function instead of a method
fn get_or_exit<A: Clone>(a: &Result<A, AlanError>) -> A {
    match a {
        Ok(v) => v.clone(),
        Err(e) => panic!("{:?}", e),
    }
}

/// `string_concat` is a simple function that concatenates two strings
fn string_concat(a: &String, b: &String) -> String {
    format!("{}{}", a, b).to_string()
}

/// `println` is a simple function that prints basically anything
fn println<A: std::fmt::Display>(a: &A) {
    println!("{}", a);
}

/// `println_result` is a small wrapper function that makes printing Result types easy
fn println_result<A: std::fmt::Display>(a: &Result<A, AlanError>) {
    match a {
      Ok(o) => println!("{}", o),
      Err(e) => println!("{:?}", e),
    };
}

/// `stdout` is a simple function that prints basically anything without a newline attached
fn stdout<A: std::fmt::Display>(a: &A) {
    print!("{}", a);
}

/// `wait` is a function that sleeps the current thread for the specified number of milliseconds
fn wait(t: &i64) {
    std::thread::sleep(std::time::Duration::from_millis(*t as u64));
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

/// `filled` returns a filled Vec<V> of the provided value for the provided size
fn filled<V: std::clone::Clone>(i: &V, l: &i64) -> Vec<V> {
    vec![i.clone(); *l as usize]
}

/// `print_vec` pretty prints a vector assuming the input type can be displayed
fn print_vec<A: std::fmt::Display>(vs: &Vec<A>) {
    println!("[{}]", vs.iter().map(|v| format!("{}", v)).collect::<Vec<String>>().join(", "));
}

/// `print_vec_result` pretty prints a vector of result values assuming the input can be displayed
fn print_vec_result<A: std::fmt::Display>(vs: &Vec<Result<A, AlanError>>) {
    println!("[{}]", vs.iter().map(|v| match v {
        Err(e) => format!("{:?}", e),
        Ok(a) => format!("{}", a)
    }).collect::<Vec<String>>().join(", "));
}

/// `map_onearg` runs the provided single-argument function on each element of the vector,
/// returning a new vector
fn map_onearg<A, B>(v: &Vec<A>, m: fn(&A) -> B) -> Vec<B> {
    v.iter().map(|val| m(val)).collect::<Vec<B>>()
}

/// `map_twoarg` runs the provided two-argument (value, index) function on each element of the
/// vector, returning a new vector
fn map_twoarg<A, B>(v: &Vec<A>, m: fn(&A, usize) -> B) -> Vec<B> {
    v.iter().enumerate().map(|(i, val)| m(val, i)).collect::<Vec<B>>()
}

/// `map_threearg` runs the provided three-argument (value, index, vec_ref) function on each
/// element of the vector, returning a new vector
fn map_threearg<A, B>(v: &Vec<A>, m: fn(&A, usize, &Vec<A>) -> B) -> Vec<B> {
    v.iter().enumerate().map(|(i, val)| m(val, i, &v)).collect::<Vec<B>>()
}

/// `parmap_onearg` runs the provided single-argument function on each element of the vector, with
/// a different subset of the vector run in parallel across all threads.
fn parmap_onearg<A: std::marker::Sync + 'static, B: std::marker::Send + std::clone::Clone + 'static>(v: &Vec<A>, m: fn(&A) -> B) -> Vec<B> {
    let par = std::thread::available_parallelism();
    match par {
        Err(_) => map_onearg(v, m), // Fall back to sequential if there's no available parallelism
        Ok(p) if p.get() == 1 => map_onearg(v, m), // Same here
        Ok(p) => {
            let l = v.len();
            let slice_len: isize = (l / p).try_into().unwrap();
            let mut out = Vec::new();
            out.reserve_exact(l);
            if slice_len == 0 {
                // We have more CPU cores than values to parallelize, let's assume the user knows
                // what they're doing and parallelize anyway
                let mut handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
                handles.reserve_exact(l);
                for i in 0..l {
                    let v_ptr = v.as_ptr() as usize;
                    let o_ptr = out.as_ptr() as usize;
                    handles.push(std::thread::spawn(move || {
                        unsafe {
                            let val = (v_ptr as *const A).offset(i as isize).as_ref().unwrap();
                            let mut out = (o_ptr as *mut B).offset(i as isize);
                            out.write(m(val));
                        }
                    }));
                }
                for handle in handles {
                    let res = handle.join();
                    match res {
                        Err(e) => panic!("{:?}", e),
                        Ok(_) => {},
                    }
                }
            } else {
                // We have more values than CPU cores, so let's divvy this up in batches per core
                let mut handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
                handles.reserve_exact(p.into());
                for i in 0..p.into() {
                    // I wanted to do this with slices, but their size varies at compile time so
                    // I'm just going with pointers instead
                    let v_ptr = v.as_ptr() as usize;
                    let o_ptr = out.as_ptr() as usize;
                    let s: isize = (i*(slice_len as usize)).try_into().unwrap();
                    let e: isize = if i == p.get() - 1 { l.try_into().unwrap() } else { ((i+1)*(slice_len as usize)).try_into().unwrap() };
                    handles.push(std::thread::spawn(move || {
                        let v_ptr = v_ptr as *const A;
                        let o_ptr = o_ptr as *mut B;
                        for i in s..e {
                            unsafe {
                                let val = v_ptr.offset(i).as_ref().unwrap();
                                let mut out = o_ptr.offset(i);
                                out.write(m(val));
                            }
                        }
                    }));
                }
                for handle in handles {
                    let res = handle.join();
                    match res {
                        Err(e) => panic!("{:?}", e),
                        Ok(_) => {},
                    }
                }
            }
            // We need to tweak the len, the values are there but the Vec doesn't know that
            unsafe {
                out.set_len(l);
            }
            out
        }
    }
}

/// `push` pushes an element into a vector
fn push<A: std::clone::Clone>(v: &mut Vec<A>, a: &A) {
    v.push(a.clone());
}

struct GPU {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GPU {
    pub fn new() -> Result<GPU, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::default();
        let adapter_future = instance.request_adapter(&wgpu::RequestAdapterOptions::default());
        let adapter = match futures::executor::block_on(adapter_future) {
            Some(a) => Ok(a),
            None => Err("Unable to acquire an adapter"),
        }?;
        let device_future = adapter.request_device(&wgpu::DeviceDescriptor::default(), None);
        let (device, queue) = futures::executor::block_on(device_future)?;
        Ok(GPU {
            instance,
            adapter,
            device,
            queue,
        })
    }
}

fn GPU_new() -> GPU {
    // TODO: Make this safer
    match GPU::new() {
        Ok(g) => g,
        Err(_) => unreachable!(),
    }
}

fn create_buffer_init(g: &mut GPU, usage: &mut wgpu::BufferUsages, vals: &mut Vec<i32>) -> wgpu::Buffer {
    let val_slice = &vals[..];
    let val_ptr = val_slice.as_ptr();
    let val_u8_len = vals.len() * 4;
    let val_u8: &[u8] = unsafe {
        std::slice::from_raw_parts(val_ptr as *const u8, val_u8_len)
    };
    wgpu::util::DeviceExt::create_buffer_init(&g.device, &wgpu::util::BufferInitDescriptor {
        label: None, // TODO: Add a label for easier debugging?
        contents: val_u8,
        usage: *usage,
    })
}

fn create_empty_buffer(g: &mut GPU, usage: &mut wgpu::BufferUsages, size: &mut i64) -> wgpu::Buffer {
    g.device.create_buffer(&wgpu::BufferDescriptor {
        label: None, // TODO: Add a label for easier debugging?
        size: *size as u64,
        usage: *usage,
        mapped_at_creation: false, // TODO: With `create_buffer_init` does this make any sense?
    })
}

// TODO: Either add the ability to bind to const values, or come up with a better solution. For
// now, just hardwire a few buffer usage types in these functions
fn map_read_buffer_type() -> wgpu::BufferUsages {
    wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST
}

fn storage_buffer_type() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
}

type Vec_Buffer<'a> = Vec<&'a wgpu::Buffer>;
type Vec_Vec_Buffer<'a> = Vec<Vec<&'a wgpu::Buffer>>;

fn Vec_Buffer_new<'a>() -> Vec_Buffer<'a> {
  Vec::new()
}

fn Vec_Vec_Buffer_new<'a>() -> Vec_Vec_Buffer<'a> {
  Vec::new()
}

struct GPGPU<'a> {
    pub source: String,
    pub entrypoint: String,
    pub buffers: Vec_Vec_Buffer<'a>,
}

impl GPGPU<'_> {
    fn new<'a>(source: String, buffers: Vec_Vec_Buffer<'a>) -> GPGPU<'a> {
        GPGPU {
            source,
            entrypoint: "main".to_string(),
            buffers,
        }
    }
}

fn GPGPU_new<'a>(source: &mut String, buffers: &'a mut Vec_Vec_Buffer) -> GPGPU<'a> {
    GPGPU::new(source.clone(), buffers.clone())
}

fn GPGPU_new_easy<'a>(source: &mut String, buffer: &'a mut wgpu::Buffer) -> GPGPU<'a> {
    GPGPU::new(source.clone(), vec!(vec!(buffer)))
}

fn gpu_run(g: &mut GPU, gg: &mut GPGPU) {
    let module = g.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&gg.source)),
    });
    let compute_pipeline = g.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &module,
        entry_point: &gg.entrypoint,
    });
    let mut bind_groups = Vec::new();
    let mut encoder =
        g.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute_pipeline);
        for i in 0..gg.buffers.len() {
            let bind_group_layout = compute_pipeline.get_bind_group_layout(i.try_into().unwrap());
            let bind_group_buffers = &gg.buffers[i];
            let mut bind_group_entries = Vec::new();
            for j in 0..bind_group_buffers.len() {
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: j.try_into().unwrap(),
                    resource: bind_group_buffers[j].as_entire_binding(),
                });
            }
            let bind_group = g.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &bind_group_entries[..],
            });
            bind_groups.push(bind_group);
        }
        for i in 0..gg.buffers.len() { // The Rust borrow checker is forcing my hand here
            cpass.set_bind_group(i.try_into().unwrap(), &bind_groups[i], &[]);
        }
        cpass.dispatch_workgroups(1, 1, 1); // TODO: Add support for workgroups during execution
    }
    g.queue.submit(Some(encoder.finish()));
}

fn read_buffer(g: &mut GPU, b: &mut wgpu::Buffer) -> Vec<i32> { // TODO: Support other value types
    let temp_buffer = create_empty_buffer(g, &mut map_read_buffer_type(), &mut b.size().try_into().unwrap());
    let mut encoder =
        g.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_buffer_to_buffer(b, 0, &temp_buffer, 0, b.size());
    g.queue.submit(Some(encoder.finish()));
    let temp_slice = temp_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    temp_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    g.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
    if let Ok(Ok(())) = receiver.recv() {
        let data = temp_slice.get_mapped_range();
        let data_ptr = data.as_ptr();
        let data_len = data.len() / 4; // From u8 to i32
        let data_i32: &[i32] = unsafe {
            std::slice::from_raw_parts(data_ptr as *const i32, data_len)
        };
        let result = data_i32.to_vec();
        drop(data);
        temp_buffer.unmap();
        result
    } else {
        panic!("failed to run compute on gpu!")
    }
}