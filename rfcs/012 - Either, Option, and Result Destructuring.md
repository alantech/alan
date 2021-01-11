# 012 - Easier Either, Option, and Result Destructuring

## Current Status

### Proposed

2021-01-08

### Accepted

YYYY-MM-DD

#### Approvers

- David Ellis david@alantechnologies.com

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- Colton Donnelly <colton@alantechnologies.com>

## Summary

Currently, error handling in Alan is done through the `Result<T>` type, which is based on Rust's `Result<T, E>` type.
While Rust has convenient ways of retrieving the value of a `Result`, it's difficult to replicate this behavior due to a lack of pattern matching.
This RFC seeks to introduce a new language feature that would allow Alan to replicate the convenience of Rust's `Result` destructuring without introducing pattern matching.

## Expected SemBDD Impact

If Alan was at least version 1.0.0, then this would be considered a major change that breaks user code when destructuring a `Result<T>`.

## Proposal

This proposal seeks to introduce a special case when handling `if` statements in Alan.
Currently, when handling a `Result<T>`, a user might expect to write something like this in order to retrieve a value.
```
let myResult = ok(5);
if myResult.isOk() {
	let okVal = myResult.getOr(-1);
	print("should be 5: " + okVal);
} else {
	let errVal = myResult.getErr("expected error, but could not retrieve Error value");
	print("unreachable, but I know that errVal is NOT 'expected error, but could not retrieve Error value', unless that's what myResult is");
}
```

Note that, in both branches of the above `if` statement, the status (`ok` or `err`) of the `Result` is already known, but the user must then call either `getOr` or `getErr` in order to retrieve the expected value.
(Note that `getOrDefault` is not yet implemented, but only makes this process marginally easier.)
`getOrExit` was also recently added to the API, which is a good alternative, but the name implies that the function is fallible (albeit the failure case is quite dramatic), despite this use-case being categorically infallible.
Instead, it would be much easier for the user (and potentially even more efficient, although marginally) if the variable could be immediately used as if it were destructured.
Since `isOk` already guarantees that the `Result` is `ok` (when it returns `true`, otherwise the guarantee is that it's `err`), it would make sense to be able to write the above code like this:
```
let myResult = ok(5);
if myResult.isOk() {
	print("should be 5: " + myResult); // 1
} else {
	print("unreachable, but if myResult is an error, then this would print that error: " + myResult); // 2
}
```

In the above code, `myResult` was already destructured into its `ok` value at location 1
(in other words, at location 1, `myResult` is of type `int64`, not `Result<int64>`).
This allows the user to quickly and easily perform operations on that inner value without the additional step mentioned above.
Note that, if myResult were an `err` instead, it would work the exact same way, with the `Error` taking the place of `myResult`.

This should also be extended to `Option<T>` and `Either<T, U>` as well:
```
let myOption = some(true);
if myOption.isSome() {
	print("here ya go: " + myOption); // 1
} else {
	print("I've got nothing for ya"); // 2
}

let myEither = getEither(); // 3
if myEither.isMain() {
	print("main rock: " + myEither); // 4
} else {
	print("alt rock: " + myEither); // 5
}
```

In location 1, `myOption` should be of type `bool`.
Meanwhile, in location 2, it might make sense to undefine the `myOption` var - this is fairly ambiguous and needs more discussion.
In location 3, `getEither` should be considered a `fn(): Either<string, int64>`.
In location 4, `myEither` should be of type `string`, and location 5 would be `int64`.

The implementation should be relatively simple: when building the `amm` representation of the program, if the compiler detects any `if` statements (and other constructs that should have this feature too - if deemed worthy) with this pattern, then it should reassign the value at the beginning of the block.
Preferably, this would be done with the `getR` opcode, which would allow for the value to be destructured from the `Result` efficiently.

Of particular importance is not only when this sugar gets applied, but also when it doesn't.
Above, it's mentioned that the values are removed within "then" and `else` blocks - these are the only contexts where the values are automatically destructured.
```
let myResult = ok(5);
if myResult.isOk() {
	print("should be 5: " + myResult); // myResult is `int64`
} else {
	print("unreachable, but if myResult is an error, then this would print that error: " + myResult); // myResult is `Error`
}
print("either way, the value is " + myResult) // myResult is `Result<int64>`

let myOption = some(true);
if myOption.isSome() {
	print("here ya go: " + myOption); // myOption is `bool`
} else {
	print("I've got nothing for ya"); // myOption isn't available
}
print("either way, the value is " + myOption) // myOption is `Option<bool>`

let myEither = getEither(); // myEither is `Either<string, int64>`
if myEither.isMain() {
	print("main rock: " + myEither); // myEither is `string`
} else {
	print("alt rock: " + myEither); // myEither is `int64`
}
print("either way, the value is " + myEither) // myEither is `Either<string, int64>`
```

### Alternatives Considered

The simplest alternative is to accept the current way of destructuring these types.
However, this results in a lot of unnecessary overhead that any sane person will hate [(although there are fewer sane people, apparently)](https://github.com/golang/go/issues/32437).
As such, a solution should be found that is good enough for the 99% use case (this proposal), while still providing an easy solution for every other case (the API).

A relatively easy alternative would be to expose a `get` alias to `getOrExit` that would be available in the contexts described by this document, but this has 2 problems:
1. It fails to address the issue of overhead as mentioned above.
2. Having functions only available during specific syntactic contexts kinda seems a bit more absurd than the syntactic sugar proposed in this document.

Another alternative would be to add ML-style pattern matching, but that might make the language quite distracted in its design and make itself inconsistent.
If it does get decided to use introduce pattern matching, then that could be added later on without any risk to backwards-compatibility.

## Affected Components

All code dealing with `Result`, `Option`, or `Either` in conditional branches after checking their type (eg `isOk`, `isSome`, etc) will be broken.
Internally, affects the compiler but not the runtime.

## Expected Timeline

This could probably be done relatively quickly - I suspect it could be done in 1 or 2 days.
However, users should be told about this breaking change so that their code will continue to work.
