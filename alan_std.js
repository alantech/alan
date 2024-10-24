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

  valueOf() {
    return this.val;
  }

  toString() {
    return this.val.toString();
  }
}

export class I8 extends Int {
  constructor(v) {
    super(v, 8, 256, -128, 127);
  }

  build(v) {
    return new I8(this.wrap(v));
  }
}

export class U8 extends Int {
  constructor(v) {
    super(v, 8, 256, 0, 255);
  }

  build(v) {
    return new U8(this.wrap(v));
  }
}

export class I16 extends Int {
  constructor(v) {
    super(v, 16, 65_536, -32_768, 32_767);
  }

  build(v) {
    return new I16(this.wrap(v));
  }
}

export class U16 extends Int {
  constructor(v) {
    super(v, 16, 65_536, 0, 65_535);
  }

  build(v) {
    return new U16(this.wrap(v));
  }
}

export class I32 extends Int {
  constructor(v) {
    super(v, 32, 4_294_967_296, -2_147_483_648, 2_147_483_647);
  }

  build(v) {
    return new I32(this.wrap(v));
  }
}

export class U32 extends Int {
  constructor(v) {
    super(v, 32, 4_294_967_296, 0, 4_294_967_295);
  }

  build(v) {
    return new U32(this.wrap(v));
  }
}

export class I64 extends Int {
  constructor(v) {
    super(v, 64, 18_446_744_073_709_551_616n, -9_223_372_036_854_775_808n, 9_223_372_036_854_775_807n);
  }

  build(v) {
    return new I64(this.wrap(v));
  }
}

export class U64 extends Int {
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
  constructor(v) {
    super(Number(v), 32);
  }

  build(v) {
    return new F32(v);
  }
}

export class F64 extends Float {
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
    size: vals.length * 4,
    usage,
    label: `buffer_${uuidv4().replaceAll('-', '_')}`,
  });
  let ab = b.getMappedRange();
  let i32v = new Int32Array(ab);
  for (let i = 0; i < vals.length; i++) {
    i32v[i] = vals[i].valueOf();
  }
  b.unmap();
  return b;
}

export async function createEmptyBuffer(usage, size) {
  let g = await gpu();
  let b = await g.device.createBuffer({
    size: size.valueOf() * 4,
    usage,
    label: `buffer_${uuidv4().replaceAll('-', '_')}`,
  });
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
  return new I64(b.size / 4);
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
  cpass.dispatchWorkgroups(...gg.workgroupSizes);
  g.queue.submit([encoder.finish()]);
}

export async function readBuffer(b) {
  let g = await gpu();
  let tempBuffer = await createEmptyBuffer(mapReadBufferType(), b.size);
  let encoder = g.device.createCommandEncoder();
  encoder.copyBufferToBuffer(b, 0, tempBuffer, 0, b.size);
  g.queue.submit([encoder.finish()]);
  await tempBuffer.mapAsync(GPUMapMode.READ);
  let data = tempBuffer.getMappedRange(0, b.size);
  let vals = new Int32Array(data);
  let out = [];
  for (let i = 0; i < vals.length; i++) {
    out[i] = new I32(vals[i]);
  }
  tempBuffer.unmap();
  return out;
}

export async function replaceBuffer(b, v) {
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
