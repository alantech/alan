<div align="center">
  <img src="https://docs.alan-lang.org/assets/logo.png" alt="drawing" width="180"/>
  <h2>The Alan Programming Language</h2>
</div>

[![CI](https://github.com/alantech/alan/actions/workflows/rust.yml/badge.svg)](https://github.com/alantech/alan/actions/workflows/rust.yml)
[![Docs](https://img.shields.io/badge/docs-mdbook-blue)](https://docs.alan-lang.org)
[![Discord](https://img.shields.io/badge/discord-alanlang-purple)](https://discord.gg/XatB9we)
[![Reddit](https://img.shields.io/badge/reddit-alanlang-red)](https://www.reddit.com/r/alanlang)
[![Website](https://img.shields.io/badge/website-alan--lang.org-blue)](https://alan-lang.org)

**🚧 CONSTRUCTION IN PROGRESS** - This language is being reworked for a new purpose in a backwards-incompatible way. The currently published documentation is for the [v0.1](https://github.com/alantech/alan/tree/v0.1) iteration of Alan, which is more stable and capable than the v0.2 code on the `main` branch.

**🔭 Predictable runtime for all computations** - A program is represented as DAG(s) where the running time for all computations can be predicted because there is no unbounded recursion or iteration.

**⛓ Transparent GPGPU programming** - Alan's restrictions on recursion and iteration allows for automatic generation of compute shaders for your code. (Not yet implemented)

**✅ Almost no runtime errors** - No deadlocks, livelocks, undefined variables, divide-by-zero, integer under/overflow, array out-of-bounds access, etc. Due to the type system and the standard library primitives provided to you.

**⚡️ Native performance with Rust** - Alan's new compiler transforms your code into Rust before finally generating a binary for your platform, without needing to worry about memory management or GC pauses by handling Rust's borrow checker for you.

---------------------------------
<br/>

👩‍🚀 Alan is a programming language that makes the power of the GPU more accessible, with a syntax similar to a dynamic language (it's typed, but 100% inferred), and restrictions on recursion and iteration to make automatic generation of multi-threaded CPU and GPGPU versions of your code for you.

<br/>
<h2 align="center">Installation</h2>
<br/>

Currently, the only way to install `alan` is to have a working `rust` development environment along with `git` to clone this repo and install it manually:

```bash
git clone https://github.com/alantech/alan
cd alan
cargo install --path alan
```

alan v0.2 has been tested on x86-64 for Windows, Mac, and Linux, ARM64 for Linux and Mac, and RISC-V for Linux.

<br/>
<h2 align="center">Usage</h2>
<br/>

To compile, simply:

```
alan compile <source>.ln
```

This will create a file with the name `<source>` that you can run (or error if it fails to compile).

<br/>
<h2 align="center">Contribution</h2>
<br/>

**Source Installation:**

If you wish to contribute to Alan, you'll need a development environment to build Alan locally:

* git (any recent version should work)
* Rust >=1.80.0
* Node.js >=22.0.0
* A complete C toolchain (gcc, clang, msvc)

Once those are installed, simply follow the install instructions above, replacing `cargo install --path .` with a simple `cargo build` to compile and `cargo test` to run the test suite.

**Unit and Integration Tests:**

The tests are included within the Rust source files. Test coverage is not yet 100%, with the majority of unit tests in the `src/parse.rs` file defining the Alan syntax parser. The unit tests directly follow the functions they test, instead of all being at the end as is standard in Rust, because it seemed easier to read that way. These tests all pass.

Beyond that are integration tests in the `src/compile.rs` file, making up the vast majority of that file (which for release is just a single function that is a small wrapper around the transpilation code in `lntors.rs`). Few of these tests currently pass, as they were inherited from the Alan v0.1 test suite. Most are planned for revival but some may be changed or dropped.

<br/>
<h2 align="center">License</h2>
<br/>

MIT
