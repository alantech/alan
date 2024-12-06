import { v4 as uuidv4 } from 'uuid';
export { v4 as uuidv4 } from 'uuid';

export class AlanError {
  constructor(message) {
    this.message = message;
  }
}

export function nanToError(n) {
  if (Number.isNaN(n)) {
    return new AlanError("Not a Number");
  } else {
    return n;
  }
}

export function ifbool(b, t, f) {
  if (b?.val ?? b) {
    return t();
  } else {
    return f();
  }
}

export function clone(v) {
  if (v instanceof Array) {
    return v.map(clone);
  } else if (v instanceof Set) {
    return v.union(new Set());
  } else if (v instanceof Map) {
    return new Map(v.entries().map((kv) => [clone(kv[0]), clone(kv[1])]));
  } else if (v.build instanceof Function) {
    return v.build(v.val);
  } else if (v instanceof Object) {
    return Object.fromEntries(Object.entries(v).map((kv) => [kv[0], clone(kv[1])]));
  } else {
    return structuredClone(v);
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
export class Int {
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

  clz() {
    // There's no built-in operation for this, so we're gonna do a binary search. We first convert
    // any negative numbers into their positive equivalent for the bit width in question, then we
    // check if the number is less than or equal to `2 ** (bitwidth / 2)`, if so we save
    // `bitwidth / 2` as the number of leading zeroes, then continue with `2 ** (bitwidth / 4)` and
    // saving `bitwidth - bitwidth / 4` number of leading zeroes, etc. The number of iterations is
    // fixed per bit width: 8-bit takes 3 loops, 16-bit 4 loops, 32-bit 5 loops, 64-bit 6 loops.
    // There's also a short-circuit for 0 to make the loop easier to implement.
    const val = this.val < (this.bits == 64 ? 0n : 0) ? BigInt(this.val + this.size) : BigInt(this.val);
    if (val == 0n) {
      return this.build(this.bits);
    }
    let checkBit = BigInt(this.bits / 2);
    let clz = 0;
    let step = 0;
    let maxSteps = Math.log2(this.bits);
    do {
      step++;
      if (val < 2n ** checkBit) {
        clz = BigInt(this.bits) - checkBit;
        checkBit = checkBit - BigInt(Math.round(this.bits / (2 ** (step + 1))));
      } else {
        checkBit = checkBit + BigInt(Math.round(this.bits / (2 ** (step + 1))));
      }
    } while(step < maxSteps);
    return this.build(clz);
  }

  ctz() {
    // This returns all of the trailing zeros for a number. Like clz above, first convert everything
    // to an unsigned BigInt to make the work simpler
    let val = this.val < (this.bits == 64 ? 0n : 0) ? BigInt(this.val + this.size) : BigInt(this.val);
    if (val == 0n) {
      return this.build(this.bits);
    }
    // Trailing zeros is a bit different. We'll use a shift and check approach, checking the modulus
    // 2 of the value to determine the last bit and increment the count if it's zero, then right
    // shift and check again until we get 1 and abort the loop.
    let ctz = 0;
    for (let i = 0; i < this.bits; i++) {
      if (val %2n == 0n) {
        ctz++;
        val = val >> 1n;
      } else {
        break;
      }
    }
    return this.build(ctz);
  }

  ones() {
    // This returns a count of all ones for the number. No real option other than iterating through
    // each bit and summing the results. Like clz above, convert everything to an unsigned BigInt.
    let val = this.val < (this.bits == 64 ? 0n : 0) ? BigInt(this.val + this.size) : BigInt(this.val);
    let ones = 0;
    for (let i = 0; i < this.bits; i++) {
      ones += Number(val % 2n);
      val = val >> 1n;
    }
    return this.build(ones);
  }

  valueOf() {
    return this.val;
  }

  toString() {
    return this.val.toString();
  }
}

export class I8 extends Int {
  static ArrayKind = Int32Array; // GPUs don't suppoert 8-bits (uniformly)
  constructor(v) {
    super(v, 8, 256, -128, 127);
  }

  build(v) {
    return new I8(this.wrap(v));
  }
}

export class U8 extends Int {
  static ArrayKind = Uint32Array; // GPUs don't suppoert 8-bits (uniformly)
  constructor(v) {
    super(v, 8, 256, 0, 255);
  }

  build(v) {
    return new U8(this.wrap(v));
  }
}

export class I16 extends Int {
  static ArrayKind = Int32Array; // GPUs don't suppoert 16-bits (uniformly)
  constructor(v) {
    super(v, 16, 65_536, -32_768, 32_767);
  }

  build(v) {
    return new I16(this.wrap(v));
  }
}

export class U16 extends Int {
  static ArrayKind = Uint32Array; // GPUs don't suppoert 16-bits (uniformly)
  constructor(v) {
    super(v, 16, 65_536, 0, 65_535);
  }

  build(v) {
    return new U16(this.wrap(v));
  }
}

export class I32 extends Int {
  static ArrayKind = Int32Array;
  constructor(v) {
    super(v, 32, 4_294_967_296, -2_147_483_648, 2_147_483_647);
  }

  build(v) {
    return new I32(this.wrap(v));
  }
}

export class U32 extends Int {
  static ArrayKind = Uint32Array;
  constructor(v) {
    super(v, 32, 4_294_967_296, 0, 4_294_967_295);
  }

  build(v) {
    return new U32(this.wrap(v));
  }
}

export class I64 extends Int {
  static ArrayKind = Int32Array; // GPUs don't support 64-bits
  constructor(v) {
    super(v, 64, 18_446_744_073_709_551_616n, -9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n);
  }

  build(v) {
    return new I64(this.wrap(v));
  }
}

export class U64 extends Int {
  static ArrayKind = Uint32Array; // GPUs don't support 64-bits
  constructor(v) {
    super(v, 64, 18_446_744_073_709_551_616n, 0n, 18_446_744_073_709_551_615n);
  }

  build(v) {
    return new U64(this.wrap(v));
  }
}

export class Float {
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

export class F32 extends Float {
  static ArrayKind = Float32Array;
  constructor(v) {
    super(Number(v), 32);
  }

  build(v) {
    return new F32(v);
  }
}

export class F64 extends Float {
  static ArrayKind = Float32Array; // GPUs don't support 64-bits
  constructor(v) {
    super(Number(v), 64);
  }

  build(v) {
    return new F64(v);
  }
}

export class Bool {
  constructor(val) {
    this.val = Boolean(val);
    this.ArrayKind = Int8Array;
  }

  valueOf() {
    return this.val;
  }

  toString() {
    return this.val.toString();
  }
}

export class Str {
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

export function swap(a, i, j) {
  if (i.val < 0 || i.val > a.length) {
    return new AlanError(`Provided index ${i.val} is beyond the bounds of the array`);
  }
  if (j.val < 0 || j.val > a.length) {
    return new AlanError(`Provided index ${j.val} is beyond the bounds of the array`);
  }
  let temp = a[i.val];
  a[i.val] = a[j.val];
  a[j.val] = temp;
}

async function merge(left, right, sorter) {
  let arr = [];
  while (left.length && right.length) {
    if ((await sorter(left[0], right[0])) < 0) {
      arr.push(left.shift());
    } else {
      arr.push(right.shift());
    }
  }
  return [ ...arr, ...left, ...right ];
}

export async function sort(a, sorter) {
  // I really didn't want to write my own sorter, but here we are. This is a merge sort. It's not a
  // true in-place merge sort, but I fake it at the end. Should be made better in the future.
  if (a.length < 2) {
    return;
  }
  let half = Math.floor(a.length / 2);
  let right = [...a];
  let left = right.splice(0, half);
  await sort(left, sorter);
  await sort(right, sorter);
  let res = await merge(left, right, sorter);
  for (let i = 0; i < res.length; i++) {
    a[i] = res[i];
  }
}

export function cross(a, b) {
  // Assuming they're all the same type
  let type = a[0].constructor;
  return [
    new type(a[1].val * b[2].val - a[2].val * b[1].val),
    new type(a[2].val * b[0].val - a[0].val * b[2].val),
    new type(a[0].val * b[1].val - a[1].val * b[0].val),
  ];
}

export class GPU {
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

export async function gpu() {
  if (GPUS === null) {
    GPUS = await GPU.init(await GPU.list());
  }
  if (GPUS.length > 0) {
    return GPUS[0];
  } else {
    throw new AlanError("This program requires a GPU but there are no WebGPU-compliant GPUs on this machine");
  }
}

export async function createBufferInit(usage, vals) {
  let g = await gpu();
  let b = await g.device.createBuffer({
    mappedAtCreation: true,
    size: vals.length * (vals[0]?.bits ?? 32) / 8,
    usage,
    label: `buffer_${uuidv4().replaceAll('-', '_')}`,
  });
  let ab = b.getMappedRange();
  let v = new (vals[0].constructor.ArrayKind ?? Int32Array)(ab);
  for (let i = 0; i < vals.length; i++) {
    v[i] = vals[i].valueOf();
  }
  b.unmap();
  b.ValType = vals[0].constructor;
  return b;
}

export async function createEmptyBuffer(usage, size, ValKind) {
  let g = await gpu();
  let b = await g.device.createBuffer({
    size: size.valueOf() * (ValKind?.bits ?? 32) / 8,
    usage,
    label: `buffer_${uuidv4().replaceAll('-', '_')}`,
  });
  b.ValKind = ValKind;
  return b;
}

export function mapReadBufferType() {
  return GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST;
}

export function mapWriteBufferType() {
  return GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC;
}

export function storageBufferType() {
  return GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC;
}

export function bufferlen(b) {
    return new I64(b.size / ((b?.ValKind?.bits ?? 32) / 8));
}

export function bufferid(b) {
  return new Str(b.label);
}

export class GPGPU {
  constructor(source, buffers, workgroupSizes, entrypoint) {
    this.source = source;
    this.entrypoint = entrypoint ?? "main";
    this.buffers = buffers;
    this.workgroupSizes = workgroupSizes;
  }
}

export async function gpuRun(gg) {
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
    gg.workgroupSizes[0].valueOf(),
    (gg.workgroupSizes[1] ?? 1).valueOf(),
    (gg.workgroupSizes[2] ?? 1).valueOf()
  );
  cpass.end();
  g.queue.submit([encoder.finish()]);
}

export async function readBuffer(b) {
  let g = await gpu();
  await g.queue.onSubmittedWorkDone(); // Don't try to read until you're sure it's safe to
  let tempBuffer = await createEmptyBuffer(mapReadBufferType(), b.size / 4);
  let encoder = g.device.createCommandEncoder();
  encoder.copyBufferToBuffer(b, 0, tempBuffer, 0, b.size);
  g.queue.submit([encoder.finish()]);
  await tempBuffer.mapAsync(GPUMapMode.READ);
  let data = tempBuffer.getMappedRange(0, b.size);
  let vals = new (b?.ValKind?.ArrayKind ?? Int32Array)(data);
  let out = [];
  for (let i = 0; i < vals.length; i++) {
    out[i] = new (b?.ValKind ?? I32)(vals[i]);
  }
  tempBuffer.unmap();
  tempBuffer.destroy();
  return out;
}

export async function replaceBuffer(b, v) {
  if (v.length != bufferlen(b)) {
    return new AlanError("The input array is not the same size as the buffer");
  }
  let tempBuffer = await createBufferInit(mapWriteBufferType(), v);
  let g = await gpu();
  let encoder = g.device.createCommandEncoder();
  encoder.copyBufferToBuffer(tempBuffer, 0, b, 0, b.size);
  g.queue.submit([encoder.finish()]);
  tempBuffer.destroy();
}
