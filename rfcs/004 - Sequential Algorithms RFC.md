# 004 - Sequential Algorithms RFC

## Current Status

### Proposed

2020-07-19

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

There are algorithms people need to write that are inherently sequential, and often non-deterministic on the number of steps needed to take, such as with numeric approximation algorithms like Newton-Raphson, or anything written as a recursive function call. Most of the problems developers need to solve does not require this linearity, but when it really is required, it would be sorely missed by the developer. These include such venerable mechanisms as `while`, `do-while`, `for`, and etc.

Reintroducing this power to `alan` in a controlled manner that still allows the runtime to be able to plan around these algorithms *and* still provide the guarantee of termination is the purpose of this RFC. There is a grammar that will be presented that provides this in a minimally invasive form to the mental model of most developers, but has been intentionally rejected to nudge developers to find better, more deterministic and more parallelizable mechanisms, instead.

## Expected SemBDD Impact

If `alan` was beyond `1.0.0` this would be a minor update.

## Proposal

A new standard library to be added named `@std/seq`. This library would provide functions for accomplishing many different sequential patterns that depend on a special `Seq` type that behaves something like this:

```ln
type Seq {
  counter: int64
  limit: int64
}
```

However, it will be a built-in type and its internals opaque, similar to the `Array` type, as manipulation of the `limit` field after initial construction defeats a major guarantee the runtime is depending on for this to work. A new instance of this type can only be created with the `seq` function, with the signature:

```ln
fn seq(limit: int64): Seq
```

When created, the `counter` is initialized to 0 it can be "consumed" up to the `limit` number of calls. There are some interesting advanced behaviors that fall out of this model if a particular `Seq` instance is used by more than one sequential function, but it is assumed the vast majority of the time a call to a sequential function will provide it with a new `Seq` instance.

Here are the function signatures of the sequential functions that `@std/seq` will provide:

```ln
fn next(seq: Seq): Result<int64>
fn each(seq: Seq, func: function): void
fn while(seq: Seq, condFn: function, bodyFn: function): void
fn doWhile(seq: Seq, bodyFn: function): void
fn recurse(seq: Seq, recursiveFn: function, arg: any): anythingElse
fn generator(seq: Seq, generatorFn: function, initialState: any): ArrayLike<anythingElse>
```

`next` is the simplest to explain: Each time you call it, it returns the current `counter` value wrapped in a Result and then increments it. If you call past the limit, it returns an Error Result.

`each` is almost as simple: It simply runs the provided function the however many times is in the `limit` of the `seq` instance. the `func` function must be the following signature:

```ln
fn func(): void
fn func(i: int64): void
```

a pure side-effect function that may or may not take the current iteration counter.

`while` runs the `bodyFn` *up to* the `limit` number of times, but can abort early if `condFn` returns `false`. The signatures of these two functions must match:

```ln
fn condFn(): bool
fn bodyFn(): void
```

`doWhile` always runs at least once (unless the `seq` has reached its `limit` or it was constructed with an initial `limit` of `0`) and uses the return value of the function to determine if it should continue or not, so its `bodyFn` has the following signature:

```ln
fn bodyFn(): bool
```

`recurse` allows recursive functions to be defined in `alan`. This is impossible in `alan`'s grammar, so what is done is special trickery to make it possible. The `recursiveFn` has the following function signature:

```ln
fn recursiveFn(self: Self, arg: any): Result<anythingElse>
```

The `Self` type is a special type that the recursive function can use to trigger a controlled recursive call, like so:

```ln
const recursiveResult = self.recurse(someNewArg)
```

`Self` is another opaque type that the runtime can use to keep track of the function to be called recursively and how deep the recursion has gone so far. The `recursiveFn` *must* wrap its value in a `Result` type because `alan` may interject and bubble up an error of the recursion limit is reached.

The "final" function in `@std/seq` is `generator`, which allows one to define a generator function for returning a lazily-generated array of values. It returns an `ArrayLike` type instead of an `Array` type so that laziness is respected, and it has all of the functions for treating it like an `Array` to also tag along (hence why `generator` itself isn't really the "final" function), plus it's own form of `next` that executes the `generatorFn` and returns its wrapped value, or an error if past the limit.

The function signature of `generatorFn` looks like:

```ln
fn generatorFn(state: any): anythingElse
```

The `state` is a mutable argument passed in that the generator can use to keep track of any internal state it wants, with the first call given the `initialState` value.

### Alternatives Considered

There is a *very tempting* alternative to the above standard library: new syntax, instead, that may be implemented internally similarly to how the above is done, similarly to how `if-else if-else` conditional statements are decomposed into `cond` function calls.

This would allow things like:

```ln
seq(10).each(fn (i: int64) {
  print(i)
})
```

to become:

```ln
for i: int64 in seq(10) {
  print(i)
}
```

or:

```ln
let foo = "f"
seq(10).while(fn = foo != "foo", fn {
  print("I pity the foo!")
  foo = foo + "o"
})
```

to be:

```ln
let foo = "f"
while foo != "foo" {
  print("I pity the foo!")
  foo = foo + "o"
} limit 10
```

or:

```ln
let i = 15
seq(10).doWhile(fn {
  if i % 2 == 0 {
    i = i - 1
  } else {
    i = i - 3
  }
  return i > 0
})
```

could be:

```ln
let i = 15
do {
  if i % 2 == 0 {
    i = i - 1
  } else {
    i = i - 3
  }
} while i > 0 limit 10
```

then:

```
fn fibonacciRecursive(self: Self, i: int64): Result<int64> {
  if i < 2 {
    return some(1)
  } else {
    const prev = self.recurse(i - 1)
    const prevPrev = self.recurse(i - 2)
    if prev.isErr() {
      return prev
    }
    if prevPrev.isErr() {
      return prevPrev
    }
    return some((prev || 1) + (prevPrev || 1))
  }
}
print(seq(100).recurse(fibonacciRecursive, 8))
```

could be:

```ln
recursive fn fibonacciRecursive(i: int64): Result<int64> {
  if i < 2 {
    return some(1)
  } else {
    const prev = fibonacciRecursive(i - 1)
    const prevPrev = fibonacciRecursive(i - 2)
    if prev.isErr() {
      return prev
    }
    if prevPrev.isErr() {
      return prevPrev
    }
    return some((prev || 1) + (prevPrev || 1))
  }
} limit 100
print(fibonacciRecursive(8))
```

and so on.

Turning it into syntax can make it look *much* more natural to those coming from other imperative languages, with only the `limit <number>` syntax being the addition needed for the runtime to be able to calculate an upper bound on the runtime and have a guarantee that a value will eventually be returned. But this approach has been rejected for the following reasons:

1. It is a *lot* of syntactic sugar, and introduces a lot of new reserved words to support it, which complicates the compiler and the mental overhead of writing in the language needing to memorize this stuff for any source file you're looking at.
2. Any code written with this syntax is *guaranteed* to be sequential on a single CPU, and due to the nondeterminism on the number of runs, more difficult to properly schedule work around. Having built-in syntax for it will encourage its use and keep this syntax normalized amongst developers, when it really should be seen as a tool of last resort in an increasingly multicore computing world.
3. It is hoped that the community decides to prefer default linting rules that produces lint warnings (or errors?) on usage of `@std/seq` at all -- ideally anything that needs this functionality is wrapping it up in a nice-to-use library, like a [fast inverse square root](https://en.wikipedia.org/wiki/Fast_inverse_square_root) estimator using it, and "regular" code can ignore it. It is far less likely that such a linting rule would be agreed upon in the community if these were built-in keywords and statements in the language (though [this has happened in the past](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/with), but let's not try to tempt fate here?) 

## Affected Components

The proposed syntax should require no changes to the compiler excepting the definition of new opcodes to support it. The standard library code would be written to use these new opcodes, and the runtimes would need to implement the opcodes.

## Expected Timeline

This library would be based on the work involving Array opcodes, and would likely take approximately a week's worth of effort to build and test.

