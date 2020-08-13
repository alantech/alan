# The Alan Programming Language![CI](https://github.com/alantech/alan/workflows/CI/badge.svg)

<center>
  <img src="https://alan-lang.org/alan-logo.png" alt="drawing" width="180"/>
</center>

The `alan` compiler and runtime can parallelize your code without concurrent or asynchronous programming (threads, promises, channels, etc) by only allowing iteration and recursion that is guaranteed to halt (e.g. no `while (true) {}` loops)

This repository houses all the components for the Alan programming language.

<center>
  <h2><a href="https://docs.alan-lang.org">Documentation</a> | <a href="https://alan-lang.org">Homepage</a></h2>
</center>

## Install

As `alan` is not self-hosting, other languages runtimes are necessary for the compiler and runtimes. For `alan-compile` and `alan-js-runtime` Node.js is required, with a minimum version of 10.20.1. For `alan-runtime` Rust is required, with a minimum version of 1.41.1.

### Recommended Installation

It is recommended to install Alan via the published artifacts:

To install `alan-compile`, run:

```bash
npm i -g alan-compile
```

To install `alan-js-runtime`, run:

```bash
npm i -g alan-js-runtime
```

To install `alan-runtime`, run:

```bash
cargo install alan-runtime
```

### Source Installation

However, you may also install Alan directly from the source of this project. Simply run:

```
make clean
make install # currently doesn't work quite right in Linux if you're using an nvm-managed Node.js (the compiler fails to install if you prefix this last one with sudo)
```

## Usage

To compile to Alan GraphCode:

```
alan-compile <source>.ln <whateveryouwant>.agc
```

Then it can be run with:

```
alan-runtime run <whateveryouwant>.agc
```

To compile to Javascript:

```
alan-compile <source>.ln <whateveryouwant>.js
```

Which can be run with:

```
node <whateveryouwant>.js
```

But make sure you have `alan-js-runtime` installed for this to work (either globally or in a local `node_modules` directory).

To compile to Alan's first intermediate representation, `alan--`:

```
alan-compile <source>.ln <whateveryouwant>.amm
```

This is useful if you want to compile to another scope-based, garbage-collected language, but not for much else.

To compile to Alan's second intermediate representation, `alan graphcode assembler`:

```
alan-compile <source>.ln <whateveryouwant>.aga
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
  * runtime

The Apache 2.0 license is freely combinable with the GPL 3 series of licenses as well as with proprietary software, so the standard library and Javascript runtime shim library are freely combinable with your own software projects without any requirement to open source it. It also provides a patent indemnification clause: we grant you a license to any applicable patents while using this software in your own code, so there is no risk of being sued for using it.

The AGPL 3.0 license requires that any changes to the code are published and publicly accessible. This is to make sure that any advancements to the compiler and runtime are available to all. The licensing of these tools does *not* affect the licensing of the code they compile or run, however, otherwise any code compiled by GCC would be GPL 3.0 licensed and any Java code running on the OpenJDK would be GPL 2.0 licensed. (The "linking" exception added to the OpenJDK's licensing terms is not needed for Alan as the entire standard library is Apache 2.0 licensed on purpose.)

If one does need to make changes to the compiler or runtime, obeying the terms is as simple as merely forking this repository and making the changes there. You do not even need to try to upstream those changes. If you use the compiler or runtime without changes, all that is required of you is to provide a link to this repository *if you are asked for it.*

We believe this structure will encourage the greatest growth in the ecosystem. Any kind of software, from fully-freely licensed (Apache, BSD, MIT) to reciprocally-freely licensed ([A]GPL 2/3) to proprietary can be developed in the Alan language, and there are no concerns that Alan Technologies will try to apply patent infringement lawsuits on you, while the compiler and primary runtime are reciprocally-freely licensed to ensure advancements from all parties are available to be used.