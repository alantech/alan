/// Rust functions that the root scope binds.
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::OnceLock;

pub use ordered_hash_map::OrderedHashMap;
pub use uuid::Uuid;
pub use wgpu::BufferUsages;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
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

/// String-related functions

/// Converts anything that implements ToString into a string. Needed to convert all errors into
/// AlanError since it doesn't seem possible to `impl From<dyn std::error::Error>`
pub fn stringify<T: std::string::ToString>(v: T) -> String {
    v.to_string()
}

/// `splitstring` creates a vector of strings split by the specified separator string
#[inline(always)]
pub fn splitstring(a: &String, b: &String) -> Vec<String> {
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
pub fn getstring(a: &String, i: &i64) -> Result<String, AlanError> {
    a.chars()
        .nth(*i as usize)
        .map(|c| String::from(c))
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
pub fn indexstring(a: &String, b: &String) -> Result<i64, AlanError> {
    a.find(b).map(|v| v as i64).ok_or(AlanError {
        message: format!("Could not find {} in {}", b, a),
    })
}

/// Boolean-related functions

/// `ifbool` executes the true function on true, and the false function on false, returning the
/// value returned by either function
#[inline(always)]
pub fn ifbool<T>(c: &bool, mut t: impl FnMut() -> T, mut f: impl FnMut() -> T) -> T {
    if *c {
        t()
    } else {
        f()
    }
}

/// Array-related functions

/// `getarray` returns a value from an array at the location specified
#[inline(always)]
pub fn getarray<T: Clone>(a: &Vec<T>, i: &i64) -> Option<T> {
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
pub fn map_onearg<A, B>(v: &Vec<A>, mut m: impl FnMut(&A) -> B) -> Vec<B> {
    v.iter().map(|val| m(val)).collect::<Vec<B>>()
}

/// `map_twoarg` runs the provided two-argument (value, index) function on each element of the
/// vector, returning a new vector
#[inline(always)]
pub fn map_twoarg<A, B>(v: &Vec<A>, mut m: impl FnMut(&A, i64) -> B) -> Vec<B> {
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
                        let out = (o_ptr as *mut B).offset(i as isize);
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
                                let out = o_ptr.offset(i);
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
pub fn filter_onearg<A: std::clone::Clone>(v: &Vec<A>, mut f: impl FnMut(&A) -> bool) -> Vec<A> {
    v.iter()
        .filter(|val| f(val))
        .map(|val| val.clone())
        .collect::<Vec<A>>()
}

/// `filter_twoarg` runs the provided function each element of the vector plus its index,
/// returning a new vector
#[inline(always)]
pub fn filter_twoarg<A: std::clone::Clone>(
    v: &Vec<A>,
    mut f: impl FnMut(&A, i64) -> bool,
) -> Vec<A> {
    v.iter()
        .enumerate()
        .filter(|(i, val)| f(val, *i as i64))
        .map(|(_, val)| val.clone())
        .collect::<Vec<A>>()
}

/// `reduce_sametype` runs the provided function to reduce the vector into a singular value
#[inline(always)]
pub fn reduce_sametype<A: std::clone::Clone>(
    v: &Vec<A>,
    mut f: impl FnMut(&A, &A) -> A,
) -> Option<A> {
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
pub fn reduce_sametype_idx<A: std::clone::Clone>(
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
pub fn reduce_difftype<A: std::clone::Clone, B: std::clone::Clone>(
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
pub fn reduce_difftype_idx<A: std::clone::Clone, B: std::clone::Clone>(
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
pub fn concat<A: std::clone::Clone>(a: &Vec<A>, b: &Vec<A>) -> Vec<A> {
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
pub fn append<A: std::clone::Clone>(a: &mut Vec<A>, b: &Vec<A>) {
    for i in 0..b.len() {
        a.push(b[i].clone());
    }
}

/// `hasfnarray` returns true if the check function returns true for any element of the vector
#[inline(always)]
pub fn hasfnarray<T>(a: &Vec<T>, mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    return false;
}

/// `findarray` returns the first value from the vector that matches the check function, if any
#[inline(always)]
pub fn findarray<T: std::clone::Clone>(a: &Vec<T>, mut f: impl FnMut(&T) -> bool) -> Option<T> {
    for v in a {
        if f(v) {
            return Some(v.clone());
        }
    }
    return None;
}

/// `everyarray` returns true if every value in the vector matches the check function
#[inline(always)]
pub fn everyarray<T>(a: &Vec<T>, mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if !f(v) {
            return false;
        }
    }
    return true;
}

/// `somearray` returns true if any value in the vector matches the check function
#[inline(always)]
pub fn somearray<T>(a: &Vec<T>, mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    return false;
}

/// `repeatarray` returns a new array with the original array repeated N times
#[inline(always)]
pub fn repeatarray<T: std::clone::Clone>(a: &Vec<T>, c: &i64) -> Vec<T> {
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
pub fn deletearray<T: std::clone::Clone>(a: &mut Vec<T>, i: &i64) -> Result<T, AlanError> {
    match (*i as usize) >= a.len() {
        true => Err(format!(
            "Provided array index {} is beyond the bounds of the array",
            i
        )
        .into()),
        false => Ok(a.remove(*i as usize).clone()),
    }
}

/// `swaparray` swaps the values at the specified indicies (or fails if either index is out of
/// bounds). It returns a Fallible void value.
#[inline(always)]
pub fn swaparray<T>(a: &mut Vec<T>, i: &i64, j: &i64) -> Result<(), AlanError> {
    if *i < 0 {
        return Err(format!(
            "Provided array index {} is beyond the bounds of the array",
            i
        )
        .into());
    }
    if *j < 0 {
        return Err(format!(
            "Provided array index {} is beyond the bounds of the array",
            j
        )
        .into());
    }
    let i = *i as usize;
    let j = *j as usize;
    if i >= a.len() {
        return Err(format!(
            "Provided array index {} is beyond the bounds of the array",
            i
        )
        .into());
    }
    if j >= a.len() {
        return Err(format!(
            "Provided array index {} is beyond the bounds of the array",
            j
        )
        .into());
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
pub fn sortarray<T>(a: &mut Vec<T>, mut sorter: impl FnMut(&T, &T) -> i8) {
    a.sort_by(|a, b| sorter(a, b).cmp(&0));
}

/// Buffer-related functions

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
pub fn reducebuffer_difftype<A: std::clone::Clone, const S: usize, B: std::clone::Clone>(
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
pub fn hasbuffer<T: std::cmp::PartialEq, const S: usize>(a: &[T; S], v: &T) -> bool {
    for val in a {
        if val == v {
            return true;
        }
    }
    return false;
}

/// `hasfnbuffer` returns true if the check function returns true for any element of the array
#[inline(always)]
pub fn hasfnbuffer<T, const S: usize>(a: &[T; S], mut f: impl FnMut(&T) -> bool) -> bool {
    for v in a {
        if f(v) {
            return true;
        }
    }
    return false;
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
    return None;
}

/// `everybuffer` returns true if every value in the array matches the check function
#[inline(always)]
pub fn everybuffer<T, const S: usize>(a: &[T; S], mut f: impl FnMut(&T) -> bool) -> bool {
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
        false => Err(format!(
            "The provided index {} is out-of-bounds for the specified buffer",
            i
        )
        .into()),
        true => Ok(std::mem::replace(a.each_mut()[*i as usize], v.clone())),
    }
}

/// `swapbuffer` swaps the values at the specified indicies (or fails if either index is out of
/// bounds). It returns a Fallible void value.
#[inline(always)]
pub fn swapbuffer<T, const S: usize>(a: &mut [T; S], i: &i64, j: &i64) -> Result<(), AlanError> {
    if *i < 0 {
        return Err(format!(
            "Provided buffer index {} is beyond the bounds of the buffer",
            i
        )
        .into());
    }
    if *j < 0 {
        return Err(format!(
            "Provided buffer index {} is beyond the bounds of the buffer",
            j
        )
        .into());
    }
    let i = *i as usize;
    let j = *j as usize;
    if i >= a.len() {
        return Err(format!(
            "Provided buffer index {} is beyond the bounds of the buffer",
            i
        )
        .into());
    }
    if j >= a.len() {
        return Err(format!(
            "Provided buffer index {} is beyond the bounds of the buffer",
            j
        )
        .into());
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

/// Dictionary-related bindings

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
    d.keys().map(|k| k.clone()).collect::<Vec<K>>()
}

/// `valsdict` returns an array of values from the dictionary
#[inline(always)]
pub fn valsdict<K, V: std::clone::Clone>(d: &OrderedHashMap<K, V>) -> Vec<V> {
    d.values().map(|v| v.clone()).collect::<Vec<V>>()
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

/// Set-related bindings

/// `arrayset` returns an array of values in the set
#[inline(always)]
pub fn arrayset<V: std::clone::Clone>(s: &HashSet<V>) -> Vec<V> {
    s.iter().map(|v| v.clone()).collect::<Vec<V>>()
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
    a.union(b).map(|v| v.clone()).collect::<HashSet<V>>()
}

/// `intersectset` returns a new set that is the intersection of the original two sets
#[inline(always)]
pub fn intersectset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    a.intersection(b).map(|v| v.clone()).collect::<HashSet<V>>()
}

/// `differenceset` returns the difference of the original two sets (values in A not in B)
#[inline(always)]
pub fn differenceset<V: std::clone::Clone + std::hash::Hash + Eq>(
    a: &HashSet<V>,
    b: &HashSet<V>,
) -> HashSet<V> {
    a.difference(b).map(|v| v.clone()).collect::<HashSet<V>>()
}

/// `symmetric_differenceset` returns the symmetric difference of the original two sets (values in
/// A not in B *and* values in B not in A)
#[inline(always)]
pub fn symmetric_differenceset<V: std::clone::Clone + std::hash::Hash + Eq>(
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

/// Vector-related functions

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

/// GPU-related functions and types

pub struct GPU {
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
                    memory_hints: wgpu::MemoryHints::Performance,
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
    vals: &Vec<T>,
    element_size: &i8,
) -> GBuffer {
    let g = gpu();
    let val_slice = &vals[..];
    let val_ptr = val_slice.as_ptr();
    let val_u8_len = vals.len() * (*element_size as usize);
    let val_u8: &[u8] = unsafe { std::slice::from_raw_parts(val_ptr as *const u8, val_u8_len) };
    GBuffer {
        buffer: Rc::new(wgpu::util::DeviceExt::create_buffer_init(
            &g.device,
            &wgpu::util::BufferInitDescriptor {
                label: None, // TODO: Add a label for easier debugging?
                contents: val_u8,
                usage: *usage,
            },
        )),
        id: format!("buffer_{}", format!("{}", Uuid::new_v4()).replace("-", "_")),
        element_size: *element_size,
    }
}

pub fn create_empty_buffer(usage: &wgpu::BufferUsages, size: &i64, element_size: &i8) -> GBuffer {
    let g = gpu();
    GBuffer {
        buffer: Rc::new(g.device.create_buffer(&wgpu::BufferDescriptor {
            label: None, // TODO: Add a label for easier debugging?
            size: (*size as u64) * (*element_size as u64),
            usage: *usage,
            mapped_at_creation: false, // TODO: With `create_buffer_init` does this make any sense?
        })),
        id: format!("buffer_{}", format!("{}", Uuid::new_v4()).replace("-", "_")),
        element_size: *element_size,
    }
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
    pub module: Option<wgpu::ShaderModule>,
    pub compute_pipeline: Option<wgpu::ComputePipeline>,
}

impl GPGPU {
    pub fn new(source: String, buffers: Vec<Vec<GBuffer>>, workgroup_sizes: [i64; 3]) -> GPGPU {
        GPGPU {
            source,
            entrypoint: "main".to_string(),
            buffers,
            workgroup_sizes,
            module: None,
            compute_pipeline: None,
        }
    }
}

pub fn gpu_run(gg: &mut GPGPU) {
    let g = gpu();
    if gg.module.is_none() {
        gg.module = Some(g.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&gg.source)),
        }));
    }
    let module = gg.module.as_ref().unwrap();
    if gg.compute_pipeline.is_none() {
        gg.compute_pipeline = Some(g.device.create_compute_pipeline(
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
    let mut encoder = g
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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

pub fn gpu_run_list(ggs: &mut Vec<GPGPU>) {
    let g = gpu();
    let mut encoder = g
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    for gg in ggs {
        if gg.module.is_none() {
            gg.module = Some(g.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&gg.source)),
            }));
        }
        let module = gg.module.as_ref().unwrap();
        if gg.compute_pipeline.is_none() {
            gg.compute_pipeline = Some(g.device.create_compute_pipeline(
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
    }
    g.queue.submit(Some(encoder.finish()));
}

pub fn read_buffer<T: std::clone::Clone>(b: &GBuffer) -> Vec<T> {
    let g = gpu();
    let temp_buffer = create_empty_buffer(&map_read_buffer_type(), &bufferlen(b), &b.element_size);
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
        let data_len = bufferlen(b) as usize;
        let data_slice: &[T] =
            unsafe { std::slice::from_raw_parts(data_ptr as *const T, data_len) };
        let result = data_slice.to_vec();
        drop(data);
        temp_buffer.unmap();
        result
    } else {
        panic!("Failed to run compute on gpu!")
    }
}

#[allow(clippy::ptr_arg)]
pub fn replace_buffer<T>(b: &GBuffer, v: &Vec<T>) -> Result<(), AlanError> {
    if v.len() as i64 != bufferlen(b) {
        Err("The input array is not the same size as the buffer".into())
    } else {
        let g = gpu();
        let gb = create_buffer_init(&map_write_buffer_type(), &v, &b.element_size);
        let mut encoder = g
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&gb, 0, b, 0, b.size());
        g.queue.submit(Some(encoder.finish()));
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
        if self.mouse_x.is_none() {
            self.mouse_x = Some(0);
            self.mouse_y = Some(0);
            0
        } else {
            self.mouse_x.unwrap()
        }
    }

    pub fn mouse_y(&mut self) -> u32 {
        if self.mouse_y.is_none() {
            self.mouse_x = Some(0);
            self.mouse_y = Some(0);
            0
        } else {
            self.mouse_y.unwrap()
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
        self.transparent = true;
    }
}

pub struct AlanWindowFrame {
    pub context: GBuffer,
    pub framebuffer: GBuffer,
}

pub struct AlanWindow<C, R>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32>,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU>,
{
    config: WindowAttributes,
    context: AlanWindowContext,
    instance: Option<wgpu::Instance>,
    surface: Option<wgpu::Surface<'static>>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    context_buffer: Option<GBuffer>,
    buffer: Option<GBuffer>,
    context_fn: C,
    gpgpu_shader_fn: R,
    gpgpu_shaders: Option<Vec<GPGPU>>,
    inited: bool,
}

impl<C, R> AlanWindow<C, R>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32>,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU>,
{
    fn window_gpu_init(&mut self) {
        if self.context.start.is_none() {
            self.context.start = Some(std::time::Instant::now());
        }
        if self.instance.is_none() {
            self.instance = Some(wgpu::Instance::default());
        }
        if self.surface.is_none() {
            let instance = self.instance.as_ref().unwrap();
            self.surface = Some(
                instance
                    .create_surface(self.context.window.as_ref().unwrap().clone())
                    .unwrap(),
            );
        }
        if self.adapter.is_none() {
            let instance = self.instance.as_ref().unwrap();
            let surface = self.surface.as_ref().unwrap();
            self.adapter = Some(
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(), // TODO: Configure this
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                }))
                .unwrap(),
            );
            println!("Supported texture formats: {:?}", surface.get_capabilities(self.adapter.as_ref().unwrap()).formats);
        }
        if self.device.is_none() {
            // We can do both device and queue here as they're created at the same time
            let adapter = self.adapter.as_ref().unwrap();
            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: adapter.features(),
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            ))
            .unwrap();
            self.device = Some(device);
            self.queue = Some(queue);
        }
        if self.context_buffer.is_none() {
            let device = self.device.as_ref().unwrap();
            self.context_buffer = Some(GBuffer {
                buffer: Rc::new(device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: 16, // TODO: Not hardwired
                    usage: storage_buffer_type(),
                    mapped_at_creation: false,
                })),
                id: format!("buffer_{}", format!("{}", Uuid::new_v4()).replace("-", "_")),
                element_size: 4,
            });
        }
        if self.buffer.is_none() {
            let device = self.device.as_ref().unwrap();
            let mut size = self.context.window.as_ref().unwrap().inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);
            self.context.buffer_width = Some(if (4 * size.width) % 256 == 0 {
                4 * size.width
            } else {
                (4 * size.width) + (256 - ((4 * size.width) % 256))
            });
            let buffer_height = size.height;
            let buffer_size = (self.context.buffer_width.unwrap() as u64) * (buffer_height as u64);
            self.buffer = Some(GBuffer {
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
        if self.gpgpu_shaders.is_none() {
            let mut size = self.context.window.as_ref().unwrap().inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);
            self.gpgpu_shaders = Some((self.gpgpu_shader_fn)(&AlanWindowFrame {
                context: self.context_buffer.as_ref().unwrap().clone(),
                framebuffer: self.buffer.as_ref().unwrap().clone(),
            }));
        }
        self.inited = true;
    }
}

impl<C, R> ApplicationHandler for AlanWindow<C, R>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32>,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU>,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        self.context.window = Some(std::sync::Arc::new(
            event_loop.create_window(self.config.clone()).unwrap(),
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                // Cleanup the app now that we're caching things
                self.gpgpu_shaders = None;
                self.context.window = None;
                if let Some(b) = &self.buffer {
                    b.destroy();
                }
                self.buffer = None;
                if let Some(b) = &self.context_buffer {
                    b.destroy();
                }
                self.context_buffer = None;
                self.queue = None;
                self.device = None;
                self.adapter = None;
                self.surface = None;
                self.instance = None;
                event_loop.exit();
            }
            WindowEvent::Resized(mut new_size) => {
                if event_loop.exiting() {
                    return;
                }
                if !self.inited {
                    self.window_gpu_init();
                }
                // We need to create a new buffer with the right size *and* replace all instances
                // of the old buffer in the GPGPU array with the new one.
                let device = self.device.as_ref().unwrap();
                new_size.width = new_size.width.max(1);
                new_size.height = new_size.height.max(1);
                self.context.buffer_width = Some(if (4 * new_size.width) % 256 == 0 {
                    4 * new_size.width
                } else {
                    (4 * new_size.width) + (256 - ((4 * new_size.width) % 256))
                });
                let buffer_height = new_size.height;
                let buffer_size =
                    (self.context.buffer_width.unwrap() as u64) * (buffer_height as u64);
                let old_buffer_id = self.buffer.as_ref().unwrap().id.clone();
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
                for shader in self.gpgpu_shaders.as_mut().unwrap() {
                    for group in &mut shader.buffers {
                        let mut idx = None;
                        for (i, buffer) in group.iter().enumerate() {
                            if buffer.id == old_buffer_id {
                                idx = Some(i);
                                break;
                            }
                        }
                        if let Some(id) = idx {
                            group[id] = new_buffer.clone();
                        }
                    }
                }
                if let Some(b) = &self.buffer {
                    b.destroy();
                }
                self.buffer = Some(new_buffer);
                self.context.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                if event_loop.exiting() {
                    return;
                }
                let frame_start = std::time::Instant::now();
                if !self.inited {
                    self.window_gpu_init();
                }
                let window = self.context.window.as_ref().unwrap();
                // TODO: These shouldn't be set every frame
                window.set_cursor_visible(self.context.cursor_visible);
                window.set_transparent(self.context.transparent);
                let mut size = window.inner_size();
                size.width = size.width.max(1);
                size.height = size.height.max(1);
                let surface = self.surface.as_ref().unwrap();
                let adapter = self.adapter.as_ref().unwrap();
                let device = self.device.as_ref().unwrap();
                let old_context_buffer_id = self.context_buffer.as_ref().unwrap().id.clone();
                let queue = self.queue.as_ref().unwrap();
                let mut config = surface
                    .get_default_config(adapter, size.width, size.height)
                    .unwrap();
                config.usage =
                    wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT;
                config.present_mode = wgpu::PresentMode::Fifo;
                config.desired_maximum_frame_latency = 3;
                config.alpha_mode = if self.context.transparent {
                    wgpu::CompositeAlphaMode::PreMultiplied
                } else {
                    wgpu::CompositeAlphaMode::Auto
                };
                surface.configure(device, &config);
                let frame = surface.get_current_texture().unwrap();
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                let context_array = (self.context_fn)(&mut self.context);
                let context_slice = &context_array[..];
                let context_ptr = context_slice.as_ptr();
                let context_u8_len = context_array.len() * 4;
                let context_u8: &[u8] =
                    unsafe { std::slice::from_raw_parts(context_ptr as *const u8, context_u8_len) };
                let new_context_buffer = GBuffer {
                    buffer: Rc::new(wgpu::util::DeviceExt::create_buffer_init(
                        device,
                        &wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: context_u8,
                            usage: storage_buffer_type(),
                        },
                    )),
                    id: old_context_buffer_id.clone(),
                    element_size: 4,
                };
                let ggs = self.gpgpu_shaders.as_mut().unwrap();
                for gg in ggs {
                    if gg.module.is_none() {
                        gg.module =
                            Some(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                                label: None,
                                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                                    &gg.source,
                                )),
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
                        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: None,
                            timestamp_writes: None,
                        });
                        cpass.set_pipeline(compute_pipeline);
                        for i in 0..gg.buffers.len() {
                            let bind_group_layout =
                                compute_pipeline.get_bind_group_layout(i.try_into().unwrap());
                            let bind_group_buffers = &mut gg.buffers[i];
                            #[allow(clippy::needless_range_loop)]
                            for j in 0..bind_group_buffers.len() {
                                if bind_group_buffers[j].id == old_context_buffer_id {
                                    bind_group_buffers[j] = new_context_buffer.clone();
                                }
                            }
                            let mut bind_group_entries = Vec::new();
                            #[allow(clippy::needless_range_loop)]
                            for j in 0..bind_group_buffers.len() {
                                bind_group_entries.push(wgpu::BindGroupEntry {
                                    binding: j.try_into().unwrap(),
                                    resource: bind_group_buffers[j].as_entire_binding(),
                                });
                            }
                            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: None,
                                layout: &bind_group_layout,
                                entries: &bind_group_entries[..],
                            });
                            bind_groups.push(bind_group);
                        }
                        #[allow(clippy::needless_range_loop)]
                        for i in 0..gg.buffers.len() {
                            // The Rust borrow checker is forcing my hand here
                            cpass.set_bind_group(i.try_into().unwrap(), &bind_groups[i], &[]);
                        }
                        cpass.dispatch_workgroups(
                            // TODO: Can I avoid this branching somehow?
                            match gg.workgroup_sizes[0] {
                                -1 => size.width,
                                -2 => size.height,
                                otherwise => otherwise.try_into().unwrap(),
                            },
                            match gg.workgroup_sizes[1] {
                                -1 => size.width,
                                -2 => size.height,
                                otherwise => otherwise.try_into().unwrap(),
                            },
                            match gg.workgroup_sizes[2] {
                                -1 => size.width,
                                -2 => size.height,
                                otherwise => otherwise.try_into().unwrap(),
                            },
                        );
                    }
                }
                self.context_buffer.as_ref().unwrap().destroy();
                self.context_buffer = Some(new_context_buffer);
                encoder.copy_buffer_to_texture(
                    wgpu::TexelCopyBufferInfo {
                        buffer: &self.buffer.as_ref().unwrap().buffer,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: self.context.buffer_width,
                            rows_per_image: None,
                        },
                    },
                    frame.texture.as_image_copy(),
                    frame.texture.size(),
                );
                queue.submit(Some(encoder.finish()));
                frame.present();
                let render_time = frame_start.elapsed();
                self.context
                    .window
                    .as_ref()
                    .unwrap()
                    .set_title(&format!("Render time: {:.3}", render_time.as_secs_f64()));
                self.context.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.context.mouse_x.is_some() {
                    self.context.mouse_x = Some(position.x as u32);
                    self.context.mouse_y = Some(position.y as u32);
                }
            }
            _ => {} // Ignore all other events
        }
    }
}

pub fn run_window<C, R>(
    mut initial_context_fn: impl FnMut(&mut AlanWindowContext),
    context_fn: C,
    gpgpu_shader_fn: R,
) -> Result<(), AlanError>
where
    C: FnMut(&mut AlanWindowContext) -> Vec<u32>,
    R: Fn(&AlanWindowFrame) -> Vec<GPGPU>,
{
    let mut context = AlanWindowContext {
        window: None,
        start: None,
        buffer_width: None,
        mouse_x: None,
        mouse_y: None,
        cursor_visible: true,
        transparent: false,
    };
    initial_context_fn(&mut context);
    let config = Window::default_attributes().with_transparent(context.transparent);
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll); // TODO: This should also be configurable
    let mut app = AlanWindow {
        config,
        context,
        instance: None,
        surface: None,
        adapter: None,
        device: None,
        queue: None,
        context_buffer: None,
        buffer: None,
        context_fn,
        gpgpu_shader_fn,
        gpgpu_shaders: None,
        inited: false,
    };
    match event_loop.run_app(&mut app) {
        Ok(_) => Ok(()),
        Err(e) => Err(AlanError {
            message: format!("{:?}", e),
        }),
    }
}
