# 008 - Safe Global Data Storage RFC

## Current Status

### Proposed

2020-08-07

### Accepted

2020-08-07

#### Approvers

- Luis de Pombo <luis@alantechnologies.com>

### Implementation

- [ ] Implemented:
  - [@std/datastore (js-runtime only for now)](https://github.com/alantech/alan/pull/256) 2020-09-02
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <david@alantechnologies.com>

## Summary

While Alan does allow for module-level constants, there is no way to have mutable global state. This is to prevent a whole host of multithreading issues as well as discourage antipatterns involving global variables that make code illegible. However, sometimes persisted data between HTTP requests such as a cache, or data truly global to your application such as A/B testing configuration data, needs to be accessible and mutable, and using an external database is not an option (often for performance reasons).

Providing this functionality through a standard library makes it clear to the reader exactly where such global state is being used and/or manipulated, and the standard library can make sure it is done in a threadsafe way under the hood (if necessary, the js-runtime has no such concerns).

## Expected SemBDD Impact

If Alan was at 1.0.0 or beyond, this would be a minor version update as all existing code should continue to function as before.

## Proposal

As global state storage and retrieval is intended to allow cross-cutting mutable changes, any use of this is inherently *dangerous* and can cause unexpected bugs in logic if mutations that violate the expected range of values for *any one consumer*. Don't use this without a good reason.

But to make accidental mutations less likely, it is proposed that global state is stored in a two stage process, first a namespace key and then within that namespace a value key for the actual field. If you are intentionally digging into, say, the `"@std/http"`'s namespace to adjust the `"maximumConcurrentRequests"` allowed (assuming such a use came about), the impact is more obvious and less likely to collide with, say, a PostgreSQL client's maximum concurrent requests global.

The proposed API is simple:

```ln
from @std/app import start, print, exit
from @std/datastore import namespace, set, getOr, has, del

on start {
  const foo = namespace("foo")
  foo.set("bar", 1)
  foo.has("bar").print() // Prints 'true'
  foo.getOr("bar", 0).print() // Prints '1'
  foo.del("bar").print() // Prints 'true'
  emit exit 0
}
```

A new standard library named `@std/datastore` is proposed that exports five functions, `namespace`, `set`, `has`, `getOr`, and `del`. All namespaces and keys are required to be strings (as they take on a similar role to module and variable names), while the value can be any data type.

Namespaces are lazily created on usage, and all value reads must provide a default value in case the namespace+key pair does not exist. This also provides the typing information necessary to determine how to read the value back out for the Rust runtime, and for the compiler to use in the following statements. Checking for the existence of a namespace-key pair can be done with the `has` function, but since the namespace+key pair can be `del`eted at any time, it would not be safe to have an unguarded `get` function, and is not provided.

Under the hood, this would be implemented with a few opcodes corresponding to the four CRUD-like functions. The `namespace` function will be pure Alan code meant to make working with these opcodes easier. The opcode function signatures are as follows:

```ln
// The datastore fixed and variable set opcodes
fn dssetf(ns: string, key: string, val: int8): void
fn dssetf(ns: string, key: string, val: int16): void
fn dssetf(ns: string, key: string, val: int32): void
fn dssetf(ns: string, key: string, val: int64): void
fn dssetf(ns: string, key: string, val: float32): void
fn dssetf(ns: string, key: string, val: float64): void
fn dssetf(ns: string, key: string, val: bool): void
fn dssetv(ns: string, key: string, val: any): void

// The datastore has opcode
fn dshas(ns: string, key: string): bool

// The datastore del opcode
fn dsdel(ns: string, key: string): bool

// The datastore fixed and variable get opcodes
fn dsgetf(ns: string, key: string): Result<int8>
fn dsgetf(ns: string, key: string): Result<int16>
fn dsgetf(ns: string, key: string): Result<int32>
fn dsgetf(ns: string, key: string): Result<int64>
fn dsgetf(ns: string, key: string): Result<float32>
fn dsgetf(ns: string, key: string): Result<float64>
fn dsgetf(ns: string, key: string): Result<bool>
fn dsgetv(ns: string, key: string): Result<any>
```

The get opcodes return a result object, but being able to choose which version of the opcode to use depends on type data from the user, so that is not exposed to the developer.

Internally, the js-runtime can simply use a global object for the namespaces, with each namespace being a sub-object. The rust runtime has more work to do. If we stick to Rust's standard library likely a [RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html) of a `HashMap` of `HashMap`s of `HandlerMemory` objects owned by the VM. This assumes a pattern of heavier reads than writes, which may not be a good idea. [Dashmap](https://github.com/xacrimon/dashmap) looks like a good library to use, though we would need to decide if nesting it within itself is a good idea or if we should just concatenate the namespace to the key with some unprintable character(s) in between and keep it one layer deep.

### Alternatives Considered

The first major alternative considered is to simply not do this and require users to use Redis, PostgreSQL, etc external databases to handle this concern. The dual issues of performance and ease of development overrode this, though. Requiring IO for this is certainly going to be slower than staying in-process, and requiring a database for this use-case makes the development and deployment of any application that requires this behavior far more complex.

The second alternative considered was baking this into the language level, allowing `let` variables in module scope, having them namespaced automatically to the module scope, and allowing mutation outside of the module only to variables that were `export`ed. This appeared to have many benefits to "regularity" and a separation between public and private mutable module-level globals, but it also introduced many irregularities -- accessing these variables requires using `Result` methods which is not necessary for other `let` variables, they cannot be deleted so they would have to be wrapped in an `Maybe`, `Array`, or `HashMap` type if the user wants to be able to do so (and would be different based on user preference), and they are less obviously a performance impact to developers, which may encourage their mis-use. Extra "gotchas" in the syntax of the language and bad developer incentive structure nixed this approach (though its pretty nice, otherwise).

A final alternative was to have the `namespace` automatically defined for you in your own module and that you cannot access the namespace of other modules -- they would have to export helper functions to let you mutate their own namespace. This was the most tempting alternative as the safety is increased, but it also increases the "magic" of the `@std/datastore` standard library, where its functions "know" what module they're called from and do something different in that situation. That would likely require compiler support and no library written by an end user would have that power. This makes that library less "fair" than the rest: only the root scope is allowed right now to have special behavior, but what it creates is available to all libraries. It also introduces some sharp edges in the language: if someone wraps the official `@std/datastore` in a mocked version, what should the compiler do with it? Will it accidentally shift all namespaces from the module name to `"@std/datastore"`? Or will it give this mock `@std/datastore` module the same "powers" as the real one? If the former, users accidentally break functionality when they were just trying to, say, prevent key deletions. If the latter, there will be users who *will* mock it and insert completely arbitrary functionality to get access to this "called from" behavior for their own purposes, making codebases hard to parse.

So this final alternative is rejected due to the module-level complexities it introduces to the language.

## Affected Components

Just a new standard library needs to be written along with 6 new opcodes in the two runtimes.

## Expected Timeline

This could be built in a day, once we have time to tackle it.

