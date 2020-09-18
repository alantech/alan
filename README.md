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

For Linux:

```bash
wget https://github.com/alantech/alan/releases/download/v0.1.7/alan-ubuntu.tar.gz
tar -xzf alan-ubuntu.tar.gz
sudo mv alan /usr/local/bin/alan
```

For MacOS:

```bash
curl -OL https://github.com/alantech/alan/releases/download/v0.1.7/alan-macos.tar.gz
tar -xzf alan-macos.tar.gz
# sudo mkdir -p /usr/local/bin if the folder does not exist
sudo mv alan /usr/local/bin/alan
```

For Windows:

```ps1
Invoke-WebRequest -OutFile alan-windows.zip -Uri https://github.com/alantech/alan/releases/download/v0.1.7/alan-windows.zip
Expand-Archive -Path alan-windows.zip -DestinationPath C:\windows
```

### Source Installation

If you wish to contribute to Alan, or if your operating system and/or CPU architecture do not match the above, you'll need a development environment to build Alan locally:

* git (any recent version should work)
* Node.js >=10.20.1, <14.0.0
* Rust >=1.41.1
* A complete C toolchain (gcc, clang, msvc)
* Python >=2.7, <3.0 (and named `python2` in your PATH)

Once those are installed, simply:

```bash
git clone https://github.com/alantech/alan
cd alan
make
sudo make install
```

## Usage

### Recommended Usage

To compile to Alan GraphCode and then run it with the AVM:

```
alan compile <source>.ln <whateveryouwant>.agc
alan run <whateveryouwant>.agc
```

You can also compile-and-run a source file with a simple:

```
alan <source>.ln
```


### Advanced Usage

#### Transpile Alan to Javascript and run it with Node.js

```
alan compile <source>.ln <whateveryouwant>.js
node <whateveryouwant>.js
```

Make sure you have `alan-js-runtime` installed for this to work (either globally or in a local `node_modules` directory). You can either add it to the `package.json` of the project that will house the output code, or add it globally:


```bash
npm i -g alan-js-runtime
```

#### Transpile Alan to its intermediate representations:

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

## Contact

Please reach out on [Discord](https://discord.gg/XatB9we) or email us at hello at alantechnologies dot com.

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
