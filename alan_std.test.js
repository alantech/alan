import assert from "node:assert";

import * as alanStd from "./alan_std.js";

assert(new alanStd.AlanError("foo") instanceof alanStd.AlanError, "AlanError constructor");
assert(new alanStd.AlanError("foo").message === "foo", "AlanError message");

assert(alanStd.nanToError(5) === 5, "nanToError normal number");
assert(alanStd.nanToError(NaN).message === "Not a Number", "nanToError NaN");

assert(alanStd.clampI8(0) === 0, "clampI8 0");
assert(alanStd.clampI8(150) === 127, "clampI8 150");
assert(alanStd.clampI8(-150) === -128, "clampI8 -150");

assert(alanStd.clampI16(0) === 0, "clampI16 0");
assert(alanStd.clampI16(40_000) === 32_767, "clampI16 40k");
assert(alanStd.clampI16(-40_000) === -32_768, "clampI16 -40k");

assert(alanStd.clampI32(0) === 0, "clampI32 0");
assert(alanStd.clampI32(3_000_000_000) === 2_147_483_647, "clampI32 3B");
assert(alanStd.clampI32(-3_000_000_000) === -2_147_483_648, "clampI32 -3B");

assert(alanStd.clampI64(0n) === 0n, "clampI64 0n");
assert(alanStd.clampI64(10_000_000_000_000_000_000n) === 9_223_372_036_854_775_807n, "clampI64 10Q");
assert(alanStd.clampI64(-10_000_000_000_000_000_000n) === -9_223_372_036_854_775_808n, "clampI64 -10Q");

assert(alanStd.parseI64("0") === 0n, "parseI64 0");
assert(alanStd.parseI64("foo") instanceof alanStd.AlanError, "parseI64 foo");

assert(alanStd.clampU8(0) === 0, "clampU8 0");
assert(alanStd.clampU8(350) === 255, "clampU8 350");
assert(alanStd.clampU8(-150) === 0, "clampU8 -150");

assert(alanStd.clampU16(0) === 0, "clampU16 0");
assert(alanStd.clampU16(80_000) === 65_535, "clampU16 80k");
assert(alanStd.clampU16(-40_000) === 0, "clampU16 -40k");

assert(alanStd.clampU32(0) === 0, "clampU32 0");
assert(alanStd.clampU32(6_000_000_000) === 4_294_967_295, "clampU32 6B");
assert(alanStd.clampU32(-3_000_000_000) === 0, "clampU32 -3B");

assert(alanStd.clampU64(0n) === 0n, "clampU64 0n");
assert(alanStd.clampU64(20_000_000_000_000_000_000n) === 18_446_744_073_709_551_615n, "clampU64 20Q");
assert(alanStd.clampU64(-10_000_000_000_000_000_000n) === 0n, "clampU64 -10Q");

assert(alanStd.parseU64("0") === 0n, "parseU64 0");
assert(alanStd.parseU64("foo") instanceof alanStd.AlanError, "parseU64 foo");

assert(alanStd.ifbool(true, () => true, () => false) === true, "ifbool true");
assert(alanStd.ifbool(false, () => true, () => false) === false, "ifbool false");

assert(alanStd.wrappingAddI8(1, 2) === 3, "wrappingAddI8 1 + 2 = 3");
assert(alanStd.wrappingAddI8(127, 1) === -128, "wrappingAddI8 127 + 1 = -128");

assert(alanStd.wrappingSubI8(1, 2) === -1, "wrappingSubI8 1 - 2 = -1");
assert(alanStd.wrappingSubI8(-128, 1) === 127, "wrappingSubI8 -128 - 1 = 127");

assert(alanStd.wrappingMulI8(64, 64) === 0, "wrappingMulI8 64 * 64 = 0");

assert(alanStd.wrappingDivI8(-128, 2) == -64, "wrappingDivI8 -128 / 2 = -64");

assert(alanStd.wrappingModI8(5, 2) == 1, "wrappingModI8 5 % 2 = 1");

assert(alanStd.wrappingPowI8(2, 8) == 0, "wrappingPowI8 2 ^ 8 = 0");

assert(alanStd.wrappingShlI8(-128, 1) == 0, "wrappingShlI8 -128 << 1 = 0");

assert(alanStd.wrappingShrI8(-128, 1) == 64, "wrappingShrI8 -128 >> 1 = 64");

assert(alanStd.rotateLeftI8(-128, 1) == 1, "rotateLeftI8 -128 <<< 1 = 1");

assert(alanStd.rotateRightI8(64, 1) == -128, "rotateRightI8 64 >>> 1 = -128");

assert(alanStd.wrappingAddI16(1, 2) === 3, "wrappingAddI16 1 + 2 = 3");
assert(alanStd.wrappingAddI16(32_767, 1) === -32_768, "wrappingAddI16 32_767 + 1 = -32_768");

assert(alanStd.wrappingSubI16(1, 2) === -1, "wrappingSubI16 1 - 2 = -1");
assert(alanStd.wrappingSubI16(-32_768, 1) === 32_767, "wrappingSubI16 -32_768 - 1 = 32_767");

assert(alanStd.wrappingMulI16(256, 256) === 0, "wrappingMulI16 256 * 256 = 0");

assert(alanStd.wrappingDivI16(-32_768, 2) == -16_384, "wrappingDivI16 -32_768 / 2 = -16_384");

assert(alanStd.wrappingModI16(5, 2) == 1, "wrappingModI16 5 % 2 = 1");

assert(alanStd.wrappingPowI16(2, 16) == 0, "wrappingPowI16 2 ^ 16 = 0");

assert(alanStd.wrappingShlI16(-32_768, 1) == 0, "wrappingShlI16 -32_768 << 1 = 0");

assert(alanStd.wrappingShrI16(-32_768, 1) == 16_384, "wrappingShrI16 -32_768 >> 1 = 16_384");

assert(alanStd.rotateLeftI16(-32_768, 1) == 1, "rotateLeftI16 -32_768 <<< 1 = 1");

assert(alanStd.rotateRightI16(16_384, 1) == -32_768, "rotateRightI16 16_384 >>> 1 = -32_768");

assert(alanStd.wrappingAddI32(1, 2) === 3, "wrappingAddI32 1 + 2 = 3");
assert(alanStd.wrappingAddI32(2_147_483_647, 1) === -2_147_483_648, "wrappingAddI32 2_147_483_647 + 1 = -2_147_483_648");

assert(alanStd.wrappingSubI32(1, 2) === -1, "wrappingSubI32 1 - 2 = -1");
assert(alanStd.wrappingSubI32(-2_147_483_648, 1) === 2_147_483_647, "wrappingSubI32 -2_147_483_648 - 1 = 2_147_483_647");

assert(alanStd.wrappingMulI32(65_536, 65_536) === 0, "wrappingMulI32 65_536 * 65_536 = 0");

assert(alanStd.wrappingDivI32(-2_147_483_648, 2) == -1_073_741_824, "wrappingDivI32 -2_147_483_648 / 2 = -1_073_741_824");

assert(alanStd.wrappingModI32(5, 2) == 1, "wrappingModI32 5 % 2 = 1");

assert(alanStd.wrappingPowI32(2, 32) == 0, "wrappingPowI32 2 ^ 32 = 0");

assert(alanStd.wrappingShlI32(-2_147_483_648, 1) == 0, "wrappingShlI32 -2_147_483_648 << 1 = 0");

assert(alanStd.wrappingShrI32(-2_147_483_648, 1) == 1_073_741_824, "wrappingShrI32 -2_147_483_648 >> 1 = 1_073_741_824");

assert(alanStd.rotateLeftI32(-2_147_483_648, 1) == 1, "rotateLeftI32 -2_147_483_648 <<< 1 = 1");

assert(alanStd.rotateRightI32(1_073_741_824, 1) == -2_147_483_648, "rotateRightI32 1_073_741_824 >>> 1 = -2_147_483_648");
