const EventEmitter = require('events')
const util = require('util')
const exec = util.promisify ? util.promisify(require('child_process').exec) : () => {} // browsers

const e = new EventEmitter()

module.exports = {
  // Type conversion opcodes (mostly no-ops in JS, unless we implement a strict mode)
  i8f64:   a => a,
  i16f64:  a => a,
  i32f64:  a => a,
  i64f64:  a => a,
  f32f64:  a => a,
  strf64:  a => parseFloat(a),
  boolf64: a => a ? 1.0 : 0.0,

  i8f32:   a => a,
  i16f32:  a => a,
  i32f32:  a => a,
  i64f32:  a => a,
  f64f32:  a => a,
  strf32:  a => parseFloat(a),
  boolf32: a => a ? 1.0 : 0.0,

  i8i64:   a => a,
  i16i64:  a => a,
  i32i64:  a => a,
  f32i64:  a => Math.floor(a),
  f64i64:  a => Math.floor(a),
  stri64:  a => parseInt(a), // intentionally allowing other bases here
  booli64: a => a ? 1 : 0,

  i8i32:   a => a,
  i16i32:  a => a,
  i64i32:  a => a,
  f32i32:  a => Math.floor(a),
  f64i32:  a => Math.floor(a),
  stri32:  a => parseInt(a),
  booli64: a => a ? 1 : 0,

  i8i16:   a => a,
  i32i16:  a => a,
  i64i16:  a => a,
  f32i16:  a => Math.floor(a),
  f64i16:  a => Math.floor(a),
  stri16:  a => parseInt(a),
  booli16: a => a ? 1 : 0,

  i16i8:   a => a,
  i32i8:   a => a,
  i64i8:   a => a,
  f32i8:   a => Math.floor(a),
  f64i8:   a => Math.floor(a),
  stri8:   a => parseInt(a),
  booli8:  a => a ? 1 : 0,

  i8bool:  a => a !== 0,
  i16bool: a => a !== 0,
  i32bool: a => a !== 0,
  i64bool: a => a !== 0,
  f32bool: a => a !== 0.0,
  f64bool: a => a !== 0.0,
  strbool: a => a === "true",

  i8str:   a => a.toString(),
  i16str:  a => a.toString(),
  i32str:  a => a.toString(),
  i64str:  a => a.toString(),
  f32str:  a => a.toString(),
  f64str:  a => a.toString(),
  boolstr: a => a.toString(),

  // Arithmetic opcodes
  addi8:   (a, b) => a + b,
  addi16:  (a, b) => a + b,
  addi32:  (a, b) => a + b,
  addi64:  (a, b) => a + b,
  addf32:  (a, b) => a + b,
  addf64:  (a, b) => a + b,

  subi8:   (a, b) => a - b,
  subi16:  (a, b) => a - b,
  subi32:  (a, b) => a - b,
  subi64:  (a, b) => a - b,
  subf32:  (a, b) => a - b,
  subf64:  (a, b) => a - b,

  negi8:   a => 0 - a,
  negi16:  a => 0 - a,
  negi32:  a => 0 - a,
  negi64:  a => 0 - a,
  negf32:  a => 0.0 - a,
  negf64:  a => 0.0 - a,

  muli8:   (a, b) => a * b,
  muli16:  (a, b) => a * b,
  muli32:  (a, b) => a * b,
  muli64:  (a, b) => a * b,
  mulf32:  (a, b) => a * b,
  mulf64:  (a, b) => a * b,

  divi8:   (a, b) => Math.floor(a / b),
  divi16:  (a, b) => Math.floor(a / b),
  divi32:  (a, b) => Math.floor(a / b),
  divi64:  (a, b) => Math.floor(a / b),
  divf32:  (a, b) => a / b,
  divf64:  (a, b) => a / b,

  modi8:   (a, b) => a % b,
  modi16:  (a, b) => a % b,
  modi32:  (a, b) => a % b,
  modi64:  (a, b) => a % b,

  powi8:   (a, b) => Math.floor(a ** b), // If 'b' is negative, it would produce a fraction
  powi16:  (a, b) => Math.floor(a ** b),
  powi32:  (a, b) => Math.floor(a ** b),
  powi64:  (a, b) => Math.floor(a ** b),
  powf32:  (a, b) => a ** b,
  powf64:  (a, b) => a ** b,

  sqrtf32: a => Math.sqrt(a),
  sqrtf64: a => Math.sqrt(a),

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

  noti8:   a => ~a,
  noti16:  a => ~a,
  noti32:  a => ~a,
  noti64:  a => ~a,
  notbool: a => !a,

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
  repstr:  (a, b) => new Array(b).fill(a).join(''),
  // TODO: templ, after maps are figured out
  matches: (a, b) => RegExp(b).test(a),
  indstr:  (a, b) => a.indexOf(b),
  lenstr:  a => a.length,
  trim:    a => a.trim(),
  copyfrom:(arr, ind) => arr[ind],
  copytof:  (arr, ind, val) => { arr[ind] = val }, // These do the same thing in JS
  copytov:  (arr, ind, val) => { arr[ind] = val },
  register:(arr, ind) => arr[ind], // Only on references to inner arrays

  // Array opcodes TODO more to come
  newarr:  size => new Array(), // Ignored because JS push doesn't behave as desired
  pusharr: (arr, val, size) => arr.push(val),
  lenarr: arr => arr.length,
  indarrf:(arr, val) => arr.indexOf(val),
  indarrv:(arr, val) => arr.indexOf(val),
  join:   (arr, sep) => arr.join(sep),
  map:    (arr, fn) => arr.map(fn),

  // Map opcodes TODO after maps are figured out

  // Ternary functions
  // TODO: pair and condarr after arrays are figured out
  condfn:  (cond, fn) => cond ? fn() : undefined,

  // Copy opcodes (for let reassignment)
  copyi8:   a => a,
  copyi16:  a => a,
  copyi32:  a => a,
  copyi64:  a => a,
  copyvoid: a => a,
  copyf32:  a => a,
  copyf64:  a => a,
  copybool: a => a,
  copystr:  a => a,
  // Actually the recommended deep clone mechanism: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/assign#Deep_Clone
  copyarr:  a => JSON.parse(JSON.stringify(a)),
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
  noerr:    () => '',
  errorstr: a => a.toString(),
  someM:     a => ({
    isSome: true,
    val: JSON.parse(JSON.stringify(a)),
  }),
  noneM:     () => ({
    isSome: false,
  }),
  isSome:   a => a.isSome,
  isNone:   a => !a.isSome,
  getOrM:   (a, b) => a.isSome ? a.val : b,
  okR:      a => ({
    isOk: true,
    val: JSON.parse(JSON.stringify(a)),
  }),
  err:      a => ({
    isOk: false,
    error: a,
  }),
  isOk:     a => a.isOk,
  isErr:    a => !a.isOk,
  getOrR:   (a, b) => a.isOk ? a.val : b,
  getR:     (a) => {
    if (a.isOk) {
      return a.val
    } else {
      throw new Error('runtime error: illegal access')
    }
  },
  getErr:   (a, b) => a.isOk ? b : a.error,
  resfrom:  (arr, ind) => ind >= 0 && ind < arr.length ? {
    isOk: true,
    val: arr[ind],
  } : {
    isOk: false,
    error: 'out-of-bounds access',
  },
  mainE:     a => ({
    isMain: true,
    main: JSON.parse(JSON.stringify(a)),
  }),
  altE:      a => ({
    isMain: false,
    alt: JSON.parse(JSON.stringify(a)),
  }),
  isMain:   a => a.isMain,
  isAlt:    a => !a.isMain,
  mainOr:   (a, b) => a.isMain ? a.main : b,
  altOr:    (a, b) => a.isMain ? b : a.alt,

  // IO opcodes
  asyncopcodes: ['waitop', 'execop'],
  waitop: a => new Promise(resolve => setTimeout(resolve, a)),
  execop: a => exec(a),

  // "Special" opcodes
  stdoutp: out => process.stdout.write(out),
  exitop:  code => process.exit(code),

  // Event bookkeeping
  emit:    (name, payload) => e.emit(name, payload),
  on:      (name, cb) => e.on(name, cb),
  emitter: e,
}
