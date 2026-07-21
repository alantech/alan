/// Rust functions that the root scope binds.
use std::collections::{HashMap, HashSet};
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, OnceLock};


pub use ordered_hash_map::OrderedHashMap;
pub use uuid::Uuid;
pub use wgpu::BufferUsages;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowAttributes, WindowId};

/// The `AlanError` type is a *cloneable* error that all errors are implemented as within Alan, to
/// simplify error handling. In the future it will have a stack trace based on the Alan source
/// code, but for now only a simple error message is provided.
#[derive(Clone, Debug)]
pub struct AlanError {
    pub message: String,
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

// String-related functions

/// Converts anything that implements ToString into a string. Needed to convert all errors into
/// AlanError since it doesn't seem possible to `impl From<dyn std::error::Error>`
pub fn stringify<T: std::string::ToString>(v: T) -> String {
    v.to_string()
}

/// `splitstring` creates a vector of strings split by the specified separator string
#[inline(always)]
pub fn splitstring(a: &str, b: &str) -> Vec<String> {
    // For now, special handling if the split string is an empty string to make it behave as
    // expected as creating a character array
    if b.is_empty() {
        a.chars().map(|v| v.to_string()).collect::<Vec<String>>()
    } else {
        a.split(b).map(|v| v.to_string()).collect::<Vec<String>>()
    }
}

/// `getstring` returns the character at the specified index (TODO: What is a "character" in Alan?)
#[inline(always)]
pub fn getstring(a: &str, i: &i64) -> Result<String, AlanError> {
    a.chars()
        .nth(*i as usize)
        .map(String::from)
        .ok_or(AlanError {
            message: format!(
                "Index {} is out-of-bounds for a string length of {}",
                i,
                a.chars().collect::<Vec<char>>().len()
            ),
        })
}

/// `indexstring` finds the index where the specified substring starts, if possible
#[inline(always)]
pub fn indexstring(a: &str, b: &str) -> Result<i64, AlanError> {
    a.find(b).map(|v| v as i64).ok_or(AlanError {
        message: format!("Could not find {b} in {a}"),
    })
}

// Array-related functions

/// `getarray` returns a value from an array at the location specified
#[inline(always)]
pub fn getarray<T: Clone>(a: &[T], i: &i64) -> Option<T> {
    a.get(*i as usize).cloned()
}

/// `filled` returns a filled Vec<V> of the provided value for the provided size
#[inline(always)]
pub fn filled<V: std::clone::Clone>(i: &V, l: &i64) -> Vec<V> {
    vec![i.clone(); *l as usize]
}

/// `map_onearg` runs the provided single-argument function on each element of the vector,
/// returning a new vector
#[inline(always)]
pub fn map_onearg<A, B>(v: &[A], m: impl FnMut(&A) -> B) -> Vec<B> {
    v.iter().map(m).collect::<Vec<B>>()
}

/// `map_twoarg` runs the provided two-argument (value, index) function on each element of the
/// vector, returning a new vector
#[inline(always)]
pub fn map_twoarg<A, B>(v: &[A], mut m: impl FnMut(&A, i64) -> B) -> Vec<B> {
    v.iter()
        .enumerate()
        .map(|(i, val)| m(val, i as i64))
        .collect::<Vec<B>>()
}

/// `parmap_onearg` runs the provided single-argument function on each element of the vector, with
/// a different subset of the vector run in parallel across all threads.
pub fn parmap_onearg<
    A: std::marker::Sync + 'static,
    B: std::marker::Send + std::clone::Clone + 'static,
>(
    v: &[A],
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
                        let val = (v_ptr as *const A).add(i).as_ref().unwrap();
                        let out = (o_ptr as *mut B).add(i);
                        out.write(m(val));
                    }));
                }
                for handle in handles {
                    let res = handle.join();
                    if let Err(e) = res {
                        panic!("{e:?}")
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
                                let out = o_ptr.offset(i);
                                out.write(m(val));
                            }
                        }
                    }));
                }
                for handle in handles {
                    let res = handle.join();
                    if let Err(e) = res {
                        panic!("{e:?}")
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
pub fn filter_onearg<A: std::clone::Clone>(v: &[A], mut f: impl FnMut(&A) -> bool) -> Vec<A> {
    v.iter().filter(|val| f(val)).cloned().collect::<Vec<A>>()
}

/// `filter_twoarg` runs the provided function each element of the vector plus its index,
/// returning a new vector
#[inline(always)]
pub fn filter_twoarg<A: std::clone::Clone>(v: &[A], mut f: impl FnMut(&A, i64) -> bool) -> Vec<A> {
    v.iter()
        .enumerate()
        .filter(|(i, val)| f(val, *i as i64))
        .map(|(_, val)| val.clone())
        .collect::<Vec<A>>()
}

/// `reduce_sametype` runs the provided function to reduce the vector into a singular value
#[inline(always)]
pub fn reduce_sametype<A: std::clone::Clone>(v: &[A], mut f: impl FnMut(&A, &A) -> A) -> Option<A> {
    // The built-in iter `reduce` is awkward for our use case
    if v.is_empty() {
        None
    } else if v.len() == 1 {
        Some(v[0].clone())
    } else {
        let mut out = v[0].clone();
        for val in v.iter().skip(1) {
            out = f(&out, val);
        }
        Some(out)
    }
}

/// `reduce_sametype_idx` runs the provided function to reduce the vector into a singular value
#[inline(always)]
pub fn reduce_sametype_idx<A: std::clone::Clone>(
    v: &[A],
    mut f: impl FnMut(&A, &A, &i64) -> A,
) -> Option<A> {
    // The built-in iter `reduce` is awkward for our use case
    if v.is_empty() {
        None
    } else if v.len() == 1 {
        Some(v[0].clone())
    } else {
        let mut out = v[0].clone();
        for (i, val) in v.iter().enumerate().skip(1) {
            out = f(&out, val, &(i as i64));
        }
        Some(out)
    }
}

/// `reduce_difftype` runs the provided function and initial value to reduce the vector into a
/// singular value. Because an initial value is provided, it always returns at least that value
#[inline(always)]
pub fn reduce_difftype<A: std::clone::Clone, B: std::clone::Clone>(
    v: &[A],
    i: &B,
    mut f: impl FnMut(&B, &A) -> B,
) -> B {
    let mut out = i.clone();
    for val in v {
        out = f(&out, val);
    }
    out
}

/// `reduce_difftype_idx` runs the provided function and initial value to reduce the vector into a
/// singular value. Because an initial value is provided, it always returns at least that value
#[inline(always)]
pub fn reduce_difftype_idx<A: std::clone::Clone, B: std::clone::Clone>(
    v: &[A],
    i: &B,
    mut f: impl FnMut(&B, &A, &i64) -> B,
) -> B {
    let mut out = i.clone();
    for (i, val) in v.iter().enumerate() {
        out = f(&out, val, &(i as i64));
    }
    out
}

/// `concat` returns a new vector combining the two vectors provided
#[inline(always)]
pub fn concat<A: std::clone::Clone>(a: &[A], b: &[A]) -> Vec<A> {
    let mut out = Vec::new();
    for v in a {
        out.push(v.clone());
    }
    for v in b {
        out.push(v.clone());
    }
    out
}

/// `append` mutates the first vector copying the second vector into it
#[inline(always)]
pub fn append<A: std::clone::Clone>(a: &mut Vec<A>, b: &[A]) {
    for v in b {
        a.push(v.clone());
    }
}

/// `hasfnarray` returns true if the check function returns true for any element of the vector
#[inline(always)]
pub fn hasfnarray<T>(a: &[T], mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    false
}

/// `findarray` returns the first value from the vector that matches the check function, if any
#[inline(always)]
pub fn findarray<T: std::clone::Clone>(a: &[T], mut f: impl FnMut(&T) -> bool) -> Option<T> {
    for v in a {
        if f(v) {
            return Some(v.clone());
        }
    }
    None
}

/// `everyarray` returns true if every value in the vector matches the check function
#[inline(always)]
pub fn everyarray<T>(a: &[T], mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if !f(v) {
            return false;
        }
    }
    true
}

/// `somearray` returns true if any value in the vector matches the check function
#[inline(always)]
pub fn somearray<T>(a: &[T], mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    false
}

/// `repeatarray` returns a new array with the original array repeated N times
#[inline(always)]
pub fn repeatarray<T: std::clone::Clone>(a: &[T], c: &i64) -> Vec<T> {
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
pub fn storearray<T: std::clone::Clone>(a: &mut Vec<T>, i: &i64, v: &T) -> Result<(), AlanError> {
    let idx = *i as usize;
    match idx > a.len() {
        true => {
            Err(format!("Provided array index {i} is greater than the length of the array").into())
        }
        false => {
            if idx < a.len() {
                a.insert(idx, v.clone());
            } else {
                a.push(v.clone());
            }
            Ok(())
        }
    }
}

/// `deletearray` deletes a value at the specified index, but fails if the index is out-of-bounds.
/// If it succeeds, it returns the value wrapped in a Fallible.
#[inline(always)]
pub fn deletearray<T: std::clone::Clone>(a: &mut Vec<T>, i: &i64) -> Result<T, AlanError> {
    match (*i as usize) >= a.len() {
        true => Err(format!("Provided array index {i} is beyond the bounds of the array").into()),
        false => Ok(a.remove(*i as usize).clone()),
    }
}

/// `swaparray` swaps the values at the specified indicies (or fails if either index is out of
/// bounds). It returns a Fallible void value.
#[inline(always)]
pub fn swaparray<T>(a: &mut [T], i: &i64, j: &i64) -> Result<(), AlanError> {
    if *i < 0 {
        return Err(format!("Provided array index {i} is beyond the bounds of the array").into());
    }
    if *j < 0 {
        return Err(format!("Provided array index {j} is beyond the bounds of the array").into());
    }
    let i = *i as usize;
    let j = *j as usize;
    if i >= a.len() {
        return Err(format!("Provided array index {i} is beyond the bounds of the array").into());
    }
    if j >= a.len() {
        return Err(format!("Provided array index {j} is beyond the bounds of the array").into());
    }
    if i == j {
        return Ok(());
    }
    if i < j {
        let (i_section, j_section) = a.split_at_mut(j);
        std::mem::swap(&mut i_section[i], &mut j_section[0]);
    } else {
        let (j_section, i_section) = a.split_at_mut(j);
        std::mem::swap(&mut j_section[j], &mut i_section[0]);
    }
    Ok(())
}

/// `sortarray` is a thin wrapper around `sort_by` allowing for the sort decision to be done by
/// numeric operation rather than the `Ordering` enum, which is not exposed
#[inline(always)]
pub fn sortarray<T>(a: &mut [T], mut sorter: impl FnMut(&T, &T) -> i8) {
    a.sort_by(|a, b| sorter(a, b).cmp(&0));
}

// Buffer-related functions

/// `getbuffer` returns the value at the given index presuming it exists
#[inline(always)]
pub fn getbuffer<T: std::clone::Clone, const S: usize>(b: &[T; S], i: &i64) -> Option<T> {
    b.get(*i as usize).cloned()
}

/// `mapbuffer_onearg` runs the provided single-argument function on each element of the buffer,
/// returning a new buffer
#[inline(always)]
pub fn mapbuffer_onearg<A, const N: usize, B>(v: &[A; N], mut m: impl FnMut(&A) -> B) -> [B; N] {
    std::array::from_fn(|i| m(&v[i]))
}

/// `mapbuffer_twoarg` runs the provided two-argument (value, index) function on each element of the
/// buffer, returning a new buffer
#[inline(always)]
pub fn mapbuffer_twoarg<A, const N: usize, B: std::marker::Copy>(
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
pub fn reducebuffer_sametype<A: std::clone::Clone, const S: usize>(
    b: &[A; S],
    mut f: impl FnMut(&A, &A) -> A,
) -> Option<A> {
    // The built-in iter `reduce` is awkward for our use case
    if b.is_empty() {
        None
    } else if b.len() == 1 {
        Some(b[0].clone())
    } else {
        let mut out = b[0].clone();
        for v in b.iter().skip(1) {
            out = f(&out, v);
        }
        Some(out)
    }
}

/// `reducebuffer_difftype` runs the provided function and initial value to reduce the buffer into a
/// singular value. Because an initial value is provided, it always returns at least that value
#[inline(always)]
pub fn reducebuffer_difftype<A: std::clone::Clone, const S: usize, B: std::clone::Clone>(
    b: &[A; S],
    i: &B,
    mut f: impl FnMut(&B, &A) -> B,
) -> B {
    let mut out = i.clone();
    for v in b {
        out = f(&out, v);
    }
    out
}

/// `hasbuffer` returns true if the specified value exists anywhere in the array
#[inline(always)]
pub fn hasbuffer<T: std::cmp::PartialEq, const S: usize>(a: &[T; S], v: &T) -> bool {
    for val in a {
        if val == v {
            return true;
        }
    }
    false
}

/// `hasfnbuffer` returns true if the check function returns true for any element of the array
#[inline(always)]
pub fn hasfnbuffer<T, const S: usize>(a: &[T; S], mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    false
}

/// `findbuffer` returns the first value from the buffer that matches the check function, if any
#[inline(always)]
pub fn findbuffer<T: std::clone::Clone, const S: usize>(
    a: &[T; S],
    mut f: impl FnMut(&T) -> bool,
) -> Option<T> {
    for v in a {
        if f(v) {
            return Some(v.clone());
        }
    }
    None
}

/// `everybuffer` returns true if every value in the array matches the check function
#[inline(always)]
pub fn everybuffer<T, const S: usize>(a: &[T; S], mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if !f(v) {
            return false;
        }
    }
    true
}

/// `concatbuffer` mutates the first buffer given with the values of the other two. It depends on
/// the provided buffer to be the right size to fit the data from both of the other buffers.
#[inline(always)]
pub fn concatbuffer<T: std::clone::Clone, const S: usize, const N: usize, const O: usize>(
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
pub fn repeatbuffertoarray<T: std::clone::Clone, const S: usize>(a: &[T; S], c: &i64) -> Vec<T> {
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
pub fn storebuffer<T: std::clone::Clone, const S: usize>(
    a: &mut [T; S],
    i: &i64,
    v: &T,
) -> Result<T, AlanError> {
    match (*i as usize) < a.len() {
        false => {
            Err(format!("The provided index {i} is out-of-bounds for the specified buffer").into())
        }
        true => Ok(std::mem::replace(a.each_mut()[*i as usize], v.clone())),
    }
}

/// `swapbuffer` swaps the values at the specified indicies (or fails if either index is out of
/// bounds). It returns a Fallible void value.
#[inline(always)]
pub fn swapbuffer<T, const S: usize>(a: &mut [T; S], i: &i64, j: &i64) -> Result<(), AlanError> {
    if *i < 0 {
        return Err(format!("Provided buffer index {i} is beyond the bounds of the buffer").into());
    }
    if *j < 0 {
        return Err(format!("Provided buffer index {j} is beyond the bounds of the buffer").into());
    }
    let i = *i as usize;
    let j = *j as usize;
    if i >= a.len() {
        return Err(format!("Provided buffer index {i} is beyond the bounds of the buffer").into());
    }
    if j >= a.len() {
        return Err(format!("Provided buffer index {j} is beyond the bounds of the buffer").into());
    }
    if i == j {
        return Ok(());
    }
    if i < j {
        let (i_section, j_section) = a.split_at_mut(j);
        std::mem::swap(&mut i_section[i], &mut j_section[0]);
    } else {
        let (j_section, i_section) = a.split_at_mut(i);
        std::mem::swap(&mut j_section[j], &mut i_section[0]);
    }
    Ok(())
}

/// `sortbuffer` is a thin wrapper around `sort_by` allowing for the sort decision to be done by
/// numeric operation rather than the `Ordering` enum, which is not exposed
#[inline(always)]
pub fn sortbuffer<T, const S: usize>(a: &mut [T; S], mut sorter: impl FnMut(&T, &T) -> i8) {
    a.sort_by(|a, b| sorter(a, b).cmp(&0));
}

// Dictionary-related bindings

/// `getdict` returns the value for the given key, if it exists
#[inline(always)]
pub fn getdict<K: std::hash::Hash + Eq, V: std::clone::Clone>(
    d: &OrderedHashMap<K, V>,
    k: &K,
) -> Option<V> {
    d.get(k).cloned()
}

/// `keysdict` returns an array of keys from the dictionary
#[inline(always)]
pub fn keysdict<K: std::clone::Clone, V>(d: &OrderedHashMap<K, V>) -> Vec<K> {
    d.keys().cloned().collect::<Vec<K>>()
}

/// `valsdict` returns an array of values from the dictionary
#[inline(always)]
pub fn valsdict<K, V: std::clone::Clone>(d: &OrderedHashMap<K, V>) -> Vec<V> {
    d.values().cloned().collect::<Vec<V>>()
}

/// `arraydict` returns an array of key-value tuples representing the dictionary
#[inline(always)]
pub fn arraydict<K: std::clone::Clone, V: std::clone::Clone>(
    d: &OrderedHashMap<K, V>,
) -> Vec<(K, V)> {
    d.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<(K, V)>>()
}

/// `concatdict` returns a new dictionary containing the key-value pairs of the original two
/// dictionaries. Insertion order follows the first dictionary followed by the second dictionary.
/// In cases of key collision, the insertion order of the first dictionary is followed but with the
/// second dictionary's value.
#[inline(always)]
pub fn concatdict<K: std::clone::Clone + std::hash::Hash + Eq, V: std::clone::Clone>(
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

// Set-related bindings

/// `arrayset` returns an array of values in the set
#[inline(always)]
pub fn arrayset<V: std::clone::Clone>(s: &HashSet<V>) -> Vec<V> {
    s.iter().cloned().collect::<Vec<V>>()
}

/// `unionset` returns a new set that is the union of the original two sets
#[inline(always)]
pub fn unionset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    // Rust's own `union` method returns a specialized `Union` type to eliminate duplication, which
    // is much more efficient in certain circumstances, but it doesn't appear to implement all of
    // the functions of a `HashSet`, so I am only using it internally to generate a new `HashSet`
    // that I can be sure is usable everywhere.
    a.union(b).cloned().collect::<HashSet<V>>()
}

/// `intersectset` returns a new set that is the intersection of the original two sets
#[inline(always)]
pub fn intersectset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    a.intersection(b).cloned().collect::<HashSet<V>>()
}

/// `differenceset` returns the difference of the original two sets (values in A not in B)
#[inline(always)]
pub fn differenceset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    a.difference(b).cloned().collect::<HashSet<V>>()
}

/// `symmetric_differenceset` returns the symmetric difference of the original two sets (values in
/// A not in B *and* values in B not in A)
#[inline(always)]
pub fn symmetric_differenceset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    a.symmetric_difference(b).cloned().collect::<HashSet<V>>()
}

/// `productset` returns the product of the original two sets (a set of tuples of all combinations
/// of values in each set)
#[inline(always)]
pub fn productset<V: std::clone::Clone + std::hash::Hash + Eq>(
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

// Vector-related functions

pub fn cross_f32(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn cross_f64(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// GPU-related functions and types

static INSTANCE: OnceLock<wgpu::Instance> = OnceLock::new();

/// The shared `wgpu::Instance` for the whole program. Both the GPGPU device(s)
/// and any window surface must come from the same instance for them to be
/// compatible with each other.
pub fn instance() -> &'static wgpu::Instance {
    INSTANCE.get_or_init(wgpu::Instance::default)
}

pub struct GPU {
    pub adapter: wgpu::Adapter,
    /// Lazily-initialized (device, queue) pair. Created on first access.
    device: OnceLock<(wgpu::Device, wgpu::Queue)>,
}
impl GPU {
    pub fn list() -> Vec<wgpu::Adapter> {
        let instance = instance();
        let mut out = Vec::new();
        let adapters_future = instance.enumerate_adapters(wgpu::Backends::all());
        let adapters = futures::executor::block_on(adapters_future);
        for adapter in adapters {
            if adapter.get_downlevel_capabilities().is_webgpu_compliant() {
                out.push(adapter);
            }
        }
        out
    }

    pub fn init(adapters: Vec<wgpu::Adapter>) -> Vec<GPU> {
        adapters
            .into_iter()
            .map(|adapter| GPU {
                adapter,
                device: OnceLock::new(),
            })
            .collect()
    }
    /// Move an adapter capable of presenting to the given surface to the front.
    pub fn prefer_surface(adapters: &mut Vec<wgpu::Adapter>, surface: &wgpu::Surface<'_>) {
        if let Some(pos) = adapters
            .iter()
            .position(|a| a.is_surface_supported(surface))
        {
            let preferred = adapters.remove(pos);
            adapters.insert(0, preferred);
        }
    }
    /// Lazily create the device and queue for this adapter on first access.
    fn get_device(&self) -> &wgpu::Device {
        &self.device.get_or_init(|| {
            let info = self.adapter.get_info();
            futures::executor::block_on(
                self.adapter
                    .request_device(&wgpu::DeviceDescriptor {
                        label: Some(&format!("{} on {}", info.name, info.backend.to_str())),
                        required_features: wgpu::Features::empty(),
                        required_limits: self.adapter.limits(),
                        experimental_features: wgpu::ExperimentalFeatures::disabled(),
                        memory_hints: wgpu::MemoryHints::Performance,
                        trace: wgpu::Trace::Off,
                    }),
            )
            .expect("Failed to create GPU device")
        })
        .0
    }
    /// Lazily create the device and queue for this adapter on first access.
    fn get_queue(&self) -> &wgpu::Queue {
        self.get_device();
        &self.device.get().unwrap().1
    }
}

static GPUS: OnceLock<Vec<GPU>> = OnceLock::new();
static SUBGROUP_MAX_SIZE: OnceLock<u32> = OnceLock::new();

fn gpus_init() -> &'static Vec<GPU> {
    GPUS.get_or_init(|| {
        // Create a temporary test window to find a presentation-capable
        // adapter. This guarantees that `gpu()` returns a device that can
        // drive both GPGPU compute *and* window surfaces, so all `GBuffer`s
        // allocated via the standard path end up on the "right" device.
        // May fail on some platforms (e.g., macOS from non-main thread),
        // in which case we fall back to the default adapter list.
        struct TestAdapterFinder {
            found_surface: Option<wgpu::Surface<'static>>,
        }
        impl ApplicationHandler<()> for TestAdapterFinder {
            fn resumed(&mut self, event_loop: &ActiveEventLoop) {
                match event_loop.create_window(Window::default_attributes()) {
                    Ok(window) => {
                        self.found_surface = match instance().create_surface(window) {
                            Ok(s) => Some(s),
                            Err(_) => None,
                        };
                    }
                    Err(_) => {}
                }
                event_loop.exit();
            }
            fn window_event(
                &mut self,
                _event_loop: &ActiveEventLoop,
                _id: WindowId,
                _event: WindowEvent,
            ) {
            }
        }
        let adapters = std::panic::catch_unwind(|| {
            let event_loop: EventLoop<()> = match EventLoop::new() {
                Ok(el) => el,
                Err(_) => return None,
            };
            let mut finder = TestAdapterFinder { found_surface: None };
            if event_loop.run_app(&mut finder).is_err() {
                return None;
            }
            let test_surface = finder.found_surface?;
            let mut adapters = GPU::list();
            GPU::prefer_surface(&mut adapters, &test_surface);
            Some(adapters)
        })
        .unwrap_or_else(|_| None)
        .unwrap_or_else(GPU::list);
        GPU::init(adapters)
    })
}

fn gpu() -> &'static GPU {
    match gpus_init().first() {
        Some(g) => g,
        None => panic!(
            "This program requires a GPU but there are no WebGPU-compliant GPUs on this machine"
        ),
    }
}


fn subgroup_max_size() -> u32 {
    *SUBGROUP_MAX_SIZE.get_or_init(|| {
        let g = gpu();
        g.adapter.get_info().subgroup_max_size
    })
}

pub fn optimal_local_group(global: [i64; 3]) -> [i64; 3] {
    let total_global = (global[0] as u64) * (global[1] as u64) * (global[2] as u64);
    if total_global == 0 {
        return [1, 1, 1];
    }
    let g = gpu();
    let sub_max = subgroup_max_size();
    let max_invocations = g.adapter.limits().max_compute_invocations_per_workgroup as u64;
    // Target totalInvocationsPerWorkgroup so that totalWorkgroups ~ subgroup_max_size
    let mut target = (total_global as f64 / sub_max as f64).ceil() as u64;
    // Clamp to [8, maxInvocations]
    target = target.max(8).min(max_invocations);
    // Snap to nearest multiple of 8 (hardware alignment)
    target = target.div_ceil(8) * 8;
    // Shape: prefer S*S*1, then D*8*1
    let sqrt = (target as f64).sqrt() as u64;
    if sqrt >= 8 && sqrt * sqrt == target {
        return [sqrt as i64, sqrt as i64, 1];
    }
    if target.is_multiple_of(8) {
        return [(target / 8) as i64, 8, 1];
    }
    [target as i64, 1, 1]
}

#[derive(Clone)]
pub struct GBuffer {
    buffer: Rc<wgpu::Buffer>,
    id: String,
    element_size: i8,
}

impl PartialEq for GBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for GBuffer {}

impl Hash for GBuffer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.buffer.hash(state);
    }
}

impl Deref for GBuffer {
    type Target = Rc<wgpu::Buffer>;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl DerefMut for GBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

pub fn create_buffer_init<T>(
    usage: &wgpu::BufferUsages,
    vals: &[T],
    element_size: &i8,
) -> Result<GBuffer, AlanError> {
    let g = gpu();
    let val_ptr = vals.as_ptr();
    let val_u8_len = vals.len() * (*element_size as usize);
    let limits = g.get_device().limits();
    if limits.max_buffer_size < val_u8_len as u64 {
        return Err(AlanError { message: format!("Cannot load the array into the GPU, as it is too large. GBuffer on your GPU only supports up to {} bytes per buffer", limits.max_buffer_size), });
    }

    // Create an empty buffer first
    let buf = GBuffer {
        buffer: Rc::new(g.get_device().create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: val_u8_len as u64,
            usage: *usage,
            mapped_at_creation: false,
        })),
        id: format!("buffer_{}", format!("{}", Uuid::new_v4()).replace("-", "_")),
        element_size: *element_size,
    };

    // Create a staging buffer with the data
    let val_u8: &[u8] = unsafe { std::slice::from_raw_parts(val_ptr as *const u8, val_u8_len) };
    let staging_buffer = g.get_device().create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: val_u8_len as u64,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
        mapped_at_creation: true,
    });

    // Write data to staging buffer
    {
        let view = staging_buffer.slice(..);
        match view.get_mapped_range_mut() {
            Ok(mut range) => range.copy_from_slice(val_u8),
            Err(e) => {
                return Err(AlanError {
                    message: format!("Somehow got an invalid range on full slice {e:?}"),
                })
            }
        };
    }
    staging_buffer.unmap();

    // Copy from staging buffer to target buffer
    let mut encoder = g.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_buffer_to_buffer(&staging_buffer, 0, &buf, 0, val_u8_len as u64);

    // Submit and wait for the copy to complete
    let submission_index = g.get_queue().submit(Some(encoder.finish()));
    match g.get_device().poll(wgpu::PollType::Wait {
        submission_index: Some(submission_index),
        timeout: None,
    }) {
        Ok(_) => Ok(()),
        Err(e) => Err(AlanError {
            message: format!("Failed to create buffer {e:?}"),
        }),
    }?;

    Ok(buf)
}

pub fn create_empty_buffer(
    usage: &wgpu::BufferUsages,
    size: &i64,
    element_size: &i8,
) -> Result<GBuffer, AlanError> {
    let g = gpu();
    let limits = g.get_device().limits();
    if limits.max_buffer_size < *size as u64 {
        return Err(AlanError { message: format!("Cannot load the array into the GPU, as it is too large. GBuffer on your GPU only supports up to {} bytes per buffer", limits.max_buffer_size), });
    }
    Ok(GBuffer {
        buffer: Rc::new(g.get_device().create_buffer(&wgpu::BufferDescriptor {
            label: None, // TODO: Add a label for easier debugging?
            size: (*size as u64) * (*element_size as u64),
            usage: *usage,
            mapped_at_creation: false, // TODO: With `create_buffer_init` does this make any sense?
        })),
        id: format!("buffer_{}", format!("{}", Uuid::new_v4()).replace("-", "_")),
        element_size: *element_size,
    })
}

// TODO: Either add the ability to bind to const values, or come up with a better solution. For
// now, just hardwire a few buffer usage types in these functions
#[inline(always)]
pub fn map_read_buffer_type() -> wgpu::BufferUsages {
    wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST
}

#[inline(always)]
pub fn map_write_buffer_type() -> wgpu::BufferUsages {
    wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC
}

#[inline(always)]
pub fn storage_buffer_type() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
}

#[inline(always)]
pub fn bufferlen(gb: &GBuffer) -> i64 {
    (gb.size() as i64) / (gb.element_size as i64)
}

#[inline(always)]
pub fn buffer_id(b: &GBuffer) -> String {
    b.id.clone()
}

pub struct GPGPU {
    pub source: String,
    pub entrypoint: String,
    pub buffers: Vec<Vec<GBuffer>>,
    pub workgroup_sizes: [i64; 3],
    pub local_workgroup_size: [i64; 3],
    pub module: Option<wgpu::ShaderModule>,
    pub compute_pipeline: Option<wgpu::ComputePipeline>,
}

impl GPGPU {
    pub fn new(
        source: String,
        buffers: Vec<Vec<GBuffer>>,
        workgroup_sizes: [i64; 3],
        local_workgroup_size: [i64; 3],
    ) -> GPGPU {
        GPGPU {
            source,
            entrypoint: "main".to_string(),
            buffers,
            workgroup_sizes,
            local_workgroup_size,
            module: None,
            compute_pipeline: None,
        }
    }
}

pub fn gpu_run(gg: &mut GPGPU) {
    let g = gpu();
    if gg.module.is_none() {
        gg.module = Some(g.get_device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&gg.source)),
        }));
    }
    let module = gg.module.as_ref().unwrap();
    if gg.compute_pipeline.is_none() {
        gg.compute_pipeline = Some(g.get_device().create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module,
                entry_point: Some(&gg.entrypoint),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            },
        ));
    }
    let compute_pipeline = gg.compute_pipeline.as_ref().unwrap();
    let mut bind_groups = Vec::new();
    let mut encoder = g.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(compute_pipeline);
        for i in 0..gg.buffers.len() {
            let bind_group_layout = compute_pipeline.get_bind_group_layout(i.try_into().unwrap());
            let bind_group_buffers = &gg.buffers[i];
            let mut bind_group_entries = Vec::new();
            #[allow(clippy::needless_range_loop)] // Not needless clippy
            for j in 0..bind_group_buffers.len() {
                bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: j.try_into().unwrap(),
                    resource: bind_group_buffers[j].as_entire_binding(),
                });
            }
            let bind_group = g.get_device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &bind_group_entries[..],
            });
            bind_groups.push(bind_group);
        }
        #[allow(clippy::needless_range_loop)] // Not needless clippy
        for i in 0..gg.buffers.len() {
            // The Rust borrow checker is forcing my hand here
            cpass.set_bind_group(i.try_into().unwrap(), &bind_groups[i], &[]);
        }
        let lx = gg.local_workgroup_size[0];
        let ly = gg.local_workgroup_size[1];
        let x = if gg.workgroup_sizes[0] > 0 {
            ((gg.workgroup_sizes[0] + lx - 1) / lx).try_into().unwrap()
        } else {
            gg.workgroup_sizes[0].try_into().unwrap()
        };
        let y = if gg.workgroup_sizes[1] > 0 {
            ((gg.workgroup_sizes[1] + ly - 1) / ly).try_into().unwrap()
        } else {
            gg.workgroup_sizes[1].try_into().unwrap()
        };
        let z = gg.workgroup_sizes[2].try_into().unwrap();
        cpass.dispatch_workgroups(x, y, z);
    }

    g.get_queue().submit(Some(encoder.finish()));
}

pub fn gpu_run_list(ggs: &mut Vec<GPGPU>) {
    let g = gpu();
    let mut encoder = g.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    for gg in ggs {
        if gg.module.is_none() {
            gg.module = Some(g.get_device().create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&gg.source)),
            }));
        }
        let module = gg.module.as_ref().unwrap();
        if gg.compute_pipeline.is_none() {
            gg.compute_pipeline = Some(g.get_device().create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: None,
                    module,
                    entry_point: Some(&gg.entrypoint),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                },
            ));
        }
        let compute_pipeline = gg.compute_pipeline.as_ref().unwrap();
        let mut bind_groups = Vec::new();
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(compute_pipeline);
            for i in 0..gg.buffers.len() {
                let bind_group_layout =
                    compute_pipeline.get_bind_group_layout(i.try_into().unwrap());
                let bind_group_buffers = &gg.buffers[i];
                let mut bind_group_entries = Vec::new();
                #[allow(clippy::needless_range_loop)] // Not needless clippy
                for j in 0..bind_group_buffers.len() {
                    bind_group_entries.push(wgpu::BindGroupEntry {
                        binding: j.try_into().unwrap(),
                        resource: bind_group_buffers[j].as_entire_binding(),
                    });
                }
                let bind_group = g.get_device().create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &bind_group_entries[..],
                });
                bind_groups.push(bind_group);
            }
            #[allow(clippy::needless_range_loop)] // Not needless clippy
            for i in 0..gg.buffers.len() {
                // The Rust borrow checker is forcing my hand here
                cpass.set_bind_group(i.try_into().unwrap(), &bind_groups[i], &[]);
            }
            let lx = gg.local_workgroup_size[0];
            let ly = gg.local_workgroup_size[1];
            let x = if gg.workgroup_sizes[0] > 0 {
                ((gg.workgroup_sizes[0] + lx - 1) / lx).try_into().unwrap()
            } else {
                gg.workgroup_sizes[0].try_into().unwrap()
            };
            let y = if gg.workgroup_sizes[1] > 0 {
                ((gg.workgroup_sizes[1] + ly - 1) / ly).try_into().unwrap()
            } else {
                gg.workgroup_sizes[1].try_into().unwrap()
            };
            let z = gg.workgroup_sizes[2].try_into().unwrap();
            cpass.dispatch_workgroups(x, y, z);
        }
    }

    g.get_queue().submit(Some(encoder.finish()));
}

pub fn read_buffer<T: std::clone::Clone>(b: &GBuffer) -> Vec<T> {
    let g = gpu();

    // Wait for all work to finish before reading out to avoid race conditions
    let _ = g.get_device().poll(wgpu::PollType::wait_indefinitely());

    let temp_buffer = create_empty_buffer(&map_read_buffer_type(), &bufferlen(b), &b.element_size)
        .expect("The buffer already exists so a new one the same size should always work");
    let mut encoder = g.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_buffer_to_buffer(b, 0, &temp_buffer, 0, b.size());
    let submission_index = g.get_queue().submit(Some(encoder.finish()));
    let temp_slice = temp_buffer.slice(..);
    temp_slice.map_async(
        wgpu::MapMode::Read,
        |_| { /* Not needed for us; single threaded GPU access in Alan (for now) */ },
    );
    let _ = g.get_device().poll(wgpu::PollType::Wait {
        submission_index: Some(submission_index),
        timeout: None,
    });
    let data = temp_slice
        .get_mapped_range()
        .expect("The full buffer should always be mappable");
    let data_ptr = data.as_ptr();
    let data_len = bufferlen(b) as usize;
    let data_slice: &[T] = unsafe { std::slice::from_raw_parts(data_ptr as *const T, data_len) };
    let result = data_slice.to_vec();
    drop(data);
    temp_buffer.unmap();
    result
}

#[allow(clippy::ptr_arg)]
pub fn replace_buffer<T>(b: &GBuffer, v: &[T]) -> Result<(), AlanError> {
    if v.len() as i64 != bufferlen(b) {
        Err("The input array is not the same size as the buffer".into())
    } else if !b.usage().contains(wgpu::BufferUsages::COPY_DST) {
        Err(
            "The destination buffer does not have COPY_DST usage flag required for copy operations"
                .into(),
        )
    } else {
        let g = gpu();
        let gb = create_buffer_init(&storage_buffer_type(), v, &b.element_size)
            .expect("The buffer already exists so a new one the same size should always work");
        let mut encoder = g.get_device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&gb, 0, b, 0, b.size());
        let submission_index = g.get_queue().submit(Some(encoder.finish()));
        let _ = g.get_device().poll(wgpu::PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        });
        gb.destroy();
        Ok(())
    }
}

/// Window-related types and functions
pub struct AlanWindowContext {
    window: Option<std::sync::Arc<Window>>,
    start: Option<std::time::Instant>,
    buffer_width: Option<u32>,
    mouse_x: Option<u32>,
    mouse_y: Option<u32>,
    mouse_left: bool,
    mouse_right: bool,
    mouse_middle: bool,
    mouse_wheel_dx: f32,
    mouse_wheel_dy: f32,
    cursor_visible: bool,
    transparent: bool,
}

impl AlanWindowContext {
    pub fn width(&self) -> u32 {
        match self.window.as_ref() {
            Some(win) => win.inner_size().width.max(1),
            None => 0,
        }
    }

    pub fn height(&self) -> u32 {
        match self.window.as_ref() {
            Some(win) => win.inner_size().height.max(1),
            None => 0,
        }
    }

    pub fn buffer_width(&self) -> u32 {
        self.buffer_width.unwrap_or(0) / 4
    }

    pub fn runtime(&self) -> u32 {
        match self.start.as_ref() {
            Some(time) => u32::from_le_bytes(time.elapsed().as_secs_f32().to_le_bytes()),
            None => 0,
        }
    }

    pub fn mouse_x(&mut self) -> u32 {
        match self.mouse_x {
            Some(x) => x,
            None => {
                self.mouse_x = Some(0);
                self.mouse_y = Some(0);
                0
            }
        }
    }

    pub fn mouse_y(&mut self) -> u32 {
        match self.mouse_y {
            Some(y) => y,
            None => {
                self.mouse_x = Some(0);
                self.mouse_y = Some(0);
                0
            }
        }
    }

    pub fn cursor_visible(&mut self) {
        self.cursor_visible = true;
    }

    pub fn cursor_invisible(&mut self) {
        self.cursor_visible = false;
    }

    pub fn transparent(&mut self) {
        self.transparent = true;
    }

    pub fn opaque(&mut self) {
        self.transparent = false;
    }

    pub fn mouse_left(&mut self) -> u32 {
        self.mouse_left as u32
    }

    pub fn mouse_right(&mut self) -> u32 {
        self.mouse_right as u32
    }

    pub fn mouse_middle(&mut self) -> u32 {
        self.mouse_middle as u32
    }

    pub fn mouse_wheel_x(&mut self) -> f32 {
        let v = self.mouse_wheel_dx;
        self.mouse_wheel_dx = 0.0;
        v
    }

    pub fn mouse_wheel_y(&mut self) -> f32 {
        let v = self.mouse_wheel_dy;
        self.mouse_wheel_dy = 0.0;
        v
    }
}

pub struct AlanWindowFrame {
    pub context: GBuffer,
    pub framebuffer: GBuffer,
    pub width: u32,
    pub height: u32,
}

/// User events sent to the shared event loop
enum UserEvent {
    /// Request to create a new window; blocks on `done_tx` until window closes
    NewWindow {
        config: WindowAttributes,
        context: AlanWindowContext,
        context_fn: Box<dyn FnMut(&mut AlanWindowContext) -> Vec<u32> + Send>,
        gpgpu_shader_fn: Box<dyn Fn(&AlanWindowFrame) -> Vec<GPGPU> + Send>,
        done_tx: Sender<()>,
    },
}

/// Per-window GPU and rendering state
struct WindowState {
    context: AlanWindowContext,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    context_buffer: Option<GBuffer>,
    buffer: Option<GBuffer>,
    cached_surface_config: Option<wgpu::SurfaceConfiguration>,
    cached_size: winit::dpi::PhysicalSize<u32>,
    context_fn: Box<dyn FnMut(&mut AlanWindowContext) -> Vec<u32>>,
    gpgpu_shader_fn: Box<dyn Fn(&AlanWindowFrame) -> Vec<GPGPU>>,
    gpgpu_shaders: Option<Vec<GPGPU>>,
    inited: bool,
    /// Signals the calling thread that this window has closed
    done_tx: Option<Sender<()>>,
}

/// Manages all open windows in the shared event loop
struct WindowManager {
    windows: HashMap<WindowId, WindowState>,
    pending: VecDeque<UserEvent>,
}

impl WindowManager {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            pending: VecDeque::new(),
        }
    }

    fn gpu_init(&mut self, id: WindowId) {
        let ws = self.windows.get_mut(&id).unwrap();
        if ws.context.start.is_none() {
            ws.context.start = Some(std::time::Instant::now());
        }
        if ws.surface.is_none() {
            let window = ws.context.window.as_ref().unwrap().clone();
            ws.surface = Some(instance().create_surface(window).unwrap());
        }
        if ws.device.is_none() {
            let g = gpu();
            ws.device = Some(g.get_device().clone());
            ws.queue = Some(g.get_queue().clone());
        }
        if ws.context_buffer.is_none() {
            let device = ws.device.as_ref().unwrap();
            ws.context_buffer = Some(GBuffer {
                buffer: Rc::new(device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: 256,
                    usage: storage_buffer_type(),
                    mapped_at_creation: false,
                })),
                id: format!("buffer_{}", format!("{}", Uuid::new_v4()).replace("-", "_")),
                element_size: 4,
            });
        }
        if ws.buffer.is_none() {
            let device = ws.device.as_ref().unwrap();
            let mut size = ws.context.window.as_ref().unwrap().inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);
            ws.context.buffer_width =
                Some(if (4 * size.width) % 256 == 0 { 4 * size.width } else { (4 * size.width) + (256 - ((4 * size.width) % 256)) });
            let buffer_size = (ws.context.buffer_width.unwrap() as u64) * (size.height as u64);
            ws.buffer = Some(GBuffer {
                buffer: Rc::new(device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: buffer_size,
                    usage: storage_buffer_type(),
                    mapped_at_creation: false,
                })),
                id: format!("buffer_{}", format!("{}", Uuid::new_v4()).replace("-", "_")),
                element_size: 4,
            });
        }
        if ws.gpgpu_shaders.is_none() {
            let mut size = ws.context.window.as_ref().unwrap().inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);
            ws.gpgpu_shaders =
                Some((ws.gpgpu_shader_fn)(&AlanWindowFrame {
                    context: ws.context_buffer.as_ref().unwrap().clone(),
                    framebuffer: ws.buffer.as_ref().unwrap().clone(),
                    width: size.width,
                    height: size.height,
                }));
        }
        ws.inited = true;
    }

    fn render_frame(&mut self, id: WindowId) {
        let ws = match self.windows.get_mut(&id) {
            Some(ws) => ws,
            None => return,
        };
        if !ws.inited {
            self.gpu_init(id);
        }
        let ws = match self.windows.get_mut(&id) {
            Some(ws) => ws,
            None => return,
        };
        let window = match ws.context.window.as_ref() {
            Some(w) => Arc::clone(w),
            None => return,
        };
        window.set_cursor_visible(ws.context.cursor_visible);
        window.set_transparent(ws.context.transparent);
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        let surface = match ws.surface.as_ref() {
            Some(s) => s,
            None => return,
        };
        let g = gpu();
        let device = match ws.device.as_ref() {
            Some(d) => d,
            None => return,
        };
        let queue = match ws.queue.as_ref() {
            Some(q) => q,
            None => return,
        };
        if ws.cached_surface_config.is_none() || ws.cached_size != size {
            let mut config = match surface.get_default_config(&g.adapter, size.width, size.height) {
                Some(c) => c,
                None => return,
            };
            config.usage =
                wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT;
            config.present_mode = wgpu::PresentMode::AutoVsync;
            config.desired_maximum_frame_latency = 1;
            config.alpha_mode = if ws.context.transparent {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else {
                wgpu::CompositeAlphaMode::Auto
            };
            surface.configure(device, &config);
            ws.cached_surface_config = Some(config);
            ws.cached_size = size;
        }
        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f)
            | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
            _ => return,
        };
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let context_array = (ws.context_fn)(&mut ws.context);
        let context_slice = &context_array[..];
        let context_ptr = context_slice.as_ptr();
        let context_u8_len = context_array.len() * 4;
        let context_u8: &[u8] =
            unsafe { std::slice::from_raw_parts(context_ptr as *const u8, context_u8_len) };
        let ctx_buf = match ws.context_buffer.as_ref() {
            Some(b) => b,
            None => return,
        };
        queue.write_buffer(&ctx_buf.buffer, 0, context_u8);
        let ggs = match ws.gpgpu_shaders.as_mut() {
            Some(g) => g,
            None => return,
        };
        for gg in ggs {
            if gg.module.is_none() {
                gg.module = Some(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&gg.source)),
                }));
            }
            let module = gg.module.as_ref().unwrap();
            if gg.compute_pipeline.is_none() {
                gg.compute_pipeline = Some(device.create_compute_pipeline(
                    &wgpu::ComputePipelineDescriptor {
                        label: None,
                        layout: None,
                        module,
                        entry_point: Some(&gg.entrypoint),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        cache: None,
                    },
                ));
            }
            let compute_pipeline = gg.compute_pipeline.as_ref().unwrap();
            let mut bind_groups = Vec::new();
            {
                let mut cpass =
                    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: None,
                        timestamp_writes: None,
                    });
                cpass.set_pipeline(compute_pipeline);
                for i in 0..gg.buffers.len() {
                    let bind_group_layout =
                        compute_pipeline.get_bind_group_layout(i.try_into().unwrap());
                    let bind_group_buffers = &gg.buffers[i];
                    let mut bind_group_entries = Vec::new();
                    #[allow(clippy::needless_range_loop)]
                    for j in 0..bind_group_buffers.len() {
                        bind_group_entries.push(wgpu::BindGroupEntry {
                            binding: j.try_into().unwrap(),
                            resource: bind_group_buffers[j].as_entire_binding(),
                        });
                    }
                    let bind_group =
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &bind_group_layout,
                            entries: &bind_group_entries[..],
                        });
                    bind_groups.push(bind_group);
                }
                #[allow(clippy::needless_range_loop)]
                for i in 0..gg.buffers.len() {
                    cpass.set_bind_group(i.try_into().unwrap(), &bind_groups[i], &[]);
                }
                let lx = gg.local_workgroup_size[0];
                let ly = gg.local_workgroup_size[1];
                cpass.dispatch_workgroups(
                    ((gg.workgroup_sizes[0] + lx - 1) / lx) as u32,
                    ((gg.workgroup_sizes[1] + ly - 1) / ly) as u32,
                    gg.workgroup_sizes[2] as u32,
                );
            }
        }
        let framebuffer = match ws.buffer.as_ref() {
            Some(b) => b,
            None => return,
        };
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &framebuffer.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: ws.context.buffer_width,
                    rows_per_image: None,
                },
            },
            frame.texture.as_image_copy(),
            frame.texture.size(),
        );
        queue.submit(Some(encoder.finish()));
        queue.present(frame);
        let frame_start = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_secs(0))
            .unwrap();
        let render_time = frame_start.elapsed();
        window.set_title(&format!("Render time: {:.3}", render_time.as_secs_f64()));
        window.request_redraw();
    }
}

impl ApplicationHandler<UserEvent> for WindowManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
        self.pending.push_back(event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        while let Some(event) = self.pending.pop_front() {
            match event {
                UserEvent::NewWindow {
                    config,
                    mut context,
                    context_fn,
                    gpgpu_shader_fn,
                    done_tx,
                } => {
                    let window = Arc::new(event_loop.create_window(config).unwrap());
                    context.window = Some(window.clone());
                    let id = window.id();
                    self.windows.insert(
                        id,
                        WindowState {
                            context,
                            surface: None,
                            device: None,
                            queue: None,
                            context_buffer: None,
                            buffer: None,
                            cached_surface_config: None,
                            cached_size: winit::dpi::PhysicalSize::new(0, 0),
                            context_fn,
                            gpgpu_shader_fn,
                            gpgpu_shaders: None,
                            inited: false,
                            done_tx: Some(done_tx),
                        },
                    );
                    window.request_redraw();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(ws) = self.windows.remove(&id) {
                    if let Some(b) = &ws.buffer {
                        b.destroy();
                    }
                    if let Some(b) = &ws.context_buffer {
                        b.destroy();
                    }
                    if let Some(tx) = ws.done_tx {
                        let _ = tx.send(());
                    }
                }
                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(mut new_size) => {
                if event_loop.exiting() {
                    return;
                }
                if let Some(ws) = self.windows.get(&id) {
                    if ws.inited {
                        let device = ws.device.as_ref().unwrap();
                        new_size.width = new_size.width.max(1);
                        new_size.height = new_size.height.max(1);
                        let buffer_width = if (4 * new_size.width) % 256 == 0 {
                            4 * new_size.width
                        } else {
                            (4 * new_size.width) + (256 - ((4 * new_size.width) % 256))
                        };
                        let buffer_size = (buffer_width as u64) * (new_size.height as u64);
                        let new_buffer = GBuffer {
                            buffer: Rc::new(device.create_buffer(&wgpu::BufferDescriptor {
                                label: None,
                                size: buffer_size,
                                usage: storage_buffer_type(),
                                mapped_at_creation: false,
                            })),
                            id: format!("buffer_{}", format!("{}", Uuid::new_v4()).replace("-", "_")),
                            element_size: 4,
                        };
                        if let Some(ws) = self.windows.get_mut(&id) {
                            if let Some(b) = &ws.buffer {
                                b.destroy();
                            }
                            ws.buffer = Some(new_buffer);
                            ws.context.buffer_width = Some(buffer_width);
                            ws.gpgpu_shaders =
                                Some((ws.gpgpu_shader_fn)(&AlanWindowFrame {
                                    context: ws.context_buffer.as_ref().unwrap().clone(),
                                    framebuffer: ws.buffer.as_ref().unwrap().clone(),
                                    width: new_size.width,
                                    height: new_size.height,
                                }));
                        }
                        if let Some(ws) = self.windows.get(&id) {
                            ws.context.window.as_ref().unwrap().request_redraw();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if event_loop.exiting() {
                    return;
                }
                self.render_frame(id);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(ws) = self.windows.get_mut(&id) {
                    if ws.context.mouse_x.is_some() {
                        ws.context.mouse_x = Some(position.x as u32);
                        ws.context.mouse_y = Some(position.y as u32);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                if let Some(ws) = self.windows.get_mut(&id) {
                    match button {
                        MouseButton::Left => ws.context.mouse_left = pressed,
                        MouseButton::Right => ws.context.mouse_right = pressed,
                        MouseButton::Middle => ws.context.mouse_middle = pressed,
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(ws) = self.windows.get_mut(&id) {
                    match delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            ws.context.mouse_wheel_dx += x;
                            ws.context.mouse_wheel_dy += y;
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            ws.context.mouse_wheel_dx += pos.x as f32;
                            ws.context.mouse_wheel_dy += pos.y as f32;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy<UserEvent>> = OnceLock::new();

pub fn run_window<C, R>(
    mut initial_context_fn: impl FnMut(&mut AlanWindowContext),
    context_fn: C,
    gpgpu_shader_fn: R,
) -> Result<(), AlanError>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32> + Send + 'static,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU> + Send + 'static,
{
    let mut context = AlanWindowContext {
        window: None,
        start: None,
        buffer_width: None,
        mouse_x: None,
        mouse_y: None,
        mouse_left: false,
        mouse_right: false,
        mouse_middle: false,
        mouse_wheel_dx: 0.0,
        mouse_wheel_dy: 0.0,
        cursor_visible: true,
        transparent: false,
    };
    initial_context_fn(&mut context);
    let config = Window::default_attributes().with_transparent(context.transparent);
    let (tx, rx) = channel();

    // First call: create and run the event loop on the current thread.
    // Subsequent calls: send requests via the proxy and block on the channel.
    if EVENT_LOOP_PROXY.get().is_none() {
        let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event()
            .build()
            .map_err(|e| AlanError {
                message: format!("Failed to create event loop: {}", e),
            })?;
        let proxy = event_loop.create_proxy();
        EVENT_LOOP_PROXY.set(proxy).unwrap();

        // Send the first window request
        EVENT_LOOP_PROXY
            .get()
            .unwrap()
            .send_event(UserEvent::NewWindow {
                config,
                context,
                context_fn: Box::new(context_fn),
                gpgpu_shader_fn: Box::new(gpgpu_shader_fn),
                done_tx: tx,
            })
            .map_err(|e| AlanError {
                message: format!("Failed to send window request: {}", e),
            })?;

        let mut manager = WindowManager::new();
        event_loop.run_app(&mut manager).map_err(|e| AlanError {
            message: format!("Event loop error: {}", e),
        })?;

        // After run_app returns, all windows have closed; drain the notification
        rx.recv().map_err(|e| AlanError {
            message: format!("Window channel closed: {}", e),
        })?;
        Ok(())
    } else {
        // Event loop already running — send request via proxy
        EVENT_LOOP_PROXY
            .get()
            .unwrap()
            .send_event(UserEvent::NewWindow {
                config,
                context,
                context_fn: Box::new(context_fn),
                gpgpu_shader_fn: Box::new(gpgpu_shader_fn),
                done_tx: tx,
            })
            .map_err(|e| AlanError {
                message: format!("Failed to send window request: {}", e),
            })?;
        rx.recv().map_err(|e| AlanError {
            message: format!("Window channel closed: {}", e),
        })?;
        Ok(())
    }
}
