# The Alan Virtual Machine (and CLI App)

A virtual machine in Rust to compile and run AGC or Alan Graphcode, Alan's bytecode format.

This runtime is licensed AGPL 3.0 but the [alan standard library](https://github.com/alantech/alan/tree/main/std) and the [Javascript runtime shim](https://github.com/alantech/alan/tree/main/js-runtime) are licensed Apache 2.0 so you can freely distribute your compiled code.

## Install

The AVM requires the [compiler](https://github.com/alantech/alan/tree/main/compiler) to have been built and the `alan-compile` binary to exist before this project can be built directly with:

```
cargo build
```

It is recommended to build the AVM from the root of the repository with a simple `make` call, instead.

## Usage

```
cargo run -- compile <sourcefile>.ln <binfile>.agc
cargo run -- run  <binfile>.agc
```

The binary file has to be `.agc` format.
To run an optimized build:

```
cargo build --release
./target/release/alan run <binfile>.agc
```

## Development

The AVM is backed by a single-threaded, or basic, [Tokio](https://tokio.rs/) scheduler and uses a [Rayon](https://crates.io/crates/rayon)
threadpool to run cpu bound opcodes.

## Contribution Agreement

To contribute to the AVM you need to sign a Contributor License Agreement. Alan Technologies will retain the right to relicense this code in licenses other than AGPL 3.0 concurrently or in the future to convert to a newer license.

## License

AGPL 3.0
