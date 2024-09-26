import assert from "node:assert";

import * as alanStd from "./alan_std.js";

assert(new alanStd.AlanError("foo") instanceof alanStd.AlanError, "AlanError constructor");
assert(new alanStd.AlanError("foo").message === "foo", "AlanError message");

assert(alanStd.nanToError(5) === 5, "nanToError normal number");
assert(alanStd.nanToError(NaN).message === "Not a Number", "nanToError NaN");

assert(new alanStd.I8(0).val === 0, "I8 0");
assert(new alanStd.I8(150).val === 127, "I8 150");
assert(new alanStd.I8(-150).val === -128, "I8 -150");

assert(new alanStd.I16(0).val === 0, "clampI16 0");
assert(new alanStd.I16(40_000).val === 32_767, "clampI16 40k");
assert(new alanStd.I16(-40_000).val === -32_768, "clampI16 -40k");

assert(new alanStd.I32(0).val === 0, "clampI32 0");
assert(new alanStd.I32(3_000_000_000).val === 2_147_483_647, "clampI32 3B");
assert(new alanStd.I32(-3_000_000_000).val === -2_147_483_648, "clampI32 -3B");

assert(new alanStd.I64(0n).val === 0n, "clampI64 0n");
assert(new alanStd.I64(10_000_000_000_000_000_000n).val === 9_223_372_036_854_775_807n, "clampI64 10Q");
assert(new alanStd.I64(-10_000_000_000_000_000_000n).val === -9_223_372_036_854_775_808n, "clampI64 -10Q");

assert(new alanStd.U8(0).val === 0, "clampU8 0");
assert(new alanStd.U8(350).val === 255, "clampU8 350");
assert(new alanStd.U8(-150).val === 0, "clampU8 -150");

assert(new alanStd.U16(0).val === 0, "clampU16 0");
assert(new alanStd.U16(80_000).val === 65_535, "clampU16 80k");
assert(new alanStd.U16(-40_000).val === 0, "clampU16 -40k");

assert(new alanStd.U32(0).val === 0, "clampU32 0");
assert(new alanStd.U32(6_000_000_000).val === 4_294_967_295, "clampU32 6B");
assert(new alanStd.U32(-3_000_000_000).val === 0, "clampU32 -3B");

assert(new alanStd.U64(0n).val === 0n, "clampU64 0n");
assert(new alanStd.U64(20_000_000_000_000_000_000n).val === 18_446_744_073_709_551_615n, "clampU64 20Q");
assert(new alanStd.U64(-10_000_000_000_000_000_000n).val === 0n, "clampU64 -10Q");

assert(alanStd.ifbool(true, () => true, () => false) === true, "ifbool true");
assert(alanStd.ifbool(false, () => true, () => false) === false, "ifbool false");

assert(new alanStd.I8(1).wrappingAdd(new alanStd.I8(2)).val === 3, "wrappingAddI8 1 + 2 = 3");
assert(new alanStd.I8(127).wrappingAdd(new alanStd.I8(1)).val === -128, "wrappingAddI8 127 + 1 = -128");

assert(new alanStd.I8(1).wrappingSub(new alanStd.I8(2)).val === -1, "wrappingSubI8 1 - 2 = -1");
assert(new alanStd.I8(-128).wrappingSub(new alanStd.I8(1)).val === 127, "wrappingSubI8 -128 - 1 = 127");

assert(new alanStd.I8(64).wrappingMul(new alanStd.I8(64)).val === 0, "wrappingMulI8 64 * 64 = 0");

assert(new alanStd.I8(-128).wrappingDiv(new alanStd.I8(2)).val == -64, "wrappingDivI8 -128 / 2 = -64");

assert(new alanStd.I8(5).wrappingMod(new alanStd.I8(2)).val == 1, "wrappingModI8 5 % 2 = 1");

assert(new alanStd.I8(2).wrappingPow(new alanStd.I8(8)).val == 0, "wrappingPowI8 2 ^ 8 = 0");

assert(new alanStd.I8(-128).wrappingShl(new alanStd.I8(1)).val == 0, "wrappingShlI8 -128 << 1 = 0");

assert(new alanStd.I8(-128).wrappingShr(new alanStd.I8(1)).val == 64, "wrappingShrI8 -128 >> 1 = 64");

assert(new alanStd.I8(-128).rotateLeft(new alanStd.I8(1)).val == 1, "rotateLeftI8 -128 <<< 1 = 1");
assert(new alanStd.I8(0b01010101).rotateLeft(new alanStd.I8(0)).val == 0b01010101, "rotateLeftI8 0b01010101 <<< 0 = 0b01010101");
assert(new alanStd.I8(0b01010101).rotateLeft(new alanStd.I8(1)).val == 0b10101010 - 256, "rotateLeftI8 0b01010101 <<< 1 = 0b10101010");
assert(new alanStd.I8(0b01010101).rotateLeft(new alanStd.I8(2)).val == 0b01010101, "rotateLeftI8 0b01010101 <<< 2 = 0b01010101");
assert(new alanStd.I8(0b01010101).rotateLeft(new alanStd.I8(3)).val == 0b10101010 - 256, "rotateLeftI8 0b01010101 <<< 3 = 0b10101010");

assert(new alanStd.I8(64).rotateRight(new alanStd.I8(1)).val == 32, "rotateRightI8 64 >>> 1 = 32");
assert(new alanStd.I8(0b01010101).rotateRight(new alanStd.I8(0)).val == 0b01010101, "rotateRightI8 0b01010101 <<< 0 = 0b01010101");
assert(new alanStd.I8(0b01010101).rotateRight(new alanStd.I8(1)).val == 0b10101010 - 256, "rotateRightI8 0b01010101 <<< 1 = 0b10101010");
assert(new alanStd.I8(0b01010101).rotateRight(new alanStd.I8(2)).val == 0b01010101, "rotateRightI8 0b01010101 <<< 2 = 0b01010101");
assert(new alanStd.I8(0b01010101).rotateRight(new alanStd.I8(3)).val == 0b10101010 - 256, "rotateRightI8 0b01010101 <<< 3 = 0b10101010");

assert(new alanStd.I16(1).wrappingAdd(new alanStd.I16(2)).val === 3, "wrappingAddI16 1 + 2 = 3");
assert(new alanStd.I16(32_767).wrappingAdd(new alanStd.I16(1)).val === -32_768, "wrappingAddI16 32_767 + 1 = -32_768");

assert(new alanStd.I16(1).wrappingSub(new alanStd.I16(2)).val === -1, "wrappingSubI16 1 - 2 = -1");
assert(new alanStd.I16(-32_768).wrappingSub(new alanStd.I16(1)).val === 32_767, "wrappingSubI16 -32_768 - 1 = 32_767");

assert(new alanStd.I16(256).wrappingMul(new alanStd.I16(256)).val === 0, "wrappingMulI16 256 * 256 = 0");

assert(new alanStd.I16(-32_768).wrappingDiv(new alanStd.I16(2)) == -16_384, "wrappingDivI16 -32_768 / 2 = -16_384");

assert(new alanStd.I16(5).wrappingMod(new alanStd.I16(2)) == 1, "wrappingModI16 5 % 2 = 1");

assert(new alanStd.I16(2).wrappingPow(new alanStd.I16(16)) == 0, "wrappingPowI16 2 ^ 16 = 0");

assert(new alanStd.I16(-32_768).wrappingShl(new alanStd.I16(1)) == 0, "wrappingShlI16 -32_768 << 1 = 0");

assert(new alanStd.I16(-32_768).wrappingShr(new alanStd.I16(1)) == 16_384, "wrappingShrI16 -32_768 >> 1 = 16_384");

assert(new alanStd.I16(-32_768).rotateLeft(new alanStd.I16(1)) == 1, "rotateLeftI16 -32_768 <<< 1 = 1");

assert(new alanStd.I16(16_384).rotateRight(new alanStd.I16(1)) == 8_192, "rotateRightI16 16_384 >>> 1 = -32_768");

assert(new alanStd.I32(1).wrappingAdd(new alanStd.I32(2)).val === 3, "wrappingAddI32 1 + 2 = 3");
assert(new alanStd.I32(2_147_483_647).wrappingAdd(new alanStd.I32(1)).val === -2_147_483_648, "wrappingAddI32 2_147_483_647 + 1 = -2_147_483_648");

assert(new alanStd.I32(1).wrappingSub(new alanStd.I32(2)).val === -1, "wrappingSubI32 1 - 2 = -1");
assert(new alanStd.I32(-2_147_483_648).wrappingSub(new alanStd.I32(1)).val === 2_147_483_647, "wrappingSubI32 -2_147_483_648 - 1 = 2_147_483_647");

assert(new alanStd.I32(65_536).wrappingMul(new alanStd.I32(65_536)).val === 0, "wrappingMulI32 65_536 * 65_536 = 0");

assert(new alanStd.I32(-2_147_483_648).wrappingDiv(new alanStd.I32(2)) == -1_073_741_824, "wrappingDivI32 -2_147_483_648 / 2 = -1_073_741_824");

assert(new alanStd.I32(5).wrappingMod(new alanStd.I32(2)) == 1, "wrappingModI32 5 % 2 = 1");

assert(new alanStd.I32(2).wrappingPow(new alanStd.I32(32)) == 0, "wrappingPowI32 2 ^ 32 = 0");

assert(new alanStd.I32(-2_147_483_648).wrappingShl(new alanStd.I32(1)) == 0, "wrappingShlI32 -2_147_483_648 << 1 = 0");

assert(new alanStd.I32(-2_147_483_648).wrappingShr(new alanStd.I32(1)) == 1_073_741_824, "wrappingShrI32 -2_147_483_648 >> 1 = 1_073_741_824");

assert(new alanStd.I32(-2_147_483_648).rotateLeft(new alanStd.I32(1)) == 1, "rotateLeftI32 -2_147_483_648 <<< 1 = 1");

assert(new alanStd.I32(1_073_741_824).rotateRight(new alanStd.I32(1)) == 536_870_912, "rotateRightI32 1_073_741_824 >>> 1 = 536_870_912");

assert(new alanStd.I64(1n).wrappingAdd(new alanStd.I64(2n)).val === 3n, "wrappingAddI64 1 + 2 = 3");
assert(new alanStd.I64(9_223_372_036_854_775_807n).wrappingAdd(new alanStd.I64(1n)).val === -9_223_372_036_854_775_808n, "wrappingAddI64 9_223_372_036_854_775_807 + 1 = -9_223_372_036_854_775_808");

assert(new alanStd.I64(1n).wrappingSub(new alanStd.I64(2n)).val === -1n, "wrappingSubI64 1 - 2 = -1");
assert(new alanStd.I64(-9_223_372_036_854_775_808n).wrappingSub(new alanStd.I64(1n)).val === 9_223_372_036_854_775_807n, "wrappingSubI64 -9_223_372_036_854_775_808 - 1 = 9_223_372_036_854_775_807");

assert(new alanStd.I64(4_294_967_296n).wrappingMul(new alanStd.I64(4_294_967_296n)).val === 0n, "wrappingMulI64 4_294_967_296 * 4_294_967_296 = 0");

assert(new alanStd.I64(-9_223_372_036_854_775_808n).wrappingDiv(new alanStd.I64(2n)) == -4_611_686_018_427_387_904n, "wrappingDivI64 -9_223_372_036_854_775_808 / 2 = −4_611_686_018_427_387_904");

assert(new alanStd.I64(5n).wrappingMod(new alanStd.I64(2n)) == 1n, "wrappingModI64 5 % 2 = 1");

assert(new alanStd.I64(2n).wrappingPow(new alanStd.I64(64n)) == 0n, "wrappingPowI64 2 ^ 64 = 0");

assert(new alanStd.I64(-9_223_372_036_854_775_808n).wrappingShl(new alanStd.I64(1n)) == 0n, "wrappingShlI64 -9_223_372_036_854_775_808 << 1 = 0");

assert(new alanStd.I64(-9_223_372_036_854_775_808n).wrappingShr(new alanStd.I64(1n)) == 4_611_686_018_427_387_904n, "wrappingShrI64 -9_223_372_036_854_775_808 >> 1 = 4_611_686_018_427_387_904");

assert(new alanStd.I64(-9_223_372_036_854_775_808n).rotateLeft(new alanStd.I64(1n)) == 1n, "rotateLeftI64 -9_223_372_036_854_775_808 <<< 1 = 1");

assert(new alanStd.I64(4_611_686_018_427_387_904n).rotateRight(new alanStd.I64(1n)) == 2_305_843_009_213_693_952n, "rotateRightI64 4_611_686_018_427_387_904 >>> 1 = 2_305_843_009_213_693_952");

assert(new alanStd.U8(1).wrappingAdd(new alanStd.U8(2)).val === 3, "wrappingAddU8 1 + 2 = 3");
assert(new alanStd.U8(255).wrappingAdd(new alanStd.U8(1)).val === 0, "wrappingAddU8 255 + 1 = 0");

assert(new alanStd.U8(1).wrappingSub(new alanStd.U8(2)).val === 255, "wrappingSubU8 1 - 2 = 255");
assert(new alanStd.U8(255).wrappingSub(new alanStd.U8(1)).val === 254, "wrappingSubU8 255 - 1 = 254");

assert(new alanStd.U8(64).wrappingMul(new alanStd.U8(64)).val === 0, "wrappingMulU8 64 * 64 = 0");

assert(new alanStd.U8(128).wrappingDiv(new alanStd.U8(2)) == 64, "wrappingDivU8 128 / 2 = 64");

assert(new alanStd.U8(5).wrappingMod(new alanStd.U8(2)) == 1, "wrappingModU8 5 % 2 = 1");

assert(new alanStd.U8(2).wrappingPow(new alanStd.U8(8)) == 0, "wrappingPowU8 2 ^ 8 = 0");

assert(new alanStd.U8(0).not() == 255, "notU8 0 = 255");

assert(new alanStd.U8(128).wrappingShl(new alanStd.U8(1)) == 0, "wrappingShlU8 128 << 1 = 0");

assert(new alanStd.U8(128).wrappingShr(new alanStd.U8(1)) == 64, "wrappingShrU8 128 >> 1 = 64");

assert(new alanStd.U8(128).rotateLeft(new alanStd.U8(1)) == 1, "rotateLeftU8 128 <<< 1 = 1");
assert(new alanStd.U8(0b01010101).rotateLeft(new alanStd.U8(0)) == 0b01010101, "rotateLeftU8 0b01010101 <<< 0 = 0b01010101");
assert(new alanStd.U8(0b01010101).rotateLeft(new alanStd.U8(1)) == 0b10101010, "rotateLeftU8 0b01010101 <<< 1 = 0b10101010");
assert(new alanStd.U8(0b01010101).rotateLeft(new alanStd.U8(2)) == 0b01010101, "rotateLeftU8 0b01010101 <<< 2 = 0b01010101");
assert(new alanStd.U8(0b01010101).rotateLeft(new alanStd.U8(3)) == 0b10101010, "rotateLeftU8 0b01010101 <<< 3 = 0b10101010");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(0)) == 0b00000001, "rotateLeftU8 0b00000001 <<< 0 = 0b00000001");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(1)) == 0b00000010, "rotateLeftU8 0b00000001 <<< 1 = 0b00000010");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(2)) == 0b00000100, "rotateLeftU8 0b00000001 <<< 2 = 0b00000100");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(3)) == 0b00001000, "rotateLeftU8 0b00000001 <<< 3 = 0b00001000");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(4)) == 0b00010000, "rotateLeftU8 0b00000001 <<< 4 = 0b00010000");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(5)) == 0b00100000, "rotateLeftU8 0b00000001 <<< 5 = 0b00100000");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(6)) == 0b01000000, "rotateLeftU8 0b00000001 <<< 6 = 0b01000000");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(7)) == 0b10000000, "rotateLeftU8 0b00000001 <<< 7 = 0b10000000");
assert(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(8)) == 0b00000001, "rotateLeftU8 0b00000001 <<< 8 = 0b00000001");

assert(new alanStd.U8(64).rotateRight(new alanStd.U8(1)) == 32, "rotateRightU8 64 >>> 1 = 32");
assert(new alanStd.U8(0b01010101).rotateRight(new alanStd.U8(0)) == 0b01010101, "rotateRightU8 0b01010101 >>> 0 = 0b01010101");
assert(new alanStd.U8(0b01010101).rotateRight(new alanStd.U8(1)) == 0b10101010, "rotateRightU8 0b01010101 >>> 1 = 0b10101010");
assert(new alanStd.U8(0b01010101).rotateRight(new alanStd.U8(2)) == 0b01010101, "rotateRightU8 0b01010101 >>> 2 = 0b01010101");
assert(new alanStd.U8(0b01010101).rotateRight(new alanStd.U8(3)) == 0b10101010, "rotateRightU8 0b01010101 >>> 3 = 0b10101010");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(0)) == 0b00000001, "rotateRightU8 0b00000001 >>> 0 = 0b00000001");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(1)) == 0b10000000, "rotateRightU8 0b00000001 >>> 1 = 0b10000000");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(2)) == 0b01000000, "rotateRightU8 0b00000001 >>> 2 = 0b01000000");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(3)) == 0b00100000, "rotateRightU8 0b00000001 >>> 3 = 0b00100000");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(4)) == 0b00010000, "rotateRightU8 0b00000001 >>> 4 = 0b00010000");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(5)) == 0b00001000, "rotateRightU8 0b00000001 >>> 5 = 0b00001000");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(6)) == 0b00000100, "rotateRightU8 0b00000001 >>> 6 = 0b00000100");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(7)) == 0b00000010, "rotateRightU8 0b00000001 >>> 7 = 0b00000010");
assert(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(8)) == 0b00000001, "rotateRightU8 0b00000001 >>> 8 = 0b00000008");
assert(new alanStd.U8(100).rotateRight(new alanStd.U8(2)) == 25, "rotateRightU8 100 >>> 2 = 25");

assert(new alanStd.U16(1).wrappingAdd(new alanStd.U16(2)).val === 3, "wrappingAddU16 1 + 2 = 3");
assert(new alanStd.U16(65_535).wrappingAdd(new alanStd.U16(1)).val === 0, "wrappingAddU16 65_535 + 1 = 0");

assert(new alanStd.U16(1).wrappingSub(new alanStd.U16(2)).val === 65_535, "wrappingSubU16 1 - 2 = 65_535");
assert(new alanStd.U16(65_535).wrappingSub(new alanStd.U16(1)).val === 65_534, "wrappingSubU16 65_535 - 1 = 65_534");

assert(new alanStd.U16(256).wrappingMul(new alanStd.U16(256)).val === 0, "wrappingMulU16 256 * 256 = 0");

assert(new alanStd.U16(128).wrappingDiv(new alanStd.U16(2)) == 64, "wrappingDivU16 128 / 2 = 64");

assert(new alanStd.U16(5).wrappingMod(new alanStd.U16(2)) == 1, "wrappingModU16 5 % 2 = 1");

assert(new alanStd.U16(2).wrappingPow(new alanStd.U16(16)) == 0, "wrappingPowU16 2 ^ 16 = 0");

assert(new alanStd.U16(0).not() == 65_535, "notU16 0 = 65_535");

assert(new alanStd.U16(32_768).wrappingShl(new alanStd.U16(1)) == 0, "wrappingShlU16 32_768 << 1 = 0");

assert(new alanStd.U16(128).wrappingShr(new alanStd.U16(1)) == 64, "wrappingShrU16 128 >> 1 = 64");

assert(new alanStd.U16(32_768).rotateLeft(new alanStd.U16(1)) == 1, "rotateLeftU16 128 <<< 1 = 1");

assert(new alanStd.U16(64).rotateRight(new alanStd.U16(1)) == 32, "rotateRightU16 64 >>> 1 = 128");

assert(new alanStd.U32(1).wrappingAdd(new alanStd.U32(2)).val === 3, "wrappingAddU32 1 + 2 = 3");
assert(new alanStd.U32(4_294_967_295).wrappingAdd(new alanStd.U32(1)).val === 0, "wrappingAddU32 4_294_967_295 + 1 = 0");

assert(new alanStd.U32(1).wrappingSub(new alanStd.U32(2)).val === 4_294_967_295, "wrappingSubU32 1 - 2 = 4_294_967_295");
assert(new alanStd.U32(4_294_967_295).wrappingSub(new alanStd.U32(1)).val === 4_294_967_294, "wrappingSubU32 4_294_967_295 - 1 = 4_294_967_294");

assert(new alanStd.U32(65_536).wrappingMul(new alanStd.U32(65_536)).val === 0, "wrappingMulU32 65_536 * 65_536 = 0");

assert(new alanStd.U32(128).wrappingDiv(new alanStd.U32(2)) == 64, "wrappingDivU32 128 / 2 = 64");

assert(new alanStd.U32(5).wrappingMod(new alanStd.U32(2)) == 1, "wrappingModU32 5 % 2 = 1");

assert(new alanStd.U32(2).wrappingPow(new alanStd.U32(32)) == 0, "wrappingPowU32 2 ^ 32 = 0");

assert(new alanStd.U32(0).not() == 4_294_967_295, "notU32 0 = 4_294_967_295");

assert(new alanStd.U32(2_147_483_648).wrappingShl(new alanStd.U32(1)) == 0, "wrappingShlU32 2_147_483_648 << 1 = 0");

assert(new alanStd.U32(128).wrappingShr(new alanStd.U32(1)) == 64, "wrappingShrU32 128 >> 1 = 64");

assert(new alanStd.U32(2_147_483_648).rotateLeft(new alanStd.U32(1)) == 1, "rotateLeftU32 2_147_483_648 <<< 1 = 1");

assert(new alanStd.U32(64).rotateRight(new alanStd.U32(1)) == 32, "rotateRightU32 64 >>> 1 = 32");

assert(new alanStd.U64(1n).wrappingAdd(new alanStd.U64(2n)).val === 3n, "wrappingAddU64 1 + 2 = 3");
assert(new alanStd.U64(18_446_744_073_709_551_615n).wrappingAdd(new alanStd.U64(1n)).val === 0n, "wrappingAddU64 18_446_744_073_709_551_615 + 1 = 0");

assert(new alanStd.U64(1n).wrappingSub(new alanStd.U64(2n)).val === 18_446_744_073_709_551_615n, "wrappingSubU64 1 - 2 = 18_446_744_073_709_551_615");
assert(new alanStd.U64(18_446_744_073_709_551_615n).wrappingSub(new alanStd.U64(1n)).val === 18_446_744_073_709_551_614n, "wrappingSubU64 18_446_744_073_709_551_615 - 1 = 18_446_744_073_709_551_614");

assert(new alanStd.U64(4_294_967_296n).wrappingMul(new alanStd.U64(4_294_967_296n)).val === 0n, "wrappingMulU64 4_294_967_296 * 4_294_967_296 = 0");

assert(new alanStd.U64(128n).wrappingDiv(new alanStd.U64(2n)) == 64n, "wrappingDivU64 128 / 2 = 64");

assert(new alanStd.U64(5n).wrappingMod(new alanStd.U64(2n)) == 1n, "wrappingModU64 5 % 2 = 1");

assert(new alanStd.U64(2n).wrappingPow(new alanStd.U64(64n)) == 0n, "wrappingPowU64 2 ^ 64 = 0");

assert(new alanStd.U64(0n).not() == 18_446_744_073_709_551_615n, "notU64 0 = 18_446_744_073_709_551_615");

assert(new alanStd.U64(9_223_372_036_854_775_808n).wrappingShl(new alanStd.U64(1n)) == 0n, "wrappingShlU64 9_223_372_036_854_775_808 << 1 = 0");

assert(new alanStd.U64(128n).wrappingShr(new alanStd.U64(1n)) == 64n, "wrappingShrU64 128 >> 1 = 64");

assert(new alanStd.U64(9_223_372_036_854_775_808n).rotateLeft(new alanStd.U64(1n)) == 1n, "rotateLeftU64 9_223_372_036_854_775_808 <<< 1 = 1");
assert(new alanStd.U64(100n).rotateLeft(new alanStd.U64(2n)) == 400n, "rotateLeftU64 100 <<< 2 = 400");

assert(new alanStd.U64(64n).rotateRight(new alanStd.U64(1n)) == 32n, "rotateRightU64 64 >>> 1 = 32");
