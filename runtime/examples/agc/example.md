# `alan` to `alan--` to `alangraphcode` example

This is a handwritten example of what the translation of a trivial `alan` application could look like in `alan--` and `alangraphcode` with explanations along the way. The `alan--.md` and `bytecodes.md` files in the `meta` repo provide a lot of context here that I'm going to assume is known.

First, the example `alan` application, `example.ln` (say ".ln" out loud; it sounds kinda like "EL-EN" / "alan"):

```
from @std/app import start, exit

on start {
  emit exit 2 + 1
}
```

This example imports the standard start and exit events, registers a handler function for the start event, and then executes a simple addition of constants and emits that as the exit code for the application. Absolutely trivial, but allows us to avoid the `string` problem for a moment.

The translation from `alan` to `alan--` eliminates all `imports` and `exports`, inlining everything into the output file, with `@std` imports converted to intrinsic global references, producing `example.amm` (for `Alan Minus Minus`)

```
const __unnamed_string_val_0: string = "Hello, Dear World!\n"
on __std_app_start fn (): void {
  const __unnamed_1: int64 = 2
  const __unnamed_2: int64 = 1
  const __unnamed_3 = add(__unnamed_1, __unnamed_2)
  emit __std_app_stdout __unnamed_string_val_0
  emit __std_app_exit __unnamed_3
}
```

`alan--` is intended to be a strict subset of `alan` so this code will execute correctly in the interpreter, as well. (Integration testing that `.ln`, `.amm`, and the eventual `.agc` files all run and produce the same results is an goal.)

The handler function for the now-renamed `__std_app_start` event contains 4 statements. This will directly correspond to the `alangraphcode` statements and is consistent with all of the internal `Box` objects that would be created for the original `alan` code.

The `alangraphcode` (".agc" should be clear here) file is binary, but with specially-chosen constants to be human-readable. A hexdump for `example.agc` below:

```

00000000: 6167 6330 3030 3031 0000 0000 0000 0020  agc00001........
00000010: 0000 0000 0000 000F 4865 6C6C 6F2C 2044  ........Hello, D
00000020: 6561 7220 576F 726C 6421 5C6E 0000 0000  ear World!\n....
00000030: 6861 6e64 6c65 723a 8022 7374 6172 7422  handler:."start"
00000040: 0000 0000 0000 0018 6c69 6e65 6e6f 3a20  ........lineno:
00000050: 0000 0000 0000 0000 0000 0000 0000 0000  ................
00000060: 7365 7420 6936 343a 0000 0000 0000 0000  set i64:........
00000070: 0000 0000 0000 0002 6c69 6e65 6e6f 3a20  ........lineno:
00000080: 0000 0000 0000 0001 0000 0000 0000 0000  ................
00000090: 7365 7420 6936 343a 0000 0000 0000 0008  set i64:........
000000a0: 0000 0000 0000 0001 6c69 6e65 6e6f 3a20  ........lineno:
000000b0: 0000 0000 0000 0002 0000 0000 0000 0002  ................
000000c0: 0000 0000 0000 0000 0000 0000 0000 0001  ................
000000d0: 6164 6420 6936 343a 0000 0000 0000 0000  add i64:........
000000e0: 0000 0000 0000 0010 6c69 6e65 6e6f 3a20  ........lineno:
000000f0: 0000 0000 0000 0003 0000 0000 0000 0000  ................
00000100: 656D 6974 2074 6F3A 8022 7374 6F75 7422  emit to:."stout"
00000110: FFFF FFFF FFFF FFFF 6C69 6E65 6E6F 3A20  ........lineno:
00000120: 0000 0000 0000 0004 0000 0000 0000 0001  ................
00000130: 0000 0000 0000 0002 656d 6974 2074 6f3a  ........emit to:
00000140: 8022 6578 6974 2220 0000 0000 0000 0010  ."exit" ........
```

Every 4 blocks of hexadecimal digits is an 8-byte (64-bit) chunk of memory. Sticking to 64-bit blocks of memory as the unit within `alangraphcode` (though allowing byte-level addressing for data less than 64-bits in size) should make CPU pipelining of operations on the code less prone to stalling, but will negatively impact caching for larger applications and/or older machines. Whether or not this decision is the right one, it feels more future-oriented as CPU cache sizes go up, int64/float64 becomes the standard storage unit, and CPUs continue to perform more and more trickery in their pipelines to eke out performance gains at the cost of greater complexity and poorer predictability.

We should definitely revisit, however, if binary sizes are too massive and/or performance is unexpectedly low.

The first 64-bit integer specifies the version of the `alangraphcode` in play. It should be considered a name for the format instead of a monotonically-increasing integer that the `runtime` would have a list of supported versions it could run (either directly or through a rewriting mechanism). I've currently chosen a 64-bit integer that corresponds to the ASCII/UTF-8 string "agc00001" and the intention is to monotonically increase the ASCII numbers until or unless a real fork (ie, multiple simultaneous bytecodes supported) is necessary. Hopefully that doesn't come to pass, but it should be possible with this mechanism.

The next 64-bit integer is the number of bytes of *global* memory shared between all handlers is necessary. This memory is always read-only and immediately follows its size declaration, which in this case is 32 bytes (hex `20` or four 64-bit chunks) as our `example.amm` has one `const` declaration in the outer scope. We only use 28 bytes, but the bytecode is zero padded to the next 64-bit interval. The length-prefixed string, `Hello, Dear World!\n` is 20 bytes long in UTF-8 is what follows within its 8 byte (uint64) length declaration.

So, moving on, the next 64-bit integer declares a new event handler with the value that corresponds to an ASCII `handler:`. Since this example only uses built-in events and does not declare custom events, the custom event declaration portion of the `alangraphcode` format is skipped. For simplicity in the `alangraphcode` format (and because there's no reason not to), all custom `events` must be declared after the global constants and before all `handlers`. `alan` has no such restriction and there's no need to have such a restriction in `alan--`, but it may emit in the same fashion to keep the final compile step simpler.

The next 64-bit integer is the ID for the event the handler is being registered for. It looks *mostly* like ASCII/UTF-8, but it is not. The 64-bit integers in `alangraphcode` are all *signed* integers by default (though we will support unsigned values, addresses & events & etc internal representations are 64-bit signed integers), so if the first hexadecimal byte in an integer is `80` or higher (up to `FF`) it is a negative number, so the built-in events (since there will not be that many) are all placed in the `80` "namespace" and the remaining 7 bytes can be used for ASCII-like values. In this case the value `"start"`

Following that is the number of bytes of memory the `handler` requires to do its work. In this case, 24 bytes, which is hex `18` hence a value of `0000 0000 0000 0018`.

Each actual instruction to be run has a unique ID and metadata about the instructions it requires to be run before the instruction data is added. The uniqueness of these IDs must only be guaranteed per handler, so they can be trivially monotonically increasing, hence the cheeky prefix 64-bit integer with the ASCII value `lineno: ` for each of them, and you can see there are four of these strings in the file.

The 64-bit integer following that is the first ID, which is zero. The 64-bit integer after that is also zero, but it is indicating the number of instructions it is *directly* dependent upon before it can be executed, so the first instruction has no dependencies and could be executed immediately. This means the next 64-bit integer is the actual opcode to execute, which in this case is a number equivalent to `set i64:`. The next 2 64-bit integers indicate the memory address (0, the first address for the handler's memory) and the value to set (2 in this case).

The next 5 64-bit integers follow the same pattern. `lineno: ` followed by a 1 for the ID, 0 for no dependencies, `set i64:` for the opcode, 8 for the address (the next 64-bit integer starting point for the handler's memory), and 1 for the value.

Now we have 8 64-bit integers with a slightly different pattern. `lineno: ` followed by a 2 for the ID, 2 for the number of dependencies, 0 for the first dependency ID, 1 for the second dependency ID, then `add i64:` for the opcode, 0 for the first memory address of the inputs (assumed to be two contiguous 64-bit integers, as they are here) and 16 for the output memory address (10 in hexadecimal).

This demonstrates an instruction that depends on the execution of the prior two instructions before it can be run, while the first two instructions could *theoretically* be run in parallel. In such a trivial example it makes zero sense and the Execution Planner should agree with me on that, having an internal model of execution time for the instructions and synchronization time for multiple threads executing the instructions, but on some theoretical machine with very slow constant assignment relative to thread communication, it could run those in parallel to improve the performance.

The next 6 64-bit integers start with a `lineno: ` followed by a 3 for the ID, 0 for the number of dependencies, `emit to:` to declare event emission, a special 64-bit number beginning with `0x80` and followed by the ASCII-like `"stout" ` and finally -1 (hex `FFFF FFFF FFFF FFFF`) for the memory address to pull the event data from, which is a signed 64-bit integer referencing the first address in global memory.

The final 7 64-bit integers start with a `lineno: ` followed by a 4 for the ID, 1 for the number of dependencies, a 2 for the dependency on the prior instruction, `emit to:` to declare event emission, a special 64-bit number beginning with `0x80` and followed by the ASCII-like `"exit" ` and finally 16 (hex `10`) for the memory address to pull the event data from, which is a 64-bit integer.