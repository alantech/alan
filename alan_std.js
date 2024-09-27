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

export class Int {
  constructor(val, bits, size, lower, upper) {
    if (bits == 64) {
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
    if (this.bits == 64) {
      return this.build(this.val / a.val);
    } else {
      return this.build(Math.floor(this.val / a.val));
    }
  }

  wrappingMod(a) {
    return this.build(this.val % a.val);
  }

  wrappingPow(a) {
    if (this.bits == 64) {
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
