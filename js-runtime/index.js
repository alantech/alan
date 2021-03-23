require('cross-fetch/polyfill')
const EventEmitter = require('events')
const http = require('http')
const util = require('util')

const xxh = require('xxhashjs')

const exec = util.promisify ? util.promisify(require('child_process').exec) : () => {} // browsers

const e = new EventEmitter()

const INT8MAX = 2 ** 7 - 1
const INT8MIN = -(2 ** 7)
const INT16MAX = 2 ** 15 - 1
const INT16MIN = -(2 ** 15)
const INT32MAX = 2 ** 31 - 1
const INT32MIN = -(2 ** 31)
const INT64MAX = 2n ** 63n - 1n
const INT64MIN = -(2n ** 63n)

// Hashing opcodes (hashv is recursive, needs to be defined outside of the export object)
const hashcore = (hasher, a) => {
  // TODO: We have to turn these values into ArrayBuffers of the right type. There's currently an
  // issue if a floating point number that is also an integer is provided -- the duck typing here
  // will treat it as an i64 instead of an f64 so the hash will be different between the JS and
  // Rust implementations. There are a few ways to solve this, but they all have tradeoffs. Will
  // revisit this in the future.
  let buffer = new ArrayBuffer(8)
  if (typeof a === 'number') {
    if (a === parseInt(a)) {
      const view = new BigInt64Array(buffer)
      view.set([BigInt(a)], 0)
    } else {
      const view = new Float64Array(buffer)
      view.set([a], 0)
    }
  } else if (typeof a === 'bigint') {
    const view = new BigInt64Array(buffer)
    view.set([a], 0)
  } else if (typeof a === 'string') {
    // If it's a string, we treat it like an array of 64-bit integers with a prefixed 64-bit length
    // to match the behavior of the Rust runtime
    const len = a.length
    const len8 = Math.ceil(len / 8) * 8
    buffer = new ArrayBuffer(8 + len8)
    const lenview = new BigInt64Array(buffer)
    lenview.set([BigInt(len)], 0)
    const strview = new Int8Array(buffer)
    // The following only works in the ASCII subset for now, since JS chose to use utf16 instead of
    // utf8. TODO: Find a pure Javascript library that converts utf16 codepoints to utf8, or write
    // one. :/
    strview.set(a.split('').map(s => s.charCodeAt(0)), 8)
  } else {
    // Booleans are treated as if they are 64-bit integers
    const val = a ? BigInt(1) : BigInt(0)
    const view = new BigInt64Array(buffer)
    view.set([val], 0)
  }
  for (let i = 0; i < buffer.byteLength; i += 8) {
    const piece = buffer.slice(i, i + 8)
    hasher.update(piece)
  }
  return hasher
}
const hashf = a => BigInt.asIntN(64, hashcore(xxh.h64().init(0xfa57), a).digest())
const hashv = arr => {
  // The Rust runtime considers strings a variable type, but they are more like a fixed type for JS
  if (typeof arr === 'string') return hashf(arr)
  const hasher = xxh.h64().init(0xfa57)
  let stack = [arr]
  while (stack.length > 0) {
    let arr = stack.pop()
    for (const elem of arr) {
      if (elem instanceof Array) {
        stack.push(elem)
      } else {
        hashcore(hasher, elem)
      }
    }
  }
  return BigInt.asIntN(64, hasher.digest())
}

const copyarr = a => {
  try {
    return JSON.parse(JSON.stringify(a))
  } catch (e) {
    if (typeof a[0] === 'bigint') {
      return a.map(v => BigInt(v))
    } else {
      return a.map(v => copyarr(v))
    }
  }
}

// Not very OOP, but since the HTTP server is a singleton right now, store open connections here
const httpConns = {}

// The shared mutable state for the datastore library
const ds = {}

module.exports = {
  // Type conversion opcodes (mostly no-ops in JS, unless we implement a strict mode)
  i8f64:    a => a,
  i16f64:   a => a,
  i32f64:   a => a,
  i64f64:   a => parseFloat(a.toString()),
  f32f64:   a => a,
  strf64:   a => parseFloat(a),
  boolf64:  a => a ? 1.0 : 0.0,

  i8f32:    a => a,
  i16f32:   a => a,
  i32f32:   a => a,
  i64f32:   a => parseFloat(a.toString()),
  f64f32:   a => a,
  strf32:   a => parseFloat(a),
  boolf32:  a => a ? 1.0 : 0.0,

  i8i64:    a => BigInt(a),
  i16i64:   a => BigInt(a),
  i32i64:   a => BigInt(a),
  f32i64:   a => BigInt(Math.floor(a)),
  f64i64:   a => BigInt(Math.floor(a)),
  stri64:   a => BigInt(parseInt(a)), // intentionally allowing other bases here
  booli64:  a => a ? 1n : 0n,

  i8i32:    a => a,
  i16i32:   a => a,
  i64i32:   a => Number(BigInt.asIntN(32, a)),
  f32i32:   a => Math.floor(a),
  f64i32:   a => Math.floor(a),
  stri32:   a => parseInt(a),
  booli32:  a => a ? 1 : 0,

  i8i16:    a => a,
  i32i16:   a => a,
  i64i16:   a => Number(BigInt.asIntN(16, a)),
  f32i16:   a => Math.floor(a),
  f64i16:   a => Math.floor(a),
  stri16:   a => parseInt(a),
  booli16:  a => a ? 1 : 0,

  i16i8:    a => a,
  i32i8:    a => a,
  i64i8:    a => Number(BigInt.asIntN(8, a)),
  f32i8:    a => Math.floor(a),
  f64i8:    a => Math.floor(a),
  stri8:    a => parseInt(a),
  booli8:   a => a ? 1 : 0,

  i8bool:   a => a !== 0,
  i16bool:  a => a !== 0,
  i32bool:  a => a !== 0,
  i64bool:  a => a !== 0n,
  f32bool:  a => a !== 0.0,
  f64bool:  a => a !== 0.0,
  strbool:  a => a === "true",

  i8str:    a => a.toString(),
  i16str:   a => a.toString(),
  i32str:   a => a.toString(),
  i64str:   a => a.toString(),
  f32str:   a => a.toString(),
  f64str:   a => a.toString(),
  boolstr:  a => a.toString(),

  // Arithmetic opcodes
  addi8:   (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 0 && a > INT8MAX - b) return [false, 'overflow']
    if (a < 0 && b < 0 && a < INT8MIN - b) return [false, 'underflow']
    return [true, a + b]
  },
  addi16:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 0 && a > INT16MAX - b) return [false, 'overflow']
    if (a < 0 && b < 0 && a < INT16MIN - b) return [false, 'underflow']
    return [true, a + b]
  },
  addi32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 0 && a > INT32MAX - b) return [false, 'overflow']
    if (a < 0 && b < 0 && a < INT32MIN - b) return [false, 'underflow']
    return [true, a + b]
  },
  addi64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0n && b > 0n && a > INT64MAX - b) return [false, 'overflow']
    if (a < 0n && b < 0n && a < INT64MIN - b) return [false, 'underflow']
    return [true, a + b]
  },
  addf32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    const out = a + b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },
  addf64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    const out = a + b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },

  subi8:   (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b < 0 && a > INT8MAX + b) return [false, 'overflow']
    if (a < 0 && b > 0 && a < INT8MIN + b) return [false, 'underflow']
    return [true, a - b]
  },
  subi16:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b < 0 && a > INT16MAX + b) return [false, 'overflow']
    if (a < 0 && b > 0 && a < INT16MIN + b) return [false, 'underflow']
    return [true, a - b]
  },
  subi32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b < 0 && a > INT32MAX + b) return [false, 'overflow']
    if (a < 0 && b > 0 && a < INT32MIN + b) return [false, 'underflow']
    return [true, a - b]
  },
  subi64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0n && b < 0n && a > INT32MAX + b) return [false, 'overflow']
    if (a < 0n && b > 0n && a < INT32MIN + b) return [false, 'underflow']
    return [true, a - b]
  },
  subf32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    const out = a - b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },
  subf64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    const out = a - b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },

  negi8:    a => 0 - a,
  negi16:   a => 0 - a,
  negi32:   a => 0 - a,
  negi64:   a => 0n - a,
  negf32:   a => 0.0 - a,
  negf64:   a => 0.0 - a,

  absi8:    a => Math.abs(a),
  absi16:   a => Math.abs(a),
  absi32:   a => Math.abs(a),
  absi64:   a => a > 0n ? a : -a,
  absf32:   a => Math.abs(a),
  absf64:   a => Math.abs(a),

  muli8:   (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 0 && a > INT8MAX / b) return [false, 'overflow']
    if (a < 0 && b < 0 && a < INT8MIN / b) return [false, 'underflow']
    return [true, a * b]
  },
  muli16:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 0 && a > INT16MAX / b) return [false, 'overflow']
    if (a < 0 && b < 0 && a < INT16MIN / b) return [false, 'underflow']
    return [true, a * b]
  },
  muli32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 0 && a > INT32MAX / b) return [false, 'overflow']
    if (a < 0 && b < 0 && a < INT32MIN / b) return [false, 'underflow']
    return [true, a * b]
  },
  muli64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0n && b > 0n && a > INT64MAX / b) return [false, 'overflow']
    if (a < 0n && b < 0n && a < INT64MIN / b) return [false, 'underflow']
    return [true, a * b]
  },
  mulf32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    const out = a * b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },
  mulf64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    const out = a * b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },

  divi8:   (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (b === 0) return [false, 'divide-by-zero']
    return [true, Math.floor(a / b)]
  },
  divi16:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (b === 0) return [false, 'divide-by-zero']
    return [true, Math.floor(a / b)]
  },
  divi32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (b === 0) return [false, 'divide-by-zero']
    return [true, Math.floor(a / b)]
  },
  divi64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (b === 0n) return [false, 'divide-by-zero']
    return [true, a / b]
  },
  divf32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (b === 0.0) return [false, 'divide-by-zero']
    const out = a / b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },
  divf64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (b === 0.0) return [false, 'divide-by-zero']
    const out = a / b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },

  modi8:   (a, b) => a % b,
  modi16:  (a, b) => a % b,
  modi32:  (a, b) => a % b,
  modi64:  (a, b) => a % b,

  powi8:   (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 1 && a > INT8MAX ** (1 / b)) return [false, 'overflow']
    if (a < 0 && b > 1 && a < INT8MIN ** (1 / b)) return [false, 'underflow']
    return [true, Math.floor(a ** b)]
  },
  powi16:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 1 && a > INT16MAX ** (1 / b)) return [false, 'overflow']
    if (a < 0 && b > 1 && a < INT16MIN ** (1 / b)) return [false, 'underflow']
    return [true, Math.floor(a ** b)]
  },
  powi32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 1 && a > INT32MAX ** (1 / b)) return [false, 'overflow']
    if (a < 0 && b > 1 && a < INT32MIN ** (1 / b)) return [false, 'underflow']
    return [true, Math.floor(a ** b)]
  },
  powi64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    if (a > 0 && b > 1n) {
      const af = parseFloat(a.toString())
      const bf = parseFloat(b.toString())
      const maxf = parseFloat(INT64MAX.toString())
      if (af > maxf ** (1 / bf)) return [false, 'overflow']
    }
    if (a < 0n && b > 1n) {
      const af = parseFloat(a.toString())
      const bf = parseFloat(b.toString())
      const minf = parseFloat(INT64MIN.toString())
      if (af < minf ** (1 / bf)) return [false, 'underflow']
    }
    return [true, a ** b]
  },
  powf32:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    const out = a ** b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },
  powf64:  (ra, rb) => {
    if (!ra[0]) return ra
    if (!rb[0]) return rb
    const a = ra[1]
    const b = rb[1]
    const out = a ** b
    if (out === Number.POSITIVE_INFINITY) return [false, 'overflow']
    if (out === Number.NEGATIVE_INFINITY) return [false, 'underflow']
    return [true, out]
  },

  sqrtf32:  a => Math.sqrt(a),
  sqrtf64:  a => Math.sqrt(a),

  // Boolean and bitwise opcodes
  andi8:   (a, b) => a & b,
  andi16:  (a, b) => a & b,
  andi32:  (a, b) => a & b,
  andi64:  (a, b) => a & b,
  andbool: (a, b) => a && b,

  ori8:    (a, b) => a | b,
  ori16:   (a, b) => a | b,
  ori32:   (a, b) => a | b,
  ori64:   (a, b) => a | b,
  orbool:  (a, b) => a || b,

  xori8:   (a, b) => a ^ b,
  xori16:  (a, b) => a ^ b,
  xori32:  (a, b) => a ^ b,
  xori64:  (a, b) => a ^ b,
  xorbool: (a, b) => !!(a ^ b),

  noti8:    a => ~a,
  noti16:   a => ~a,
  noti32:   a => ~a,
  noti64:   a => ~a,
  notbool:  a => !a,

  nandi8:  (a, b) => ~(a & b),
  nandi16: (a, b) => ~(a & b),
  nandi32: (a, b) => ~(a & b),
  nandi64: (a, b) => ~(a & b),
  nandboo: (a, b) => !(a && b),

  nori8:   (a, b) => ~(a | b),
  nori16:  (a, b) => ~(a | b),
  nori32:  (a, b) => ~(a | b),
  nori64:  (a, b) => ~(a | b),
  norbool: (a, b) => !(a || b),

  xnori8:  (a, b) => ~(a ^ b),
  xnori16: (a, b) => ~(a ^ b),
  xnori32: (a, b) => ~(a ^ b),
  xnori64: (a, b) => ~(a ^ b),
  xnorboo: (a, b) => !(a ^ b),

  // Equality and order opcodes
  eqi8:    (a, b) => a === b,
  eqi16:   (a, b) => a === b,
  eqi32:   (a, b) => a === b,
  eqi64:   (a, b) => a === b,
  eqf32:   (a, b) => a === b,
  eqf64:   (a, b) => a === b,
  eqstr:   (a, b) => a === b,
  eqbool:  (a, b) => a === b,

  neqi8:   (a, b) => a !== b,
  neqi16:  (a, b) => a !== b,
  neqi32:  (a, b) => a !== b,
  neqi64:  (a, b) => a !== b,
  neqf32:  (a, b) => a !== b,
  neqf64:  (a, b) => a !== b,
  neqstr:  (a, b) => a !== b,
  neqbool: (a, b) => a !== b,

  lti8:    (a, b) => a < b,
  lti16:   (a, b) => a < b,
  lti32:   (a, b) => a < b,
  lti64:   (a, b) => a < b,
  ltf32:   (a, b) => a < b,
  ltf64:   (a, b) => a < b,
  ltstr:   (a, b) => a < b,

  ltei8:   (a, b) => a <= b,
  ltei16:  (a, b) => a <= b,
  ltei32:  (a, b) => a <= b,
  ltei64:  (a, b) => a <= b,
  ltef32:  (a, b) => a <= b,
  ltef64:  (a, b) => a <= b,
  ltestr:  (a, b) => a <= b,

  gti8:    (a, b) => a > b,
  gti16:   (a, b) => a > b,
  gti32:   (a, b) => a > b,
  gti64:   (a, b) => a > b,
  gtf32:   (a, b) => a > b,
  gtf64:   (a, b) => a > b,
  gtstr:   (a, b) => a > b,

  gtei8:   (a, b) => a >= b,
  gtei16:  (a, b) => a >= b,
  gtei32:  (a, b) => a >= b,
  gtei64:  (a, b) => a >= b,
  gtef32:  (a, b) => a >= b,
  gtef64:  (a, b) => a >= b,
  gtestr:  (a, b) => a >= b,

  // String opcodes
  catstr:  (a, b) => a.concat(b),
  split:   (a, b) => a.split(b),
  repstr:  (a, b) => new Array(parseInt(b.toString())).fill(a).join(''),
  // TODO: templ, after maps are figured out
  matches: (a, b) => RegExp(b).test(a),
  indstr:  (a, b) => {
    const ind = BigInt(a.indexOf(b))
    return ind > -1 ? [ true, ind, ] : [ false, 'substring not found', ]
  },
  lenstr:   a => BigInt(a.length),
  trim:     a => a.trim(),
  copyfrom:(arr, ind) => JSON.parse(JSON.stringify(arr[ind])),
  copytof: (arr, ind, val) => { arr[ind] = val }, // These do the same thing in JS
  copytov: (arr, ind, val) => { arr[ind] = val },
  register:(arr, ind) => arr[ind], // Only on references to inner arrays

  // Array opcodes TODO more to come
  newarr:   size => new Array(), // Ignored because JS push doesn't behave as desired
  pusharr: (arr, val, size) => arr.push(val),
  poparr:   arr => arr.length > 0 ? [ true, arr.pop(), ] : [ false, 'cannot pop empty array', ],
  lenarr:   arr => BigInt(arr.length),
  indarrf: (arr, val) => {
    const ind = BigInt(arr.indexOf(val))
    return ind > -1 ? [ true, ind, ] : [ false, 'element not found', ]
  },
  indarrv: (arr, val) => {
    const ind = BigInt(arr.indexOf(val))
    return ind > -1 ? [ true, ind, ] : [ false, 'element not found', ]
  },
  delindx: (arr, idx) => {
    const spliced = arr.splice(parseInt(idx.toString()), 1)
    if (spliced.length === 1 && parseInt(idx.toString()) >= 0) {
      return [ true, spliced[0] ]
    } else {
      return [ false, `cannot remove idx ${idx} from array with length ${arr.length}` ]
    }
  },
  join:    (arr, sep) => arr.join(sep),
  map:     async (arr, fn) => await Promise.all(arr.map((v, i) => fn(v, BigInt(i)))),
  mapl:    async (arr, fn) => await Promise.all(arr.map((v, i) => fn(v, BigInt(i)))),
  reparr:  (arr, n) => Array.from(new Array(parseInt(n.toString()) * arr.length))
    .map((_, i) => typeof arr[i % arr.length] === 'bigint' ?
      BigInt(arr[i % arr.length]) :
      JSON.parse(JSON.stringify(arr[i % arr.length]))
    ),
  each:    async (arr, fn) => { // Thrown away but awaited to maintain consistent execution
    await Promise.all(arr.map((v, i) => fn(v, BigInt(i))))
  },
  eachl:   async (arr, fn) => { // Thrown away but awaited to maintain consistent execution
    await Promise.all(arr.map((v, i) => fn(v, BigInt(i))))
  },
  find:    async (arr, fn) => {
    let val = undefined
    const len = arr.length
    for (let i = 0; i < len && val === undefined; i++) {
      if (await fn(arr[i])) {
        val = arr[i]
      }
    }
    if (val === undefined) {
      return [
        false,
        'no element matches',
      ]
    } else {
      return [
        true,
        val,
      ]
    }
  },
  findl:   async (arr, fn) => {
    let val = undefined
    const len = arr.length
    for (let i = 0; i < len && val === undefined; i++) {
      if (await fn(arr[i])) {
        val = arr[i]
      }
    }
    if (val === undefined) {
      return [
        false,
        'no element matches',
      ]
    } else {
      return [
        true,
        val,
      ]
    }
  },
  every:   async (arr, fn) => {
    const len = arr.length
    for (let i = 0; i < len; i++) {
      if (!await fn(arr[i])) return false
    }
    return true
  },
  everyl:  async (arr, fn) => {
    const len = arr.length
    for (let i = 0; i < len; i++) {
      if (!await fn(arr[i])) return false
    }
    return true
  },
  some:    async (arr, fn) => {
    const len = arr.length
    for (let i = 0; i < len; i++) {
      if (await fn(arr[i])) return true
    }
    return false
  },
  somel:    async (arr, fn) => {
    const len = arr.length
    for (let i = 0; i < len; i++) {
      if (await fn(arr[i])) return true
    }
    return false
  },
  filter:  async (arr, fn) => {
    let out = []
    let len = arr.length
    for (let i = 0; i < len; i++) {
      if (await fn(arr[i])) out.push(arr[i])
    }
    return out
  },
  filterl: async (arr, fn) => {
    let out = []
    let len = arr.length
    for (let i = 0; i < len; i++) {
      if (await fn(arr[i])) out.push(arr[i])
    }
    return out
  },
  reducel: async (arr, fn) => {
    let cumu = arr[0]
    let len = arr.length
    for (let i = 1; i < len; i++) {
      cumu = await fn(cumu, arr[i])
    }
    return cumu
  },
  reducep: async (arr, fn) => {
    let cumu = arr[0]
    let len = arr.length
    for (let i = 1; i < len; i++) {
      cumu = await fn(cumu, arr[i])
    }
    return cumu
  },
  foldl:   async (obj, fn) => {
    const [arr, init] = obj
    let cumu = init
    let len = arr.length
    for (let i = 0; i < len; i++) {
      cumu = await fn(cumu, arr[i])
    }
    return cumu
  },
  foldp:   async (obj, fn) => {
    const [arr, init] = obj
    let cumu = init
    let len = arr.length
    for (let i = 0; i < len; i++) {
      cumu = await fn(cumu, arr[i])
    }
    return [cumu] // This path is expected to return an array of folded values per thread
  },
  catarr:  (a, b) => [...a, ...b],

  // Map opcodes TODO after maps are figured out

  // Ternary functions
  // TODO: pair and condarr after arrays are figured out
  condfn:  async (cond, fn) => cond ? await fn() : undefined,

  // Copy opcodes (for let reassignment)
  copyi8:   a => JSON.parse(JSON.stringify(a)),
  copyi16:  a => JSON.parse(JSON.stringify(a)),
  copyi32:  a => JSON.parse(JSON.stringify(a)),
  copyi64:  a => BigInt(a),
  copyvoid: a => JSON.parse(JSON.stringify(a)),
  copyf32:  a => JSON.parse(JSON.stringify(a)),
  copyf64:  a => JSON.parse(JSON.stringify(a)),
  copybool: a => JSON.parse(JSON.stringify(a)),
  copystr:  a => JSON.parse(JSON.stringify(a)),
  // Actually the recommended deep clone mechanism: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/assign#Deep_Clone
  // Doesn't work with BigInt :(
  // copyarr:  a => JSON.parse(JSON.stringify(a)),
  // Implementation is now recursive with a try-catch wrapper, so not great for perf
  copyarr,
  zeroed:  () => null,

  // Trig opcodes
  lnf64:    a => Math.log(a),
  logf64:   a => Math.log(a) / Math.log(10),
  sinf64:   a => Math.sin(a),
  cosf64:   a => Math.cos(a),
  tanf64:   a => Math.tan(a),
  asinf64:  a => Math.asin(a),
  acosf64:  a => Math.acos(a),
  atanf64:  a => Math.atan(a),
  sinhf64:  a => Math.sinh(a),
  coshf64:  a => Math.cosh(a),
  tanhf64:  a => Math.tanh(a),

  // Error, Maybe, Result, Either opcodes
  error:    a => a,
  reff:      a => a, // Just an alias for error but without the type mangling in the compiler
  refv:      a => a, // Just an alias for error but without the type mangling in the compiler
  noerr:   () => '',
  errorstr: a => a.toString(),
  someM:    a => [
    true,
    a,
  ],
  noneM:   () => [
    false,
  ],
  isSome:   a => a[0],
  isNone:   a => !a[0],
  getOrM:  (a, b) => a[0] ? a[1] : b,
  getM:     a => {
    if (a[0]) {
      return a[1]
    } else {
      throw new Error('runtime error: illegal access')
    }
  },
  okR:      a => [
    true,
    a,
  ],
  err:      a => [
    false,
    a,
  ],
  isOk:     a => a[0],
  isErr:    a => !a[0],
  getOrR:  (a, b) => a[0] ? a[1] : b,
  getOrRS: (a, b) => a[0] ? a[1] : b,
  getR:    (a) => {
    if (a[0]) {
      return a[1]
    } else {
      throw new Error('runtime error: illegal access')
    }
  },
  getErr:  (a, b) => a[0] ? b : a[1],
  resfrom: (arr, rind) => {
    if (!rind[0]) return rind
    const ind = rind[1]
    if (ind >= 0 && ind < arr.length) {
      return [
        true,
        arr[ind],
      ]
    } else {
      return [
        false,
        'out-of-bounds access',
      ]
    }
  },
  mainE:    a => [
    true,
    a,
  ],
  altE:     a => [
    false,
    a,
  ],
  isMain:   a => a[0],
  isAlt:    a => !a[0],
  mainOr:  (a, b) => a[0] ? a[1] : b,
  altOr:   (a, b) => a[0] ? b : a[1],
  getMain:  a => {
    if (a[0]) {
      return a[1]
    } else {
      throw new Error('runtime error: illegal access')
    }
  },
  getAlt:  a => {
    if (!a[0]) {
      return a[1]
    } else {
      throw new Error('runtime error: illegal access')
    }
  },

  // Hashing opcodes (hashv is recursive, needs to be defined elsewhere)
  hashf,
  hashv,

  // In Node.js the datastore opcodes don't have to be IO opcodes, but in the Rust runtime they do,
  // because of the multithreaded nature of the Rust runtime. Not sure if they should be "fake"
  // async here or not.
  dssetf:  (ns, key, val) => {
    ds[`${ns}:${key}`] = val
  },
  dssetv:  (ns, key, val) => {
    ds[`${ns}:${key}`] = val
  },
  dshas:   (ns, key) => ds.hasOwnProperty(`${ns}:${key}`),
  dsdel:   (ns, key) => {
    const fullKey = `${ns}:${key}`
    const toDelete = ds.hasOwnProperty(fullKey)
    if (toDelete) delete ds[fullKey]
    return toDelete
  },
  dsgetf:  (ns, key) => {
    const fullKey = `${ns}:${key}`
    if (ds.hasOwnProperty(fullKey)) {
      return [ true, ds[`${ns}:${key}`], ]
    } else {
      return [ false, 'namespace-key pair not found', ]
    }
  },
  dsgetv:  (ns, key) => {
    const fullKey = `${ns}:${key}`
    if (ds.hasOwnProperty(fullKey)) {
      return [ true, ds[`${ns}:${key}`], ]
    } else {
      return [ false, 'namespace-key pair not found', ]
    }
  },
  newseq:  (limit) => [0n, limit],
  seqnext: (seq) => {
    if (seq[0] < seq[1]) {
      const out = [true, seq[0]]
      seq[0]++
      return out
    } else {
      return [false, 'error: sequence out-of-bounds']
    }
  },
  seqeach: async (seq, func) => {
    while (seq[0] < seq[1]) {
      await func(seq[0])
      seq[0]++
    }
  },
  seqwhile:async (seq, condFn, bodyFn) => {
    while (seq[0] < seq[1] && await condFn()) {
      await bodyFn()
      seq[0]++
    }
  },
  seqdo:   async (seq, bodyFn) => {
    let ok = true
    do {
      ok = await bodyFn()
      seq[0]++
    } while (seq[0] < seq[1] && ok)
  },
  selfrec: async (self, arg) => {
    const [seq, recurseFn] = self
    if (seq[0] < seq[1]) {
      seq[0]++
      return recurseFn(self, arg)
    } else {
      return [false, 'error: sequence out-of-bounds']
    }
  },
  seqrec: (seq, recurseFn) => [seq, recurseFn],

  // IO opcodes
  httpreq:  async req => {
    const [ method, url, headers, body, ] = req
    try {
      const response = await fetch(url, {
        method,
        headers,
        body: body.length > 0 ? body : undefined,
      });
      const rstatus = response.status
      const rheaders = [...response.headers.entries()].map(kv => [kv[0] + '', kv[1] + ''])
      const rbody = await response.text()
      return [ true, [ rstatus, rheaders, rbody, 0n ] ]
    } catch (e) {
      return [ false, e.toString() ]
    }
  },
  httplsn:  async () => {
    const server = http.createServer((req, res) => {
      const connId = Number(hashf(Math.random().toString()))
      httpConns[connId] = {
        req,
        res,
      }
      let body = ''
      req.on('data', d => {
        body += d
      })
      req.on('end', () => {
        e.emit('__conn', [
          req.method,
          req.url,
          Object.entries(req.headers),
          body,
          connId,
        ])
      })
    })
    const listenResult = await new Promise(resolve => {
      server.on('error', e => resolve(e))
      server.listen({
        port: 8000,
      }, () => resolve(true))
    })
    if (listenResult === true) {
      console.log("HTTP server listening on port 8000")
    } else {
      console.error(`HTTP server failed to listen to port 8000: ${e}`)
    }
  },
  httpsend: async (ires) => {
    const [ status, headers, body, connId, ] = ires
    const conn = httpConns[connId]
    if (!conn) return [ false, 'connection not found', ]
    delete httpConns[connId]
    return new Promise(resolve => {
      conn.res.on('close', () => resolve([ false, 'client hangup', ]))
      conn.res
        .writeHead(Number(status), headers.reduce((acc, kv) => {
          acc[kv[0]] = kv[1]
          return acc
        }, {}))
        .end(body, () => resolve([ true, 'ok', ]))
    })
  },
  waitop:   async (a) => await new Promise(resolve => setTimeout(resolve, Number(a))),
  syncop:   async (f, a) => await f(a),
  execop:   async (cmd) => {
    try {
      const res = await exec(cmd)
      const { stdout, stderr } = res
      return [ 0, stdout, stderr ]
    } catch (e) {
      return [ e.signal, e.stdout, e.stderr ]
    }
  },

  // "Special" opcodes
  stdoutp:  out => process.stdout.write(out),
  stderrp:  err => process.stderr.write(err),
  exitop:   code => process.exit(parseInt(code.toString())),

  // Event bookkeeping
  emit:    (name, payload) => e.emit(name, payload),
  on:      (name, cb) => e.on(name, cb),
  emitter:  e,
}

module.exports.asyncopcodes = Object.keys(module.exports).filter(k => module.exports[k].constructor.name === 'AsyncFunction')
