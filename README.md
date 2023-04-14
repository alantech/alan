<div align="center">
  <img src="https://alan-lang.org/logo.png" alt="drawing" width="180"/>
  <h2>The Alan Programming Language</h2>
</div>

[![CI](https://github.com/alantech/alan/workflows/CI/badge.svg)](https://github.com/alantech/alan/actions?query=workflow%3ACI)
[![Docs](https://img.shields.io/badge/docs-mdbook-blue)](https://docs.alan-lang.org)
[![Discord](https://img.shields.io/badge/discord-alanlang-purple)](https://discord.gg/XatB9we)
[![Reddit](https://img.shields.io/badge/reddit-alanlang-red)](https://www.reddit.com/r/alanlang)
<!--
[![Website](https://img.shields.io/badge/website-alan--lang.org-blue)](https://alan-lang.org)
-->

**🔭 Predictable runtime for all computations** - A program is represented as DAG(s) where the running time for all computations can be predicted because there is no unbounded recursion or iteration.

**⛓ Automatic IO concurrency and parallelism across events and arrays** - Alan exploits opportunities for IO concurrency or CPU parallelization across machines in a cluster via arrays and a static event loop without threads, channels, promises, futures, locks, etc.

**✅ Almost no runtime errors** - No deadlocks, livelocks, undefined variables, divide-by-zero, integer under/overflow, array out-of-bounds access, etc.

**⚡️ No GC pauses** - Alan’s runtime manages memory allocation, access, and deallocation for you like Java, Python, or Javascript. However, Alan’s static event system and [automatic event-oriented memory model](https://alan-lang.org/alan_overview.html#memory-management) does so without garbage collector pauses.

---------------------------------
<br/>

👩‍🚀 Alan is a programming language that does concurrency for you and can thus separate how the software is written from how it runs.
To learn more about Alan, take a look at [runnable examples](https://docs.alan-lang.org/examples.html) or the most [Frequently Asked Questions](https://github.com/alantech/alan/blob/main/FAQ.md).

<br/>
<h2 align="center">Installation</h2>
<br/>

For MacOS it is recommended to install Alan via the [Homebrew](https://brew.sh) package manager.

**MacOS**

```bash
brew install alantech/homebrew-core/alan
```

For Linux and Windows it is recommended to install Alan via the [published artifacts](https://github.com/alantech/alan/releases). Simply download the zip or tar.gz file for your operating system, and extract the `alan` executable to somewhere in your `$PATH`, make sure it's marked executable (if not on Windows), and you're ready to roll.

**Linux**

```bash
wget https://github.com/alantech/alan/releases/latest/download/alan-ubuntu.tar.gz
tar -xzf alan-ubuntu.tar.gz
sudo mv alan /usr/local/bin/alan
```

**Windows**

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

Note: To better understand if we are building something people want to use we currently [log an event](https://github.com/alantech/alan/blob/main/avm/src/vm/telemetry.rs) when running an Alan command. Feel free to turn this off by setting the `ALAN_TELEMETRY_OFF` environment variable to `true`, but if you do please let us know how you are using Alan and how often!

<br/>
<h2 align="center">Contribution</h2>
<br/>

**Source Installation:**

If you wish to contribute to Alan, or if your operating system and/or CPU architecture do not match the above, you'll need a development environment to build Alan locally:

* git (any recent version should work)
* Node.js >=10.20.1
* Rust >=1.45.0
* A complete C toolchain (gcc, clang, msvc)

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

The AGPL 3.0 license requires that any changes to the code are published and publicly accessible. This is to make sure that any advancements to the compiler and AVM are available to all. The licensing of these tools does not affect the licensing of the code they compile or run. Similarly, the [GCC compiler collection](https://gcc.gnu.org) is GPL 3.0 licensed, but code compiled by it is not GPL 3.0 licensed.
