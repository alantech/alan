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

assert(alanStd.wrappingAddI64(1n, 2n) === 3n, "wrappingAddI64 1 + 2 = 3");
assert(alanStd.wrappingAddI64(9_223_372_036_854_775_807n, 1n) === -9_223_372_036_854_775_808n, "wrappingAddI64 9_223_372_036_854_775_807 + 1 = -9_223_372_036_854_775_808");

assert(alanStd.wrappingSubI64(1n, 2n) === -1n, "wrappingSubI64 1 - 2 = -1");
assert(alanStd.wrappingSubI64(-9_223_372_036_854_775_808n, 1n) === 9_223_372_036_854_775_807n, "wrappingSubI64 -9_223_372_036_854_775_808 - 1 = 9_223_372_036_854_775_807");

assert(alanStd.wrappingMulI64(4_294_967_296n, 4_294_967_296n) === 0n, "wrappingMulI64 4_294_967_296 * 4_294_967_296 = 0");

assert(alanStd.wrappingDivI64(-9_223_372_036_854_775_808n, 2n) == -4_611_686_018_427_387_904n, "wrappingDivI64 -9_223_372_036_854_775_808 / 2 = −4_611_686_018_427_387_904");

assert(alanStd.wrappingModI64(5n, 2n) == 1n, "wrappingModI64 5 % 2 = 1");

assert(alanStd.wrappingPowI64(2n, 64n) == 0n, "wrappingPowI64 2 ^ 64 = 0");

assert(alanStd.wrappingShlI64(-9_223_372_036_854_775_808n, 1n) == 0n, "wrappingShlI64 -9_223_372_036_854_775_808 << 1 = 0");

assert(alanStd.wrappingShrI64(-9_223_372_036_854_775_808n, 1n) == 4_611_686_018_427_387_904n, "wrappingShrI64 -9_223_372_036_854_775_808 >> 1 = 4_611_686_018_427_387_904");

assert(alanStd.rotateLeftI64(-9_223_372_036_854_775_808n, 1n) == 1n, "rotateLeftI64 -9_223_372_036_854_775_808 <<< 1 = 1");

assert(alanStd.rotateRightI64(4_611_686_018_427_387_904n, 1n) == -9_223_372_036_854_775_808n, "rotateRightI64 4_611_686_018_427_387_904 >>> 1 = -9_223_372_036_854_775_808");

assert(alanStd.wrappingAddU8(1, 2) === 3, "wrappingAddU8 1 + 2 = 3");
assert(alanStd.wrappingAddU8(255, 1) === 0, "wrappingAddU8 255 + 1 = 0");

assert(alanStd.wrappingSubU8(1, 2) === 255, "wrappingSubU8 1 - 2 = 255");
assert(alanStd.wrappingSubU8(255, 1) === 254, "wrappingSubU8 255 - 1 = 254");

assert(alanStd.wrappingMulU8(64, 64) === 0, "wrappingMulU8 64 * 64 = 0");

assert(alanStd.wrappingDivU8(128, 2) == 64, "wrappingDivU8 128 / 2 = 64");

assert(alanStd.wrappingModU8(5, 2) == 1, "wrappingModU8 5 % 2 = 1");

assert(alanStd.wrappingPowU8(2, 8) == 0, "wrappingPowU8 2 ^ 8 = 0");

assert(alanStd.notU8(0) == 255, "notU8 0 = 255");

assert(alanStd.wrappingShlU8(128, 1) == 0, "wrappingShlU8 128 << 1 = 0");

assert(alanStd.wrappingShrU8(128, 1) == 64, "wrappingShrU8 128 >> 1 = 64");

assert(alanStd.rotateLeftU8(128, 1) == 1, "rotateLeftU8 128 <<< 1 = 1");

assert(alanStd.rotateRightU8(64, 1) == 128, "rotateRightU8 64 >>> 1 = 128");

assert(alanStd.wrappingAddU16(1, 2) === 3, "wrappingAddU16 1 + 2 = 3");
assert(alanStd.wrappingAddU16(65_535, 1) === 0, "wrappingAddU16 65_535 + 1 = 0");

assert(alanStd.wrappingSubU16(1, 2) === 65_535, "wrappingSubU16 1 - 2 = 65_535");
assert(alanStd.wrappingSubU16(65_535, 1) === 65_534, "wrappingSubU16 65_535 - 1 = 65_534");

assert(alanStd.wrappingMulU16(256, 256) === 0, "wrappingMulU16 256 * 256 = 0");

assert(alanStd.wrappingDivU16(128, 2) == 64, "wrappingDivU16 128 / 2 = 64");

assert(alanStd.wrappingModU16(5, 2) == 1, "wrappingModU16 5 % 2 = 1");

assert(alanStd.wrappingPowU16(2, 16) == 0, "wrappingPowU16 2 ^ 8 = 0");

assert(alanStd.notU16(0) == 65_535, "notU16 0 = 65_535");

assert(alanStd.wrappingShlU16(32_768, 1) == 0, "wrappingShlU16 32_768 << 1 = 0");

assert(alanStd.wrappingShrU16(128, 1) == 64, "wrappingShrU16 128 >> 1 = 64");

assert(alanStd.rotateLeftU16(32_768, 1) == 1, "rotateLeftU16 128 <<< 1 = 1");

assert(alanStd.rotateRightU16(64, 1) == 128, "rotateRightU16 64 >>> 1 = 128");
