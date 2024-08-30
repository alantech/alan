/// Rust functions that the root scope binds.
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::OnceLock;

use ordered_hash_map::OrderedHashMap;

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

/// `fallible_error` create an Err for the given fallible type
#[inline(always)]
fn fallible_error<T>(m: &String) -> Result<T, AlanError> {
    Err(m.clone().into())
}

/// `bare_error` create a bare Error value, useful when trying to extract the actual error from a
/// Fallible type
#[inline(always)]
fn bare_error(m: &String) -> AlanError {
    AlanError { message: m.clone() }
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

/// `stringtoi16` tries to convert a string into an i16
#[inline(always)]
fn stringtoi16(s: &String) -> Result<i16, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `stringtoi32` tries to convert a string into an i32
#[inline(always)]
fn stringtoi32(s: &String) -> Result<i32, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `stringtoi64` tries to convert a string into an i64
#[inline(always)]
fn stringtoi64(s: &String) -> Result<i64, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
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

/// `stringtou16` tries to convert a string into an u16
#[inline(always)]
fn stringtou16(s: &String) -> Result<u16, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `stringtou32` tries to convert a string into an u32
#[inline(always)]
fn stringtou32(s: &String) -> Result<u32, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// `stringtou64` tries to convert a string into an u64
#[inline(always)]
fn stringtou64(s: &String) -> Result<u64, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
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

/// `stringtof64` tries to convert a string into an f64
#[inline(always)]
fn stringtof64(s: &String) -> Result<f64, AlanError> {
    match s.parse() {
        Ok(v) => Ok(v),
        Err(_) => Err("Not a Number".into()),
    }
}

/// String-related functions

/// `splitstring` creates a vector of strings split by the specified separator string
#[inline(always)]
fn splitstring(a: &String, b: &String) -> Vec<String> {
    a.split(b).map(|v| v.to_string()).collect::<Vec<String>>()
}

/// `lenstring` returns the length of the string (the number of characters, not bytes)
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

/// `indexstring` finds the index where the specified substring starts, if possible
#[inline(always)]
fn indexstring(a: &String, b: &String) -> Result<i64, AlanError> {
    match a.find(b) {
        Some(v) => Ok(v as i64),
        None => Err(format!("Could not find {} in {}", b, a).into()),
    }
}

/// Boolean-related functions

/// `ifbool` executes the true function on true, and the false function on false, returning the
/// value returned by either function
#[inline(always)]
fn ifbool<T>(c: &bool, mut t: impl FnMut() -> T, mut f: impl FnMut() -> T) -> T {
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

/// `reduce_sametype_idx` runs the provided function to reduce the vector into a singular value
#[inline(always)]
fn reduce_sametype_idx<A: std::clone::Clone>(
    v: &Vec<A>,
    mut f: impl FnMut(&A, &A, &i64) -> A,
) -> Option<A> {
    // The built-in iter `reduce` is awkward for our use case
    if v.len() == 0 {
        None
    } else if v.len() == 1 {
        Some(v[0].clone())
    } else {
        let mut out = v[0].clone();
        for i in 1..v.len() {
            out = f(&out, &v[i], &(i as i64));
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

/// `reduce_difftype_idx` runs the provided function and initial value to reduce the vector into a
/// singular value. Because an initial value is provided, it always returns at least that value
#[inline(always)]
fn reduce_difftype_idx<A: std::clone::Clone, B: std::clone::Clone>(
    v: &Vec<A>,
    i: &B,
    mut f: impl FnMut(&B, &A, &i64) -> B,
) -> B {
    let mut out = i.clone();
    for i in 0..v.len() {
        out = f(&out, &v[i], &(i as i64));
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

/// `append` mutates the first vector copying the second vector into it
#[inline(always)]
fn append<A: std::clone::Clone>(a: &mut Vec<A>, b: &Vec<A>) {
    for i in 0..b.len() {
        a.push(b[i].clone());
    }
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

/// `somearray` returns true if any value in the vector matches the check function
#[inline(always)]
fn somearray<T>(a: &Vec<T>, mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    return false;
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

/// `storearray` inserts a new value at the specified index, but fails if the index is greater than
/// the length of the length of the array (so there would be at least one "gap" in the array in
/// that situation)
#[inline(always)]
fn storearray<T: std::clone::Clone>(a: &mut Vec<T>, i: &i64, v: &T) -> Result<(), AlanError> {
    match (*i as usize) > a.len() {
        true => Err(format!(
            "Provided array index {} is greater than the length of the array",
            i
        )
        .into()),
        false => {
            a.insert(*i as usize, v.clone());
            Ok(())
        }
    }
}

/// `deletearray` deletes a value at the specified index, but fails if the index is out-of-bounds.
/// If it succeeds, it returns the value wrapped in a Fallible.
#[inline(always)]
fn deletearray<T: std::clone::Clone>(a: &mut Vec<T>, i: &i64) -> Result<T, AlanError> {
    match (*i as usize) >= a.len() {
        true => Err(format!(
            "Provided array index {} is beyond the bounds of the array",
            i
        )
        .into()),
        false => Ok(a.remove(*i as usize).clone()),
    }
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

/// `storebuffer` stores the provided value in the specified index. If the index is out-of-bounds
/// for the buffer it fails, otherwise it returns the old value.
#[inline(always)]
fn storebuffer<T: std::clone::Clone, const S: usize>(
    a: &mut [T; S],
    i: &i64,
    v: &T,
) -> Result<T, AlanError> {
    match (*i as usize) < a.len() {
        false => Err(format!(
            "The provided index {} is out-of-bounds for the specified buffer",
            i
        )
        .into()),
        true => Ok(std::mem::replace(a.each_mut()[*i as usize], v.clone())),
    }
}

/// Dictionary-related bindings

/// `storedict` stores the provided key-value pair into the dictionary
#[inline(always)]
fn storedict<K: std::clone::Clone + std::hash::Hash + Eq, V: std::clone::Clone>(
    d: &mut OrderedHashMap<K, V>,
    k: &K,
    v: &V,
) {
    d.insert(k.clone(), v.clone());
}

/// `getdict` returns the value for the given key, if it exists
#[inline(always)]
fn getdict<K: std::hash::Hash + Eq, V: std::clone::Clone>(
    d: &OrderedHashMap<K, V>,
    k: &K,
) -> Option<V> {
    d.get(k).cloned()
}

/// `keysdict` returns an array of keys from the dictionary
#[inline(always)]
fn keysdict<K: std::clone::Clone, V>(d: &OrderedHashMap<K, V>) -> Vec<K> {
    d.keys().map(|k| k.clone()).collect::<Vec<K>>()
}

/// `valsdict` returns an array of values from the dictionary
#[inline(always)]
fn valsdict<K, V: std::clone::Clone>(d: &OrderedHashMap<K, V>) -> Vec<V> {
    d.values().map(|v| v.clone()).collect::<Vec<V>>()
}

/// `arraydict` returns an array of key-value tuples representing the dictionary
#[inline(always)]
fn arraydict<K: std::clone::Clone, V: std::clone::Clone>(d: &OrderedHashMap<K, V>) -> Vec<(K, V)> {
    d.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<(K, V)>>()
}

/// `concatdict` returns a new dictionary containing the key-value pairs of the original two
/// dictionaries. Insertion order follows the first dictionary followed by the second dictionary.
/// In cases of key collision, the insertion order of the first dictionary is followed but with the
/// second dictionary's value.
#[inline(always)]
fn concatdict<K: std::clone::Clone + std::hash::Hash + Eq, V: std::clone::Clone>(
    a: &OrderedHashMap<K, V>,
    b: &OrderedHashMap<K, V>,
) -> OrderedHashMap<K, V> {
    let mut out = OrderedHashMap::new();
    for k in a.keys() {
        if b.contains_key(k) {
            out.insert(k.clone(), b.get(k).unwrap().clone());
        } else {
            out.insert(k.clone(), a.get(k).unwrap().clone());
        }
    }
    for k in b.keys() {
        if !a.contains_key(k) {
            out.insert(k.clone(), b.get(k).unwrap().clone());
        }
    }
    out
}

/// Set-related bindings

/// `storeset` stores the provided value into the set
#[inline(always)]
fn storeset<V: std::clone::Clone + std::hash::Hash + Eq>(s: &mut HashSet<V>, v: &V) {
    s.insert(v.clone());
}

/// `arrayset` returns an array of values in the set
#[inline(always)]
fn arrayset<V: std::clone::Clone>(s: &HashSet<V>) -> Vec<V> {
    s.iter().map(|v| v.clone()).collect::<Vec<V>>()
}

/// `unionset` returns a new set that is the union of the original two sets
#[inline(always)]
fn unionset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    // Rust's own `union` method returns a specialized `Union` type to eliminate duplication, which
    // is much more efficient in certain circumstances, but it doesn't appear to implement all of
    // the functions of a `HashSet`, so I am only using it internally to generate a new `HashSet`
    // that I can be sure is usable everywhere.
    a.union(b).map(|v| v.clone()).collect::<HashSet<V>>()
}

/// `intersectset` returns a new set that is the intersection of the original two sets
#[inline(always)]
fn intersectset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    a.intersection(b).map(|v| v.clone()).collect::<HashSet<V>>()
}

/// `differenceset` returns the difference of the original two sets (values in A not in B)
#[inline(always)]
fn differenceset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    a.difference(b).map(|v| v.clone()).collect::<HashSet<V>>()
}

/// `symmetric_differenceset` returns the symmetric difference of the original two sets (values in
/// A not in B *and* values in B not in A)
#[inline(always)]
fn symmetric_differenceset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    a.symmetric_difference(b)
        .map(|v| v.clone())
        .collect::<HashSet<V>>()
}

/// `productset` returns the product of the original two sets (a set of tuples of all combinations
/// of values in each set)
#[inline(always)]
fn productset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<(V, V)> {
    let mut out = HashSet::new();
    for va in a.iter() {
        for vb in b.iter() {
            out.insert((va.clone(), vb.clone()));
        }
    }
    out
}

/// Process exit-related bindings

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

#[derive(Clone)]
struct GBuffer(Rc<wgpu::Buffer>);

impl PartialEq for GBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.0.global_id() == other.0.global_id()
    }
}

impl Eq for GBuffer {}

impl Hash for GBuffer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.global_id().hash(state);
    }
}

impl Deref for GBuffer {
    type Target = Rc<wgpu::Buffer>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn create_buffer_init(usage: &wgpu::BufferUsages, vals: &Vec<i32>) -> GBuffer {
    let g = gpu();
    let val_slice = &vals[..];
    let val_ptr = val_slice.as_ptr();
    let val_u8_len = vals.len() * 4;
    let val_u8: &[u8] = unsafe { std::slice::from_raw_parts(val_ptr as *const u8, val_u8_len) };
    GBuffer(Rc::new(wgpu::util::DeviceExt::create_buffer_init(
        &g.device,
        &wgpu::util::BufferInitDescriptor {
            label: None, // TODO: Add a label for easier debugging?
            contents: val_u8,
            usage: *usage,
        },
    )))
}

fn create_empty_buffer(usage: &wgpu::BufferUsages, size: &i64) -> GBuffer {
    let g = gpu();
    GBuffer(Rc::new(g.device.create_buffer(&wgpu::BufferDescriptor {
        label: None, // TODO: Add a label for easier debugging?
        size: *size as u64,
        usage: *usage,
        mapped_at_creation: false, // TODO: With `create_buffer_init` does this make any sense?
    })))
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

#[inline(always)]
fn bufferlen(gb: &GBuffer) -> i64 {
    (gb.size() / 4) as i64 // TODO: Support more than i32/u32/f32 values
}

#[inline(always)]
fn buffer_id(b: &GBuffer) -> String {
    let mut out = format!("{:?}", b.global_id());
    out.retain(|c| {
        ('A'..='Z').contains(&c) || ('a'..='z').contains(&c) || ('0'..='9').contains(&c)
    });
    out
}

struct GPGPU {
    pub source: String,
    pub entrypoint: String,
    pub buffers: Vec<Vec<GBuffer>>,
    pub workgroup_sizes: [i64; 3],
}

impl GPGPU {
    fn new(source: String, buffers: Vec<Vec<GBuffer>>, workgroup_sizes: [i64; 3]) -> GPGPU {
        GPGPU {
            source,
            entrypoint: "main".to_string(),
            buffers,
            workgroup_sizes,
        }
    }
}

#[inline(always)]
fn GPGPU_new(source: &String, buffers: &Vec<Vec<GBuffer>>, max_global_id: &[i64; 3]) -> GPGPU {
    GPGPU::new(source.clone(), buffers.clone(), *max_global_id)
}

fn GPGPU_new_easy(source: &String, buffer: &GBuffer) -> GPGPU {
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
    GPGPU::new(source.clone(), vec![vec![buffer.clone()]], [x, y, z])
}

fn gpu_run(gg: &GPGPU) {
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

fn read_buffer(b: &GBuffer) -> Vec<i32> {
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
fn println_void(_void: &()) {
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
