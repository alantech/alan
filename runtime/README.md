# alan runtime

A runtime in Rust to run AGC or Alan Graphcode, alan's bytecode format.

This runtime is licensed AGPL 3.0 but the [alan standard library](https://github.com/alantech/alan/tree/master/std) and the [Javascript runtime shim](https://github.com/alantech/alan/tree/master/js-runtime) are licensed Apache 2.0 so you can freely distribute your compiled code.

## Install

```
cargo build
```

## Usage

```
cargo run -- run  <sourcefile>
```

The source file has to be `.agc` format.
To run an optimized build:

```
cargo build --release
./target/release/alan-runtime run <sourcefile>
```

## Development

The `alan runtime` is backed by a single-threaded, or basic, [Tokio](https://tokio.rs/) scheduler and uses a [Rayon](https://crates.io/crates/rayon)
threadpool to run cpu bound opcodes.

## Contribution Agreement

To contribute to the `alan runtime` you need to sign a Contributor License Agreement (TODO: figure this out), Alan Technologies will retain the right to relicense this code in licenses other than AGPL 3.0 concurrently or in the future to convert to a newer license.

## License

AGPL 3.0
