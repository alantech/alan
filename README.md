# The Alan Programming Language [![CI](https://github.com/alantech/alan/workflows/CI/badge.svg)](https://github.com/alantech/alan/actions?query=workflow%3ACI)

<div align="center">
  <img src="https://alan-lang.org/alan-logo.png" alt="drawing" width="180"/>
</div>

The `alan` compiler and runtime can parallelize your code without concurrent or asynchronous programming (threads, promises, channels, etc) by only allowing iteration and recursion that is guaranteed to halt (e.g. no `while (true) {}` loops)

This repository houses all the components for the Alan programming language.

<div align="center">
  <h2><a href="https://docs.alan-lang.org">Documentation</a> | <a href="https://alan-lang.org">Homepage</a> | <a href="https://github.com/alantech/alan/releases">Download</a></h2>
</div>

## Install

### Recommended Installation

It is recommended to install Alan via the [published artifacts](https://github.com/alantech/alan/releases). Simply download the zip or tar.gz file for your operating system, and extract the `alan` executable to somewhere in your `$PATH`, make sure it's marked executable (if not on Windows), and you're ready to roll.

If your operating system or machine architecture are not supported (only Windows, Mac, and Ubuntu on x86-64 have pregenerated binaries), you'll need to do a source installation, instead.

If you use the `alan` command's transpiling feature to generate Javascript, that code depends on the `alan-js-runtime` shim to work. You can either add it to the `package.json` of the project that will house the output code, or add it globally:

To install `alan-js-runtime`, run:

```bash
npm i -g alan-js-runtime
```

### Source Installation

As `alan` is not self-hosting, other languages runtimes are necessary to build it. For the compiler and JS runtime shim Node.js is required, with a minimum version of 10.20.1. For the AVM Rust is required, with a minimum version of 1.41.1.

Then, simply clone the repo, enter it, and run:

```
make && [sudo] make install
```

(adding the `sudo` or excluding it depending on whether you need root permissions to add to `/usr/local/bin` or not).

If you are doing this because your platform is not supported, it will have to build Node.js + V8 from scratch in the process, and will take a long time.

## Usage

To compile to Alan GraphCode:

```
alan compile <source>.ln <whateveryouwant>.agc
```

Then it can be run with:

```
alan run <whateveryouwant>.agc
```

To compile to Javascript:

```
alan compile <source>.ln <whateveryouwant>.js
```

Which can be run with:

```
node <whateveryouwant>.js
```

But make sure you have `alan-js-runtime` installed for this to work (either globally or in a local `node_modules` directory).

To compile to Alan's first intermediate representation, `alan--`:

```
alan compile <source>.ln <whateveryouwant>.amm
```

This is useful if you want to compile to another scope-based, garbage-collected language, but not for much else.

To compile to Alan's second intermediate representation, `alan graphcode assembler`:

```
alan compile <source>.ln <whateveryouwant>.aga
```

This is useful for debugging what exactly the runtime is doing with your code, or as a target format if you want to run another language on top of Alan's runtime, but not for much else.

## Integration tests

Integration tests are in `/bdd` and defined using [Shellspec](https://shellspec.info/). To run all integration tests:
```
make bdd
```

To run a single test file:
```
make bdd testfile=bdd/spec/001_event_spec.sh
```

To run a single test group use the line number corresponding to a `Describe`:
```
make bdd testfile=bdd/spec/001_event_spec.sh:30
```

## License

The Alan Programming Language is made up of multiple sub-projects housed within this monorepo. Each subdirectory has its own license file and the project as a whole uses two licenses: The Apache 2.0 license and the Affero GPL 3.0 license, with the breakdown as follows:

* Apache 2.0
  * bdd
  * js-runtime
  * std
* AGPL 3.0
  * compiler
  * avm

The Apache 2.0 license is freely combinable with the GPL 3 series of licenses as well as with proprietary software, so the standard library and Javascript runtime library are freely combinable with your own software projects without any requirement to open source it.

The AGPL 3.0 license requires that any changes to the code are published and publicly accessible. This is to make sure that any advancements to the compiler and AVM are available to all. The licensing of these tools does not affect the licensing of the code they compile or run. Similarly the [GCC compiler collection](https://gcc.gnu.org) is GPL 3.0 licensed, but code compiled by it is not GPL 3.0 licensed.
