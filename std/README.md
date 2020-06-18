# std

The alan standard library defined in alan.

## Usage

This project is not structured like a normal alan library because it is not meant to be installed like external libraries. It is meant to be consumed by the alan [compiler](https://github.com/alantech/alan/tree/master/compiler) to target the appropriate opcodes in the alan runtimes.

## How it works

These `.ln` files are not quite the same as others. The compiler loads these files with a partially populated private module scope consisting of all of the built-in events and opcode functions which the standard library can build upon and/or expose, usually with a more user-friendly name. The `root.ln` is particularly special as it defines the root scope that all other modules inherit from, including the other `std` files in here, so anything exported from `root.ln` can be assumed to exist in all scopes.

## License

Apache 2.0
