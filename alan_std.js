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

export function clampI8(n) {
  return Math.max(-128, Math.min(127, Number(n)));
}

export function clampI16(n) {
  return Math.max(-32768, Math.min(32767, Number(n)));
}

export function clampI32(n) {
  return Math.max(-2147483648, Math.min(2147483647, Number(n)));
}

export function clampI64(n) {
  return BigInt.asIntN(64, BigInt(n));
}

export function parseI64(s) {
  try {
    return clampI64(BigInt(s));
  } catch (e) {
    return AlanError(e.message);
  }
}

export function clampU8(n) {
  return Math.max(0, Math.min(255, Number(n)));
}

export function clampU16(n) {
  return Math.max(0, Math.min(65535, Number(n)));
}

export function clampU32(n) {
  return Math.max(0, Math.min(4294967295, Number(n)));
}

export function clampU64(n) {
  return BigInt.asUintN(64, BigInt(n));
}

export function parseU64(s) {
  try {
    return clampU64(BigInt(s));
  } catch (e) {
    return AlanError(e.message);
  }
}

export function ifbool(b, t, f) {
  if (b) {
    return t();
  } else {
    return f();
  }
}

export function wrappingAddI8(a, b) {
  let v = a + b;
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function wrappingSubI8(a, b) {
  let v = a - b;
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function wrappingMulI8(a, b) {
  let v = a * b;
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function wrappingDivI8(a, b) {
  let v = Math.floor(a / b);
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function wrappingModI8(a, b) {
  let v = a % b;
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function wrappingPowI8(a, b) {
  let v = Math.floor(a ** b);
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function wrappingShlI8(a, b) {
  // TODO: Thoroughly test this, there may be wonkiness with where the significant negative digit is located
  let v = a << (7 & b);
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function wrappingShrI8(a, b) {
  // TODO: Thoroughly test this, there may be wonkiness with where the significant negative digit is located
  let v = a >> (7 & b);
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function rotateLeftI8(a, b) {
  // TODO: Thoroughly test this, there may be wonkiness with where the significant negative digit is located
  let c = b;
  while (c > 7) {
    c -= 8;
  }
  if (c == 0) {
    return a;
  }
  let lhs = -1 << c;
  let rhs = -1 ^ lhs;
  let p1 = a & lhs;
  let p2 = a & rhs;
  return (p1 << (8 - c)) + (p2 >> c);
}

export function rotateRightI8(a, b) {
  // TODO: Thoroughly test this, there may be wonkiness with where the significant negative digit is located
  let c = b;
  while (c > 7) {
    c -= 8;
  }
  if (c == 0) {
    return a;
  }
  let rhs = -1 << c;
  let lhs = -1 ^ rhs;
  let p1 = a & lhs;
  let p2 = a & rhs;
  return (p1 >> (8 - c)) + (p2 << c);
}
