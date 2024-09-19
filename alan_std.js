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
  return Math.max(-32_768, Math.min(32_767, Number(n)));
}

export function clampI32(n) {
  return Math.max(-2_147_483_648, Math.min(2_147_483_647, Number(n)));
}

export function clampI64(n) {
  let v = BigInt(n);
  if (v > 9_223_372_036_854_775_807n) {
    return 9_223_372_036_854_775_807n;
  }
  if (v < -9_223_372_036_854_775_808n) {
    return -9_223_372_036_854_775_808n;
  }
  return v;
}

export function parseI64(s) {
  try {
    return clampI64(BigInt(s));
  } catch (e) {
    return new AlanError(e.message);
  }
}

export function clampU8(n) {
  return Math.max(0, Math.min(255, Number(n)));
}

export function clampU16(n) {
  return Math.max(0, Math.min(65_535, Number(n)));
}

export function clampU32(n) {
  return Math.max(0, Math.min(4_294_967_295, Number(n)));
}

export function clampU64(n) {
  let v = BigInt(n);
  if (v > 18_446_744_073_709_551_615n) {
    return 18_446_744_073_709_551_615n;
  }
  if (v < 0n) {
    return 0n;
  }
  return v;
}

export function parseU64(s) {
  try {
    return clampU64(BigInt(s));
  } catch (e) {
    return new AlanError(e.message);
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
  let c = a < 0 ? a + 256 : a;
  let v = c << b;
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function wrappingShrI8(a, b) {
  let c = a < 0 ? a + 256 : a;
  let v = c >> b;
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function rotateLeftI8(a, b) {
  let c = b;
  while (c > 7) {
    c -= 8;
  }
  if (c == 0) {
    return a;
  }
  let lhs = clampI8(-1 << c);
  let rhs = clampI8(-1 ^ lhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrI8(p1, 8 - c) + wrappingShlI8(p2, c);
}

export function rotateRightI8(a, b) {
  let c = b;
  while (c > 7) {
    c -= 8;
  }
  if (c == 0) {
    return a;
  }
  let rhs = clampI8(-1 << c);
  let lhs = clampI8(-1 ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrI8(p1, 8 - c) + wrappingShlI8(p2, c);
}

export function wrappingAddI16(a, b) {
  let v = a + b;
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function wrappingSubI16(a, b) {
  let v = a - b;
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function wrappingMulI16(a, b) {
  let v = a * b;
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function wrappingDivI16(a, b) {
  let v = Math.floor(a / b);
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function wrappingModI16(a, b) {
  let v = a % b;
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function wrappingPowI16(a, b) {
  let v = Math.floor(a ** b);
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function wrappingShlI16(a, b) {
  let c = a < 0 ? a + 65_536 : a;
  let v = c << b;
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function wrappingShrI16(a, b) {
  let c = a < 0 ? a + 65_536 : a;
  let v = c >> b;
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function rotateLeftI16(a, b) {
  let c = b;
  while (c > 15) {
    c -= 16;
  }
  if (c == 0) {
    return a;
  }
  let lhs = clampI16(-1 << c);
  let rhs = clampI16(-1 ^ lhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrI16(p1, 16 - c) + wrappingShlI16(p2, c);
}

export function rotateRightI16(a, b) {
  let c = b;
  while (c > 15) {
    c -= 16;
  }
  if (c == 0) {
    return a;
  }
  let rhs = clampI16(-1 << c);
  let lhs = clampI16(-1 ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrI16(p1, 16 - c) + wrappingShlI16(p2, c);
}
