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

export function wrappingAddI32(a, b) {
  let v = a + b;
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingSubI32(a, b) {
  let v = a - b;
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingMulI32(a, b) {
  let v = a * b;
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingDivI32(a, b) {
  let v = Math.floor(a / b);
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingModI32(a, b) {
  let v = a % b;
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingPowI32(a, b) {
  let v = Math.floor(a ** b);
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingShlI32(a, b) {
  let c = a < 0 ? a + 4_294_967_296 : a;
  let v = c << b;
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingShrI32(a, b) {
  // There's something broken with right-shift. MDN says it's a "sign propagating right shift"
  // so a negative number will remain negative after the shift (which is a trash choice, but
  // okay). However, even if I convert an i32 into a u32 inside of Number, where it's *not*
  // negative, but the 32nd bit is 1, it will treat it as the sign bit in the operation and
  // output a negative number.
  //
  // But all is not lost. I'm converting the value into a BigInt after making it a u32 and then
  // converting back to a Number at the end to get this to work right.
  let c = a < 0 ? BigInt(a) + 4_294_967_296n : BigInt(a);
  let v = c >> BigInt(b);
  while (v > 2_147_483_647n) {
    v -= 4_294_967_296n;
  }
  while (v < -2_147_483_648n) {
    v += 4_294_967_296n;
  }
  return Number(v);
}

export function rotateLeftI32(a, b) {
  let c = b;
  while (c > 31) {
    c -= 32;
  }
  if (c == 0) {
    return a;
  }
  let lhs = clampI32(-1 << c);
  let rhs = clampI32(-1 ^ lhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrI32(p1, 32 - c) + wrappingShlI32(p2, c);
}

export function rotateRightI32(a, b) {
  let c = b;
  while (c > 31) {
    c -= 32;
  }
  if (c == 0) {
    return a;
  }
  let rhs = clampI32(-1 << c);
  let lhs = clampI32(-1 ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrI32(p1, 32 - c) + wrappingShlI32(p2, c);
}

export function wrappingAddI64(a, b) {
  let v = a + b;
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingSubI64(a, b) {
  let v = a - b;
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingMulI64(a, b) {
  let v = a * b;
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingDivI64(a, b) {
  let v = a / b;
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingModI64(a, b) {
  let v = a % b;
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingPowI64(a, b) {
  let v = a ** b;
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingShlI64(a, b) {
  let c = a < 0n ? a + 18_446_744_073_709_551_616n : a;
  let v = c << b;
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingShrI64(a, b) {
  let c = a < 0n ? a + 18_446_744_073_709_551_616n : a;
  let v = c >> b;
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function rotateLeftI64(a, b) {
  let c = b;
  while (c > 63n) {
    c -= 64n;
  }
  if (c == 0n) {
    return a;
  }
  let lhs = clampI64(-1n << c);
  let rhs = clampI64(-1n ^ lhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrI64(p1, 64n - c) + wrappingShlI64(p2, c);
}

export function rotateRightI64(a, b) {
  let c = b;
  while (c > 63n) {
    c -= 64n;
  }
  if (c == 0n) {
    return a;
  }
  let rhs = clampI64(-1n << c);
  let lhs = clampI64(-1n ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrI64(p1, 64n - c) + wrappingShlI64(p2, c);
}

export function wrappingAddU8(a, b) {
  let v = a + b;
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function wrappingSubU8(a, b) {
  let v = a - b;
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function wrappingMulU8(a, b) {
  let v = a * b;
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function wrappingDivU8(a, b) {
  let v = Math.floor(a / b);
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function wrappingModU8(a, b) {
  let v = a % b;
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function wrappingPowU8(a, b) {
  let v = Math.floor(a ** b);
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function notU8(a) {
  let v = ~a;
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function wrappingShlU8(a, b) {
  let v = a << b;
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function wrappingShrU8(a, b) {
  let v = a >> b;
  while (v > 255) {
    v -= 256;
  }
  while (v < 0) {
    v += 256;
  }
  return v;
}

export function rotateLeftU8(a, b) {
  let c = b;
  while (c > 7) {
    c -= 8;
  }
  if (c == 0) {
    return a;
  }
  let lhs = clampU8(255 << c);
  let rhs = clampU8(255 ^ lhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrU8(p1, 8 - c) + wrappingShlU8(p2, c);
}

export function rotateRightU8(a, b) {
  let c = b;
  while (c > 7) {
    c -= 8;
  }
  if (c == 0) {
    return a;
  }
  let rhs = clampU8(255 << c);
  let lhs = clampU8(255 ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrU8(p1, 8 - c) + wrappingShlU8(p2, c);
}

export function wrappingAddU16(a, b) {
  let v = a + b;
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function wrappingSubU16(a, b) {
  let v = a - b;
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function wrappingMulU16(a, b) {
  let v = a * b;
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function wrappingDivU16(a, b) {
  let v = Math.floor(a / b);
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function wrappingModU16(a, b) {
  let v = a % b;
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function wrappingPowU16(a, b) {
  let v = Math.floor(a ** b);
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function notU16(a) {
  let v = ~a;
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function wrappingShlU16(a, b) {
  let v = a << b;
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function wrappingShrU16(a, b) {
  let v = a >> b;
  while (v > 65_535) {
    v -= 65_536;
  }
  while (v < 0) {
    v += 65_536;
  }
  return v;
}

export function rotateLeftU16(a, b) {
  let c = b;
  while (c > 15) {
    c -= 16;
  }
  if (c == 0) {
    return a;
  }
  let lhs = clampU16(65_535 << c);
  let rhs = clampU16(65_535 ^ lhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrU16(p1, 16 - c) + wrappingShlU16(p2, c);
}

export function rotateRightU16(a, b) {
  let c = b;
  while (c > 15) {
    c -= 16;
  }
  if (c == 0) {
    return a;
  }
  let rhs = clampU16(65_535 << c);
  let lhs = clampU16(65_535 ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrU16(p1, 16 - c) + wrappingShlU16(p2, c);
}
