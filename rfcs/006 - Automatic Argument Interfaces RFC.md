# 004 - Automatic Argument Interfaces RFC

## Current Status

### Proposed

2020-07-22

### Accepted

YYYY-MM-DD

#### Approvers

- Luis de Pombo <luis@alantechnologies.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <david@alantechnologies.com>

## Summary

Interfaces declare how an unknown type can be used by a function, specifying which functions or operators can consume it, and which fields it must have (if it is a user-defined type and not a built-in type). This allows one to write generic functions that can work on many different types of data, even types the original author is not yet aware of, as long as it has the "right shape." Having an explicit interface type certainly produces better self-documenting code, but it also gets in the way of an exploratory phase as you're writing your generic function -- adding a new function, operator, or field access in your function also requires you to edit the interface definition to get this to compile.

This is not necessary, however, as the *usage* of the argument in the function itself defines a set of functions, operators, and fields that the interface would need, so it should be possible to have the compiler automatically infer the implied interface from the function's usage. This provides the user the exploratory benefits of a dynamic language with the runtime performance benefits of a static language (as the compiler will convert the various usages of the function into the underlying static types), and completes the capabilities of the type inference engine such that variables, function return types, and now arguments would no longer require an explicit type declaration by the user. Combine this with a formatting tool that can extract the inferred types and insert it into the source tree, and you can have your cake and eat it, too!

## Expected SemBDD Impact

If `alan` was at 1.0.0 or beyond, this would be a minor update: old code would continue to function, while new code would have the option of having argument types inferred.

## Proposal

Consider the following application:

```ln
from @std/app import start, print, exit

interface f64 {
  toFloat64(f64): float64
}

fn average(a: f64, b: f64) = (a.toFloat64() + b.toFloat64()) / 2.0

on start {
  print("average of 1 and 1.5 equals " + average(1.0, 1.5).toString())
  print("average of 1 and 2 equals " + average(1, 2).toString())
  print("average of true and false equals " + average(true, false).toString())
  emit exit 0
}
```

This prints the following:

```
average of 1 and 1.5 equals 1.25
average of 1 and 2 equals 1.5
average of true and false equals 0.5
```

The `f64` interface makes it explicit that any variable in a function with that type must have a `toFloat64` converter function associated with it, but this is also immediately obvious from the usage of the `a` and `b` variables in the `average` function itself.

The `f64` interface isn't adding any new knowledge for the compiler, though it does provide documentation about what types are allowed without needing to read the entire function body. This doesn't matter in this particular example as the function body is the same size as the interface body, but for functions with many LOC this can become a burden in the other direction where the "shape" of multiple variables are all "mixed up" within a block of imperative code and static types speed comprehension up.

Developers who prefer composing their code through many small functions in a large call graph have no problems with dynamic types as they're more self-contained in any particular function and refactoring a single function in a dynamic language will "just work" while in a static language the type information needs to be updated throughout the call graph, making it tedious. Developers that prefer to work with fewer, larger imperative-style functions and a shallower call graph have less overhead needed to update those types and gain more from their centralized definition, while in a dynamic language they don't have that central definition so they can get confused on what is allowed or isn't on any particular type and run into production bugs due to that. These stylistic differences between the programmers leads to the dynamic vs static divide and why they always shout over each other without understanding. Thank you for coming to my TED talk.

Type inference, while having [existed for decades](https://books.google.com/books?id=QcYl_ylrHmcC&q=%22type+inference%22#v=snippet&q=%22type%20inference%22&f=false), is recently coming into vogue due to a confluence of factors, primarily that dynamic interpreted languages have raised a generation of developers used to the large call graph of simple functions style while simultaneously Moore's Law petering out has made the performance issues of these dynamic languages a larger problem, as you can't just upgrade your servers every few years and handle a greater scale of usage "for free." So static languages that can better appeal to these developers gain a greater marketshare of the entire developer community.

`alan` already has type inference for function returns and variables, built on top of opcodes that are mostly statically-typed so composition of them produces other known statically-typed inputs and outputs. (The exception being for Generic types resolved with Interface types in some of the opcodes. The compiler does some special trickery with Interface type substitution at the opcode level to get that to work as expected.) The argument types can be properly constrained by generating inferred interface types based on the usage of the argument within the function body, which will allow the compiler to still error if impossible types are provided instead of failing at runtime, eg:

```ln
from @std/app import start, print, exit

interface f64 {
  toFloat64(f64): float64
}

fn average(a: f64, b: f64) = (a.toFloat64() + b.toFloat64()) / 2.0

on start {
  print("average of [1, 2, 3] and [4, 5, 6] equals " + average([1, 2, 3], [4, 5, 6]).toString())
  emit exit 0
}
```

returns the compile time error (right now) as:

```
Unable to find matching function for name and argument type set
average(<Array<int64>>, <Array<int64>>)
```

after which the developer can decide if they are accidentally passing in invalid data, or if they need to modify `average` to support Arrays (which don't have a `toFloat64` function). Fortunately multiple dispatch can make that modification easy as that case can simply be a different function entirely, allowing the original `average` to remain short and easy-to-follow.

The nice thing about type inference situation is different developer teams could set up their own linting rules on where along the spectrum they want to be, from implicit types everywhere, to explicit types everywhere, with any combination of partially-explicit typing in-between, and a fully type inferred language can look very compact and "clean" to dynamic typing enthusiasts:

```ln
from @std/app import start, print, exit

fn average(a, b) = (a.toFloat64() + b.toFloat64()) / 2.0

on start {
  print("average of 1 and 1.5 equals " + average(1.0, 1.5).toString())
  print("average of 1 and 2 equals " + average(1, 2).toString())
  print("average of true and false equals " + average(true, false).toString())
  emit exit 0
}
```

Depending on the implementation this precise version may or may not behave exactly the same as the original example, depending on how the type inference for each variable works. I propose that it should *not* be the same. Each variable should have a separately-inferred interface type so the actual types of `a` and `b` could be different at runtime instead of the same. Translated back to the existing version of `alan`, this would be like:

```ln
from @std/app import start, print, exit

interface f64 {
  toFloat64(f64): float64
}
interface f64_2 = f64

fn average(a: f64, b: f64_2) = (a.toFloat64() + b.toFloat64()) / 2.0

on start {
  print("average of 2.5 and false equals " + average(2.5, false).toString())
  emit exit 0
}
```

which prints:

```
average of 2.5 and false equals 1.25
```

This may or may not be a nonsensical behavior in this codebase, but the other way (where the automatic types are inferred together, or at least combined if they match) is more restrictive and may prevent the behavior the user wants.

But if the user does want an automatically inferred type that explicitly matches, I propose a new "type" name, `auto`, such that the way to write the original example with the exact same behavior at compile time would be:

```ln
fn average(a: auto, b: auto) = (a.toFloat64() + b.toFloat64()) / 2.0
```

and now the compiler would know that it should infer these two types into the same interface.

This is not quite enough, though. Suppose there was some function `foo` that took three arguments that should be inferred, where two of them should constrain to the same type and one constrained separately. Technically the following would work:

```ln
fn foo(a: auto, b: auto, c) ...
```

but if that `foo` was changed to take four arguments, two that need to be one type and two that need to be another, it would fail. So the final proposal is that the `auto` keyword can also have any number of integer digits appended to it to distinguish between them, eg `auto`, `auto0`, `auto1`, [`auto9001`](https://www.youtube.com/watch?v=SiMHTK15Pik), etc. Eg,

```ln
fn foo(a: auto0, b: auto0, c: auto1, d: auto1) ...
```

This advanced sort of constraint on the automatically inferred types would likely be part of a developer flow to convert some dynamically-typed code into statically-typed code, to give an envisioned "type inference development tool" hints on how to construct the automatically-inferred interfaces for the developer. (A tool to let developers automatically add or remove typing information from a given source file, or perhaps even within a range of lines of said source file, so a developer that doesn't understand the types within a given source file can have the tool generate said types for their inspection, or a developer can strip the types away, refactor the code, and then regenerate types that they can then adjust the naming to their liking.)

### Alternatives Considered

The primary alternative considered is not doing this at all. Type inference places a much higher burden on the compiler to "get things right" and bugs in the compiler could cause wonky inferences in special cases. Further, type inference increases the flexibility of the language and its grammar and that is not always a good thing. Lisp's extreme flexibility is likely the primary cause of [the Lisp Curse](http://www.winestockwebdesign.com/Essays/Lisp_Curse.html) as distinct sub-grammars and conventions between different sub-groups in the Lisp community harmed cooperation and collaboration due to a ["Tower of Babel"](https://en.wikipedia.org/wiki/Tower_of_Babel)-like situation versus less-flexible languages.

More than the other forms of type inference already in the language, this one is heavily implicit instead of explicit, as well, which produces a "magic" to the syntax that can be harder for newbies to expand beyond without exposure to hard static typing, whereas comparing the equality of two `int64`s returning a `bool` is more "obvious."

However, this was ultimately rejected because of the idea of tooling, inspired by the [tooling of Go](https://golang.org/cmd/go/), that could facilitate "translation" between the camps along the static-to-dynamic spectrum. This will allow `alan` to address a larger swath of the developer community with varying opinions, needs, and group sizes.

Argument type inference without the `auto`, `auto0`, `auto1`, etc syntax was also considered as significantly simpler, but the lack of flexibility with the inferred argument types and inability to reproduce certain statically-typed function behaviors in the inferred system felt wrong. However, the implementation of inferred types may go through that version as a step along the way to producing the full argument type inference system described in this RFC.

## Affected Components

This affects solely the first stage of the compiler and the new BDD tests that will be necessary.

## Expected Timeline

The initial type inference logic will likely take 1-2 weeks to iron out all of the kinks: if the same operator symbol is defined with multiple precedence levels, there may be *multiple* interface solutions possible, and how to handle that is still up-in-the-air. Also required properties may take more time with nested properties and/or array accesses, eg `foo[0].bar.baz` would need to infer that `foo` is an array type with an interface that has a `bar` required property and that `bar` property needs to have an inferred interface that has a `baz` property. Not impossible, but hard.

Once that's done, another week to add support into the language grammar and compiler for the `auto` type/keyword and related names (`auto0`, `auto1`, etc), and then adjust the interface inference to reuse the inferred type generation between arguments.

Finally, TBD on the tooling idea to automatically adjust the source file to include/exclude types. That may depend on `alan` having a language server to function correctly.
