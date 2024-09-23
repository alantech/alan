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
  // Because this bitwise arithmetic in JS Number is a bit of a mess, let's convert to u8, do the
  // work there, then convert back
  let v = rotateLeftU8(a < 0 ? a + 256 : a, b < 0 ? b + 256 : b);
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
}

export function rotateRightI8(a, b) {
  let v = rotateLeftU8(a < 0 ? a + 256 : a, b < 0 ? b + 256 : b);
  while (v > 127) {
    v -= 256;
  }
  while (v < -128) {
    v += 256;
  }
  return v;
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
  let v = rotateLeftU16(a < 0 ? a + 65_536 : a, b < 0 ? b + 65_536 : b);
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
}

export function rotateRightI16(a, b) {
  let v = rotateRightU16(a < 0 ? a + 65_536 : a, b < 0 ? b + 65_536 : b);
  while (v > 32_767) {
    v -= 65_536;
  }
  while (v < -32_768) {
    v += 65_536;
  }
  return v;
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
  let v = rotateLeftU32(a < 0 ? a + 4_294_967_296 : a, b < 0 ? b + 4_294_967_296 : b);
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
}

export function rotateRightI32(a, b) {
  let v = rotateRightU32(a < 0 ? a + 4_294_967_296 : a, b < 0 ? b + 4_294_967_296 : b);
  while (v > 2_147_483_647) {
    v -= 4_294_967_296;
  }
  while (v < -2_147_483_648) {
    v += 4_294_967_296;
  }
  return v;
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
  let v = rotateLeftU64(a < 0n ? a + 18_446_744_073_709_551_616n : a, b < 0n ? b + 18_446_744_073_709_551_616n : b);
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function rotateRightI64(a, b) {
  let v = rotateRightU64(a < 0n ? a + 18_446_744_073_709_551_616n : a, b < 0n ? b + 18_446_744_073_709_551_616n : b);
  while (v > 9_223_372_036_854_775_807n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < -9_223_372_036_854_775_808n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
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
  let lhs = 255 & (255 << 8 - c);
  let rhs = 255 & (255 ^ lhs);
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
  let rhs = 255 & (255 << c);
  let lhs = 255 & (255 ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShlU8(p1, 8 - c) + wrappingShrU8(p2, c);
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
  let lhs = 65_535 & (65_535 << 16 - c);
  let rhs = 65_535 & (65_535 ^ lhs);
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
  let rhs = 65_535 & (65_535 << c);
  let lhs = 65_535 & (65_535 ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShlU16(p1, 16 - c) + wrappingShrU16(p2, c);
}

export function wrappingAddU32(a, b) {
  let v = a + b;
  while (v > 4_294_967_295) {
    v -= 4_294_967_296;
  }
  while (v < 0) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingSubU32(a, b) {
  let v = a - b;
  while (v > 4_294_967_295) {
    v -= 4_294_967_296;
  }
  while (v < 0) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingMulU32(a, b) {
  let v = a * b;
  while (v > 4_294_967_295) {
    v -= 4_294_967_296;
  }
  while (v < 0) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingDivU32(a, b) {
  let v = Math.floor(a / b);
  while (v > 4_294_967_295) {
    v -= 4_294_967_296;
  }
  while (v < 0) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingModU32(a, b) {
  let v = a % b;
  while (v > 4_294_967_295) {
    v -= 4_294_967_296;
  }
  while (v < 0) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingPowU32(a, b) {
  let v = Math.floor(a ** b);
  while (v > 4_294_967_295) {
    v -= 4_294_967_296;
  }
  while (v < 0) {
    v += 4_294_967_296;
  }
  return v;
}

export function notU32(a) {
  let v = ~a;
  while (v > 4_294_967_295) {
    v -= 4_294_967_296;
  }
  while (v < 0) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingShlU32(a, b) {
  let v = a << b;
  while (v > 4_294_967_295) {
    v -= 4_294_967_296;
  }
  while (v < 0) {
    v += 4_294_967_296;
  }
  return v;
}

export function wrappingShrU32(a, b) {
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
  while (v > 4_294_967_295n) {
    v -= 4_294_967_296n;
  }
  while (v < 0n) {
    v += 4_294_967_296n;
  }
  return Number(v);
}

export function rotateLeftU32(a, b) {
  let c = BigInt(b);
  while (c > 31n) {
    c -= 32n;
  }
  if (c == 0n) {
    return a;
  }
  let lhs = 4_294_967_295n & (4_294_967_295n << c);
  let rhs = 4_294_967_295n & (4_294_967_295n ^ lhs);
  let p1 = BigInt(a) & lhs;
  let p2 = BigInt(a) & rhs;
  return wrappingShrU32(Number(p1), 32 - Number(c)) + wrappingShlU32(Number(p2), Number(c));
}

export function rotateRightU32(a, b) {
  let c = BigInt(b);
  while (c > 31n) {
    c -= 32n;
  }
  if (c == 0n) {
    return a;
  }
  let rhs = BigInt(clampU32(4_294_967_295n << c));
  let lhs = BigInt(clampU32(4_294_967_295n ^ rhs));
  let p1 = BigInt(a) & lhs;
  let p2 = BigInt(a) & rhs;
  return wrappingShlU32(Number(p1), 32 - Number(c)) + wrappingShrU32(Number(p2), Number(c));
}

export function wrappingAddU64(a, b) {
  let v = a + b;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingSubU64(a, b) {
  let v = a - b;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingMulU64(a, b) {
  let v = a * b;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingDivU64(a, b) {
  let v = a / b;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingModU64(a, b) {
  let v = a % b;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingPowU64(a, b) {
  let v = a ** b;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function notU64(a) {
  let v = ~a;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingShlU64(a, b) {
  let v = a << b;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function wrappingShrU64(a, b) {
  let v = a >> b;
  while (v > 18_446_744_073_709_551_615n) {
    v -= 18_446_744_073_709_551_616n;
  }
  while (v < 0n) {
    v += 18_446_744_073_709_551_616n;
  }
  return v;
}

export function rotateLeftU64(a, b) {
  let c = b;
  while (c > 63n) {
    c -= 64n;
  }
  if (c == 0n) {
    return a;
  }
  let lhs = 18_446_744_073_709_551_615n & (18_446_744_073_709_551_615n << c);
  let rhs = 18_446_744_073_709_551_615n & (18_446_744_073_709_551_615n ^ lhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShrU64(p1, 64n - c) + wrappingShlU64(p2, c);
}

export function rotateRightU64(a, b) {
  let c = b;
  while (c > 63n) {
    c -= 64n;
  }
  if (c == 0n) {
    return a;
  }
  let rhs = 18_446_744_073_709_551_615n & (18_446_744_073_709_551_615n << c);
  let lhs = 18_446_744_073_709_551_615n & (18_446_744_073_709_551_615n ^ rhs);
  let p1 = a & lhs;
  let p2 = a & rhs;
  return wrappingShlU64(p1, 64n - c) + wrappingShrU64(p2, c);
}
