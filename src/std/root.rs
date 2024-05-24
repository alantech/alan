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

/// `f64toi8` casts an f64 to an i8.
fn f64toi8(f: &f64) -> i8 {
    *f as i8
}

/// `f32toi8` casts an f32 to an i8.
fn f32toi8(f: &f32) -> i8 {
    *f as i8
}

/// `i64toi8` casts an i64 to an i8.
fn i64toi8(i: &i64) -> i8 {
    *i as i8
}

/// `i32toi8` casts an i32 to an i8.
fn i32toi8(i: &i32) -> i8 {
    *i as i8
}

/// `i16toi8` casts an i16 to an i8.
fn i16toi8(i: &i16) -> i8 {
    *i as i8
}

/// `get_or_i8` unwraps a Result<i8, AlanError> with the default value if it is an error
fn get_or_i8(r: &Result<i8, AlanError>, default: &i8) -> i8 {
    match r {
        Ok(v) => *v,
        Err(_) => *default,
    }
}

/// `addi8` safely adds two i8s together, returning a Result-wrapped i8 (or an error on overflow)
fn addi8(a: &i8, b: &i8) -> Result<i8, AlanError> {
    match a.checked_add(*b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `addi8_result` safely adds two Result<i8, AlanError>s together, returning a Result-wrapped i8 (or an error on overflow)
fn addi8_result(a: &Result<i8, AlanError>, b: &Result<i8, AlanError>) -> Result<i8, AlanError> {
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
fn subi8(a: &i8, b: &i8) -> Result<i8, AlanError> {
    match a.checked_sub(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `subi8_result` safely subtracts two i8s, returning a Result-wrapped i8 (or an error on underflow)
fn subi8_result(a: &Result<i8, AlanError>, b: &Result<i8, AlanError>) -> Result<i8, AlanError> {
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
fn muli8(a: &i8, b: &i8) -> Result<i8, AlanError> {
    match a.checked_mul(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `muli8_result` safely multiplies two Result<i8, AlanError>s, returning a Result-wrapped i8 (or an error on under/overflow)
fn muli8_result(a: &Result<i8, AlanError>, b: &Result<i8, AlanError>) -> Result<i8, AlanError> {
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
fn divi8(a: &i8, b: &i8) -> Result<i8, AlanError> {
    match a.checked_div(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `divi8_result` safely divides two Result<i8, AlanError>s, returning a Result-wrapped i8 (or an error on divide-by-zero)
fn divi8_result(a: &Result<i8, AlanError>, b: &Result<i8, AlanError>) -> Result<i8, AlanError> {
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
fn modi8(a: &i8, b: &i8) -> Result<i8, AlanError> {
    match a.checked_rem(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi8_result` safely divides two Result<i8, AlanError>s, returning a Result-wrapped remainder in i8 (or an error on divide-by-zero)
fn modi8_result(a: &Result<i8, AlanError>, b: &Result<i8, AlanError>) -> Result<i8, AlanError> {
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
fn powi8(a: &i8, b: &i8) -> Result<i8, AlanError> {
    // TODO: Support b being negative correctly
    match a.checked_pow(*b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `powi8_result` safely raises the first Result<i8, AlanError> to the second Result<i8, AlanError>, returning a Result-wrapped i8 (or an error on under/overflow)
fn powi8_result(a: &Result<i8, AlanError>, b: &Result<i8, AlanError>) -> Result<i8, AlanError> {
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

/// `mini8_result` returns the smaller of the two Result<i8, AlanError> values
fn mini8_result(a: &Result<i8, AlanError>, b: &Result<i8, AlanError>) -> Result<i8, AlanError> {
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

/// `maxi8_result` returns the larger of the two Result<i8, AlanError> values
fn maxi8_result(a: &Result<i8, AlanError>, b: &Result<i8, AlanError>) -> Result<i8, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a > b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `f64toi16` casts an f64 to an i16.
fn f64toi16(f: &f64) -> i16 {
    *f as i16
}

/// `f32toi16` casts an f32 to an i16.
fn f32toi16(f: &f32) -> i16 {
    *f as i16
}

/// `i64toi16` casts an i64 to an i16.
fn i64toi16(i: &i64) -> i16 {
    *i as i16
}

/// `i32toi16` casts an i32 to an i16.
fn i32toi16(i: &i32) -> i16 {
    *i as i16
}

/// `i8toi16` casts an i8 to an i16.
fn i8toi16(i: &i8) -> i16 {
    *i as i16
}

/// `get_or_i16` unwraps a Result<i16, AlanError> with the default value if it is an error
fn get_or_i16(r: &Result<i16, AlanError>, default: &i16) -> i16 {
    match r {
        Ok(v) => *v,
        Err(_) => *default,
    }
}

/// `addi16` safely adds two i16s together, returning a Result-wrapped i16 (or an error on overflow)
fn addi16(a: &i16, b: &i16) -> Result<i16, AlanError> {
    match a.checked_add(*b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `addi16_result` safely adds two Result<i16, AlanError>s together, returning a Result-wrapped i16 (or an error on overflow)
fn addi16_result(a: &Result<i16, AlanError>, b: &Result<i16, AlanError>) -> Result<i16, AlanError> {
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

/// `subi16` safely subtracts two i16s, returning a Result-wrapped i16 (or an error on underflow)
fn subi16(a: &i16, b: &i16) -> Result<i16, AlanError> {
    match a.checked_sub(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `subi16_result` safely subtracts two i16s, returning a Result-wrapped i16 (or an error on underflow)
fn subi16_result(a: &Result<i16, AlanError>, b: &Result<i16, AlanError>) -> Result<i16, AlanError> {
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

/// `muli16` safely multiplies two i16s, returning a Result-wrapped i16 (or an error on under/overflow)
fn muli16(a: &i16, b: &i16) -> Result<i16, AlanError> {
    match a.checked_mul(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `muli16_result` safely multiplies two Result<i16, AlanError>s, returning a Result-wrapped i16 (or an error on under/overflow)
fn muli16_result(a: &Result<i16, AlanError>, b: &Result<i16, AlanError>) -> Result<i16, AlanError> {
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

/// `divi16` safely divides two i16s, returning a Result-wrapped i16 (or an error on divide-by-zero)
fn divi16(a: &i16, b: &i16) -> Result<i16, AlanError> {
    match a.checked_div(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `divi16_result` safely divides two Result<i16, AlanError>s, returning a Result-wrapped i16 (or an error on divide-by-zero)
fn divi16_result(a: &Result<i16, AlanError>, b: &Result<i16, AlanError>) -> Result<i16, AlanError> {
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

/// `modi16` safely divides two i16s, returning a Result-wrapped remainder in i16 (or an error on divide-by-zero)
fn modi16(a: &i16, b: &i16) -> Result<i16, AlanError> {
    match a.checked_rem(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi16_result` safely divides two Result<i16, AlanError>s, returning a Result-wrapped remainder in i16 (or an error on divide-by-zero)
fn modi16_result(a: &Result<i16, AlanError>, b: &Result<i16, AlanError>) -> Result<i16, AlanError> {
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

/// `powi16` safely raises the first i16 to the second i16, returning a Result-wrapped i16 (or an error on under/overflow)
fn powi16(a: &i16, b: &i16) -> Result<i16, AlanError> {
    // TODO: Support b being negative correctly
    match a.checked_pow(*b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `powi16_result` safely raises the first Result<i16, AlanError> to the second Result<i16, AlanError>, returning a Result-wrapped i16 (or an error on under/overflow)
fn powi16_result(a: &Result<i16, AlanError>, b: &Result<i16, AlanError>) -> Result<i16, AlanError> {
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

/// `mini16` returns the smaller of the two i16 values
fn mini16(a: &i16, b: &i16) -> i16 {
    if a < b { *a } else { *b }
}

/// `mini16_result` returns the smaller of the two Result<i16, AlanError> values
fn mini16_result(a: &Result<i16, AlanError>, b: &Result<i16, AlanError>) -> Result<i16, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a < b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `maxi16` returns the larger of the two i16 values
fn maxi16(a: &i16, b: &i16) -> i16 {
    if a > b { *a } else { *b }
}

/// `maxi16_result` returns the larger of the two Result<i16, AlanError> values
fn maxi16_result(a: &Result<i16, AlanError>, b: &Result<i16, AlanError>) -> Result<i16, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a > b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `get_or_i32` unwraps a Result<i32, AlanError> with the default value if it is an error
fn get_or_i32(r: &Result<i32, AlanError>, default: &i32) -> i32 {
    match r {
        Ok(v) => *v,
        Err(_) => *default,
    }
}

/// `f64toi32` casts an f64 to an i32.
fn f64toi32(f: &f64) -> i32 {
    *f as i32
}

/// `f32toi32` casts an f32 to an i32.
fn f32toi32(f: &f32) -> i32 {
    *f as i32
}

/// `i64toi32` casts an i64 to an i32.
fn i64toi32(i: &i64) -> i32 {
    *i as i32
}

/// `i16toi32` casts an i16 to an i32.
fn i16toi32(i: &i16) -> i32 {
    *i as i32
}

/// `i8toi32` casts an i8 to an i32.
fn i8toi32(i: &i8) -> i32 {
    *i as i32
}

/// `addi32` safely adds two i32s together, returning a Result-wrapped i32 (or an error on overflow)
fn addi32(a: &i32, b: &i32) -> Result<i32, AlanError> {
    match a.checked_add(*b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `addi32_result` safely adds two Result<i32, AlanError>s together, returning a Result-wrapped i32 (or an error on overflow)
fn addi32_result(a: &Result<i32, AlanError>, b: &Result<i32, AlanError>) -> Result<i32, AlanError> {
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

/// `subi32` safely subtracts two i32s, returning a Result-wrapped i32 (or an error on underflow)
fn subi32(a: &i32, b: &i32) -> Result<i32, AlanError> {
    match a.checked_sub(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `subi32_result` safely subtracts two i32s, returning a Result-wrapped i32 (or an error on underflow)
fn subi32_result(a: &Result<i32, AlanError>, b: &Result<i32, AlanError>) -> Result<i32, AlanError> {
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

/// `muli32` safely multiplies two i32s, returning a Result-wrapped i32 (or an error on under/overflow)
fn muli32(a: &i32, b: &i32) -> Result<i32, AlanError> {
    match a.checked_mul(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `muli32_result` safely multiplies two Result<i32, AlanError>s, returning a Result-wrapped i32 (or an error on under/overflow)
fn muli32_result(a: &Result<i32, AlanError>, b: &Result<i32, AlanError>) -> Result<i32, AlanError> {
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

/// `divi32` safely divides two i32s, returning a Result-wrapped i32 (or an error on divide-by-zero)
fn divi32(a: &i32, b: &i32) -> Result<i32, AlanError> {
    match a.checked_div(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `divi32_result` safely divides two Result<i32, AlanError>s, returning a Result-wrapped i32 (or an error on divide-by-zero)
fn divi32_result(a: &Result<i32, AlanError>, b: &Result<i32, AlanError>) -> Result<i32, AlanError> {
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

/// `modi32` safely divides two i32s, returning a Result-wrapped remainder in i32 (or an error on divide-by-zero)
fn modi32(a: &i32, b: &i32) -> Result<i32, AlanError> {
    match a.checked_rem(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi32_result` safely divides two Result<i32, AlanError>s, returning a Result-wrapped remainder in i32 (or an error on divide-by-zero)
fn modi32_result(a: &Result<i32, AlanError>, b: &Result<i32, AlanError>) -> Result<i32, AlanError> {
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

/// `powi32` safely raises the first i32 to the second i32, returning a Result-wrapped i32 (or an error on under/overflow)
fn powi32(a: &i32, b: &i32) -> Result<i32, AlanError> {
    // TODO: Support b being negative correctly
    match a.checked_pow(*b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `powi32_result` safely raises the first Result<i32, AlanError> to the second Result<i32, AlanError>, returning a Result-wrapped i32 (or an error on under/overflow)
fn powi32_result(a: &Result<i32, AlanError>, b: &Result<i32, AlanError>) -> Result<i32, AlanError> {
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

/// `mini32` returns the smaller of the two i32 values
fn mini32(a: &i32, b: &i32) -> i32 {
    if a < b { *a } else { *b }
}

/// `mini32_result` returns the smaller of the two Result<i32, AlanError> values
fn mini32_result(a: &Result<i32, AlanError>, b: &Result<i32, AlanError>) -> Result<i32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a < b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `maxi32` returns the larger of the two i32 values
fn maxi32(a: &i32, b: &i32) -> i32 {
    if a > b { *a } else { *b }
}

/// `maxi32_result` returns the larger of the two Result<i32, AlanError> values
fn maxi32_result(a: &Result<i32, AlanError>, b: &Result<i32, AlanError>) -> Result<i32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a > b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `f64toi64` casts an f64 to an i64.
fn f64toi64(f: &f64) -> i64 {
    *f as i64
}

/// `f32toi64` casts an f32 to an i64.
fn f32toi64(f: &f32) -> i64 {
    *f as i64
}

/// `i8toi64` casts an i8 to an i64.
fn i8toi64(i: &i8) -> i64 {
    *i as i64
}

/// `i16toi64` casts an i16 to an i64.
fn i16toi64(i: &i16) -> i64 {
    *i as i64
}

/// `i32toi64` casts an i32 to an i64.
fn i32toi64(i: &i32) -> i64 {
    *i as i64
}

/// `get_or_i64` unwraps a Result<i64, AlanError> with the default value if it is an error
fn get_or_i64(r: &Result<i64, AlanError>, default: &i64) -> i64 {
    match r {
        Ok(v) => *v,
        Err(_) => *default,
    }
}

/// `addi64` safely adds two i64s together, returning a Result-wrapped i64 (or an error on overflow)
fn addi64(a: &i64, b: &i64) -> Result<i64, AlanError> {
    match a.checked_add(*b) {
        Some(c) => Ok(c),
        None => Err("Overflow".into()),
    }
}

/// `addi64_result` safely adds two Result<i64, AlanError>s together, returning a Result-wrapped i64 (or an error on overflow)
fn addi64_result(a: &Result<i64, AlanError>, b: &Result<i64, AlanError>) -> Result<i64, AlanError> {
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
fn subi64(a: &i64, b: &i64) -> Result<i64, AlanError> {
    match a.checked_sub(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow".into()),
    }
}

/// `subi64_result` safely subtracts two Result<i64, AlanError>s, returning a Result-wrapped i64 (or an error on underflow)
fn subi64_result(a: &Result<i64, AlanError>, b: &Result<i64, AlanError>) -> Result<i64, AlanError> {
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
fn muli64(a: &i64, b: &i64) -> Result<i64, AlanError> {
    match a.checked_mul(*b) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `muli64_result` safely multiplies two Result<i64, AlanError>s, returning a Result-wrapped i64 (or an error on under/overflow)
fn muli64_result(a: &Result<i64, AlanError>, b: &Result<i64, AlanError>) -> Result<i64, AlanError> {
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
fn divi64(a: &i64, b: &i64) -> Result<i64, AlanError> {
    match a.checked_div(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `divi64_result` safely divides two Resul_i64s, returning a Result-wrapped i64 (or an error on divide-by-zero)
fn divi64_result(a: &Result<i64, AlanError>, b: &Result<i64, AlanError>) -> Result<i64, AlanError> {
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
fn modi64(a: &i64, b: &i64) -> Result<i64, AlanError> {
    match a.checked_rem(*b) {
        Some(c) => Ok(c),
        None => Err("Divide-by-zero".into()),
    }
}

/// `modi64_result` safely divides two Result<i64, AlanError>s, returning a Result-wrapped remainder in i64 (or an error on divide-by-zero)
fn modi64_result(a: &Result<i64, AlanError>, b: &Result<i64, AlanError>) -> Result<i64, AlanError> {
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
fn powi64(a: &i64, b: &i64) -> Result<i64, AlanError> {
    // TODO: Support b being negative correctly
    match a.checked_pow(*b as u32) {
        Some(c) => Ok(c),
        None => Err("Underflow or Overflow".into()),
    }
}

/// `powi64_result` safely raises the first Result<i64, AlanError> to the second Result<i64, AlanError>, returning a Result-wrapped i64 (or an error on under/overflow)
fn powi64_result(a: &Result<i64, AlanError>, b: &Result<i64, AlanError>) -> Result<i64, AlanError> {
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

/// `mini64_result` returns the smaller of the two Result<i64, AlanError> values
fn mini64_result(a: &Result<i64, AlanError>, b: &Result<i64, AlanError>) -> Result<i64, AlanError> {
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

/// `maxi64_result` returns the larger of the two Result<i64, AlanError> values
fn maxi64_result(a: &Result<i64, AlanError>, b: &Result<i64, AlanError>) -> Result<i64, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a > b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `get_or_f32` unwraps a Result<f32, AlanError> with the default value if it is an error
fn get_or_f32(r: &Result<f32, AlanError>, default: &f32) -> f32 {
    match r {
        Ok(v) => *v,
        Err(_) => *default,
    }
}

/// `f64tof32` casts an f64 to an f32.
fn f64tof32(f: &f64) -> f32 {
    *f as f32
}

/// `i64tof32` casts an i64 to an f32.
fn i64tof32(i: &i64) -> f32 {
    *i as f32
}

/// `i32tof32` casts an i32 to an f32.
fn i32tof32(i: &i32) -> f32 {
    *i as f32
}

/// `i16tof32` casts an i16 to an f32.
fn i16tof32(i: &i16) -> f32 {
    *i as f32
}

/// `i8tof32` casts an i8 to an f32.
fn i8tof32(i: &i8) -> f32 {
    *i as f32
}

/// `addf32` safely adds two f32s together, returning a Result-wrapped f32 (or an error on overflow)
fn addf32(a: &f32, b: &f32) -> Result<f32, AlanError> {
    match a + b {
        f32::MAX => Err("Overflow".into()),
        f32::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `addf32_result` safely adds two Result<f32, AlanError>s together, returning a Result-wrapped f32 (or an error on overflow)
fn addf32_result(a: &Result<f32, AlanError>, b: &Result<f32, AlanError>) -> Result<f32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a + b {
                f32::MAX => Err("Overflow".into()),
                f32::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `subf32` safely subtracts two f32s, returning a Result-wrapped f32 (or an error on underflow)
fn subf32(a: &f32, b: &f32) -> Result<f32, AlanError> {
    match a - b {
        f32::MAX => Err("Overflow".into()),
        f32::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `subf32_result` safely subtracts two f32s, returning a Result-wrapped f32 (or an error on underflow)
fn subf32_result(a: &Result<f32, AlanError>, b: &Result<f32, AlanError>) -> Result<f32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a - b {
                f32::MAX => Err("Overflow".into()),
                f32::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `mulf32` safely multiplies two f32s, returning a Result-wrapped f32 (or an error on under/overflow)
fn mulf32(a: &f32, b: &f32) -> Result<f32, AlanError> {
    match a * b {
        f32::MAX => Err("Overflow".into()),
        f32::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `mulf32_result` safely multiplies two Result<f32, AlanError>s, returning a Result-wrapped f32 (or an error on under/overflow)
fn mulf32_result(a: &Result<f32, AlanError>, b: &Result<f32, AlanError>) -> Result<f32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a * b {
                f32::MAX => Err("Overflow".into()),
                f32::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `divf32` safely divides two f32s, returning a Result-wrapped f32 (or an error on divide-by-zero)
fn divf32(a: &f32, b: &f32) -> Result<f32, AlanError> {
    match a / b {
        f32::MAX => Err("Overflow".into()),
        f32::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `divf32_result` safely divides two Result<f32, AlanError>s, returning a Result-wrapped f32 (or an error on divide-by-zero)
fn divf32_result(a: &Result<f32, AlanError>, b: &Result<f32, AlanError>) -> Result<f32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a / b {
                f32::MAX => Err("Overflow".into()),
                f32::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `sqrtf32` takes the square root of an f32, returning an f32
fn sqrtf32(f: &f32) -> f32 {
    f.sqrt()
}

/// `sqrtf32_result` takes the square root of a Result<f32, AlanError>, returning a Result<f32, AlanError>
fn sqrtf32_result(f: &Result<f32, AlanError>) -> Result<f32, AlanError> {
    match f {
      Err(e) => Err(e.clone()),
      Ok(v) => Ok(v.sqrt()),
    }
}

/// `powf32` safely raises the first f32 to the second f32, returning a Result-wrapped f32 (or an error on under/overflow)
fn powf32(a: &f32, b: &f32) -> Result<f32, AlanError> {
    match a.powf(*b) {
        f32::MAX => Err("Overflow".into()),
        f32::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `powf32_result` safely raises the first Result<f32, AlanError> to the second Result<f32, AlanError>, returning a Result-wrapped f32 (or an error on under/overflow)
fn powf32_result(a: &Result<f32, AlanError>, b: &Result<f32, AlanError>) -> Result<f32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.powf(*b) {
                f32::MAX => Err("Overflow".into()),
                f32::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `minf32` returns the smaller of the two f32 values
fn minf32(a: &f32, b: &f32) -> f32 {
    if a < b { *a } else { *b }
}

/// `minf32_result` returns the smaller of the two Result<f32, AlanError> values
fn minf32_result(a: &Result<f32, AlanError>, b: &Result<f32, AlanError>) -> Result<f32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a < b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `maxf32` returns the larger of the two f32 values
fn maxf32(a: &f32, b: &f32) -> f32 {
    if a > b { *a } else { *b }
}

/// `maxf32_result` returns the larger of the two Result<f32, AlanError> values
fn maxf32_result(a: &Result<f32, AlanError>, b: &Result<f32, AlanError>) -> Result<f32, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a > b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `f32tof64` casts an f32 to an f64.
fn f32tof64(f: &f32) -> f64 {
    *f as f64
}

/// `i8tof64` casts an i8 to an f64.
fn i8tof64(i: &i8) -> f64 {
    *i as f64
}

/// `i16tof64` casts an i16 to an f64.
fn i16tof64(i: &i16) -> f64 {
    *i as f64
}

/// `i32tof64` casts an i32 to an f64.
fn i32tof64(i: &i32) -> f64 {
    *i as f64
}

/// `i64tof64` casts an i64 to an f64.
fn i64tof64(i: &i64) -> f64 {
    *i as f64
}

/// `get_or_f64` unwraps a Result<f64, AlanError> with the default value if it is an error
fn get_or_f64(r: &Result<f64, AlanError>, default: &f64) -> f64 {
    match r {
        Ok(v) => *v,
        Err(_) => *default,
    }
}

/// `addf64` safely adds two f64s together, returning a Result-wrapped f64 (or an error on overflow)
fn addf64(a: &f64, b: &f64) -> Result<f64, AlanError> {
    match a + b {
        f64::MAX => Err("Overflow".into()),
        f64::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `addf64_result` safely adds two Result<f64, AlanError>s together, returning a Result-wrapped f64 (or an error on overflow)
fn addf64_result(a: &Result<f64, AlanError>, b: &Result<f64, AlanError>) -> Result<f64, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a + b {
                f64::MAX => Err("Overflow".into()),
                f64::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `subf64` safely subtracts two f64s, returning a Result-wrapped f64 (or an error on underflow)
fn subf64(a: &f64, b: &f64) -> Result<f64, AlanError> {
    match a - b {
        f64::MAX => Err("Overflow".into()),
        f64::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `subf64_result` safely subtracts two Result<f64, AlanError>s, returning a Result-wrapped f64 (or an error on underflow)
fn subf64_result(a: &Result<f64, AlanError>, b: &Result<f64, AlanError>) -> Result<f64, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a - b {
                f64::MAX => Err("Overflow".into()),
                f64::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `mulf64` safely multiplies two f64s, returning a Result-wrapped f64 (or an error on under/overflow)
fn mulf64(a: &f64, b: &f64) -> Result<f64, AlanError> {
    match a * b {
        f64::MAX => Err("Overflow".into()),
        f64::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `mulf64_result` safely multiplies two Result<f64, AlanError>s, returning a Result-wrapped f64 (or an error on under/overflow)
fn mulf64_result(a: &Result<f64, AlanError>, b: &Result<f64, AlanError>) -> Result<f64, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a * b {
                f64::MAX => Err("Overflow".into()),
                f64::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `divf64` safely divides two f64s, returning a Result-wrapped f64 (or an error on divide-by-zero)
fn divf64(a: &f64, b: &f64) -> Result<f64, AlanError> {
    match a / b {
        f64::MAX => Err("Overflow".into()),
        f64::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `divf64_result` safely divides two Resul_f64s, returning a Result-wrapped f64 (or an error on divide-by-zero)
fn divf64_result(a: &Result<f64, AlanError>, b: &Result<f64, AlanError>) -> Result<f64, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a / b {
                f64::MAX => Err("Overflow".into()),
                f64::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `sqrtf64` takes the square root of an f64, returning an f64
fn sqrtf64(f: &f64) -> f64 {
    f.sqrt()
}

/// `sqrtf64_result` takes the square root of a Result<f64, AlanError>, returning a Result<f64, AlanError>
fn sqrtf64_result(f: &Result<f64, AlanError>) -> Result<f64, AlanError> {
    match f {
      Err(e) => Err(e.clone()),
      Ok(v) => Ok(v.sqrt()),
    }
}

/// `powf64` safely raises the first f64 to the second f64, returning a Result-wrapped f64 (or an error on under/overflow)
fn powf64(a: &f64, b: &f64) -> Result<f64, AlanError> {
    match a.powf(*b) {
        f64::MAX => Err("Overflow".into()),
        f64::MIN => Err("Underflow".into()),
        o => if o.is_nan() {
          Err("Not a Number".into())
        } else {
          Ok(o)
        }
    }
}

/// `powf64_result` safely raises the first Result<f64, AlanError> to the second Result<f64, AlanError>, returning a Result-wrapped f64 (or an error on under/overflow)
fn powf64_result(a: &Result<f64, AlanError>, b: &Result<f64, AlanError>) -> Result<f64, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => match a.powf(*b) {
                f64::MAX => Err("Overflow".into()),
                f64::MIN => Err("Underflow".into()),
                o => if o.is_nan() {
                  Err("Not a Number".into())
                } else {
                  Ok(o)
                }
            }
        }
    }
}

/// `minf64` returns the smaller of the two f64 values
fn minf64(a: &f64, b: &f64) -> f64 {
    if a < b { *a } else { *b }
}

/// `minf64_result` returns the smaller of the two Result<f64, AlanError> values
fn minf64_result(a: &Result<f64, AlanError>, b: &Result<f64, AlanError>) -> Result<f64, AlanError> {
    match a {
        Err(e) => Err(e.clone()),
        Ok(a) => match b {
            Err(e) => Err(e.clone()),
            Ok(b) => if a < b { Ok(*a) } else { Ok(*b) }
        }
    }
}

/// `maxf64` returns the larger of the two f64 values
fn maxf64(a: &f64, b: &f64) -> f64 {
    if a > b { *a } else { *b }
}

/// `maxf64_result` returns the larger of the two Result<f64, AlanError> values
fn maxf64_result(a: &Result<f64, AlanError>, b: &Result<f64, AlanError>) -> Result<f64, AlanError> {
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

/// `i8tobool` converts an integer into a boolean
fn i8tobool(a: &i8) -> bool {
    *a != 0
}

/// `i16tobool` converts an integer into a boolean
fn i16tobool(a: &i16) -> bool {
    *a != 0
}

/// `i32tobool` converts an integer into a boolean
fn i32tobool(a: &i32) -> bool {
    *a != 0
}

/// `i64tobool` converts an integer into a boolean
fn i64tobool(a: &i64) -> bool {
    *a != 0
}

/// `f32tobool` converts an integer into a boolean
fn f32tobool(a: &f32) -> bool {
    *a != 0.0
}

/// `f64tobool` converts an integer into a boolean
fn f64tobool(a: &f64) -> bool {
    *a != 0.0
}

/// `stringtobool` converts a string into a boolean. "true" is true and everything else is false
fn stringtobool(a: &String) -> bool {
    a.as_str() == "true"
}

/// `and` performs a boolean `and`
fn and(a: &bool, b: &bool) -> bool {
    *a && *b
}

/// `or` performs a boolean `or`
fn or(a: &bool, b: &bool) -> bool {
    *a || *b
}

/// `xor` performs a boolean `xor`
fn xor(a: &bool, b: &bool) -> bool {
    *a ^ *b
}

/// `not` performs a boolean `not`
fn not(a: &bool) -> bool {
    !*a
}

/// `nand` performs a boolean `nand` (considering how computers are built, why is this not a
/// built-in operator?)
fn nand(a: &bool, b: &bool) -> bool {
    !(*a && *b)
}

/// `nor` performs a boolean `nor`
fn nor(a: &bool, b: &bool) -> bool {
    !(*a || *b)
}

/// `xnor` performs a boolean `xnor` (aka `eq`)
fn xnor(a: &bool, b: &bool) -> bool {
    *a == *b
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

/// `vec_len` returns the length of a vector
fn vec_len<A>(v: &Vec<A>) -> i64 {
    v.len() as i64
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
        // Let's ask the adapter for everything it can do
        let features = adapter.features();
        let limits = adapter.limits();
        let device_future = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: features,
            required_limits: limits,
        }, None);
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
    pub workgroup_sizes: [i64; 3],
}

impl GPGPU<'_> {
    fn new<'a>(source: String, buffers: Vec_Vec_Buffer<'a>, workgroup_sizes: [i64; 3]) -> GPGPU<'a> {
        GPGPU {
            source,
            entrypoint: "main".to_string(),
            buffers,
            workgroup_sizes,
        }
    }
}

fn GPGPU_new<'a>(source: &mut String, buffers: &'a mut Vec_Vec_Buffer) -> GPGPU<'a> {
    GPGPU::new(source.clone(), buffers.clone(), [1, 1, 1]) // TODO: Expose this
}

fn GPGPU_new_easy<'a>(source: &mut String, buffer: &'a mut wgpu::Buffer) -> GPGPU<'a> {
    // In order to support larger arrays, we need to split the buffer length across them. Each of
    // indices is allowed to be up to 65535 (yes, a 16-bit integer) leading to a maximum length of
    // 65535^3, or about 2.815x10^14 elements (about 281 trillion elements). Not quite up to the
    // 64-bit address space limit 2^64 or about 1.845x10^19 or about 18 quintillion elements, but
    // enough for exactly 1PB of 32-bit numbers in an array, so we should be good.
    // For now, the 65535 limit should be hardcoded by the shader author and an early exit
    // conditional check if the shader is operating on a nonexistent array index. This may change
    // in the future if the performance penalty of the bounds check is considered too high.
    //
    // Explaining the equation itself, the array length, L, needs to be split into X, Y, and Z
    // parts where L = X + A*Y + B*Z, with X, Y, and Z bound between 0 and 65534 (inclusive) while
    // A is 65535 and B is 65535^2 or 4294836225. Computing each dimension is to take the original
    // length of the array (which is the buffer size divided by 4 because we're only supporting
    // 32-bit numbers for now) and then getting the division and remainder first by the B constant,
    // and the Z limit becomes the division + 1, while the remainder is executed division and
    // remainder on the A constant, division + 1, and this remainder becomes the X limit (plus 1).
    // Including this big explanation in case I've made an off-by-one error here ;)
    let l: i64 = (buffer.size() / 4).try_into().unwrap();
    let z_div = l / 4294836225;
    let z = z_div + 1;
    let z_rem = l.wrapping_rem(4294836225);
    let y_div = z_rem / 65535;
    let y = y_div + 1;
    let y_rem = z_rem.wrapping_rem(65535);
    let x = y_rem + 1;
    GPGPU::new(source.clone(), vec!(vec!(buffer)), [x, y, z])
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
        cpass.dispatch_workgroups(gg.workgroup_sizes[0].try_into().unwrap(), gg.workgroup_sizes[1].try_into().unwrap(), gg.workgroup_sizes[2].try_into().unwrap());
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