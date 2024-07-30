/// Rust functions that the root scope binds.
use std::hash::Hasher;
use std::sync::OnceLock;

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

impl From<String> for AlanError {
    fn from(s: String) -> AlanError {
        AlanError { message: s }
    }
}

/// Functions for (potentially) every type

/// `clone` clones the input type
#[inline(always)]
fn clone<T: std::clone::Clone>(v: &T) -> T {
    v.clone()
}

/// `hash` hashes the input type
#[inline(always)]
fn hash<T>(v: &T) -> i64 {
    let mut hasher = std::hash::DefaultHasher::new();
    let v_len = std::mem::size_of::<T>();
    let v_raw = unsafe { std::slice::from_raw_parts(v as *const T as usize as *const u8, v_len) };
    hasher.write(v_raw);
    hasher.finish() as i64
}

/// `hasharray` hashes the input array one element at a time
#[inline(always)]
fn hasharray<T>(v: &Vec<T>) -> i64 {
    let mut hasher = std::hash::DefaultHasher::new();
    let v_len = std::mem::size_of::<T>();
    for r in v {
        let v_raw =
            unsafe { std::slice::from_raw_parts(r as *const T as usize as *const u8, v_len) };
        hasher.write(v_raw);
    }
    hasher.finish() as i64
}

/// `hashstring` hashes the input string
#[inline(always)]
fn hashstring(v: &String) -> i64 {
    let mut hasher = std::hash::DefaultHasher::new();
    hasher.write(v.as_str().as_bytes());
    hasher.finish() as i64
}

/// Fallible, Maybe, and Either functions

/// `maybe_get_or` gets the Option's value or returns the default if not present.
#[inline(always)]
fn maybe_get_or<T: std::clone::Clone>(v: &Option<T>, d: &T) -> T {
    match v {
        Some(val) => val.clone(),
        None => d.clone(),
    }
}

/// `fallible_get_or` gets the Fallible (Result with pre-bound error) value or returns the default
/// if not present.
#[inline(always)]
fn fallible_get_or<T: std::clone::Clone>(v: &Result<T, AlanError>, d: &T) -> T {
    match v {
        Ok(val) => val.clone(),
        Err(_) => d.clone(),
    }
}

/// `maybe_none` creates a None for the given maybe type
#[inline(always)]
fn maybe_none<T>() -> Option<T> {
    None
}

/// `fallible_error` create an Err for the given fallible type
#[inline(always)]
fn fallible_error<T>(m: &String) -> Result<T, AlanError> {
    Err(m.clone().into())
}

/// `maybe_exists` returns a boolean on whether or not the Maybe has a value
#[inline(always)]
fn maybe_exists<T>(v: &Option<T>) -> bool {
    v.is_some()
}

/// Signed Integer-related functions

/// `stringtoi8` tries to convert a string into an i8
#[inline(always)]
fn stringtoi8(s: &String) -> Result<i8, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64toi8` casts an f64 to an i8.
#[inline(always)]
fn f64toi8(f: &f64) -> i8 {
    *f as i8
}

/// `f32toi8` casts an f32 to an i8.
#[inline(always)]
fn f32toi8(f: &f32) -> i8 {
    *f as i8
}

/// `i64toi8` casts an i64 to an i8.
#[inline(always)]
fn i64toi8(i: &i64) -> i8 {
    *i as i8
}

/// `i32toi8` casts an i32 to an i8.
#[inline(always)]
fn i32toi8(i: &i32) -> i8 {
    *i as i8
}

/// `i16toi8` casts an i16 to an i8.
#[inline(always)]
fn i16toi8(i: &i16) -> i8 {
    *i as i8
}

/// `u64toi8` casts an u64 to an i8.
#[inline(always)]
fn u64toi8(i: &u64) -> i8 {
    *i as i8
}

/// `u32toi8` casts an u32 to an i8.
#[inline(always)]
fn u32toi8(i: &u32) -> i8 {
    *i as i8
}

/// `u16toi8` casts an u16 to an i8.
#[inline(always)]
fn u16toi8(i: &u16) -> i8 {
    *i as i8
}

/// `u8toi8` casts an u8 to an i8.
#[inline(always)]
fn u8toi8(i: &u8) -> i8 {
    *i as i8
}

/// `addi8` safely adds two i8s together, returning a potentially wrapped i8
#[inline(always)]
fn addi8(a: &i8, b: &i8) -> i8 {
    a.wrapping_add(*b)
}

/// `subi8` safely subtracts two i8s, returning a potentially wrapped i8
#[inline(always)]
fn subi8(a: &i8, b: &i8) -> i8 {
    a.wrapping_sub(*b)
}

/// `muli8` safely multiplies two i8s, returning a potentially wrapped i8
#[inline(always)]
fn muli8(a: &i8, b: &i8) -> i8 {
    a.wrapping_mul(*b)
}

/// `divi8` safely divides two i8s, returning a potentially wrapped i8
#[inline(always)]
fn divi8(a: &i8, b: &i8) -> i8 {
    a.wrapping_div(*b)
}

/// `modi8` safely divides two i8s, returning a potentially wrapped remainder in i8
#[inline(always)]
fn modi8(a: &i8, b: &i8) -> i8 {
    a.wrapping_rem(*b)
}

/// `powi8` safely raises the first i8 to the second i8, returning a potentially wrapped i8
#[inline(always)]
fn powi8(a: &i8, b: &i8) -> i8 {
    // TODO: Support b being negative correctly
    a.wrapping_pow(*b as u32)
}

/// `mini8` returns the smaller of the two i8 values
#[inline(always)]
fn mini8(a: &i8, b: &i8) -> i8 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxi8` returns the larger of the two i8 values
#[inline(always)]
fn maxi8(a: &i8, b: &i8) -> i8 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `negi8` negates the `i8` provided
#[inline(always)]
fn negi8(a: &i8) -> i8 {
    -(*a)
}

/// `andi8` performs a bitwise `and`
#[inline(always)]
fn andi8(a: &i8, b: &i8) -> i8 {
    *a & *b
}

/// `ori8` performs a bitwise `or`
#[inline(always)]
fn ori8(a: &i8, b: &i8) -> i8 {
    *a | *b
}

/// `xori8` performs a bitwise `xor`
#[inline(always)]
fn xori8(a: &i8, b: &i8) -> i8 {
    *a ^ *b
}

/// `noti8` performs a bitwise `not`
#[inline(always)]
fn noti8(a: &i8) -> i8 {
    !*a
}

/// `nandi8` performs a bitwise `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandi8(a: &i8, b: &i8) -> i8 {
    !(*a & *b)
}

/// `nori8` performs a bitwise `nor`
#[inline(always)]
fn nori8(a: &i8, b: &i8) -> i8 {
    !(*a | *b)
}

/// `xnori8` performs a bitwise `xnor`
#[inline(always)]
fn xnori8(a: &i8, b: &i8) -> i8 {
    !(*a ^ *b)
}

/// `eqi8` compares two i8s and returns if they are equal
#[inline(always)]
fn eqi8(a: &i8, b: &i8) -> bool {
    *a == *b
}

/// `neqi8` compares two i8s and returns if they are not equal
#[inline(always)]
fn neqi8(a: &i8, b: &i8) -> bool {
    *a != *b
}

/// `lti8` compares two i8s and returns if the first is smaller than the second
#[inline(always)]
fn lti8(a: &i8, b: &i8) -> bool {
    *a < *b
}

/// `ltei8` compares two i8s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn ltei8(a: &i8, b: &i8) -> bool {
    *a <= *b
}

/// `gti8` compares two i8s and returns if the first is larger than the second
#[inline(always)]
fn gti8(a: &i8, b: &i8) -> bool {
    *a > *b
}

/// `gtei8` compares two i8s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gtei8(a: &i8, b: &i8) -> bool {
    *a >= *b
}

/// `shli8` shifts the bits of the i8 to the left and truncates any overage (a cheap way to
/// accomplish something like `((a as u8 * 2) & 255) as i8`)
#[inline(always)]
fn shli8(a: &i8, b: &i8) -> i8 {
    a.wrapping_shl(*b as u32)
}

/// `shri8` shifts the bits of the i8 to the right and truncates any overage (a cheap way to
/// accomplish something like `((a as u8 / 2) & 255) as i8`)
#[inline(always)]
fn shri8(a: &i8, b: &i8) -> i8 {
    a.wrapping_shr(*b as u32)
}

/// `wrli8` wraps the bits of an i8 to the left (so a wrap of 1 makes the most significant bit the
/// least significant and increases the significance of all others)
#[inline(always)]
fn wrli8(a: &i8, b: &i8) -> i8 {
    a.rotate_left(*b as u32)
}

/// `wrri8` wraps the bits of an i8 to the right (so a wrap of 1 makes the least significant bit the
/// most significant and decreases the significance of all others)
#[inline(always)]
fn wrri8(a: &i8, b: &i8) -> i8 {
    a.rotate_right(*b as u32)
}

/// `stringtoi16` tries to convert a string into an i16
#[inline(always)]
fn stringtoi16(s: &String) -> Result<i16, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64toi16` casts an f64 to an i16.
#[inline(always)]
fn f64toi16(f: &f64) -> i16 {
    *f as i16
}

/// `f32toi16` casts an f32 to an i16.
#[inline(always)]
fn f32toi16(f: &f32) -> i16 {
    *f as i16
}

/// `i64toi16` casts an i64 to an i16.
#[inline(always)]
fn i64toi16(i: &i64) -> i16 {
    *i as i16
}

/// `i32toi16` casts an i32 to an i16.
#[inline(always)]
fn i32toi16(i: &i32) -> i16 {
    *i as i16
}

/// `i8toi16` casts an i8 to an i16.
#[inline(always)]
fn i8toi16(i: &i8) -> i16 {
    *i as i16
}

/// `u64toi16` casts an u64 to an i16.
#[inline(always)]
fn u64toi16(i: &u64) -> i16 {
    *i as i16
}

/// `u32toi16` casts an u32 to an i16.
#[inline(always)]
fn u32toi16(i: &u32) -> i16 {
    *i as i16
}

/// `u16toi16` casts an u16 to an i16.
#[inline(always)]
fn u16toi16(i: &u16) -> i16 {
    *i as i16
}

/// `u8toi16` casts an u8 to an i16.
#[inline(always)]
fn u8toi16(i: &u8) -> i16 {
    *i as i16
}

/// `addi16` safely adds two i16s together, returning a potentially wrapped i16
#[inline(always)]
fn addi16(a: &i16, b: &i16) -> i16 {
    a.wrapping_add(*b)
}

/// `subi16` safely subtracts two i16s, returning a potentially wrapped i16
#[inline(always)]
fn subi16(a: &i16, b: &i16) -> i16 {
    a.wrapping_sub(*b)
}

/// `muli16` safely multiplies two i16s, returning a potentially wrapped i16
#[inline(always)]
fn muli16(a: &i16, b: &i16) -> i16 {
    a.wrapping_mul(*b)
}

/// `divi16` safely divides two i16s, returning a potentially wrapped i16
#[inline(always)]
fn divi16(a: &i16, b: &i16) -> i16 {
    a.wrapping_div(*b)
}

/// `modi16` safely divides two i16s, returning a potentially wrapped remainder in i16
#[inline(always)]
fn modi16(a: &i16, b: &i16) -> i16 {
    a.wrapping_rem(*b)
}

/// `powi16` safely raises the first i16 to the second i16, returning a potentially wrapped i16
#[inline(always)]
fn powi16(a: &i16, b: &i16) -> i16 {
    // TODO: Support b being negative correctly
    a.wrapping_pow(*b as u32)
}

/// `mini16` returns the smaller of the two i16 values
#[inline(always)]
fn mini16(a: &i16, b: &i16) -> i16 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxi16` returns the larger of the two i16 values
#[inline(always)]
fn maxi16(a: &i16, b: &i16) -> i16 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `negi16` negates the `i16` provided
#[inline(always)]
fn negi16(a: &i16) -> i16 {
    -(*a)
}

/// `andi16` performs a bitwise `and`
#[inline(always)]
fn andi16(a: &i16, b: &i16) -> i16 {
    *a & *b
}

/// `ori16` performs a bitwise `or`
#[inline(always)]
fn ori16(a: &i16, b: &i16) -> i16 {
    *a | *b
}

/// `xori16` performs a bitwise `xor`
#[inline(always)]
fn xori16(a: &i16, b: &i16) -> i16 {
    *a ^ *b
}

/// `noti16` performs a bitwise `not`
#[inline(always)]
fn noti16(a: &i16) -> i16 {
    !*a
}

/// `nandi16` performs a bitwise `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandi16(a: &i16, b: &i16) -> i16 {
    !(*a & *b)
}

/// `nori16` performs a bitwise `nor`
#[inline(always)]
fn nori16(a: &i16, b: &i16) -> i16 {
    !(*a | *b)
}

/// `xnori16` performs a bitwise `xnor`
#[inline(always)]
fn xnori16(a: &i16, b: &i16) -> i16 {
    !(*a ^ *b)
}

/// `eqi16` compares two i16s and returns if they are equal
#[inline(always)]
fn eqi16(a: &i16, b: &i16) -> bool {
    *a == *b
}

/// `neqi16` compares two i16s and returns if they are not equal
#[inline(always)]
fn neqi16(a: &i16, b: &i16) -> bool {
    *a != *b
}

/// `lti16` compares two i16s and returns if the first is smaller than the second
#[inline(always)]
fn lti16(a: &i16, b: &i16) -> bool {
    *a < *b
}

/// `ltei16` compares two i16s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn ltei16(a: &i16, b: &i16) -> bool {
    *a <= *b
}

/// `gti16` compares two i16s and returns if the first is larger than the second
#[inline(always)]
fn gti16(a: &i16, b: &i16) -> bool {
    *a > *b
}

/// `gtei16` compares two i16s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gtei16(a: &i16, b: &i16) -> bool {
    *a >= *b
}

/// `shli16` shifts the bits of the i16 to the left and truncates any overage (a cheap way to
/// accomplish something like `(a as u16 * 2) as i16`)
#[inline(always)]
fn shli16(a: &i16, b: &i16) -> i16 {
    a.wrapping_shl(*b as u32)
}

/// `shri16` shifts the bits of the i16 to the right and truncates any overage (a cheap way to
/// accomplish something like `(a as u16 / 2) as i16`)
#[inline(always)]
fn shri16(a: &i16, b: &i16) -> i16 {
    a.wrapping_shr(*b as u32)
}

/// `wrli16` wraps the bits of an i16 to the left (so a wrap of 1 makes the most significant bit the
/// least significant and increases the significance of all others)
#[inline(always)]
fn wrli16(a: &i16, b: &i16) -> i16 {
    a.rotate_left(*b as u32)
}

/// `wrri16` wraps the bits of an i16 to the right (so a wrap of 1 makes the least significant bit the
/// most significant and decreases the significance of all others)
#[inline(always)]
fn wrri16(a: &i16, b: &i16) -> i16 {
    a.rotate_right(*b as u32)
}

/// `stringtoi32` tries to convert a string into an i32
#[inline(always)]
fn stringtoi32(s: &String) -> Result<i32, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64toi32` casts an f64 to an i32.
#[inline(always)]
fn f64toi32(f: &f64) -> i32 {
    *f as i32
}

/// `f32toi32` casts an f32 to an i32.
#[inline(always)]
fn f32toi32(f: &f32) -> i32 {
    *f as i32
}

/// `i64toi32` casts an i64 to an i32.
#[inline(always)]
fn i64toi32(i: &i64) -> i32 {
    *i as i32
}

/// `i16toi32` casts an i16 to an i32.
#[inline(always)]
fn i16toi32(i: &i16) -> i32 {
    *i as i32
}

/// `i8toi32` casts an i8 to an i32.
#[inline(always)]
fn i8toi32(i: &i8) -> i32 {
    *i as i32
}

/// `u64toi32` casts an u64 to an i32.
#[inline(always)]
fn u64toi32(i: &u64) -> i32 {
    *i as i32
}

/// `u32toi32` casts an u32 to an i32.
#[inline(always)]
fn u32toi32(i: &u32) -> i32 {
    *i as i32
}

/// `u16toi32` casts an u16 to an i32.
#[inline(always)]
fn u16toi32(i: &u16) -> i32 {
    *i as i32
}

/// `u8toi32` casts an u8 to an i32.
#[inline(always)]
fn u8toi32(i: &u8) -> i32 {
    *i as i32
}

/// `addi32` safely adds two i32s together, returning a potentially wrapped i32
#[inline(always)]
fn addi32(a: &i32, b: &i32) -> i32 {
    a.wrapping_add(*b)
}

/// `subi32` safely subtracts two i32s, returning a potentially wrapped i32
#[inline(always)]
fn subi32(a: &i32, b: &i32) -> i32 {
    a.wrapping_sub(*b)
}

/// `muli32` safely multiplies two i32s, returning a potentially wrapped i32
#[inline(always)]
fn muli32(a: &i32, b: &i32) -> i32 {
    a.wrapping_mul(*b)
}

/// `divi32` safely divides two i32s, returning a potentially wrapped i32
#[inline(always)]
fn divi32(a: &i32, b: &i32) -> i32 {
    a.wrapping_div(*b)
}

/// `modi32` safely divides two i32s, returning a potentially wrapped remainder in i32
#[inline(always)]
fn modi32(a: &i32, b: &i32) -> i32 {
    a.wrapping_rem(*b)
}

/// `powi32` safely raises the first i32 to the second i32, returning a potentially wrapped i32
#[inline(always)]
fn powi32(a: &i32, b: &i32) -> i32 {
    // TODO: Support b being negative correctly
    a.wrapping_pow(*b as u32)
}

/// `mini32` returns the smaller of the two i32 values
#[inline(always)]
fn mini32(a: &i32, b: &i32) -> i32 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxi32` returns the larger of the two i32 values
#[inline(always)]
fn maxi32(a: &i32, b: &i32) -> i32 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `negi32` negates the `i32` provided
#[inline(always)]
fn negi32(a: &i32) -> i32 {
    -(*a)
}

/// `andi32` performs a bitwise `and`
#[inline(always)]
fn andi32(a: &i32, b: &i32) -> i32 {
    *a & *b
}

/// `ori32` performs a bitwise `or`
#[inline(always)]
fn ori32(a: &i32, b: &i32) -> i32 {
    *a | *b
}

/// `xori32` performs a bitwise `xor`
#[inline(always)]
fn xori32(a: &i32, b: &i32) -> i32 {
    *a ^ *b
}

/// `noti32` performs a bitwise `not`
#[inline(always)]
fn noti32(a: &i32) -> i32 {
    !*a
}

/// `nandi32` performs a bitwise `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandi32(a: &i32, b: &i32) -> i32 {
    !(*a & *b)
}

/// `nori32` performs a bitwise `nor`
#[inline(always)]
fn nori32(a: &i32, b: &i32) -> i32 {
    !(*a | *b)
}

/// `xnori32` performs a bitwise `xnor`
#[inline(always)]
fn xnori32(a: &i32, b: &i32) -> i32 {
    !(*a ^ *b)
}

/// `eqi32` compares two i32s and returns if they are equal
#[inline(always)]
fn eqi32(a: &i32, b: &i32) -> bool {
    *a == *b
}

/// `neqi32` compares two i32s and returns if they are not equal
#[inline(always)]
fn neqi32(a: &i32, b: &i32) -> bool {
    *a != *b
}

/// `lti32` compares two i32s and returns if the first is smaller than the second
#[inline(always)]
fn lti32(a: &i32, b: &i32) -> bool {
    *a < *b
}

/// `ltei32` compares two i32s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn ltei32(a: &i32, b: &i32) -> bool {
    *a <= *b
}

/// `gti32` compares two i32s and returns if the first is larger than the second
#[inline(always)]
fn gti32(a: &i32, b: &i32) -> bool {
    *a > *b
}

/// `gtei32` compares two i32s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gtei32(a: &i32, b: &i32) -> bool {
    *a >= *b
}

/// `shli32` shifts the bits of the i32 to the left and truncates any overage (a cheap way to
/// accomplish something like `(a as u32 * 2) as i32`)
#[inline(always)]
fn shli32(a: &i32, b: &i32) -> i32 {
    a.wrapping_shl(*b as u32)
}

/// `shri32` shifts the bits of the i32 to the right and truncates any overage (a cheap way to
/// accomplish something like `(a as u32 / 2) as i32`)
#[inline(always)]
fn shri32(a: &i32, b: &i32) -> i32 {
    a.wrapping_shr(*b as u32)
}

/// `wrli32` wraps the bits of an i32 to the left (so a wrap of 1 makes the most significant bit the
/// least significant and increases the significance of all others)
#[inline(always)]
fn wrli32(a: &i32, b: &i32) -> i32 {
    a.rotate_left(*b as u32)
}

/// `wrri32` wraps the bits of an i32 to the right (so a wrap of 1 makes the least significant bit the
/// most significant and decreases the significance of all others)
#[inline(always)]
fn wrri32(a: &i32, b: &i32) -> i32 {
    a.rotate_right(*b as u32)
}

/// `stringtoi64` tries to convert a string into an i64
#[inline(always)]
fn stringtoi64(s: &String) -> Result<i64, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64toi64` casts an f64 to an i64.
#[inline(always)]
fn f64toi64(f: &f64) -> i64 {
    *f as i64
}

/// `f32toi64` casts an f32 to an i64.
#[inline(always)]
fn f32toi64(f: &f32) -> i64 {
    *f as i64
}

/// `i8toi64` casts an i8 to an i64.
#[inline(always)]
fn i8toi64(i: &i8) -> i64 {
    *i as i64
}

/// `i16toi64` casts an i16 to an i64.
#[inline(always)]
fn i16toi64(i: &i16) -> i64 {
    *i as i64
}

/// `i32toi64` casts an i32 to an i64.
#[inline(always)]
fn i32toi64(i: &i32) -> i64 {
    *i as i64
}

/// `u8toi64` casts an u8 to an i64.
#[inline(always)]
fn u8toi64(i: &u8) -> i64 {
    *i as i64
}

/// `u16toi64` casts an u16 to an i64.
#[inline(always)]
fn u16toi64(i: &u16) -> i64 {
    *i as i64
}

/// `u32toi64` casts an u32 to an i64.
#[inline(always)]
fn u32toi64(i: &u32) -> i64 {
    *i as i64
}

/// `u64toi64` casts an u64 to an i64.
#[inline(always)]
fn u64toi64(i: &u64) -> i64 {
    *i as i64
}

/// `addi64` safely adds two i64s together, returning a potentially wrapped i64
#[inline(always)]
fn addi64(a: &i64, b: &i64) -> i64 {
    a.wrapping_add(*b)
}

/// `subi64` safely subtracts two i64s, returning a potentially wrapped i64
#[inline(always)]
fn subi64(a: &i64, b: &i64) -> i64 {
    a.wrapping_sub(*b)
}

/// `muli64` safely multiplies two i64s, returning a potentially wrapped i64
#[inline(always)]
fn muli64(a: &i64, b: &i64) -> i64 {
    a.wrapping_mul(*b)
}

/// `divi64` safely divides two i64s, returning a potentially wrapped i64
#[inline(always)]
fn divi64(a: &i64, b: &i64) -> i64 {
    a.wrapping_div(*b)
}

/// `modi64` safely divides two i64s, returning a potentially wrapped remainder in i64
#[inline(always)]
fn modi64(a: &i64, b: &i64) -> i64 {
    a.wrapping_rem(*b)
}

/// `powi64` safely raises the first i64 to the second i64, returning a potentially wrapped i64
#[inline(always)]
fn powi64(a: &i64, b: &i64) -> i64 {
    // TODO: Support b being negative correctly
    a.wrapping_pow(*b as u32)
}

/// `mini64` returns the smaller of the two i64 values
#[inline(always)]
fn mini64(a: &i64, b: &i64) -> i64 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxi64` returns the larger of the two i64 values
#[inline(always)]
fn maxi64(a: &i64, b: &i64) -> i64 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `negi64` negates the `i64` provided
#[inline(always)]
fn negi64(a: &i64) -> i64 {
    -(*a)
}

/// `andi64` performs a bitwise `and`
#[inline(always)]
fn andi64(a: &i64, b: &i64) -> i64 {
    *a & *b
}

/// `ori64` performs a bitwise `or`
#[inline(always)]
fn ori64(a: &i64, b: &i64) -> i64 {
    *a | *b
}

/// `xori64` performs a bitwise `xor`
#[inline(always)]
fn xori64(a: &i64, b: &i64) -> i64 {
    *a ^ *b
}

/// `noti64` performs a bitwise `not`
#[inline(always)]
fn noti64(a: &i64) -> i64 {
    !*a
}

/// `nandi64` performs a bitwise `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandi64(a: &i64, b: &i64) -> i64 {
    !(*a & *b)
}

/// `nori64` performs a bitwise `nor`
#[inline(always)]
fn nori64(a: &i64, b: &i64) -> i64 {
    !(*a | *b)
}

/// `xnori64` performs a bitwise `xnor`
#[inline(always)]
fn xnori64(a: &i64, b: &i64) -> i64 {
    !(*a ^ *b)
}

/// `eqi64` compares two i64s and returns if they are equal
#[inline(always)]
fn eqi64(a: &i64, b: &i64) -> bool {
    *a == *b
}

/// `neqi64` compares two i64s and returns if they are not equal
#[inline(always)]
fn neqi64(a: &i64, b: &i64) -> bool {
    *a != *b
}

/// `lti64` compares two i64s and returns if the first is smaller than the second
#[inline(always)]
fn lti64(a: &i64, b: &i64) -> bool {
    *a < *b
}

/// `ltei64` compares two i64s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn ltei64(a: &i64, b: &i64) -> bool {
    *a <= *b
}

/// `gti64` compares two i64s and returns if the first is larger than the second
#[inline(always)]
fn gti64(a: &i64, b: &i64) -> bool {
    *a > *b
}

/// `gtei64` compares two i64s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gtei64(a: &i64, b: &i64) -> bool {
    *a >= *b
}

/// `shli64` shifts the bits of the i64 to the left and truncates any overage (a cheap way to
/// accomplish something like `(a as u64 * 2) as i64`)
#[inline(always)]
fn shli64(a: &i64, b: &i64) -> i64 {
    a.wrapping_shl(*b as u32)
}

/// `shri64` shifts the bits of the i64 to the right and truncates any overage (a cheap way to
/// accomplish something like `(a as u64 / 2) as i64`)
#[inline(always)]
fn shri64(a: &i64, b: &i64) -> i64 {
    a.wrapping_shr(*b as u32)
}

/// `wrli64` wraps the bits of an i64 to the left (so a wrap of 1 makes the most significant bit the
/// least significant and increases the significance of all others)
#[inline(always)]
fn wrli64(a: &i64, b: &i64) -> i64 {
    a.rotate_left(*b as u32)
}

/// `wrri64` wraps the bits of an i64 to the right (so a wrap of 1 makes the least significant bit the
/// most significant and decreases the significance of all others)
#[inline(always)]
fn wrri64(a: &i64, b: &i64) -> i64 {
    a.rotate_right(*b as u32)
}

/// Unsigned Integer-related functions

/// `stringtou8` tries to convert a string into an u8
#[inline(always)]
fn stringtou8(s: &String) -> Result<u8, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64tou8` casts an f64 to an u8.
#[inline(always)]
fn f64tou8(f: &f64) -> u8 {
    *f as u8
}

/// `f32tou8` casts an f32 to an u8.
#[inline(always)]
fn f32tou8(f: &f32) -> u8 {
    *f as u8
}

/// `i64tou8` casts an i64 to an u8.
#[inline(always)]
fn i64tou8(i: &i64) -> u8 {
    *i as u8
}

/// `i32tou8` casts an i32 to an u8.
#[inline(always)]
fn i32tou8(i: &i32) -> u8 {
    *i as u8
}

/// `i16tou8` casts an i16 to an u8.
#[inline(always)]
fn i16tou8(i: &i16) -> u8 {
    *i as u8
}

/// `i8tou8` casts an i8 to an u8.
#[inline(always)]
fn i8tou8(i: &i8) -> u8 {
    *i as u8
}

/// `u64tou8` casts an u64 to an u8.
#[inline(always)]
fn u64tou8(i: &u64) -> u8 {
    *i as u8
}

/// `u32tou8` casts an u32 to an u8.
#[inline(always)]
fn u32tou8(i: &u32) -> u8 {
    *i as u8
}

/// `u16tou8` casts an u16 to an u8.
#[inline(always)]
fn u16tou8(i: &u16) -> u8 {
    *i as u8
}

/// `addu8` safely adds two u8s together, returning a potentially wrapped u8
#[inline(always)]
fn addu8(a: &u8, b: &u8) -> u8 {
    a.wrapping_add(*b)
}

/// `subu8` safely subtracts two u8s, returning a potentially wrapped u8
#[inline(always)]
fn subu8(a: &u8, b: &u8) -> u8 {
    a.wrapping_sub(*b)
}

/// `mulu8` safely multiplies two u8s, returning a potentially wrapped u8
#[inline(always)]
fn mulu8(a: &u8, b: &u8) -> u8 {
    a.wrapping_mul(*b)
}

/// `divu8` safely divides two u8s, returning a potentially wrapped u8
#[inline(always)]
fn divu8(a: &u8, b: &u8) -> u8 {
    a.wrapping_div(*b)
}

/// `modu8` safely divides two u8s, returning a potentially wrapped remainder in u8
#[inline(always)]
fn modu8(a: &u8, b: &u8) -> u8 {
    a.wrapping_rem(*b)
}

/// `powu8` safely raises the first u8 to the second u8, returning a potentially wrapped u8
#[inline(always)]
fn powu8(a: &u8, b: &u8) -> u8 {
    // TODO: Support b being negative correctly
    a.wrapping_pow(*b as u32)
}

/// `minu8` returns the smaller of the two u8 values
#[inline(always)]
fn minu8(a: &u8, b: &u8) -> u8 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxu8` returns the larger of the two u8 values
#[inline(always)]
fn maxu8(a: &u8, b: &u8) -> u8 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `andu8` performs a bitwise `and`
#[inline(always)]
fn andu8(a: &u8, b: &u8) -> u8 {
    *a & *b
}

/// `oru8` performs a bitwise `or`
#[inline(always)]
fn oru8(a: &u8, b: &u8) -> u8 {
    *a | *b
}

/// `xoru8` performs a bitwise `xor`
#[inline(always)]
fn xoru8(a: &u8, b: &u8) -> u8 {
    *a ^ *b
}

/// `notu8` performs a bitwise `not`
#[inline(always)]
fn notu8(a: &u8) -> u8 {
    !*a
}

/// `nandu8` performs a bitwise `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandu8(a: &u8, b: &u8) -> u8 {
    !(*a & *b)
}

/// `noru8` performs a bitwise `nor`
#[inline(always)]
fn noru8(a: &u8, b: &u8) -> u8 {
    !(*a | *b)
}

/// `xnoru8` performs a bitwise `xnor`
#[inline(always)]
fn xnoru8(a: &u8, b: &u8) -> u8 {
    !(*a ^ *b)
}

/// `equ8` compares two u8s and returns if they are equal
#[inline(always)]
fn equ8(a: &u8, b: &u8) -> bool {
    *a == *b
}

/// `nequ8` compares two u8s and returns if they are not equal
#[inline(always)]
fn nequ8(a: &u8, b: &u8) -> bool {
    *a != *b
}

/// `ltu8` compares two u8s and returns if the first is smaller than the second
#[inline(always)]
fn ltu8(a: &u8, b: &u8) -> bool {
    *a < *b
}

/// `lteu8` compares two u8s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn lteu8(a: &u8, b: &u8) -> bool {
    *a <= *b
}

/// `gtu8` compares two u8s and returns if the first is larger than the second
#[inline(always)]
fn gtu8(a: &u8, b: &u8) -> bool {
    *a > *b
}

/// `gteu8` compares two u8s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gteu8(a: &u8, b: &u8) -> bool {
    *a >= *b
}

/// `shlu8` shifts the bits of the u8 to the left and truncates any overage (a cheap way to
/// accomplish something like `((a as u8 * 2) & 255) as u8`)
#[inline(always)]
fn shlu8(a: &u8, b: &u8) -> u8 {
    a.wrapping_shl(*b as u32)
}

/// `shru8` shifts the bits of the u8 to the right and truncates any overage (a cheap way to
/// accomplish something like `((a as u8 / 2) & 255) as u8`)
#[inline(always)]
fn shru8(a: &u8, b: &u8) -> u8 {
    a.wrapping_shr(*b as u32)
}

/// `wrlu8` wraps the bits of an u8 to the left (so a wrap of 1 makes the most significant bit the
/// least significant and increases the significance of all others)
#[inline(always)]
fn wrlu8(a: &u8, b: &u8) -> u8 {
    a.rotate_left(*b as u32)
}

/// `wrru8` wraps the bits of an u8 to the right (so a wrap of 1 makes the least significant bit the
/// most significant and decreases the significance of all others)
#[inline(always)]
fn wrru8(a: &u8, b: &u8) -> u8 {
    a.rotate_right(*b as u32)
}

/// `stringtou16` tries to convert a string into an u16
#[inline(always)]
fn stringtou16(s: &String) -> Result<u16, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64tou16` casts an f64 to an u16.
#[inline(always)]
fn f64tou16(f: &f64) -> u16 {
    *f as u16
}

/// `f32tou16` casts an f32 to an u16.
#[inline(always)]
fn f32tou16(f: &f32) -> u16 {
    *f as u16
}

/// `i64tou16` casts an i64 to an u16.
#[inline(always)]
fn i64tou16(i: &i64) -> u16 {
    *i as u16
}

/// `i32tou16` casts an i32 to an u16.
#[inline(always)]
fn i32tou16(i: &i32) -> u16 {
    *i as u16
}

/// `i16tou16` casts an i16 to an u16.
#[inline(always)]
fn i16tou16(i: &i16) -> u16 {
    *i as u16
}

/// `i8tou16` casts an i8 to an u16.
#[inline(always)]
fn i8tou16(i: &i8) -> u16 {
    *i as u16
}

/// `u64tou16` casts an u64 to an u16.
#[inline(always)]
fn u64tou16(i: &u64) -> u16 {
    *i as u16
}

/// `u32tou16` casts an u32 to an u16.
#[inline(always)]
fn u32tou16(i: &u32) -> u16 {
    *i as u16
}

/// `u8tou16` casts an u8 to an u16.
#[inline(always)]
fn u8tou16(i: &u8) -> u16 {
    *i as u16
}

/// `addu16` safely adds two u16s together, returning a potentially wrapped u16
#[inline(always)]
fn addu16(a: &u16, b: &u16) -> u16 {
    a.wrapping_add(*b)
}

/// `subu16` safely subtracts two u16s, returning a potentially wrapped u16
#[inline(always)]
fn subu16(a: &u16, b: &u16) -> u16 {
    a.wrapping_sub(*b)
}

/// `mulu16` safely multiplies two u16s, returning a potentially wrapped u16
#[inline(always)]
fn mulu16(a: &u16, b: &u16) -> u16 {
    a.wrapping_mul(*b)
}

/// `divu16` safely divides two u16s, returning a potentially wrapped u16
#[inline(always)]
fn divu16(a: &u16, b: &u16) -> u16 {
    a.wrapping_div(*b)
}

/// `modu16` safely divides two u16s, returning a potentially wrapped remainder in u16
#[inline(always)]
fn modu16(a: &u16, b: &u16) -> u16 {
    a.wrapping_rem(*b)
}

/// `powu16` safely raises the first u16 to the second u16, returning a potentially wrapped u16
#[inline(always)]
fn powu16(a: &u16, b: &u16) -> u16 {
    // TODO: Support b being negative correctly
    a.wrapping_pow(*b as u32)
}

/// `minu16` returns the smaller of the two u16 values
#[inline(always)]
fn minu16(a: &u16, b: &u16) -> u16 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxu16` returns the larger of the two u16 values
#[inline(always)]
fn maxu16(a: &u16, b: &u16) -> u16 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `andu16` performs a bitwise `and`
#[inline(always)]
fn andu16(a: &u16, b: &u16) -> u16 {
    *a & *b
}

/// `oru16` performs a bitwise `or`
#[inline(always)]
fn oru16(a: &u16, b: &u16) -> u16 {
    *a | *b
}

/// `xoru16` performs a bitwise `xor`
#[inline(always)]
fn xoru16(a: &u16, b: &u16) -> u16 {
    *a ^ *b
}

/// `notu16` performs a bitwise `not`
#[inline(always)]
fn notu16(a: &u16) -> u16 {
    !*a
}

/// `nandu16` performs a bitwise `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandu16(a: &u16, b: &u16) -> u16 {
    !(*a & *b)
}

/// `noru16` performs a bitwise `nor`
#[inline(always)]
fn noru16(a: &u16, b: &u16) -> u16 {
    !(*a | *b)
}

/// `xnoru16` performs a bitwise `xnor`
#[inline(always)]
fn xnoru16(a: &u16, b: &u16) -> u16 {
    !(*a ^ *b)
}

/// `equ16` compares two u16s and returns if they are equal
#[inline(always)]
fn equ16(a: &u16, b: &u16) -> bool {
    *a == *b
}

/// `nequ16` compares two u16s and returns if they are not equal
#[inline(always)]
fn nequ16(a: &u16, b: &u16) -> bool {
    *a != *b
}

/// `ltu16` compares two u16s and returns if the first is smaller than the second
#[inline(always)]
fn ltu16(a: &u16, b: &u16) -> bool {
    *a < *b
}

/// `lteu16` compares two u16s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn lteu16(a: &u16, b: &u16) -> bool {
    *a <= *b
}

/// `gtu16` compares two u16s and returns if the first is larger than the second
#[inline(always)]
fn gtu16(a: &u16, b: &u16) -> bool {
    *a > *b
}

/// `gteu16` compares two u16s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gteu16(a: &u16, b: &u16) -> bool {
    *a >= *b
}

/// `shlu16` shifts the bits of the u16 to the left and truncates any overage (a cheap way to
/// accomplish something like `(a as u16 * 2) as u16`)
#[inline(always)]
fn shlu16(a: &u16, b: &u16) -> u16 {
    a.wrapping_shl(*b as u32)
}

/// `shru16` shifts the bits of the u16 to the right and truncates any overage (a cheap way to
/// accomplish something like `(a as u16 / 2) as u16`)
#[inline(always)]
fn shru16(a: &u16, b: &u16) -> u16 {
    a.wrapping_shr(*b as u32)
}

/// `wrlu16` wraps the bits of an u16 to the left (so a wrap of 1 makes the most significant bit the
/// least significant and increases the significance of all others)
#[inline(always)]
fn wrlu16(a: &u16, b: &u16) -> u16 {
    a.rotate_left(*b as u32)
}

/// `wrru16` wraps the bits of an u16 to the right (so a wrap of 1 makes the least significant bit the
/// most significant and decreases the significance of all others)
#[inline(always)]
fn wrru16(a: &u16, b: &u16) -> u16 {
    a.rotate_right(*b as u32)
}

/// `stringtou32` tries to convert a string into an u32
#[inline(always)]
fn stringtou32(s: &String) -> Result<u32, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64tou32` casts an f64 to an u32.
#[inline(always)]
fn f64tou32(f: &f64) -> u32 {
    *f as u32
}

/// `f32tou32` casts an f32 to an u32.
#[inline(always)]
fn f32tou32(f: &f32) -> u32 {
    *f as u32
}

/// `i64tou32` casts an i64 to an u32.
#[inline(always)]
fn i64tou32(i: &i64) -> u32 {
    *i as u32
}

/// `i32tou32` casts an i32 to an u32.
#[inline(always)]
fn i32tou32(i: &i32) -> u32 {
    *i as u32
}

/// `i16tou32` casts an i16 to an u32.
#[inline(always)]
fn i16tou32(i: &i16) -> u32 {
    *i as u32
}

/// `i8tou32` casts an i8 to an u32.
#[inline(always)]
fn i8tou32(i: &i8) -> u32 {
    *i as u32
}

/// `u64tou32` casts an u64 to an u32.
#[inline(always)]
fn u64tou32(i: &u64) -> u32 {
    *i as u32
}

/// `u16tou32` casts an u16 to an u32.
#[inline(always)]
fn u16tou32(i: &u16) -> u32 {
    *i as u32
}

/// `u8tou32` casts an u8 to an u32.
#[inline(always)]
fn u8tou32(i: &u8) -> u32 {
    *i as u32
}

/// `addu32` safely adds two u32s together, returning a potentially wrapped u32
#[inline(always)]
fn addu32(a: &u32, b: &u32) -> u32 {
    a.wrapping_add(*b)
}

/// `subu32` safely subtracts two u32s, returning a potentially wrapped u32
#[inline(always)]
fn subu32(a: &u32, b: &u32) -> u32 {
    a.wrapping_sub(*b)
}

/// `mulu32` safely multiplies two u32s, returning a potentially wrapped u32
#[inline(always)]
fn mulu32(a: &u32, b: &u32) -> u32 {
    a.wrapping_mul(*b)
}

/// `divu32` safely divides two u32s, returning a potentially wrapped u32
#[inline(always)]
fn divu32(a: &u32, b: &u32) -> u32 {
    a.wrapping_div(*b)
}

/// `modu32` safely divides two u32s, returning a potentially wrapped remainder in u32
#[inline(always)]
fn modu32(a: &u32, b: &u32) -> u32 {
    a.wrapping_rem(*b)
}

/// `powu32` safely raises the first u32 to the second u32, returning a potentially wrapped u32
#[inline(always)]
fn powu32(a: &u32, b: &u32) -> u32 {
    // TODO: Support b being negative correctly
    a.wrapping_pow(*b as u32)
}

/// `minu32` returns the smaller of the two u32 values
#[inline(always)]
fn minu32(a: &u32, b: &u32) -> u32 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxu32` returns the larger of the two u32 values
#[inline(always)]
fn maxu32(a: &u32, b: &u32) -> u32 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `andu32` performs a bitwise `and`
#[inline(always)]
fn andu32(a: &u32, b: &u32) -> u32 {
    *a & *b
}

/// `oru32` performs a bitwise `or`
#[inline(always)]
fn oru32(a: &u32, b: &u32) -> u32 {
    *a | *b
}

/// `xoru32` performs a bitwise `xor`
#[inline(always)]
fn xoru32(a: &u32, b: &u32) -> u32 {
    *a ^ *b
}

/// `notu32` performs a bitwise `not`
#[inline(always)]
fn notu32(a: &u32) -> u32 {
    !*a
}

/// `nandu32` performs a bitwise `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandu32(a: &u32, b: &u32) -> u32 {
    !(*a & *b)
}

/// `noru32` performs a bitwise `nor`
#[inline(always)]
fn noru32(a: &u32, b: &u32) -> u32 {
    !(*a | *b)
}

/// `xnoru32` performs a bitwise `xnor`
#[inline(always)]
fn xnoru32(a: &u32, b: &u32) -> u32 {
    !(*a ^ *b)
}

/// `equ32` compares two u32s and returns if they are equal
#[inline(always)]
fn equ32(a: &u32, b: &u32) -> bool {
    *a == *b
}

/// `nequ32` compares two u32s and returns if they are not equal
#[inline(always)]
fn nequ32(a: &u32, b: &u32) -> bool {
    *a != *b
}

/// `ltu32` compares two u32s and returns if the first is smaller than the second
#[inline(always)]
fn ltu32(a: &u32, b: &u32) -> bool {
    *a < *b
}

/// `lteu32` compares two u32s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn lteu32(a: &u32, b: &u32) -> bool {
    *a <= *b
}

/// `gtu32` compares two u32s and returns if the first is larger than the second
#[inline(always)]
fn gtu32(a: &u32, b: &u32) -> bool {
    *a > *b
}

/// `gteu32` compares two u32s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gteu32(a: &u32, b: &u32) -> bool {
    *a >= *b
}

/// `shlu32` shifts the bits of the u32 to the left and truncates any overage (a cheap way to
/// accomplish something like `(a as u32 * 2) as u32`)
#[inline(always)]
fn shlu32(a: &u32, b: &u32) -> u32 {
    a.wrapping_shl(*b as u32)
}

/// `shru32` shifts the bits of the u32 to the right and truncates any overage (a cheap way to
/// accomplish something like `(a as u32 / 2) as u32`)
#[inline(always)]
fn shru32(a: &u32, b: &u32) -> u32 {
    a.wrapping_shr(*b as u32)
}

/// `wrlu32` wraps the bits of an u32 to the left (so a wrap of 1 makes the most significant bit the
/// least significant and increases the significance of all others)
#[inline(always)]
fn wrlu32(a: &u32, b: &u32) -> u32 {
    a.rotate_left(*b as u32)
}

/// `wrru32` wraps the bits of an u32 to the right (so a wrap of 1 makes the least significant bit the
/// most significant and decreases the significance of all others)
#[inline(always)]
fn wrru32(a: &u32, b: &u32) -> u32 {
    a.rotate_right(*b as u32)
}

/// `stringtou64` tries to convert a string into an u64
#[inline(always)]
fn stringtou64(s: &String) -> Result<u64, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64tou64` casts an f64 to an u64.
#[inline(always)]
fn f64tou64(f: &f64) -> u64 {
    *f as u64
}

/// `f32tou64` casts an f32 to an u64.
#[inline(always)]
fn f32tou64(f: &f32) -> u64 {
    *f as u64
}

/// `i8tou64` casts an i8 to an u64.
#[inline(always)]
fn i8tou64(i: &i8) -> u64 {
    *i as u64
}

/// `i16tou64` casts an i16 to an u64.
#[inline(always)]
fn i16tou64(i: &i16) -> u64 {
    *i as u64
}

/// `i32tou64` casts an i32 to an u64.
#[inline(always)]
fn i32tou64(i: &i32) -> u64 {
    *i as u64
}

/// `i64tou64` casts an i64 to an u64.
#[inline(always)]
fn i64tou64(i: &i64) -> u64 {
    *i as u64
}

/// `u8tou64` casts an u8 to an u64.
#[inline(always)]
fn u8tou64(i: &u8) -> u64 {
    *i as u64
}

/// `u16tou64` casts an u16 to an u64.
#[inline(always)]
fn u16tou64(i: &u16) -> u64 {
    *i as u64
}

/// `u32tou64` casts an u32 to an u64.
#[inline(always)]
fn u32tou64(i: &u32) -> u64 {
    *i as u64
}

/// `addu64` safely adds two u64s together, returning a potentially wrapped u64
#[inline(always)]
fn addu64(a: &u64, b: &u64) -> u64 {
    a.wrapping_add(*b)
}

/// `subu64` safely subtracts two u64s, returning a potentially wrapped u64
#[inline(always)]
fn subu64(a: &u64, b: &u64) -> u64 {
    a.wrapping_sub(*b)
}

/// `mulu64` safely multiplies two u64s, returning a potentially wrapped u64
#[inline(always)]
fn mulu64(a: &u64, b: &u64) -> u64 {
    a.wrapping_mul(*b)
}

/// `divu64` safely divides two u64s, returning a potentially wrapped u64
#[inline(always)]
fn divu64(a: &u64, b: &u64) -> u64 {
    a.wrapping_div(*b)
}

/// `modu64` safely divides two u64s, returning a potentially wrapped remainder in u64
#[inline(always)]
fn modu64(a: &u64, b: &u64) -> u64 {
    a.wrapping_rem(*b)
}

/// `powu64` safely raises the first u64 to the second u64, returning a potentially wrapped u64
#[inline(always)]
fn powu64(a: &u64, b: &u64) -> u64 {
    // TODO: Support b being negative correctly
    a.wrapping_pow(*b as u32)
}

/// `minu64` returns the smaller of the two u64 values
#[inline(always)]
fn minu64(a: &u64, b: &u64) -> u64 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxu64` returns the larger of the two u64 values
#[inline(always)]
fn maxu64(a: &u64, b: &u64) -> u64 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `andu64` performs a bitwise `and`
#[inline(always)]
fn andu64(a: &u64, b: &u64) -> u64 {
    *a & *b
}

/// `oru64` performs a bitwise `or`
#[inline(always)]
fn oru64(a: &u64, b: &u64) -> u64 {
    *a | *b
}

/// `xoru64` performs a bitwise `xor`
#[inline(always)]
fn xoru64(a: &u64, b: &u64) -> u64 {
    *a ^ *b
}

/// `notu64` performs a bitwise `not`
#[inline(always)]
fn notu64(a: &u64) -> u64 {
    !*a
}

/// `nandu64` performs a bitwise `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandu64(a: &u64, b: &u64) -> u64 {
    !(*a & *b)
}

/// `noru64` performs a bitwise `nor`
#[inline(always)]
fn noru64(a: &u64, b: &u64) -> u64 {
    !(*a | *b)
}

/// `xnoru64` performs a bitwise `xnor`
#[inline(always)]
fn xnoru64(a: &u64, b: &u64) -> u64 {
    !(*a ^ *b)
}

/// `equ64` compares two u64s and returns if they are equal
#[inline(always)]
fn equ64(a: &u64, b: &u64) -> bool {
    *a == *b
}

/// `nequ64` compares two u64s and returns if they are not equal
#[inline(always)]
fn nequ64(a: &u64, b: &u64) -> bool {
    *a != *b
}

/// `ltu64` compares two u64s and returns if the first is smaller than the second
#[inline(always)]
fn ltu64(a: &u64, b: &u64) -> bool {
    *a < *b
}

/// `lteu64` compares two u64s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn lteu64(a: &u64, b: &u64) -> bool {
    *a <= *b
}

/// `gtu64` compares two u64s and returns if the first is larger than the second
#[inline(always)]
fn gtu64(a: &u64, b: &u64) -> bool {
    *a > *b
}

/// `gteu64` compares two u64s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gteu64(a: &u64, b: &u64) -> bool {
    *a >= *b
}

/// `shlu64` shifts the bits of the u64 to the left and truncates any overage (a cheap way to
/// accomplish something like `(a as u64 * 2) as u64`)
#[inline(always)]
fn shlu64(a: &u64, b: &u64) -> u64 {
    a.wrapping_shl(*b as u32)
}

/// `shru64` shifts the bits of the u64 to the right and truncates any overage (a cheap way to
/// accomplish something like `(a as u64 / 2) as u64`)
#[inline(always)]
fn shru64(a: &u64, b: &u64) -> u64 {
    a.wrapping_shr(*b as u32)
}

/// `wrlu64` wraps the bits of an u64 to the left (so a wrap of 1 makes the most significant bit the
/// least significant and increases the significance of all others)
#[inline(always)]
fn wrlu64(a: &u64, b: &u64) -> u64 {
    a.rotate_left(*b as u32)
}

/// `wrru64` wraps the bits of an u64 to the right (so a wrap of 1 makes the least significant bit the
/// most significant and decreases the significance of all others)
#[inline(always)]
fn wrru64(a: &u64, b: &u64) -> u64 {
    a.rotate_right(*b as u32)
}

/// Float-related functions

/// `stringtof32` tries to convert a string into an f32
#[inline(always)]
fn stringtof32(s: &String) -> Result<f32, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f64tof32` casts an f64 to an f32.
#[inline(always)]
fn f64tof32(f: &f64) -> f32 {
    *f as f32
}

/// `i64tof32` casts an i64 to an f32.
#[inline(always)]
fn i64tof32(i: &i64) -> f32 {
    *i as f32
}

/// `i32tof32` casts an i32 to an f32.
#[inline(always)]
fn i32tof32(i: &i32) -> f32 {
    *i as f32
}

/// `i16tof32` casts an i16 to an f32.
#[inline(always)]
fn i16tof32(i: &i16) -> f32 {
    *i as f32
}

/// `i8tof32` casts an i8 to an f32.
#[inline(always)]
fn i8tof32(i: &i8) -> f32 {
    *i as f32
}

/// `u64tof32` casts an u64 to an f32.
#[inline(always)]
fn u64tof32(i: &u64) -> f32 {
    *i as f32
}

/// `u32tof32` casts an u32 to an f32.
#[inline(always)]
fn u32tof32(i: &u32) -> f32 {
    *i as f32
}

/// `u16tof32` casts an u16 to an f32.
#[inline(always)]
fn u16tof32(i: &u16) -> f32 {
    *i as f32
}

/// `u8tof32` casts an u8 to an f32.
#[inline(always)]
fn u8tof32(i: &u8) -> f32 {
    *i as f32
}

/// `addf32` adds two f32s together, returning an f32
#[inline(always)]
fn addf32(a: &f32, b: &f32) -> f32 {
    a + b
}

/// `subf32` subtracts two f32s, returning an f32
#[inline(always)]
fn subf32(a: &f32, b: &f32) -> f32 {
    a - b
}

/// `mulf32` multiplies two f32s, returning an f32
#[inline(always)]
fn mulf32(a: &f32, b: &f32) -> f32 {
    a * b
}

/// `divf32` divides two f32s, returning an f32
#[inline(always)]
fn divf32(a: &f32, b: &f32) -> f32 {
    a / b
}

/// `sqrtf32` takes the square root of an f32, returning an f32
#[inline(always)]
fn sqrtf32(f: &f32) -> f32 {
    f.sqrt()
}

/// `powf32` safely raises the first f32 to the second f32, returning an f32
#[inline(always)]
fn powf32(a: &f32, b: &f32) -> f32 {
    a.powf(*b)
}

/// `minf32` returns the smaller of the two f32 values
#[inline(always)]
fn minf32(a: &f32, b: &f32) -> f32 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxf32` returns the larger of the two f32 values
#[inline(always)]
fn maxf32(a: &f32, b: &f32) -> f32 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `negf32` negates the `f32` provided
#[inline(always)]
fn negf32(a: &f32) -> f32 {
    -(*a)
}

/// `eqf32` compares two f32s and returns if they are equal
#[inline(always)]
fn eqf32(a: &f32, b: &f32) -> bool {
    *a == *b
}

/// `neqf32` compares two f32s and returns if they are not equal
#[inline(always)]
fn neqf32(a: &f32, b: &f32) -> bool {
    *a != *b
}

/// `ltf32` compares two f32s and returns if the first is smaller than the second
#[inline(always)]
fn ltf32(a: &f32, b: &f32) -> bool {
    *a < *b
}

/// `ltef32` compares two f32s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn ltef32(a: &f32, b: &f32) -> bool {
    *a <= *b
}

/// `gtf32` compares two f32s and returns if the first is larger than the second
#[inline(always)]
fn gtf32(a: &f32, b: &f32) -> bool {
    *a > *b
}

/// `gtef32` compares two f32s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gtef32(a: &f32, b: &f32) -> bool {
    *a >= *b
}

/// `stringtof64` tries to convert a string into an f64
#[inline(always)]
fn stringtof64(s: &String) -> Result<f64, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `f32tof64` casts an f32 to an f64.
#[inline(always)]
fn f32tof64(f: &f32) -> f64 {
    *f as f64
}

/// `i8tof64` casts an i8 to an f64.
#[inline(always)]
fn i8tof64(i: &i8) -> f64 {
    *i as f64
}

/// `i16tof64` casts an i16 to an f64.
#[inline(always)]
fn i16tof64(i: &i16) -> f64 {
    *i as f64
}

/// `i32tof64` casts an i32 to an f64.
#[inline(always)]
fn i32tof64(i: &i32) -> f64 {
    *i as f64
}

/// `i64tof64` casts an i64 to an f64.
#[inline(always)]
fn i64tof64(i: &i64) -> f64 {
    *i as f64
}

/// `u8tof64` casts an u8 to an f64.
#[inline(always)]
fn u8tof64(i: &u8) -> f64 {
    *i as f64
}

/// `u16tof64` casts an u16 to an f64.
#[inline(always)]
fn u16tof64(i: &u16) -> f64 {
    *i as f64
}

/// `u32tof64` casts an u32 to an f64.
#[inline(always)]
fn u32tof64(i: &u32) -> f64 {
    *i as f64
}

/// `u64tof64` casts an u64 to an f64.
#[inline(always)]
fn u64tof64(i: &u64) -> f64 {
    *i as f64
}

/// `addf64` adds two f64s together, returning an f64
#[inline(always)]
fn addf64(a: &f64, b: &f64) -> f64 {
    a + b
}

/// `subf64` subtracts two f64s, returning an f64
#[inline(always)]
fn subf64(a: &f64, b: &f64) -> f64 {
    a - b
}

/// `mulf64` multiplies two f64s, returning an f64
#[inline(always)]
fn mulf64(a: &f64, b: &f64) -> f64 {
    a * b
}

/// `divf64` divides two f64s, returning an f64
#[inline(always)]
fn divf64(a: &f64, b: &f64) -> f64 {
    a / b
}

/// `sqrtf64` takes the square root of an f64, returning an f64
#[inline(always)]
fn sqrtf64(f: &f64) -> f64 {
    f.sqrt()
}

/// `powf64` raises the first f64 to the second f64, returning an f64
#[inline(always)]
fn powf64(a: &f64, b: &f64) -> f64 {
    a.powf(*b)
}

/// `minf64` returns the smaller of the two f64 values
#[inline(always)]
fn minf64(a: &f64, b: &f64) -> f64 {
    if a < b {
        *a
    } else {
        *b
    }
}

/// `maxf64` returns the larger of the two f64 values
#[inline(always)]
fn maxf64(a: &f64, b: &f64) -> f64 {
    if a > b {
        *a
    } else {
        *b
    }
}

/// `negf64` negates the `f64` provided
#[inline(always)]
fn negf64(a: &f64) -> f64 {
    -(*a)
}

/// `eqf64` compares two f64s and returns if they are equal
#[inline(always)]
fn eqf64(a: &f64, b: &f64) -> bool {
    *a == *b
}

/// `neqf64` compares two f64s and returns if they are not equal
#[inline(always)]
fn neqf64(a: &f64, b: &f64) -> bool {
    *a != *b
}

/// `ltf64` compares two f64s and returns if the first is smaller than the second
#[inline(always)]
fn ltf64(a: &f64, b: &f64) -> bool {
    *a < *b
}

/// `ltef64` compares two f64s and returns if the first is smaller than or equal to the second
#[inline(always)]
fn ltef64(a: &f64, b: &f64) -> bool {
    *a <= *b
}

/// `gtf64` compares two f64s and returns if the first is larger than the second
#[inline(always)]
fn gtf64(a: &f64, b: &f64) -> bool {
    *a > *b
}

/// `gtef64` compares two f64s and returns if the first is larger than or equal to the second
#[inline(always)]
fn gtef64(a: &f64, b: &f64) -> bool {
    *a >= *b
}

/// String-related functions

/// `i8tostring` converts an i8 into a simple string representation
#[inline(always)]
fn i8tostring(a: &i8) -> String {
    format!("{}", a)
}

/// `i16tostring` converts an i16 into a simple string representation
#[inline(always)]
fn i16tostring(a: &i16) -> String {
    format!("{}", a)
}

/// `i32tostring` converts an i32 into a simple string representation
#[inline(always)]
fn i32tostring(a: &i32) -> String {
    format!("{}", a)
}

/// `i64tostring` converts an i64 into a simple string representation
#[inline(always)]
fn i64tostring(a: &i64) -> String {
    format!("{}", a)
}

/// `u8tostring` converts an u8 into a simple string representation
#[inline(always)]
fn u8tostring(a: &u8) -> String {
    format!("{}", a)
}

/// `u16tostring` converts an u16 into a simple string representation
#[inline(always)]
fn u16tostring(a: &u16) -> String {
    format!("{}", a)
}

/// `u32tostring` converts an u32 into a simple string representation
#[inline(always)]
fn u32tostring(a: &u32) -> String {
    format!("{}", a)
}

/// `u64tostring` converts an u64 into a simple string representation
#[inline(always)]
fn u64tostring(a: &u64) -> String {
    format!("{}", a)
}

/// `f32tostring` converts an f32 into a simple string representation
#[inline(always)]
fn f32tostring(a: &f32) -> String {
    format!("{}", a)
}

/// `f64tostring` converts an f64 into a simple string representation
#[inline(always)]
fn f64tostring(a: &f64) -> String {
    format!("{}", a)
}

/// `booltostring` converts a bool into a simple string representation
#[inline(always)]
fn booltostring(a: &bool) -> String {
    match a {
        true => "true".to_string(),
        false => "false".to_string(),
    }
}

/// `concatstring` is a simple function that concatenates two strings
#[inline(always)]
fn concatstring(a: &String, b: &String) -> String {
    format!("{}{}", a, b).to_string()
}

/// `repeatstring` creates a new string composed of the original string repeated `n` times
#[inline(always)]
fn repeatstring(a: &String, n: &i64) -> String {
    a.repeat(*n as usize).to_string()
}

/// `splitstring` creates a vector of strings split by the specified separator string
#[inline(always)]
fn splitstring(a: &String, b: &String) -> Vec<String> {
    a.split(b).map(|v| v.to_string()).collect::<Vec<String>>()
}

/// `lenstring` returns the length of the string (the number of characters, not bytes
#[inline(always)]
fn lenstring(a: &String) -> i64 {
    a.chars().collect::<Vec<char>>().len() as i64
}

/// `getstring` returns the character at the specified index (TODO: What is a "character" in Alan?)
#[inline(always)]
fn getstring(a: &String, i: &i64) -> Result<String, AlanError> {
    match a.chars().nth(*i as usize) {
        Some(c) => Ok(String::from(c)),
        None => Err(format!(
            "Index {} is out-of-bounds for a string length of {}",
            i,
            lenstring(a)
        )
        .into()),
    }
}

/// `trimstring` trims the string of whitespace
#[inline(always)]
fn trimstring(a: &String) -> String {
    a.trim().to_string()
}

/// `indexstring` finds the index where the specified substring starts, if possible
#[inline(always)]
fn indexstring(a: &String, b: &String) -> Result<i64, AlanError> {
    match a.find(b) {
        Some(v) => Ok(v as i64),
        None => Err(format!("Could not find {} in {}", b, a).into()),
    }
}

/// `minstring` compares two string and returns the "earlier" string (by byte ordering)
#[inline(always)]
fn minstring(a: &String, b: &String) -> String {
    if *a < *b {
        a.clone()
    } else {
        b.clone()
    }
}

/// `maxstring` compares two string and returns the "later" string (by byte ordering)
#[inline(always)]
fn maxstring(a: &String, b: &String) -> String {
    if *a > *b {
        a.clone()
    } else {
        b.clone()
    }
}

/// `eqstring` compares two string and returns if they are equal
#[inline(always)]
fn eqstring(a: &String, b: &String) -> bool {
    *a == *b
}

/// `neqstring` compares two string and returns if they are not equal
#[inline(always)]
fn neqstring(a: &String, b: &String) -> bool {
    *a != *b
}

/// `ltstring` compares two strings and returns if the first is smaller than the second
#[inline(always)]
fn ltstring(a: &String, b: &String) -> bool {
    *a < *b
}

/// `ltestring` compares two strings and returns if the first is smaller than or equal to the second
#[inline(always)]
fn ltestring(a: &String, b: &String) -> bool {
    *a <= *b
}

/// `gtstring` compares two strings and returns if the first is larger than the second
#[inline(always)]
fn gtstring(a: &String, b: &String) -> bool {
    *a > *b
}

/// `gtestring` compares two strings and returns if the first is larger than or equal to the second
#[inline(always)]
fn gtestring(a: &String, b: &String) -> bool {
    *a >= *b
}

/// `joinstring` joins an array of strings with the separator in-between
#[inline(always)]
fn joinstring(a: &Vec<String>, s: &String) -> String {
    a.join(s)
}

/// `bufferjoinstring` joins a buffer of strings with the separator in-between
#[inline(always)]
fn bufferjoinstring<const S: usize>(a: &[String; S], s: &String) -> String {
    a.join(s)
}

/// Boolean-related functions

/// `i8tobool` converts an integer into a boolean
#[inline(always)]
fn i8tobool(a: &i8) -> bool {
    *a != 0
}

/// `i16tobool` converts an integer into a boolean
#[inline(always)]
fn i16tobool(a: &i16) -> bool {
    *a != 0
}

/// `i32tobool` converts an integer into a boolean
#[inline(always)]
fn i32tobool(a: &i32) -> bool {
    *a != 0
}

/// `i64tobool` converts an integer into a boolean
#[inline(always)]
fn i64tobool(a: &i64) -> bool {
    *a != 0
}

/// `u8tobool` converts an integer into a boolean
#[inline(always)]
fn u8tobool(a: &u8) -> bool {
    *a != 0
}

/// `u16tobool` converts an integer into a boolean
#[inline(always)]
fn u16tobool(a: &u16) -> bool {
    *a != 0
}

/// `u32tobool` converts an integer into a boolean
#[inline(always)]
fn u32tobool(a: &u32) -> bool {
    *a != 0
}

/// `u64tobool` converts an integer into a boolean
#[inline(always)]
fn u64tobool(a: &u64) -> bool {
    *a != 0
}

/// `f32tobool` converts an integer into a boolean
#[inline(always)]
fn f32tobool(a: &f32) -> bool {
    *a != 0.0
}

/// `f64tobool` converts an integer into a boolean
#[inline(always)]
fn f64tobool(a: &f64) -> bool {
    *a != 0.0
}

/// `stringtobool` converts a string into a boolean. "true" is true and everything else is false
#[inline(always)]
fn stringtobool(a: &String) -> bool {
    a.as_str() == "true"
}

/// `andbool` performs a boolean `and`
#[inline(always)]
fn andbool(a: &bool, b: &bool) -> bool {
    *a && *b
}

/// `orbool` performs a boolean `or`
#[inline(always)]
fn orbool(a: &bool, b: &bool) -> bool {
    *a || *b
}

/// `xorbool` performs a boolean `xor`
#[inline(always)]
fn xorbool(a: &bool, b: &bool) -> bool {
    *a ^ *b
}

/// `notbool` performs a boolean `not`
#[inline(always)]
fn notbool(a: &bool) -> bool {
    !*a
}

/// `nandbool` performs a boolean `nand` (considering how computers are built, why is this not a
/// built-in operator?)
#[inline(always)]
fn nandbool(a: &bool, b: &bool) -> bool {
    !(*a && *b)
}

/// `norbool` performs a boolean `nor`
#[inline(always)]
fn norbool(a: &bool, b: &bool) -> bool {
    !(*a || *b)
}

/// `xnorbool` performs a boolean `xnor` (aka `eq`)
#[inline(always)]
fn xnorbool(a: &bool, b: &bool) -> bool {
    *a == *b
}

/// `eqbool` compares two bools and returns if they are equal
#[inline(always)]
fn eqbool(a: &bool, b: &bool) -> bool {
    *a == *b
}

/// `neqbool` compares two bools and returns if they are not equal
#[inline(always)]
fn neqbool(a: &bool, b: &bool) -> bool {
    *a != *b
}

/// `condbool` executes the true function on true, and the false function on false, returning the
/// value returned by either function
#[inline(always)]
fn condbool<T>(c: &bool, mut t: impl FnMut() -> T, mut f: impl FnMut() -> T) -> T {
    if *c {
        t()
    } else {
        f()
    }
}

/// Array-related functions

/// `getarray` returns a value from an array at the location specified
#[inline(always)]
fn getarray<T: Clone>(a: &Vec<T>, i: &i64) -> Option<T> {
    match a.get(*i as usize) {
        Some(v) => Some(v.clone()),
        None => None,
    }
}

/// `lenarray` returns the length of an array (Rust Vector)
#[inline(always)]
fn lenarray<T>(a: &Vec<T>) -> i64 {
    a.len() as i64
}

/// `pusharray` pushes a value onto the array
#[inline(always)]
fn pusharray<T: Clone>(a: &mut Vec<T>, v: &T) {
    a.push(v.clone());
}

/// `poparray` pops a value off of the array into an Option<T>
#[inline(always)]
fn poparray<T>(a: &mut Vec<T>) -> Option<T> {
    a.pop()
}

/// `filled` returns a filled Vec<V> of the provided value for the provided size
#[inline(always)]
fn filled<V: std::clone::Clone>(i: &V, l: &i64) -> Vec<V> {
    vec![i.clone(); *l as usize]
}

/// `vec_len` returns the length of a vector
#[inline(always)]
fn vec_len<A>(v: &Vec<A>) -> i64 {
    v.len() as i64
}

/// `map_onearg` runs the provided single-argument function on each element of the vector,
/// returning a new vector
#[inline(always)]
fn map_onearg<A, B>(v: &Vec<A>, mut m: impl FnMut(&A) -> B) -> Vec<B> {
    v.iter().map(|val| m(val)).collect::<Vec<B>>()
}

/// `map_twoarg` runs the provided two-argument (value, index) function on each element of the
/// vector, returning a new vector
#[inline(always)]
fn map_twoarg<A, B>(v: &Vec<A>, mut m: impl FnMut(&A, i64) -> B) -> Vec<B> {
    v.iter()
        .enumerate()
        .map(|(i, val)| m(val, i as i64))
        .collect::<Vec<B>>()
}

/// `parmap_onearg` runs the provided single-argument function on each element of the vector, with
/// a different subset of the vector run in parallel across all threads.
fn parmap_onearg<
    A: std::marker::Sync + 'static,
    B: std::marker::Send + std::clone::Clone + 'static,
>(
    v: &Vec<A>,
    m: fn(&A) -> B,
) -> Vec<B> {
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
                    handles.push(std::thread::spawn(move || unsafe {
                        let val = (v_ptr as *const A).offset(i as isize).as_ref().unwrap();
                        let mut out = (o_ptr as *mut B).offset(i as isize);
                        out.write(m(val));
                    }));
                }
                for handle in handles {
                    let res = handle.join();
                    match res {
                        Err(e) => panic!("{:?}", e),
                        Ok(_) => {}
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
                    let s: isize = (i * (slice_len as usize)).try_into().unwrap();
                    let e: isize = if i == p.get() - 1 {
                        l.try_into().unwrap()
                    } else {
                        ((i + 1) * (slice_len as usize)).try_into().unwrap()
                    };
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
                        Ok(_) => {}
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

/// `filter_onearg` runs the provided single-argument function on each element of the vector,
/// returning a new vector
#[inline(always)]
fn filter_onearg<A: std::clone::Clone>(v: &Vec<A>, mut f: impl FnMut(&A) -> bool) -> Vec<A> {
    v.iter()
        .filter(|val| f(val))
        .map(|val| val.clone())
        .collect::<Vec<A>>()
}

/// `filter_twoarg` runs the provided function each element of the vector plus its index,
/// returning a new vector
#[inline(always)]
fn filter_twoarg<A: std::clone::Clone>(v: &Vec<A>, mut f: impl FnMut(&A, i64) -> bool) -> Vec<A> {
    v.iter()
        .enumerate()
        .filter(|(i, val)| f(val, *i as i64))
        .map(|(_, val)| val.clone())
        .collect::<Vec<A>>()
}

/// `reduce_sametype` runs the provided function to reduce the vector into a singular value
#[inline(always)]
fn reduce_sametype<A: std::clone::Clone>(v: &Vec<A>, mut f: impl FnMut(&A, &A) -> A) -> Option<A> {
    // The built-in iter `reduce` is awkward for our use case
    if v.len() == 0 {
        None
    } else if v.len() == 1 {
        Some(v[0].clone())
    } else {
        let mut out = v[0].clone();
        for i in 1..v.len() {
            out = f(&out, &v[i]);
        }
        Some(out)
    }
}

/// `reduce_difftype` runs the provided function and initial value to reduce the vector into a
/// singular value. Because an initial value is provided, it always returns at least that value
#[inline(always)]
fn reduce_difftype<A: std::clone::Clone, B: std::clone::Clone>(
    v: &Vec<A>,
    i: &B,
    mut f: impl FnMut(&B, &A) -> B,
) -> B {
    let mut out = i.clone();
    for i in 0..v.len() {
        out = f(&out, &v[i]);
    }
    out
}

/// `concat` returns a new vector combining the two vectors provided
#[inline(always)]
fn concat<A: std::clone::Clone>(a: &Vec<A>, b: &Vec<A>) -> Vec<A> {
    let mut out = Vec::new();
    for i in 0..a.len() {
        out.push(a[i].clone());
    }
    for i in 0..b.len() {
        out.push(b[i].clone());
    }
    out
}

/// `hasarray` returns true if the specified value exists anywhere in the vector
#[inline(always)]
fn hasarray<T: std::cmp::PartialEq>(a: &Vec<T>, v: &T) -> bool {
    a.contains(v)
}

/// `hasfnarray` returns true if the check function returns true for any element of the vector
#[inline(always)]
fn hasfnarray<T>(a: &Vec<T>, mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    return false;
}

/// `findarray` returns the first value from the vector that matches the check function, if any
#[inline(always)]
fn findarray<T: std::clone::Clone>(a: &Vec<T>, mut f: impl FnMut(&T) -> bool) -> Option<T> {
    for v in a {
        if f(v) {
            return Some(v.clone());
        }
    }
    return None;
}

/// `everyarray` returns true if every value in the vector matches the check function
#[inline(always)]
fn everyarray<T>(a: &Vec<T>, mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if !f(v) {
            return false;
        }
    }
    return true;
}

/// `repeatarray` returns a new array with the original array repeated N times
#[inline(always)]
fn repeatarray<T: std::clone::Clone>(a: &Vec<T>, c: &i64) -> Vec<T> {
    let mut out = Vec::new();
    for _ in 0..*c {
        for v in a {
            out.push(v.clone());
        }
    }
    out
}

/// Buffer-related functions

/// `getbuffer` returns the value at the given index presuming it exists
#[inline(always)]
fn getbuffer<T: std::clone::Clone, const S: usize>(b: &[T; S], i: &i64) -> Option<T> {
    b.get(*i as usize).cloned()
}

/// `mapbuffer_onearg` runs the provided single-argument function on each element of the buffer,
/// returning a new buffer
#[inline(always)]
fn mapbuffer_onearg<A, const N: usize, B>(v: &[A; N], mut m: impl FnMut(&A) -> B) -> [B; N] {
    std::array::from_fn(|i| m(&v[i]))
}

/// `mapbuffer_twoarg` runs the provided two-argument (value, index) function on each element of the
/// buffer, returning a new buffer
#[inline(always)]
fn mapbuffer_twoarg<A, const N: usize, B: std::marker::Copy>(
    v: &[A; N],
    mut m: impl FnMut(&A, &i64) -> B,
) -> [B; N] {
    let mut out = [m(&v[0], &0); N];
    for i in 1..N {
        out[i] = m(&v[i], &(i as i64));
    }
    out
}

/// `reducebuffer_sametype` runs the provided function to reduce the buffer into a singular
/// value
#[inline(always)]
fn reducebuffer_sametype<A: std::clone::Clone, const S: usize>(
    b: &[A; S],
    mut f: impl FnMut(&A, &A) -> A,
) -> Option<A> {
    // The built-in iter `reduce` is awkward for our use case
    if b.len() == 0 {
        None
    } else if b.len() == 1 {
        Some(b[0].clone())
    } else {
        let mut out = b[0].clone();
        for i in 1..b.len() {
            out = f(&out, &b[i]);
        }
        Some(out)
    }
}

/// `reducebuffer_difftype` runs the provided function and initial value to reduce the buffer into a
/// singular value. Because an initial value is provided, it always returns at least that value
#[inline(always)]
fn reducebuffer_difftype<A: std::clone::Clone, const S: usize, B: std::clone::Clone>(
    b: &[A; S],
    i: &B,
    mut f: impl FnMut(&B, &A) -> B,
) -> B {
    let mut out = i.clone();
    for i in 0..b.len() {
        out = f(&out, &b[i]);
    }
    out
}

/// `hasbuffer` returns true if the specified value exists anywhere in the array
#[inline(always)]
fn hasbuffer<T: std::cmp::PartialEq, const S: usize>(a: &[T; S], v: &T) -> bool {
    for val in a {
        if val == v {
            return true;
        }
    }
    return false;
}

/// `hasfnbuffer` returns true if the check function returns true for any element of the array
#[inline(always)]
fn hasfnbuffer<T, const S: usize>(a: &[T; S], mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    return false;
}

/// `findbuffer` returns the first value from the buffer that matches the check function, if any
#[inline(always)]
fn findbuffer<T: std::clone::Clone, const S: usize>(
    a: &[T; S],
    mut f: impl FnMut(&T) -> bool,
) -> Option<T> {
    for v in a {
        if f(v) {
            return Some(v.clone());
        }
    }
    return None;
}

/// `everybuffer` returns true if every value in the array matches the check function
#[inline(always)]
fn everybuffer<T, const S: usize>(a: &[T; S], mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if !f(v) {
            return false;
        }
    }
    return true;
}

/// `concatbuffer` mutates the first buffer given with the values of the other two. It depends on
/// the provided buffer to be the right size to fit the data from both of the other buffers.
#[inline(always)]
fn concatbuffer<T: std::clone::Clone, const S: usize, const N: usize, const O: usize>(
    o: &mut [T; O],
    a: &[T; S],
    b: &[T; N],
) {
    for (i, v) in a.iter().chain(b).enumerate() {
        o[i] = v.clone();
    }
}

/// `repeatbuffertoarray` returns a new array with the original buffer repeated N times
#[inline(always)]
fn repeatbuffertoarray<T: std::clone::Clone, const S: usize>(a: &[T; S], c: &i64) -> Vec<T> {
    let mut out = Vec::new();
    for _ in 0..*c {
        for v in a {
            out.push(v.clone());
        }
    }
    out
}

/// Process exit-related bindings

/// `to_exit_code` converts an u8 into an exit code
#[inline(always)]
fn to_exit_code(i: &u8) -> std::process::ExitCode {
    (*i).into()
}

/// `get_or_exit` is basically an alias to `unwrap`, but as a function instead of a method
#[inline(always)]
fn get_or_exit<A: Clone>(a: &Result<A, AlanError>) -> A {
    match a {
        Ok(v) => v.clone(),
        Err(e) => panic!("{:?}", e),
    }
}

/// `get_or_maybe_exit` is basically an alias to `unwrap`, but as a function instead of a method
/// and for `Option` instead of `Result`
#[inline(always)]
fn get_or_maybe_exit<A: Clone>(a: &Option<A>) -> A {
    match a {
        Some(v) => v.clone(),
        None => panic!("Expected value did not exist"), // TODO: Better error message somehow?
    }
}

/// Thread-related functions

/// `wait` is a function that sleeps the current thread for the specified number of milliseconds
#[inline(always)]
fn wait(t: &i64) {
    std::thread::sleep(std::time::Duration::from_millis(*t as u64));
}

/// Time-related functions

/// `now` is a function that returns std::time::Instant for right now
#[inline(always)]
fn now() -> std::time::Instant {
    std::time::Instant::now()
}

/// `elapsed` gets the duration since the instant was created TODO: Borrow these values instead
#[inline(always)]
fn elapsed(i: &std::time::Instant) -> std::time::Duration {
    i.elapsed()
}

/// GPU-related functions and types

struct GPU {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GPU {
    pub fn list() -> Vec<wgpu::Adapter> {
        let instance = wgpu::Instance::default();
        let mut out = Vec::new();
        for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
            if adapter.get_downlevel_capabilities().is_webgpu_compliant() {
                out.push(adapter);
            }
        }
        out
    }
    pub fn init(adapters: Vec<wgpu::Adapter>) -> Vec<GPU> {
        let mut out = Vec::new();
        for adapter in adapters {
            let features = adapter.features();
            let limits = adapter.limits();
            let info = adapter.get_info();
            let device_future = adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some(&format!("{} on {}", info.name, info.backend.to_str())),
                    required_features: features,
                    required_limits: limits,
                },
                None,
            );
            match futures::executor::block_on(device_future) {
                Ok((device, queue)) => {
                    out.push(GPU {
                        adapter,
                        device,
                        queue,
                    });
                }
                Err(_) => { /* Do nothing */ }
            };
        }
        out
    }
}

static GPUS: OnceLock<Vec<GPU>> = OnceLock::new();

fn gpu() -> &'static GPU {
    match GPUS.get_or_init(|| GPU::init(GPU::list())).get(0) {
        Some(g) => g,
        None => panic!(
            "This program requires a GPU but there are no WebGPU-compliant GPUs on this machine"
        ),
    }
}

fn create_buffer_init(usage: &mut wgpu::BufferUsages, vals: &mut Vec<i32>) -> wgpu::Buffer {
    let g = gpu();
    let val_slice = &vals[..];
    let val_ptr = val_slice.as_ptr();
    let val_u8_len = vals.len() * 4;
    let val_u8: &[u8] = unsafe { std::slice::from_raw_parts(val_ptr as *const u8, val_u8_len) };
    wgpu::util::DeviceExt::create_buffer_init(
        &g.device,
        &wgpu::util::BufferInitDescriptor {
            label: None, // TODO: Add a label for easier debugging?
            contents: val_u8,
            usage: *usage,
        },
    )
}

fn create_empty_buffer(usage: &mut wgpu::BufferUsages, size: &mut i64) -> wgpu::Buffer {
    let g = gpu();
    g.device.create_buffer(&wgpu::BufferDescriptor {
        label: None, // TODO: Add a label for easier debugging?
        size: *size as u64,
        usage: *usage,
        mapped_at_creation: false, // TODO: With `create_buffer_init` does this make any sense?
    })
}

// TODO: Either add the ability to bind to const values, or come up with a better solution. For
// now, just hardwire a few buffer usage types in these functions
#[inline(always)]
fn map_read_buffer_type() -> wgpu::BufferUsages {
    wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST
}

#[inline(always)]
fn storage_buffer_type() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
}

struct GPGPU<'a> {
    pub source: String,
    pub entrypoint: String,
    pub buffers: Vec<Vec<&'a wgpu::Buffer>>,
    pub workgroup_sizes: [i64; 3],
}

impl GPGPU<'_> {
    fn new<'a>(
        source: String,
        buffers: Vec<Vec<&'a wgpu::Buffer>>,
        workgroup_sizes: [i64; 3],
    ) -> GPGPU<'a> {
        GPGPU {
            source,
            entrypoint: "main".to_string(),
            buffers,
            workgroup_sizes,
        }
    }
}

#[inline(always)]
fn GPGPU_new<'a>(
    source: &mut String,
    buffers: &'a mut Vec<Vec<&'a wgpu::Buffer>>,
    max_global_id: &mut [i64; 3],
) -> GPGPU<'a> {
    GPGPU::new(source.clone(), buffers.clone(), *max_global_id)
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
    let x = std::cmp::max(y_rem, 1);
    GPGPU::new(source.clone(), vec![vec![buffer]], [x, y, z])
}

fn gpu_run(gg: &mut GPGPU) {
    let g = gpu();
    let module = g.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&gg.source)),
    });
    let compute_pipeline = g
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point: &gg.entrypoint,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });
    let mut bind_groups = Vec::new();
    let mut encoder = g
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
        for i in 0..gg.buffers.len() {
            // The Rust borrow checker is forcing my hand here
            cpass.set_bind_group(i.try_into().unwrap(), &bind_groups[i], &[]);
        }
        cpass.dispatch_workgroups(
            gg.workgroup_sizes[0].try_into().unwrap(),
            gg.workgroup_sizes[1].try_into().unwrap(),
            gg.workgroup_sizes[2].try_into().unwrap(),
        );
    }
    g.queue.submit(Some(encoder.finish()));
}

fn read_buffer(b: &mut wgpu::Buffer) -> Vec<i32> {
    // TODO: Support other value types
    let g = gpu();
    let temp_buffer = create_empty_buffer(
        &mut map_read_buffer_type(),
        &mut b.size().try_into().unwrap(),
    );
    let mut encoder = g
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
        let data_i32: &[i32] =
            unsafe { std::slice::from_raw_parts(data_ptr as *const i32, data_len) };
        let result = data_i32.to_vec();
        drop(data);
        temp_buffer.unmap();
        result
    } else {
        panic!("failed to run compute on gpu!")
    }
}

/// Stdout/stderr-related functions

/// `println` is a simple function that prints basically anything
#[inline(always)]
fn println<A: std::fmt::Display>(a: &A) {
    println!("{}", a);
}

/// `println_result` is a small wrapper function that makes printing Result types easy
#[inline(always)]
fn println_result<A: std::fmt::Display>(a: &Result<A, AlanError>) {
    match a {
        Ok(o) => println!("{}", o),
        Err(e) => println!("{}", e.to_string()),
    };
}

/// `println_maybe` is a small wrapper function that makes printing Option types easy
#[inline(always)]
fn println_maybe<A: std::fmt::Display>(a: &Option<A>) {
    match a {
        Some(o) => println!("{}", o),
        None => println!("void"),
    };
}

/// `println_void` prints "void" if called
#[inline(always)]
fn println_void(void: &()) {
    println!("void");
}

/// `eprintln` is a simple function that prints basically anything
#[inline(always)]
fn eprintln<A: std::fmt::Display>(a: &A) {
    eprintln!("{}", a);
}

/// `eprintln_result` is a small wrapper function that makes printing Result types easy
#[inline(always)]
fn eprintln_result<A: std::fmt::Display>(a: &Result<A, AlanError>) {
    match a {
        Ok(o) => eprintln!("{}", o),
        Err(e) => eprintln!("{:?}", e),
    };
}

/// `eprintln_maybe` is a small wrapper function that makes printing Option types easy
#[inline(always)]
fn eprintln_maybe<A: std::fmt::Display>(a: &Option<A>) {
    match a {
        Some(o) => eprintln!("{}", o),
        None => eprintln!("void"),
    };
}

/// `print_vec` pretty prints a vector assuming the input type can be displayed
#[inline(always)]
fn print_vec<A: std::fmt::Display>(vs: &Vec<A>) {
    println!(
        "[{}]",
        vs.iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<String>>()
            .join(", ")
    );
}

/// `print_vec_result` pretty prints a vector of result values assuming the input can be displayed
#[inline(always)]
fn print_vec_result<A: std::fmt::Display>(vs: &Vec<Result<A, AlanError>>) {
    println!(
        "[{}]",
        vs.iter()
            .map(|v| match v {
                Err(e) => format!("{:?}", e),
                Ok(a) => format!("{}", a),
            })
            .collect::<Vec<String>>()
            .join(", ")
    );
}

/// `print_buffer` pretty prints a buffer assuming the input type can be displayed
#[inline(always)]
fn print_buffer<A: std::fmt::Display, const N: usize>(vs: &[A; N]) {
    println!(
        "[{}]",
        vs.iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<String>>()
            .join(", ")
    );
}

/// `print_duration` pretty-prints a duration value. TODO: Move this into Alan code and out of here
#[inline(always)]
fn print_duration(d: &std::time::Duration) {
    println!("{}.{:0>9}", d.as_secs(), d.subsec_nanos()); // TODO: Figure out which subsec to use
}
