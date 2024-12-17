import assert from "node:assert";

import * as alanStd from "./alan_std.js";

assert(new alanStd.AlanError("foo") instanceof alanStd.AlanError, "AlanError constructor");
assert.strictEqual(new alanStd.AlanError("foo").message, "foo", "AlanError message");

assert.strictEqual(alanStd.nanToError(5), 5, "nanToError normal number");
assert.strictEqual(alanStd.nanToError(NaN).message, "Not a Number", "nanToError NaN");

assert.strictEqual(new alanStd.I8(0).val, 0, "I8 0");
assert.strictEqual(new alanStd.I8(150).val, 127, "I8 150");
assert.strictEqual(new alanStd.I8(-150).val, -128, "I8 -150");

assert.strictEqual(new alanStd.I16(0).val, 0, "clampI16 0");
assert.strictEqual(new alanStd.I16(40_000).val, 32_767, "clampI16 40k");
assert.strictEqual(new alanStd.I16(-40_000).val, -32_768, "clampI16 -40k");

assert.strictEqual(new alanStd.I32(0).val, 0, "clampI32 0");
assert.strictEqual(new alanStd.I32(3_000_000_000).val, 2_147_483_647, "clampI32 3B");
assert.strictEqual(new alanStd.I32(-3_000_000_000).val, -2_147_483_648, "clampI32 -3B");

assert.strictEqual(new alanStd.I64(0n).val, 0n, "clampI64 0n");
assert.strictEqual(new alanStd.I64(10_000_000_000_000_000_000n).val, 9_223_372_036_854_775_807n, "clampI64 10Q");
assert.strictEqual(new alanStd.I64(-10_000_000_000_000_000_000n).val, -9_223_372_036_854_775_808n, "clampI64 -10Q");

assert.strictEqual(new alanStd.U8(0).val, 0, "clampU8 0");
assert.strictEqual(new alanStd.U8(350).val, 255, "clampU8 350");
assert.strictEqual(new alanStd.U8(-150).val, 0, "clampU8 -150");

assert.strictEqual(new alanStd.U16(0).val, 0, "clampU16 0");
assert.strictEqual(new alanStd.U16(80_000).val, 65_535, "clampU16 80k");
assert.strictEqual(new alanStd.U16(-40_000).val, 0, "clampU16 -40k");

assert.strictEqual(new alanStd.U32(0).val, 0, "clampU32 0");
assert.strictEqual(new alanStd.U32(6_000_000_000).val, 4_294_967_295, "clampU32 6B");
assert.strictEqual(new alanStd.U32(-3_000_000_000).val, 0, "clampU32 -3B");

assert.strictEqual(new alanStd.U64(0n).val, 0n, "clampU64 0n");
assert.strictEqual(new alanStd.U64(20_000_000_000_000_000_000n).val, 18_446_744_073_709_551_615n, "clampU64 20Q");
assert.strictEqual(new alanStd.U64(-10_000_000_000_000_000_000n).val, 0n, "clampU64 -10Q");

assert.strictEqual(alanStd.ifbool(true, () => true, () => false), true, "ifbool true");
assert.strictEqual(alanStd.ifbool(false, () => true, () => false), false, "ifbool false");

assert.strictEqual(new alanStd.I8(1).wrappingAdd(new alanStd.I8(2)).val, 3, "wrappingAddI8 1 + 2 = 3");
assert.strictEqual(new alanStd.I8(127).wrappingAdd(new alanStd.I8(1)).val, -128, "wrappingAddI8 127 + 1 = -128");

assert.strictEqual(new alanStd.I8(1).wrappingSub(new alanStd.I8(2)).val, -1, "wrappingSubI8 1 - 2 = -1");
assert.strictEqual(new alanStd.I8(-128).wrappingSub(new alanStd.I8(1)).val, 127, "wrappingSubI8 -128 - 1 = 127");

assert.strictEqual(new alanStd.I8(64).wrappingMul(new alanStd.I8(64)).val, 0, "wrappingMulI8 64 * 64 = 0");

assert.equal(new alanStd.I8(-128).wrappingDiv(new alanStd.I8(2)), -64, "wrappingDivI8 -128 / 2 = -64");

assert.equal(new alanStd.I8(5).wrappingMod(new alanStd.I8(2)), 1, "wrappingModI8 5 % 2 = 1");

assert.equal(new alanStd.I8(2).wrappingPow(new alanStd.I8(8)), 0, "wrappingPowI8 2 ^ 8 = 0");

assert.equal(new alanStd.I8(-128).wrappingShl(new alanStd.I8(1)), 0, "wrappingShlI8 -128 << 1 = 0");

assert.equal(new alanStd.I8(-128).wrappingShr(new alanStd.I8(1)), 64, "wrappingShrI8 -128 >> 1 = 64");

assert.equal(new alanStd.I8(-128).rotateLeft(new alanStd.I8(1)), 1, "rotateLeftI8 -128 <<< 1 = 1");
assert.equal(new alanStd.I8(0b01010101).rotateLeft(new alanStd.I8(0)), 0b01010101, "rotateLeftI8 0b01010101 <<< 0 = 0b01010101");
assert.equal(new alanStd.I8(0b01010101).rotateLeft(new alanStd.I8(1)), 0b10101010 - 256, "rotateLeftI8 0b01010101 <<< 1 = 0b10101010");
assert.equal(new alanStd.I8(0b01010101).rotateLeft(new alanStd.I8(2)), 0b01010101, "rotateLeftI8 0b01010101 <<< 2 = 0b01010101");
assert.equal(new alanStd.I8(0b01010101).rotateLeft(new alanStd.I8(3)), 0b10101010 - 256, "rotateLeftI8 0b01010101 <<< 3 = 0b10101010");

assert.equal(new alanStd.I8(64).rotateRight(new alanStd.I8(1)).val, 32, "rotateRightI8 64 >>> 1 = 32");
assert.equal(new alanStd.I8(0b01010101).rotateRight(new alanStd.I8(0)).val, 0b01010101, "rotateRightI8 0b01010101 <<< 0 = 0b01010101");
assert.equal(new alanStd.I8(0b01010101).rotateRight(new alanStd.I8(1)).val, 0b10101010 - 256, "rotateRightI8 0b01010101 <<< 1 = 0b10101010");
assert.equal(new alanStd.I8(0b01010101).rotateRight(new alanStd.I8(2)).val, 0b01010101, "rotateRightI8 0b01010101 <<< 2 = 0b01010101");
assert.equal(new alanStd.I8(0b01010101).rotateRight(new alanStd.I8(3)).val, 0b10101010 - 256, "rotateRightI8 0b01010101 <<< 3 = 0b10101010");

assert.equal(new alanStd.I8(0b01010101).reverseBits(), 0b10101010 - 256, "reverseBitsI8 0b01010101 = 0b10101010");
assert.equal(new alanStd.I8(0b00000100).reverseBits(), 0b00100000, "reverseBitsI8 0b00000100 = 0b00100000");

assert.strictEqual(new alanStd.I16(1).wrappingAdd(new alanStd.I16(2)).val, 3, "wrappingAddI16 1 + 2 = 3");
assert.strictEqual(new alanStd.I16(32_767).wrappingAdd(new alanStd.I16(1)).val, -32_768, "wrappingAddI16 32_767 + 1 = -32_768");

assert.strictEqual(new alanStd.I16(1).wrappingSub(new alanStd.I16(2)).val, -1, "wrappingSubI16 1 - 2 = -1");
assert.strictEqual(new alanStd.I16(-32_768).wrappingSub(new alanStd.I16(1)).val, 32_767, "wrappingSubI16 -32_768 - 1 = 32_767");

assert.strictEqual(new alanStd.I16(256).wrappingMul(new alanStd.I16(256)).val, 0, "wrappingMulI16 256 * 256 = 0");

assert.equal(new alanStd.I16(-32_768).wrappingDiv(new alanStd.I16(2)), -16_384, "wrappingDivI16 -32_768 / 2 = -16_384");

assert.equal(new alanStd.I16(5).wrappingMod(new alanStd.I16(2)), 1, "wrappingModI16 5 % 2 = 1");

assert.equal(new alanStd.I16(2).wrappingPow(new alanStd.I16(16)), 0, "wrappingPowI16 2 ^ 16 = 0");

assert.equal(new alanStd.I16(-32_768).wrappingShl(new alanStd.I16(1)), 0, "wrappingShlI16 -32_768 << 1 = 0");

assert.equal(new alanStd.I16(-32_768).wrappingShr(new alanStd.I16(1)), 16_384, "wrappingShrI16 -32_768 >> 1 = 16_384");

assert.equal(new alanStd.I16(-32_768).rotateLeft(new alanStd.I16(1)), 1, "rotateLeftI16 -32_768 <<< 1 = 1");

assert.equal(new alanStd.I16(16_384).rotateRight(new alanStd.I16(1)), 8_192, "rotateRightI16 16_384 >>> 1 = -32_768");

assert.equal(new alanStd.I16(1).reverseBits(), -32768, "reverseBitsI16 1 = -32768");
assert.equal(new alanStd.I16(4).reverseBits(), 8192, "reverseBitsI16 4 = 8192");

assert.strictEqual(new alanStd.I32(1).wrappingAdd(new alanStd.I32(2)).val, 3, "wrappingAddI32 1 + 2 = 3");
assert.strictEqual(new alanStd.I32(2_147_483_647).wrappingAdd(new alanStd.I32(1)).val, -2_147_483_648, "wrappingAddI32 2_147_483_647 + 1 = -2_147_483_648");

assert.strictEqual(new alanStd.I32(1).wrappingSub(new alanStd.I32(2)).val, -1, "wrappingSubI32 1 - 2 = -1");
assert.strictEqual(new alanStd.I32(-2_147_483_648).wrappingSub(new alanStd.I32(1)).val, 2_147_483_647, "wrappingSubI32 -2_147_483_648 - 1 = 2_147_483_647");

assert.strictEqual(new alanStd.I32(65_536).wrappingMul(new alanStd.I32(65_536)).val, 0, "wrappingMulI32 65_536 * 65_536 = 0");

assert.equal(new alanStd.I32(-2_147_483_648).wrappingDiv(new alanStd.I32(2)), -1_073_741_824, "wrappingDivI32 -2_147_483_648 / 2 = -1_073_741_824");

assert.equal(new alanStd.I32(5).wrappingMod(new alanStd.I32(2)), 1, "wrappingModI32 5 % 2 = 1");

assert.equal(new alanStd.I32(2).wrappingPow(new alanStd.I32(32)), 0, "wrappingPowI32 2 ^ 32 = 0");

assert.equal(new alanStd.I32(-2_147_483_648).wrappingShl(new alanStd.I32(1)), 0, "wrappingShlI32 -2_147_483_648 << 1 = 0");

assert.equal(new alanStd.I32(-2_147_483_648).wrappingShr(new alanStd.I32(1)), 1_073_741_824, "wrappingShrI32 -2_147_483_648 >> 1 = 1_073_741_824");

assert.equal(new alanStd.I32(-2_147_483_648).rotateLeft(new alanStd.I32(1)), 1, "rotateLeftI32 -2_147_483_648 <<< 1 = 1");

assert.equal(new alanStd.I32(1_073_741_824).rotateRight(new alanStd.I32(1)), 536_870_912, "rotateRightI32 1_073_741_824 >>> 1 = 536_870_912");

assert.equal(new alanStd.I32(1).reverseBits(), -2_147_483_648, "reverseBitsI32 1 = -2_147_483_648");
assert.equal(new alanStd.I32(4).reverseBits(), 536_870_912, "reverseBitsI32 4 = 536_870_912");

assert.strictEqual(new alanStd.I64(1n).wrappingAdd(new alanStd.I64(2n)).val, 3n, "wrappingAddI64 1 + 2 = 3");
assert.strictEqual(new alanStd.I64(9_223_372_036_854_775_807n).wrappingAdd(new alanStd.I64(1n)).val, -9_223_372_036_854_775_808n, "wrappingAddI64 9_223_372_036_854_775_807 + 1 = -9_223_372_036_854_775_808");

assert.strictEqual(new alanStd.I64(1n).wrappingSub(new alanStd.I64(2n)).val, -1n, "wrappingSubI64 1 - 2 = -1");
assert.strictEqual(new alanStd.I64(-9_223_372_036_854_775_808n).wrappingSub(new alanStd.I64(1n)).val, 9_223_372_036_854_775_807n, "wrappingSubI64 -9_223_372_036_854_775_808 - 1 = 9_223_372_036_854_775_807");

assert.strictEqual(new alanStd.I64(4_294_967_296n).wrappingMul(new alanStd.I64(4_294_967_296n)).val, 0n, "wrappingMulI64 4_294_967_296 * 4_294_967_296 = 0");

assert.equal(new alanStd.I64(-9_223_372_036_854_775_808n).wrappingDiv(new alanStd.I64(2n)), -4_611_686_018_427_387_904n, "wrappingDivI64 -9_223_372_036_854_775_808 / 2 = −4_611_686_018_427_387_904");

assert.equal(new alanStd.I64(5n).wrappingMod(new alanStd.I64(2n)), 1n, "wrappingModI64 5 % 2 = 1");

assert.equal(new alanStd.I64(2n).wrappingPow(new alanStd.I64(64n)), 0n, "wrappingPowI64 2 ^ 64 = 0");

assert.equal(new alanStd.I64(-9_223_372_036_854_775_808n).wrappingShl(new alanStd.I64(1n)), 0n, "wrappingShlI64 -9_223_372_036_854_775_808 << 1 = 0");

assert.equal(new alanStd.I64(-9_223_372_036_854_775_808n).wrappingShr(new alanStd.I64(1n)), 4_611_686_018_427_387_904n, "wrappingShrI64 -9_223_372_036_854_775_808 >> 1 = 4_611_686_018_427_387_904");

assert.equal(new alanStd.I64(-9_223_372_036_854_775_808n).rotateLeft(new alanStd.I64(1n)), 1n, "rotateLeftI64 -9_223_372_036_854_775_808 <<< 1 = 1");

assert.equal(new alanStd.I64(4_611_686_018_427_387_904n).rotateRight(new alanStd.I64(1n)), 2_305_843_009_213_693_952n, "rotateRightI64 4_611_686_018_427_387_904 >>> 1 = 2_305_843_009_213_693_952");

assert.equal(new alanStd.I64(1).reverseBits(), -9_223_372_036_854_775_808n, "reverseBitsI64 1 = -9_223_372_036_854_775_808");
assert.equal(new alanStd.I64(4).reverseBits(), 2_305_843_009_213_693_952n, "reverseBitsI64 4 = 2_305_843_009_213_693_952");

assert.strictEqual(new alanStd.U8(1).wrappingAdd(new alanStd.U8(2)).val, 3, "wrappingAddU8 1 + 2 = 3");
assert.strictEqual(new alanStd.U8(255).wrappingAdd(new alanStd.U8(1)).val, 0, "wrappingAddU8 255 + 1 = 0");

assert.strictEqual(new alanStd.U8(1).wrappingSub(new alanStd.U8(2)).val, 255, "wrappingSubU8 1 - 2 = 255");
assert.strictEqual(new alanStd.U8(255).wrappingSub(new alanStd.U8(1)).val, 254, "wrappingSubU8 255 - 1 = 254");

assert.strictEqual(new alanStd.U8(64).wrappingMul(new alanStd.U8(64)).val, 0, "wrappingMulU8 64 * 64 = 0");

assert.equal(new alanStd.U8(128).wrappingDiv(new alanStd.U8(2)), 64, "wrappingDivU8 128 / 2 = 64");

assert.equal(new alanStd.U8(5).wrappingMod(new alanStd.U8(2)), 1, "wrappingModU8 5 % 2 = 1");

assert.equal(new alanStd.U8(2).wrappingPow(new alanStd.U8(8)), 0, "wrappingPowU8 2 ^ 8 = 0");

assert.equal(new alanStd.U8(0).not(), 255, "notU8 0 = 255");

assert.equal(new alanStd.U8(128).wrappingShl(new alanStd.U8(1)), 0, "wrappingShlU8 128 << 1 = 0");

assert.equal(new alanStd.U8(128).wrappingShr(new alanStd.U8(1)), 64, "wrappingShrU8 128 >> 1 = 64");

assert.equal(new alanStd.U8(128).rotateLeft(new alanStd.U8(1)), 1, "rotateLeftU8 128 <<< 1 = 1");
assert.equal(new alanStd.U8(0b01010101).rotateLeft(new alanStd.U8(0)), 0b01010101, "rotateLeftU8 0b01010101 <<< 0 = 0b01010101");
assert.equal(new alanStd.U8(0b01010101).rotateLeft(new alanStd.U8(1)), 0b10101010, "rotateLeftU8 0b01010101 <<< 1 = 0b10101010");
assert.equal(new alanStd.U8(0b01010101).rotateLeft(new alanStd.U8(2)), 0b01010101, "rotateLeftU8 0b01010101 <<< 2 = 0b01010101");
assert.equal(new alanStd.U8(0b01010101).rotateLeft(new alanStd.U8(3)), 0b10101010, "rotateLeftU8 0b01010101 <<< 3 = 0b10101010");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(0)), 0b00000001, "rotateLeftU8 0b00000001 <<< 0 = 0b00000001");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(1)), 0b00000010, "rotateLeftU8 0b00000001 <<< 1 = 0b00000010");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(2)), 0b00000100, "rotateLeftU8 0b00000001 <<< 2 = 0b00000100");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(3)), 0b00001000, "rotateLeftU8 0b00000001 <<< 3 = 0b00001000");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(4)), 0b00010000, "rotateLeftU8 0b00000001 <<< 4 = 0b00010000");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(5)), 0b00100000, "rotateLeftU8 0b00000001 <<< 5 = 0b00100000");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(6)), 0b01000000, "rotateLeftU8 0b00000001 <<< 6 = 0b01000000");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(7)), 0b10000000, "rotateLeftU8 0b00000001 <<< 7 = 0b10000000");
assert.equal(new alanStd.U8(0b00000001).rotateLeft(new alanStd.U8(8)), 0b00000001, "rotateLeftU8 0b00000001 <<< 8 = 0b00000001");

assert.equal(new alanStd.U8(64).rotateRight(new alanStd.U8(1)), 32, "rotateRightU8 64 >>> 1 = 32");
assert.equal(new alanStd.U8(0b01010101).rotateRight(new alanStd.U8(0)), 0b01010101, "rotateRightU8 0b01010101 >>> 0 = 0b01010101");
assert.equal(new alanStd.U8(0b01010101).rotateRight(new alanStd.U8(1)), 0b10101010, "rotateRightU8 0b01010101 >>> 1 = 0b10101010");
assert.equal(new alanStd.U8(0b01010101).rotateRight(new alanStd.U8(2)), 0b01010101, "rotateRightU8 0b01010101 >>> 2 = 0b01010101");
assert.equal(new alanStd.U8(0b01010101).rotateRight(new alanStd.U8(3)), 0b10101010, "rotateRightU8 0b01010101 >>> 3 = 0b10101010");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(0)), 0b00000001, "rotateRightU8 0b00000001 >>> 0 = 0b00000001");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(1)), 0b10000000, "rotateRightU8 0b00000001 >>> 1 = 0b10000000");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(2)), 0b01000000, "rotateRightU8 0b00000001 >>> 2 = 0b01000000");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(3)), 0b00100000, "rotateRightU8 0b00000001 >>> 3 = 0b00100000");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(4)), 0b00010000, "rotateRightU8 0b00000001 >>> 4 = 0b00010000");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(5)), 0b00001000, "rotateRightU8 0b00000001 >>> 5 = 0b00001000");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(6)), 0b00000100, "rotateRightU8 0b00000001 >>> 6 = 0b00000100");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(7)), 0b00000010, "rotateRightU8 0b00000001 >>> 7 = 0b00000010");
assert.equal(new alanStd.U8(0b00000001).rotateRight(new alanStd.U8(8)), 0b00000001, "rotateRightU8 0b00000001 >>> 8 = 0b00000008");
assert.equal(new alanStd.U8(100).rotateRight(new alanStd.U8(2)), 25, "rotateRightU8 100 >>> 2 = 25");

assert.equal(new alanStd.U8(0b01010101).reverseBits(), 0b10101010, "reverseBitsU8 0b01010101 = 0b10101010");
assert.equal(new alanStd.U8(0b00000100).reverseBits(), 0b00100000, "reverseBitsU8 0b00000100 = 0b00100000");

assert.strictEqual(new alanStd.U16(1).wrappingAdd(new alanStd.U16(2)).val, 3, "wrappingAddU16 1 + 2 = 3");
assert.strictEqual(new alanStd.U16(65_535).wrappingAdd(new alanStd.U16(1)).val, 0, "wrappingAddU16 65_535 + 1 = 0");

assert.strictEqual(new alanStd.U16(1).wrappingSub(new alanStd.U16(2)).val, 65_535, "wrappingSubU16 1 - 2 = 65_535");
assert.strictEqual(new alanStd.U16(65_535).wrappingSub(new alanStd.U16(1)).val, 65_534, "wrappingSubU16 65_535 - 1 = 65_534");

assert.strictEqual(new alanStd.U16(256).wrappingMul(new alanStd.U16(256)).val, 0, "wrappingMulU16 256 * 256 = 0");

assert.equal(new alanStd.U16(128).wrappingDiv(new alanStd.U16(2)), 64, "wrappingDivU16 128 / 2 = 64");

assert.equal(new alanStd.U16(5).wrappingMod(new alanStd.U16(2)), 1, "wrappingModU16 5 % 2 = 1");

assert.equal(new alanStd.U16(2).wrappingPow(new alanStd.U16(16)), 0, "wrappingPowU16 2 ^ 16 = 0");

assert.equal(new alanStd.U16(0).not(), 65_535, "notU16 0 = 65_535");

assert.equal(new alanStd.U16(32_768).wrappingShl(new alanStd.U16(1)), 0, "wrappingShlU16 32_768 << 1 = 0");

assert.equal(new alanStd.U16(128).wrappingShr(new alanStd.U16(1)), 64, "wrappingShrU16 128 >> 1 = 64");

assert.equal(new alanStd.U16(32_768).rotateLeft(new alanStd.U16(1)), 1, "rotateLeftU16 128 <<< 1 = 1");

assert.equal(new alanStd.U16(64).rotateRight(new alanStd.U16(1)), 32, "rotateRightU16 64 >>> 1 = 128");

assert.equal(new alanStd.U16(1).reverseBits(), 32768, "reverseBitsU16 1 = 32768");
assert.equal(new alanStd.U16(4).reverseBits(), 8192, "reverseBitsU16 4 = 8192");

assert.strictEqual(new alanStd.U32(1).wrappingAdd(new alanStd.U32(2)).val, 3, "wrappingAddU32 1 + 2 = 3");
assert.strictEqual(new alanStd.U32(4_294_967_295).wrappingAdd(new alanStd.U32(1)).val, 0, "wrappingAddU32 4_294_967_295 + 1 = 0");

assert.strictEqual(new alanStd.U32(1).wrappingSub(new alanStd.U32(2)).val, 4_294_967_295, "wrappingSubU32 1 - 2 = 4_294_967_295");
assert.strictEqual(new alanStd.U32(4_294_967_295).wrappingSub(new alanStd.U32(1)).val, 4_294_967_294, "wrappingSubU32 4_294_967_295 - 1 = 4_294_967_294");

assert.strictEqual(new alanStd.U32(65_536).wrappingMul(new alanStd.U32(65_536)).val, 0, "wrappingMulU32 65_536 * 65_536 = 0");

assert.equal(new alanStd.U32(128).wrappingDiv(new alanStd.U32(2)), 64, "wrappingDivU32 128 / 2 = 64");

assert.equal(new alanStd.U32(5).wrappingMod(new alanStd.U32(2)), 1, "wrappingModU32 5 % 2 = 1");

assert.equal(new alanStd.U32(2).wrappingPow(new alanStd.U32(32)), 0, "wrappingPowU32 2 ^ 32 = 0");

assert.equal(new alanStd.U32(0).not(), 4_294_967_295, "notU32 0 = 4_294_967_295");

assert.equal(new alanStd.U32(2_147_483_648).wrappingShl(new alanStd.U32(1)), 0, "wrappingShlU32 2_147_483_648 << 1 = 0");

assert.equal(new alanStd.U32(128).wrappingShr(new alanStd.U32(1)), 64, "wrappingShrU32 128 >> 1 = 64");

assert.equal(new alanStd.U32(2_147_483_648).rotateLeft(new alanStd.U32(1)), 1, "rotateLeftU32 2_147_483_648 <<< 1 = 1");

assert.equal(new alanStd.U32(64).rotateRight(new alanStd.U32(1)), 32, "rotateRightU32 64 >>> 1 = 32");

assert.equal(new alanStd.U32(1).reverseBits(), 2_147_483_648, "reverseBitsU32 1 = 2_147_483_648");
assert.equal(new alanStd.U32(4).reverseBits(), 536_870_912, "reverseBitsU32 4 = 536_870_912");

assert.strictEqual(new alanStd.U64(1n).wrappingAdd(new alanStd.U64(2n)).val, 3n, "wrappingAddU64 1 + 2 = 3");
assert.strictEqual(new alanStd.U64(18_446_744_073_709_551_615n).wrappingAdd(new alanStd.U64(1n)).val, 0n, "wrappingAddU64 18_446_744_073_709_551_615 + 1 = 0");

assert.strictEqual(new alanStd.U64(1n).wrappingSub(new alanStd.U64(2n)).val, 18_446_744_073_709_551_615n, "wrappingSubU64 1 - 2 = 18_446_744_073_709_551_615");
assert.strictEqual(new alanStd.U64(18_446_744_073_709_551_615n).wrappingSub(new alanStd.U64(1n)).val, 18_446_744_073_709_551_614n, "wrappingSubU64 18_446_744_073_709_551_615 - 1 = 18_446_744_073_709_551_614");

assert.strictEqual(new alanStd.U64(4_294_967_296n).wrappingMul(new alanStd.U64(4_294_967_296n)).val, 0n, "wrappingMulU64 4_294_967_296 * 4_294_967_296 = 0");

assert.equal(new alanStd.U64(128n).wrappingDiv(new alanStd.U64(2n)), 64n, "wrappingDivU64 128 / 2 = 64");

assert.equal(new alanStd.U64(5n).wrappingMod(new alanStd.U64(2n)), 1n, "wrappingModU64 5 % 2 = 1");

assert.equal(new alanStd.U64(2n).wrappingPow(new alanStd.U64(64n)), 0n, "wrappingPowU64 2 ^ 64 = 0");

assert.equal(new alanStd.U64(0n).not(), 18_446_744_073_709_551_615n, "notU64 0 = 18_446_744_073_709_551_615");

assert.equal(new alanStd.U64(9_223_372_036_854_775_808n).wrappingShl(new alanStd.U64(1n)), 0n, "wrappingShlU64 9_223_372_036_854_775_808 << 1 = 0");

assert.equal(new alanStd.U64(128n).wrappingShr(new alanStd.U64(1n)), 64n, "wrappingShrU64 128 >> 1 = 64");

assert.equal(new alanStd.U64(9_223_372_036_854_775_808n).rotateLeft(new alanStd.U64(1n)), 1n, "rotateLeftU64 9_223_372_036_854_775_808 <<< 1 = 1");
assert.equal(new alanStd.U64(100n).rotateLeft(new alanStd.U64(2n)), 400n, "rotateLeftU64 100 <<< 2 = 400");

assert.equal(new alanStd.U64(64n).rotateRight(new alanStd.U64(1n)), 32n, "rotateRightU64 64 >>> 1 = 32");

assert.equal(new alanStd.U64(1).reverseBits(), 9_223_372_036_854_775_808n, "reverseBitsU64 1 = 9_223_372_036_854_775_808");
assert.equal(new alanStd.U64(4).reverseBits(), 2_305_843_009_213_693_952n, "reverseBitsU64 4 = 2_305_843_009_213_693_952");

assert.equal(new alanStd.U8(255).clz(), 0, "clzU8(255) = 0");
assert.equal(new alanStd.U8(0).clz(), 8, "clzU8(0) = 8");
assert.equal(new alanStd.U8(1).clz(), 7, "clzU8(1) = 7");
assert.equal(new alanStd.U8(2).clz(), 6, "clzU8(2) = 6");
assert.equal(new alanStd.U8(4).clz(), 5, "clzU8(4) = 5");
assert.equal(new alanStd.U8(8).clz(), 4, "clzU8(8) = 4");
assert.equal(new alanStd.U8(16).clz(), 3, "clzU8(16) = 3");
assert.equal(new alanStd.U8(32).clz(), 2, "clzU8(32) = 2");
assert.equal(new alanStd.U8(64).clz(), 1, "clzU8(64) = 1");
assert.equal(new alanStd.U8(128).clz(), 0, "clzU8(128) = 0");

assert.equal(new alanStd.I8(-1).clz(), 0, "clzI8(-1) = 0");
assert.equal(new alanStd.I8(0).clz(), 8, "clzI8(0) = 8");
assert.equal(new alanStd.I8(1).clz(), 7, "clzI8(1) = 7");
assert.equal(new alanStd.I8(16).clz(), 3, "clzI8(16) = 3");
assert.equal(new alanStd.I8(-128).clz(), 0, "clzI8(-128) = 0");

assert.equal(new alanStd.U16(65535).clz(), 0, "clzU16(65535) = 0");
assert.equal(new alanStd.U16(0).clz(), 16, "clzU16(0) = 16");
assert.equal(new alanStd.U16(1).clz(), 15, "clzU16(1) = 15");
assert.equal(new alanStd.U16(16).clz(), 11, "clzU16(16) = 11");
assert.equal(new alanStd.U16(32768).clz(), 0, "clzU16(32768) = 0");

assert.equal(new alanStd.I16(-1).clz(), 0, "clzI16(-1) = 0");
assert.equal(new alanStd.I16(0).clz(), 16, "clzI16(0) = 16");
assert.equal(new alanStd.I16(1).clz(), 15, "clzI16(1) = 15");
assert.equal(new alanStd.I16(16).clz(), 11, "clzI16(16) = 11");
assert.equal(new alanStd.I16(-32768).clz(), 0, "clzI16(-32768) = 0");

assert.equal(new alanStd.U32(4_294_967_295).clz(), 0, "clzU32(4_294_967_295) = 0");
assert.equal(new alanStd.U32(0).clz(), 32, "clzU32(0) = 32");
assert.equal(new alanStd.U32(1).clz(), 31, "clzU32(1) = 31");
assert.equal(new alanStd.U32(16).clz(), 27, "clzU32(32) = 27");
assert.equal(new alanStd.U32(2_147_483_648).clz(), 0, "clzU32(2_147_483_648) = 0");

assert.equal(new alanStd.I32(-1).clz(), 0, "clzI32(-1) = 0");
assert.equal(new alanStd.I32(0).clz(), 32, "clzI32(0) = 32");
assert.equal(new alanStd.I32(1).clz(), 31, "clzI32(1) = 31");
assert.equal(new alanStd.I32(16).clz(), 27, "clzI32(16) = 27");
assert.equal(new alanStd.I32(-2_147_483_648).clz(), 0, "clzI32(-2_147_483_648) = 0");

assert.equal(new alanStd.U64(18_446_744_073_709_551_615n).clz(), 0, "clzU64(18_446_744_073_709_551_615n) = 0");
assert.equal(new alanStd.U64(0).clz(), 64, "clzU64(0) = 64");
assert.equal(new alanStd.U64(1).clz(), 63, "clzU64(1) = 63");
assert.equal(new alanStd.U64(16).clz(), 59, "clzU64(64) = 59");
assert.equal(new alanStd.U64(9_223_372_036_854_775_808n).clz(), 0, "clzU64(9_223_372_036_854_775_808n) = 0");

assert.equal(new alanStd.I64(-1).clz(), 0, "clzI64(-1) = 0");
assert.equal(new alanStd.I64(0).clz(), 64, "clzI64(0) = 64");
assert.equal(new alanStd.I64(1).clz(), 63, "clzI64(1) = 63");
assert.equal(new alanStd.I64(16).clz(), 59, "clzI64(16) = 59");
assert.equal(new alanStd.I64(-9_223_372_036_854_775_808n).clz(), 0, "clzI64(-9_223_372_036_854_775_808n) = 0");

assert.equal(new alanStd.U8(0).ones(), 0, "onesU8(0) = 0");
assert.equal(new alanStd.U8(1).ones(), 1, "onesU8(1) = 1");
assert.equal(new alanStd.U8(2).ones(), 1, "onesU8(2) = 1");
assert.equal(new alanStd.U8(3).ones(), 2, "onesU8(3) = 2");
assert.equal(new alanStd.U8(255).ones(), 8, "onesU8(255) = 8");

assert.equal(new alanStd.I8(0).ones(), 0, "onesI8(0) = 0");
assert.equal(new alanStd.I8(1).ones(), 1, "onesI8(1) = 1");
assert.equal(new alanStd.I8(2).ones(), 1, "onesI8(2) = 1");
assert.equal(new alanStd.I8(3).ones(), 2, "onesI8(3) = 2");
assert.equal(new alanStd.I8(-1).ones(), 8, "onesI8(-1) = 8");

assert.equal(new alanStd.U16(0).ones(), 0, "onesU16(0) = 0");
assert.equal(new alanStd.U16(1).ones(), 1, "onesU16(1) = 1");
assert.equal(new alanStd.U16(2).ones(), 1, "onesU16(2) = 1");
assert.equal(new alanStd.U16(3).ones(), 2, "onesU16(3) = 2");
assert.equal(new alanStd.U16(65535).ones(), 16, "onesU16(65535) = 16");

assert.equal(new alanStd.I16(0).ones(), 0, "onesI16(0) = 0");
assert.equal(new alanStd.I16(1).ones(), 1, "onesI16(1) = 1");
assert.equal(new alanStd.I16(2).ones(), 1, "onesI16(2) = 1");
assert.equal(new alanStd.I16(3).ones(), 2, "onesI16(3) = 2");
assert.equal(new alanStd.I16(-1).ones(), 16, "onesI16(-1) = 16");

assert.equal(new alanStd.U32(0).ones(), 0, "onesU32(0) = 0");
assert.equal(new alanStd.U32(1).ones(), 1, "onesU32(1) = 1");
assert.equal(new alanStd.U32(2).ones(), 1, "onesU32(2) = 1");
assert.equal(new alanStd.U32(3).ones(), 2, "onesU32(3) = 2");
assert.equal(new alanStd.U32(4_294_967_295).ones(), 32, "onesU32(4_294_967_295) = 32");

assert.equal(new alanStd.I32(0).ones(), 0, "onesI32(0) = 0");
assert.equal(new alanStd.I32(1).ones(), 1, "onesI32(1) = 1");
assert.equal(new alanStd.I32(2).ones(), 1, "onesI32(2) = 1");
assert.equal(new alanStd.I32(3).ones(), 2, "onesI32(3) = 2");
assert.equal(new alanStd.I32(-1).ones(), 32, "onesI32(-1) = 32");

assert.equal(new alanStd.U64(0).ones(), 0, "onesU64(0) = 0");
assert.equal(new alanStd.U64(1).ones(), 1, "onesU64(1) = 1");
assert.equal(new alanStd.U64(2).ones(), 1, "onesU64(2) = 1");
assert.equal(new alanStd.U64(3).ones(), 2, "onesU64(3) = 2");
assert.equal(new alanStd.U64(18_446_744_073_709_551_615n).ones(), 64, "onesU64(18_446_744_073_709_551_615n) = 64");

assert.equal(new alanStd.I64(0).ones(), 0, "onesI64(0) = 0");
assert.equal(new alanStd.I64(1).ones(), 1, "onesI64(1) = 1");
assert.equal(new alanStd.I64(2).ones(), 1, "onesI64(2) = 1");
assert.equal(new alanStd.I64(3).ones(), 2, "onesI64(3) = 2");
assert.equal(new alanStd.I64(-1).ones(), 64, "onesI64(-1) = 64");

assert.equal(new alanStd.U8(0).ctz(), 8, "ctzU8(0) = 8");
assert.equal(new alanStd.U8(1).ctz(), 0, "ctzU8(1) = 0");
assert.equal(new alanStd.U8(2).ctz(), 1, "ctzU8(2) = 1");
assert.equal(new alanStd.U8(3).ctz(), 0, "ctzU8(3) = 0");
assert.equal(new alanStd.U8(128).ctz(), 7, "ctzU8(128) = 7");

assert.equal(new alanStd.I8(0).ctz(), 8, "ctzI8(0) = 8");
assert.equal(new alanStd.I8(1).ctz(), 0, "ctzI8(1) = 0");
assert.equal(new alanStd.I8(2).ctz(), 1, "ctzI8(2) = 1");
assert.equal(new alanStd.I8(3).ctz(), 0, "ctzI8(3) = 0");
assert.equal(new alanStd.I8(-128).ctz(), 7, "ctzI8(-128) = 7");

assert.equal(new alanStd.U16(0).ctz(), 16, "ctzU16(0) = 16");
assert.equal(new alanStd.U16(1).ctz(), 0, "ctzU16(1) = 0");
assert.equal(new alanStd.U16(2).ctz(), 1, "ctzU16(2) = 1");
assert.equal(new alanStd.U16(3).ctz(), 0, "ctzU16(3) = 0");
assert.equal(new alanStd.U16(32768).ctz(), 15, "ctzU16(32768) = 15");

assert.equal(new alanStd.I16(0).ctz(), 16, "ctzI16(0) = 16");
assert.equal(new alanStd.I16(1).ctz(), 0, "ctzI16(1) = 0");
assert.equal(new alanStd.I16(2).ctz(), 1, "ctzI16(2) = 1");
assert.equal(new alanStd.I16(3).ctz(), 0, "ctzI16(3) = 0");
assert.equal(new alanStd.I16(-32768).ctz(), 15, "ctzI16(-32768) = 15");

assert.equal(new alanStd.U32(0).ctz(), 32, "ctzU32(0) = 32");
assert.equal(new alanStd.U32(1).ctz(), 0, "ctzU32(1) = 0");
assert.equal(new alanStd.U32(2).ctz(), 1, "ctzU32(2) = 1");
assert.equal(new alanStd.U32(3).ctz(), 0, "ctzU32(3) = 0");
assert.equal(new alanStd.U32(2_147_483_648).ctz(), 31, "ctzU32(2_147_483_648) = 31");

assert.equal(new alanStd.I32(0).ctz(), 32, "ctzI32(0) = 32");
assert.equal(new alanStd.I32(1).ctz(), 0, "ctzI32(1) = 0");
assert.equal(new alanStd.I32(2).ctz(), 1, "ctzI32(2) = 1");
assert.equal(new alanStd.I32(3).ctz(), 0, "ctzI32(3) = 0");
assert.equal(new alanStd.I32(-2_147_483_648).ctz(), 31, "ctzI32(-2_147_483_648) = 31");

assert.equal(new alanStd.U64(0).ctz(), 64n, "ctzU64(0) = 64");
assert.equal(new alanStd.U64(1).ctz(), 0n, "ctzU64(1) = 0");
assert.equal(new alanStd.U64(2).ctz(), 1n, "ctzU64(2) = 1");
assert.equal(new alanStd.U64(3).ctz(), 0n, "ctzU64(3) = 0");
assert.equal(new alanStd.U64(9_223_372_036_854_775_808n).ctz(), 63n, "ctzU64(9_223_372_036_854_775_808) = 63");

assert.equal(new alanStd.I64(0).ctz(), 64n, "ctzI64(0) = 64");
assert.equal(new alanStd.I64(1).ctz(), 0n, "ctzI64(1) = 0");
assert.equal(new alanStd.I64(2).ctz(), 1n, "ctzI64(2) = 1");
assert.equal(new alanStd.I64(3).ctz(), 0n, "ctzI64(3) = 0");
assert.equal(new alanStd.I64(-9_223_372_036_854_775_808n).ctz(), 63n, "ctzI64(-9_223_372_036_854_775_808n) = 63");

assert.deepEqual(alanStd.cross(
  [new alanStd.F64(1), new alanStd.F64(0), new alanStd.F64(0)],
  [new alanStd.F64(0), new alanStd.F64(1), new alanStd.F64(0)],
), [new alanStd.F64(0), new alanStd.F64(0), new alanStd.F64(1)],
"cross([1, 0, 0], [0, 1, 0]) = [0, 0, 1]");

assert.deepEqual(alanStd.cross(
  [new alanStd.F64(0), new alanStd.F64(1), new alanStd.F64(0)],
  [new alanStd.F64(1), new alanStd.F64(0), new alanStd.F64(0)],
), [new alanStd.F64(0), new alanStd.F64(0), new alanStd.F64(-1)],
"cross([0, 1, 0], [1, 0, 0]) = [0, 0, -1]");
