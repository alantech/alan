<div align="center">
  <img src="https://alan-lang.org/alan-logo.png" alt="drawing" width="180"/>
  <h2>The Alan Programming Language</h2>
</div>

---------------------------------

[![CI](https://github.com/alantech/alan/workflows/CI/badge.svg)](https://github.com/alantech/alan/actions?query=workflow%3ACI)
[![Docs](https://img.shields.io/badge/docs-mdbook-blue)](https://docs.alan-lang.org)
[![Discord](https://img.shields.io/badge/discord-alanlang-purple)](https://discord.gg/XatB9we)
[![Reddit](https://img.shields.io/badge/reddit-alanlang-red)](https://www.reddit.com/r/alanlang)
<!--
[![Website](https://img.shields.io/badge/website-alan--lang.org-blue)](https://alan-lang.org)
-->

**⛓ Implicitly parallel across events, arrays and IO** - *Alan recognizes and exploits opportunities for parallelization without parallel programming (threads, channels, futures, locks, etc.)*

**✅ Almost no runtime errors** - *Null references, deadlocks, livelocks, undefined variables, divide-by-zero, integer under/overflow, array out-of-bounds access, etc, are not possible in Alan.*

**🔒 Granular third party permissions** - *Alan's module resolution mechanism allows you to prevent third party dependencies from having access to standard libraries that they should not have access to.*


---------------------------------
<br/>

👩‍🚀 Alan is a programming language that does concurrency for you and can thus separate how the software is written from how it runs.
To learn more about Alan, take a look at [runnable examples](https://docs.alan-lang.org/advanced_examples.html) or the most [Frequently Asked Questions](https://github.com/alantech/alan/blob/main/FAQ.md).

<br/>
<h2 align="center">Installation</h2>
<br/>

It is recommended to install Alan via the [published artifacts](https://github.com/alantech/alan/releases). Simply download the zip or tar.gz file for your operating system, and extract the `alan` executable to somewhere in your `$PATH`, make sure it's marked executable (if not on Windows), and you're ready to roll.

**Linux:**

```bash
wget https://github.com/alantech/alan/releases/latest/download/alan-ubuntu.tar.gz
tar -xzf alan-ubuntu.tar.gz
sudo mv alan /usr/local/bin/alan
```

**MacOS:**

```bash
curl -OL https://github.com/alantech/alan/releases/latest/download/alan-macos.tar.gz
tar -xzf alan-macos.tar.gz
# sudo mkdir -p /usr/local/bin if the folder does not exist
sudo mv alan /usr/local/bin/alan
```

**Windows:**

```ps1
Invoke-WebRequest -OutFile alan-windows.zip -Uri https://github.com/alantech/alan/releases/latest/download/alan-windows.zip
Expand-Archive -Path alan-windows.zip -DestinationPath C:\windows
```

<br/>
<h2 align="center">Usage</h2>
<br/>

To compile to Alan GraphCode and then run it with the AVM:

```
alan compile <source>.ln <whateveryouwant>.agc
alan run <whateveryouwant>.agc
```

You can also compile-and-run a source file with a simple:

```
alan <source>.ln
```

You can also [transpile Alan to Javascript](https://docs.alan-lang.org/transpile_js.html) or one of it's [intermediate representations](https://docs.alan-lang.org/compiler_internals.html).

<br/>
<h2 align="center">Contributing</h2>
<br/>

**Source Installation:**

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

**Integration tests:**

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

<br/>
<h2 align="center">License</h2>
<br/>

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
