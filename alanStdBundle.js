/**
 * Convert array of 16 byte values to UUID string format of the form:
 * XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX
 */
var byteToHex = [];
for (var i = 0; i < 256; ++i) {
  byteToHex.push((i + 0x100).toString(16).slice(1));
}
function unsafeStringify(arr, offset = 0) {
  // Note: Be careful editing this code!  It's been tuned for performance
  // and works in ways you may not expect. See https://github.com/uuidjs/uuid/pull/434
  //
  // Note to future-self: No, you can't remove the `toLowerCase()` call.
  // REF: https://github.com/uuidjs/uuid/pull/677#issuecomment-1757351351
  return (byteToHex[arr[offset + 0]] + byteToHex[arr[offset + 1]] + byteToHex[arr[offset + 2]] + byteToHex[arr[offset + 3]] + '-' + byteToHex[arr[offset + 4]] + byteToHex[arr[offset + 5]] + '-' + byteToHex[arr[offset + 6]] + byteToHex[arr[offset + 7]] + '-' + byteToHex[arr[offset + 8]] + byteToHex[arr[offset + 9]] + '-' + byteToHex[arr[offset + 10]] + byteToHex[arr[offset + 11]] + byteToHex[arr[offset + 12]] + byteToHex[arr[offset + 13]] + byteToHex[arr[offset + 14]] + byteToHex[arr[offset + 15]]).toLowerCase();
}

// Unique ID creation requires a high quality random # generator. In the browser we therefore
// require the crypto API and do not support built-in fallback to lower quality random number
// generators (like Math.random()).

var getRandomValues;
var rnds8 = new Uint8Array(16);
function rng() {
  // lazy load so that environments that need to polyfill have a chance to do so
  if (!getRandomValues) {
    // getRandomValues needs to be invoked in a context where "this" is a Crypto implementation.
    getRandomValues = typeof crypto !== 'undefined' && crypto.getRandomValues && crypto.getRandomValues.bind(crypto);
    if (!getRandomValues) {
      throw new Error('crypto.getRandomValues() not supported. See https://github.com/uuidjs/uuid#getrandomvalues-not-supported');
    }
  }
  return getRandomValues(rnds8);
}

var randomUUID = typeof crypto !== 'undefined' && crypto.randomUUID && crypto.randomUUID.bind(crypto);
var native = {
  randomUUID
};

function v4(options, buf, offset) {
  if (native.randomUUID && !buf && !options) {
    return native.randomUUID();
  }
  options = options || {};
  var rnds = options.random || (options.rng || rng)();

  // Per 4.4, set bits for version and `clock_seq_hi_and_reserved`
  rnds[6] = rnds[6] & 0x0f | 0x40;
  rnds[8] = rnds[8] & 0x3f | 0x80;

  // Copy bytes to buffer, if provided
  if (buf) {
    offset = offset || 0;
    for (var i = 0; i < 16; ++i) {
      buf[offset + i] = rnds[i];
    }
    return buf;
  }
  return unsafeStringify(rnds);
}

class AlanError {
  constructor(message) {
    this.message = message;
  }
}

function nanToError(n) {
  if (Number.isNaN(n)) {
    return new AlanError("Not a Number");
  } else {
    return n;
  }
}

function ifbool(b, t, f) {
  if (b?.val ?? b) {
    return t();
  } else {
    return f();
  }
}

// For those reading this binding support code, you might be wondering *why* all of the primitive
// types are now boxed in their own classes. The reason is that in Alan (and Rust), you can mark
// any input argument as mutable instead of the default being immutable, but in Javascript, all
// arguments are "immutable" but objects are actually pointers under-the-hood and anything you have
// pointer access to is mutable. Wrapping primitive types in objects makes it possible for the Alan
// compiler to give mutable access to them from a function (which is how all operators are defined
// in Alan). It would be *possible* to avoid this by inlining the function definition if any of the
// arguments are a mutable variant of a primitive type, but that would both make the compiler more
// complicated (and increase the maintenance burden) *and* increase the size of the generated code
// (as all of these functions would have their function bodies copied everywhere), which is a big
// problem for code that is read over the wire and re-loaded into a JIT every single page load.
// Further, that JIT is very well put together by a massive team of engineers over decades -- it'll
// be able to unbox the value and maintain the desired mutable behavior just fine, probably. ;)
class Int {
  constructor(val, bits, size, lower, upper) {
    if (bits === 64) {
      let v = BigInt(val);
      if (v > upper) {
        this.val = upper;
      } else if(v < lower) {
        this.val = lower;
      } else {
        this.val = v;
      }
    } else {
      this.val = Math.max(lower, Math.min(upper, Number(val)));
    }
    this.bits = bits;
    this.size = size;
    this.lower = lower;
    this.upper = upper;
  }

  wrap(v) {
    v = this.bits === 64 ? BigInt(v) : Number(v);
    while (v > this.upper) {
      v -= this.size;
    }
    while (v < this.lower) {
      v += this.size;
    }
    return v;
  }

  wrappingAdd(a) {
    return this.build(this.val + a.val);
  }

  wrappingSub(a) {
    return this.build(this.val - a.val);
  }

  wrappingMul(a) {
    return this.build(this.val * a.val);
  }

  wrappingDiv(a) {
    if (this.bits === 64) {
      return this.build(this.val / a.val);
    } else {
      return this.build(Math.floor(this.val / a.val));
    }
  }

  wrappingMod(a) {
    return this.build(this.val % a.val);
  }

  wrappingPow(a) {
    if (this.bits === 64) {
      return this.build(this.val ** a.val);
    } else {
      return this.build(Math.floor(this.val ** a.val));
    }
  }

  not() {
    return this.build(~this.val);
  }

  wrappingShl(a) {
    if (this.bits >= 32) {
      let b = this.val < 0 ? BigInt(this.val) + BigInt(this.size) : BigInt(this.val);
      let v = b << BigInt(a);
      return this.build(v);
    } else {
      return this.build((this.val < 0 ? this.val + this.size : this.val) << a.val);
    }
  }

  wrappingShr(a) {
    // There's something broken with right-shift. MDN says it's a "sign propagating right shift"
    // so a negative number will remain negative after the shift (which is a trash choice, but
    // okay). However, even if I convert an i32 into a u32 inside of Number, where it's *not*
    // negative, but the 32nd bit is 1, it will treat it as the sign bit in the operation and
    // output a negative number.
    //
    // But all is not lost. I'm converting the value into a BigInt after making it a u32 and then
    // converting back to a Number at the end to get this to work right.
    if (this.bits >= 32) {
      let b = this.val < 0 ? BigInt(this.val) + BigInt(this.size) : BigInt(this.val);
      let v = b >> BigInt(a);
      return this.build(v);
    } else {
      return this.build((this.val < 0 ? this.val + this.size : this.val) >> a.val);
    }
  }

  rotateLeft(a) {
    if (this.bits >=  32) {
      let b = this.val < 0 ? BigInt(this.val) + BigInt(this.size) : BigInt(this.val);
      let c = BigInt(a.val);
      while (c > BigInt(this.bits - 1)) {
        c -= BigInt(this.bits);
      }
      if (c == 0n) {
        return this.build(this.val);
      }
      let lhs = (BigInt(this.size) - 1n) & ((BigInt(this.size) - 1n) << (BigInt(this.bits) - c));
      let rhs = (BigInt(this.size) - 1n) & ((BigInt(this.size) - 1n) ^ lhs);
      let p1 = b & lhs;
      let p2 = b & rhs;
      return this.build((p1 >> (BigInt(this.bits) - c)) + (p2 << c));
    } else {
      let b = this.val < 0 ? this.val + this.size : this.val;
      let c = a.val;
      while (c > this.bits - 1) {
        c -= this.bits;
      }
      if (c == 0) {
        return this.build(this.val);
      }
      let lhs = (this.size - 1) & ((this.size - 1) << (this.bits - c));
      let rhs = (this.size - 1) & ((this.size - 1) ^ lhs);
      let p1 = b & lhs;
      let p2 = b & rhs;
      return this.build((p1 >> (this.bits - c)) + (p2 << c));
    }
  }

  rotateRight(a) {
    if (this.bits >=  32) {
      let b = this.val < 0 ? BigInt(this.val) + BigInt(this.size) : BigInt(this.val);
      let c = BigInt(a.val);
      while (c > BigInt(this.bits - 1)) {
        c -= BigInt(this.bits);
      }
      if (c == 0n) {
        return this.build(this.val);
      }
      let rhs = (BigInt(this.size) - 1n) & ((BigInt(this.size) - 1n) << c);
      let lhs = (BigInt(this.size) - 1n) & ((BigInt(this.size) - 1n) ^ rhs);
      let p1 = b & lhs;
      let p2 = b & rhs;
      return this.build((p1 << (BigInt(this.bits) - c)) + (p2 >> c));
    } else {
      let b = this.val < 0 ? this.val + this.size : this.val;
      let c = a.val;
      while (c > this.bits - 1) {
        c -= this.bits;
      }
      if (c == 0) {
        return this.build(this.val);
      }
      let rhs = (this.size - 1) & ((this.size - 1) << c);
      let lhs = (this.size - 1) & ((this.size - 1) ^ rhs);
      let p1 = b & lhs;
      let p2 = b & rhs;
      return this.build((p1 << (this.bits - c)) + (p2 >> c));
    }
  }

  valueOf() {
    return this.val;
  }

  toString() {
    return this.val.toString();
  }
}

class I8 extends Int {
  constructor(v) {
    super(v, 8, 256, -128, 127);
  }

  build(v) {
    return new I8(this.wrap(v));
  }
}

class U8 extends Int {
  constructor(v) {
    super(v, 8, 256, 0, 255);
  }

  build(v) {
    return new U8(this.wrap(v));
  }
}

class I16 extends Int {
  constructor(v) {
    super(v, 16, 65_536, -32_768, 32_767);
  }

  build(v) {
    return new I16(this.wrap(v));
  }
}

class U16 extends Int {
  constructor(v) {
    super(v, 16, 65_536, 0, 65_535);
  }

  build(v) {
    return new U16(this.wrap(v));
  }
}

class I32 extends Int {
  constructor(v) {
    super(v, 32, 4_294_967_296, -2_147_483_648, 2_147_483_647);
  }

  build(v) {
    return new I32(this.wrap(v));
  }
}

class U32 extends Int {
  constructor(v) {
    super(v, 32, 4_294_967_296, 0, 4_294_967_295);
  }

  build(v) {
    return new U32(this.wrap(v));
  }
}

class I64 extends Int {
  constructor(v) {
    super(v, 64, 18_446_744_073_709_551_616n, -9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n);
  }

  build(v) {
    return new I64(this.wrap(v));
  }
}

class U64 extends Int {
  constructor(v) {
    super(v, 64, 18_446_744_073_709_551_616n, 0n, 18_446_744_073_709_551_615n);
  }

  build(v) {
    return new U64(this.wrap(v));
  }
}

class Float {
  constructor(val, bits) {
    this.val = val;
    this.bits = bits;
  }

  valueOf() {
    return this.val;
  }

  toString() {
    return this.val.toString();
  }
}

class F32 extends Float {
  constructor(v) {
    super(Number(v), 32);
  }

  build(v) {
    return new F32(v);
  }
}

class F64 extends Float {
  constructor(v) {
    super(Number(v), 64);
  }

  build(v) {
    return new F64(v);
  }
}

class Bool {
  constructor(val) {
    this.val = Boolean(val);
  }

  valueOf() {
    return this.val;
  }

  toString() {
    return this.val.toString();
  }
}

class Str {
  constructor(val) {
    this.val = String(val);
  }

  valueOf() {
    return this.val;
  }

  toString() {
    return this.val.toString();
  }
}

class GPU {
  constructor(adapter, device, queue) {
    this.adapter = adapter;
    this.device = device;
    this.queue = queue;
  }

  static async list() {
    let out = [];
    let hp = await navigator?.gpu?.requestAdapter({ powerPreference: "high-performance", });
    let lp = await navigator?.gpu?.requestAdapter({ powerPreference: 'low-power', });
    let np = await navigator?.gpu?.requestAdapter();
    if (hp) out.push(hp);
    if (lp) out.push(lp);
    if (np) out.push(np);
    return out;
  }

  static async init(adapters) {
    let out = [];
    for (let adapter of adapters) {
      let features = adapter.features;
      let limits = adapter.limits;
      let info = adapter.info;
      let device = await adapter.requestDevice({
        label: `${info.device} on ${info.architecture}`,
        // If I don't pass these through, it defaults to a really small set of features and limits
        requiredFeatures: features,
        requiredLimits: limits,
      });
      out.push(new GPU(adapter, device, device.queue));
    }
    return out;
  }
}

let GPUS = null;

async function gpu() {
  if (GPUS === null) {
    GPUS = await GPU.init(await GPU.list());
  }
  if (GPUS.length > 0) {
    return GPUS[0];
  } else {
    throw new AlanError("This program requires a GPU but there are no WebGPU-compliant GPUs on this machine");
  }
}

async function createBufferInit(usage, vals) {
  let g = await gpu();
  let b = await g.device.createBuffer({
    mappedAtCreation: true,
    size: vals.length * 4,
    usage,
    label: `buffer_${v4().replaceAll('-', '_')}`,
  });
  let ab = b.getMappedRange();
  let i32v = new Int32Array(ab);
  for (let i = 0; i < vals.length; i++) {
    i32v[i] = vals[i].valueOf();
  }
  b.unmap();
  return b;
}

async function createEmptyBuffer(usage, size) {
  let g = await gpu();
  let b = await g.device.createBuffer({
    size,
    usage,
    label: `buffer_${v4().replaceAll('-', '_')}`,
  });
  return b;
}

function mapReadBufferType() {
  return GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST;
}

function mapWriteBufferType() {
  return GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC;
}

function storageBufferType() {
  return GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC;
}

function bufferlen(b) {
  return new I64(b.size / 4);
}

function bufferid(b) {
  return b.label;
}

class GPGPU {
  constructor(source, buffers, workgroupSizes, workgroupDims, entrypoint) {
    this.source = source;
    this.entrypoint = entrypoint ?? "main";
    this.buffers = buffers;
    this.workgroupSizes = workgroupSizes;
    this.workgroupDims = workgroupDims;
  }
}

async function gpuRun(gg) {
  let g = await gpu();
  let module = g.device.createShaderModule({
    code: gg.source,
  });
  let computePipeline = g.device.createComputePipeline({
    layout: "auto",
    compute: {
      entryPoint: gg.entrypoint,
      module,
    },
  });
  let encoder = g.device.createCommandEncoder();
  let cpass = encoder.beginComputePass();
  cpass.setPipeline(computePipeline);
  for (let i = 0; i < gg.buffers.length; i++) {
    let bindGroupLayout = computePipeline.getBindGroupLayout(i);
    let bindGroupBuffers = gg.buffers[i];
    let bindGroupEntries = [];
    for (let j = 0; j < bindGroupBuffers.length; j++) {
      bindGroupEntries.push({
        binding: j,
        resource: { buffer: bindGroupBuffers[j] }
      });
    }
    let bindGroup = g.device.createBindGroup({
      layout: bindGroupLayout,
      entries: bindGroupEntries,
    });
    cpass.setBindGroup(i, bindGroup);
  }
  cpass.dispatchWorkgroups(
    Math.ceil(gg.workgroupSizes[0] / gg.workgroupDims[0]),
    Math.ceil(gg.workgroupSizes[1] / gg.workgroupDims[1]),
    Math.ceil(gg.workgroupSizes[2] / gg.workgroupDims[2])
  );
  g.queue.submit([encoder.finish()]);
}

async function readBuffer(b) {
  let g = await gpu();
  let tempBuffer = await createEmptyBuffer(mapReadBufferType(), b.size);
  let encoder = g.device.createCommandEncoder();
  encoder.copyBufferToBuffer(b, 0, tempBuffer, 0, b.size);
  g.queue.submit([encoder.finish()]);
  await tempBuffer.mapAsync(GPUMapMode.READ);
  let data = tempBuffer.slice(0);
  tempBuffer.unmap();
  let vals = new Int32Array(data);
  let out = [];
  for (let i = 0; i < vals.length; i++) {
    out[i] = new I32(vals[i]);
  }
  return out;
}

async function replaceBuffer(b, v) {
  if (v.length != bufferlen(b)) {
    return new AlanError("The input array is not the same size as the buffer");
  }
  await b.mapAsync(GPUMapMode.WRITE);
  let data = b.slice(0);
  for (let i = 0; i < v.length; i++) {
    data[i] = v[i].valueOf();
  }
  b.unmap();
}

export { AlanError, Bool, F32, F64, Float, GPGPU, GPU, I16, I32, I64, I8, Int, Str, U16, U32, U64, U8, bufferid, bufferlen, createBufferInit, createEmptyBuffer, gpu, gpuRun, ifbool, mapReadBufferType, mapWriteBufferType, nanToError, readBuffer, replaceBuffer, storageBufferType, v4 as uuidv4 };
