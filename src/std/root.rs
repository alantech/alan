/// Rust functions that the root scope binds.

/// `to_exit_code_i64` converts a 64-bit integer into an exit code, for convenience since `i64` is the
/// default integer type in Alan.
fn to_exit_code_i64(i: i64) -> std::process::ExitCode {
    (i as u8).into()
}

/// `to_exit_code_i8` converts a 64-bit integer into an exit code, for convenience since `i64` is the
/// default integer type in Alan.
fn to_exit_code_i8(i: i8) -> std::process::ExitCode {
    (i as u8).into()
}

/// `i64toi8` casts an i64 to an i8.
fn i64toi8(i: i64) -> i8 {
    i as i8
}

/// `addi8` safely adds two i8s together, returning a Result-wrapped i8 (or an error on overflow)
fn addi8(a: i8, b: i8) -> Result<i8, Box<dyn std::error::Error>> {
    match a.checked_add(b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `subi8` safely subtracts two i8s, returning a Result-wrapped i8 (or an error on underflow)
fn subi8(a: i8, b: i8) -> Result<i8, Box<dyn std::error::Error>> {
    match a.checked_sub(b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `muli8` safely multiplies two i8s, returning a Result-wrapped i8 (or an error on under/overflow)
fn muli8(a: i8, b: i8) -> Result<i8, Box<dyn std::error::Error>> {
    match a.checked_mul(b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `divi8` safely divides two i8s, returning a Result-wrapped i8 (or an error on divide-by-zero)
fn divi8(a: i8, b: i8) -> Result<i8, Box<dyn std::error::Error>> {
    match a.checked_div(b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi8` safely divides two i8s, returning a Result-wrapped remainder in i8 (or an error on divide-by-zero)
fn modi8(a: i8, b: i8) -> Result<i8, Box<dyn std::error::Error>> {
    match a.checked_rem(b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `powi8` safely raises the first i8 to the second i8, returning a Result-wrapped i8 (or an error on under/overflow)
fn powi8(a: i8, b: i8) -> Result<i8, Box<dyn std::error::Error>> {
    // TODO: Support b being negative correctly
    match a.checked_pow(b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `mini8` returns the smaller of the two i8 values
fn mini8(a: i8, b: i8) -> i8 {
    if a < b { a } else { b }
}

/// `maxi8` returns the larger of the two i8 values
fn maxi8(a: i8, b: i8) -> i8 {
    if a > b { a } else { b }
}

/// `i64toi16` casts an i64 to an i16.
fn i64toi16(i: i64) -> i16 {
    i as i16
}

/// `addi16` safely adds two i16s together, returning a Result-wrapped i16 (or an error on overflow)
fn addi16(a: i16, b: i16) -> Result<i16, Box<dyn std::error::Error>> {
    match a.checked_add(b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `subi16` safely subtracts two i16s, returning a Result-wrapped i16 (or an error on underflow)
fn subi16(a: i16, b: i16) -> Result<i16, Box<dyn std::error::Error>> {
    match a.checked_sub(b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `muli16` safely multiplies two i16s, returning a Result-wrapped i16 (or an error on under/overflow)
fn muli16(a: i16, b: i16) -> Result<i16, Box<dyn std::error::Error>> {
    match a.checked_mul(b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `divi16` safely divides two i16s, returning a Result-wrapped i16 (or an error on divide-by-zero)
fn divi16(a: i16, b: i16) -> Result<i16, Box<dyn std::error::Error>> {
    match a.checked_div(b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi16` safely divides two i16s, returning a Result-wrapped remainder in i16 (or an error on divide-by-zero)
fn modi16(a: i16, b: i16) -> Result<i16, Box<dyn std::error::Error>> {
    match a.checked_rem(b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `powi16` safely raises the first i16 to the second i16, returning a Result-wrapped i16 (or an error on under/overflow)
fn powi16(a: i16, b: i16) -> Result<i16, Box<dyn std::error::Error>> {
    // TODO: Support b being negative correctly
    match a.checked_pow(b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `mini16` returns the smaller of the two i16 values
fn mini16(a: i16, b: i16) -> i16 {
    if a < b { a } else { b }
}

/// `maxi16` returns the larger of the two i16 values
fn maxi16(a: i16, b: i16) -> i16 {
    if a > b { a } else { b }
}

/// `i64toi32` casts an i64 to an i32.
fn i64toi32(i: i64) -> i32 {
    i as i32
}

/// `addi32` safely adds two i32s together, returning a Result-wrapped i32 (or an error on overflow)
fn addi32(a: i32, b: i32) -> Result<i32, Box<dyn std::error::Error>> {
    match a.checked_add(b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `subi32` safely subtracts two i32s, returning a Result-wrapped i32 (or an error on underflow)
fn subi32(a: i32, b: i32) -> Result<i32, Box<dyn std::error::Error>> {
    match a.checked_sub(b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `muli32` safely multiplies two i32s, returning a Result-wrapped i32 (or an error on under/overflow)
fn muli32(a: i32, b: i32) -> Result<i32, Box<dyn std::error::Error>> {
    match a.checked_mul(b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `divi32` safely divides two i32s, returning a Result-wrapped i32 (or an error on divide-by-zero)
fn divi32(a: i32, b: i32) -> Result<i32, Box<dyn std::error::Error>> {
    match a.checked_div(b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi32` safely divides two i32s, returning a Result-wrapped remainder in i32 (or an error on divide-by-zero)
fn modi32(a: i32, b: i32) -> Result<i32, Box<dyn std::error::Error>> {
    match a.checked_rem(b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `powi32` safely raises the first i32 to the second i32, returning a Result-wrapped i32 (or an error on under/overflow)
fn powi32(a: i32, b: i32) -> Result<i32, Box<dyn std::error::Error>> {
    // TODO: Support b being negative correctly
    match a.checked_pow(b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `mini32` returns the smaller of the two i32 values
fn mini32(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

/// `maxi32` returns the larger of the two i32 values
fn maxi32(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

/// `addi64` safely adds two i64s together, returning a Result-wrapped i64 (or an error on overflow)
fn addi64(a: i64, b: i64) -> Result<i64, Box<dyn std::error::Error>> {
    match a.checked_add(b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `subi64` safely subtracts two i64s, returning a Result-wrapped i64 (or an error on underflow)
fn subi64(a: i64, b: i64) -> Result<i64, Box<dyn std::error::Error>> {
    match a.checked_sub(b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `muli64` safely multiplies two i64s, returning a Result-wrapped i64 (or an error on under/overflow)
fn muli64(a: i64, b: i64) -> Result<i64, Box<dyn std::error::Error>> {
    match a.checked_mul(b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `divi64` safely divides two i64s, returning a Result-wrapped i64 (or an error on divide-by-zero)
fn divi64(a: i64, b: i64) -> Result<i64, Box<dyn std::error::Error>> {
    match a.checked_div(b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi64` safely divides two i64s, returning a Result-wrapped remainder in i64 (or an error on divide-by-zero)
fn modi64(a: i64, b: i64) -> Result<i64, Box<dyn std::error::Error>> {
    match a.checked_rem(b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `powi64` safely raises the first i64 to the second i64, returning a Result-wrapped i64 (or an error on under/overflow)
fn powi64(a: i64, b: i64) -> Result<i64, Box<dyn std::error::Error>> {
    // TODO: Support b being negative correctly
    match a.checked_pow(b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `mini64` returns the smaller of the two i64 values
fn mini64(a: i64, b: i64) -> i64 {
    if a < b { a } else { b }
}

/// `maxi64` returns the larger of the two i64 values
fn maxi64(a: i64, b: i64) -> i64 {
    if a > b { a } else { b }
}

/// `get_or_exit` is basically an alias to `unwrap`, but as a function instead of a method
fn get_or_exit<A>(a: Result<A, Box<dyn std::error::Error>>) -> A {
    a.unwrap()
}

/// `string_concat` is a simple function that concatenates two strings
fn string_concat(a: String, b: String) -> String {
    format!("{}{}", a, b).to_string()
}

/// `println` is a simple function that prints basically anything
fn println<A: std::fmt::Display>(a: A) {
    println!("{}", a);
}

/// `println_result` is a small wrapper function that makes printing Result types easy
fn println_result<A: std::fmt::Display>(a: Result<A, Box<dyn std::error::Error>>) {
    match a {
      Ok(o) => println!("{}", o),
      Err(e) => println!("{:?}", e),
    };
}

/// `stdout` is a simple function that prints basically anything without a newline attached
fn stdout<A: std::fmt::Display>(a: A) {
    print!("{}", a);
}

/// `wait` is a function that sleeps the current thread for the specified number of milliseconds
fn wait(t: i64) {
    std::thread::sleep(std::time::Duration::from_millis(t as u64));
}