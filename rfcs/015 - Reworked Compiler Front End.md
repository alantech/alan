# 000 - RFC Template

## Current Status

### Proposed

2020-03-24

### Accepted

YYYY-MM-DD

#### Approvers

- Full Name <email@example.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- Colton Donnelly <colton@alantechnologies.com>

## Summary

Rework the front end of the compiler to utilize the Hindley-Milner type system for type resolution, as well as generating Microstatements only once the program has been "solved".
Users will be able to opt-in to the new compiler front end while it's being developed by changing their Alan file to the `.lnn` extension.

## Expected SemBDD Impact

If Alan was 1.0.0, this entire change could be implemented as a single patch update, since the proposed changes only affect the internal architectural changes in the compiler.
However, the intention is to gradually move towards the new compiler front-end, which will be implemented over the course of multiple patch updates.
As such, this change will impact many patch updates, until the point it's able to replace the current front-end without any regressions to user code.

Until the new front-end is able to replace the current one, bdd tests will gradually be included in a second run which will compile programs with the new front-end.

## Proposal

This RFC proposes a new compiler front-end, with the most significant difference being the use of a Hindley-Milner-based type resolution system.
By modifying the Hindley-Milner inference systems to use Alan's interface and generic types, the compiler will be able to determine the types of functions without type annotations, while also allowing the compiler to select one of many functions depending on the input types being passed in.
To explain how the new front-end will differ from the current one, we'll use examples that grow in complexity.

It's worth noting that, since this PR is talking about the inner architecture of the compiler front-end, there is terminology that is used that is specific to the compiler's current architectural design.
Some quick definitions:
- Microstatement: The atom of the current `lntoamm` phase's internal language. The set of Microstatements is a superset of the `amm` language - a valid `amm` program is a valid program of Microstatements, but not all Microstatements are valid in `amm`.

Also, "constraint" is used in literature related to Hindley-Milner type checking to signify that a narrowing-down on the type of a value, expression, or function.
A type can only be selected if it matches constraints, and if there are constraints that conflict with each other, that results in an invalid program.

The initial motivating example is as follows:
```ln
from @std/app import print, start, exit

on start {
  const myMaybe: Maybe<int64> = none();
  const stringified = myMaybe.toString();
  emit exit 0;
}
```

Admittedly, this code does not currently work in the compiler due to a bug that results in a variable declaration's type annotation only being checked for validity, and not being used for the corresponding Microstatement in the event that it's more concrete than the function being called.
Assuming it did work, the process of translating the program with the current front-end might look like this:
1. Define the `Maybe<int64>` type and insert it as a valid type in the compiler's global scope.
2. Find all functions and values named `none`.
3. Inline the first `none` function that accepts 0 arguments as defined in `root.ln` (resulting in only the `noneM` opcode being assigned in a `CONSTDEC` Microstatement).
4. If the specified type (`Maybe<int64>`) can be applied to the output type of the most recent Microstatement (which looks like `const abc123: Maybe<any> = noneM()` - the output type is `Maybe<any>`), then replace the output type with the more specific type (the Microstatement from before should now look like `const abc123: Maybe<int64> = noneM()`)
5. Find all functions and values named `toString`.
6. Inline the first `toString` function that accepts 1 argument of type `Maybe<int64>`. This results in `toString(Maybe<Stringifiable>): string` from `root.ln` being selected, since `int64` satisfies the `Stringifiable` interface.

The new compiler front-end will behave quite similarly:
1. Define the value `abc123` with type `T`. Add a constraint that type `T` must match `Maybe<U>`, and that type `U` is `int64`.
2. Find all functions and values named `none`, and use them as the list of possibilities for the value `abc123`.
3. Define the value `abc124` with type `S`. Add a constraint that type `S` must match `string`.
4. Find all functions and values named `toString`, and use them as the list of possibilities for the value `abc124`.
5. Going back to the results of step 2, select `none` as defined in `root.ln`, since it is the only function that accepts 0 arguments.
6. Going back to the results of step 4, select `toString(Maybe<Stringifiable>): string` as defined in `root.ln`, since it is the only function that matches the constraints on `T`.
7. Inline all of the selected functions, and repeat steps 2-6 as necessary.

Let's look at another example, which the current front-end can't solve without significant backtracking:
```ln
from @std/app import print, start, exit

on start {
  let myMaybe = none();
  myMaybe = none();
  myMaybe = some(5);
  emit exit myMaybe.getOrExit().toInt8();
}
```

There's a lot more complexity here!
The current front-end would have to backtrack across all declarations, assignments, and Microstatement "rerefs" in order to ensure that the type assigned to `myMaybe` is `Maybe<int64>` once the `some(5)` assignment is reached.
However, with the new front-end, the compiler would initially assign `myMaybe` to type `Maybe<T>`, then add a constraint that `T` can be assigned by the integer literal `5`.
Then, once the `toInt8` call is reached, the compiler would add another constraint that for whatever type `T` is, there must be a function called `toInt8` that matches the signature `(T) -> int8`.
If there are no other constraints, the compiler is able to resolve `T` as an `int64` before outputting the corresponding `amm`.

The new compiler front-end will also be able to determine function signatures without requiring type annotations on parameters or the return type.
A simple example might look like:
```ln
fn add4(num) {
  return add(num, 4);
}
```

At the time of writing this RFC, this function fails to parse due to a missing type annotation on `num`, but it's intended to support gradual typing even for function parameters in Alan.
The current front-end will require substantial work to support this, requiring one of:
- An interface (internal or in `root.ln`) that ensures there's an `add` operation for the type of `num` (for example, `Addable`).
- Type-checking can only happen once the function is inlined.

However, the new type resolution system would simply add a constraint that there be a function named `add` that satisfies the signature `(T, 4 as T) -> T` (ignoring the fact that arithmetic operations in Alan actually use `Result` values).
This wouldn't require the `Addable` interface to be defined at all, and instead would allow for an anonymous interface that's only visible in the compiler to be created.
The constraints would then not need to be computed for each time the function is called - instead, we must only ensure that the value passed in to `add4` satisfies the so-called "Addable" interface.

A more complex example would be the `toString` implementation for `Maybe`, which could be rewritten as such under the new front-end:
```ln
fn toString(val) {
  if val.isSome() {
    return val.getMaybe().toString();
  } else {
    return 'none';
  }
}
```

In order to share more nuanced details about the new front-end, here are the steps the compiler might take upon having to use this function:
1. Assign the variable `val` to type `T`. There are no constraints on the type `T`.
2. Desugar the if/else. How this is done is unimportant, but the body of the `toString` function will effectively be desugared into (ignore the invalid `conditionTailFn` function body - it's required for `evalcond` to work but its contents won't be checked by the type checker, since this desugaring will actually happen just before generating the Microstatements):
```ln
const conditionalTable = newarr(2);
const condition1Boolean = val.isSome();
const condition1ThenFn = fn(): U { return val.getMaybe().toString(); };
condfn(conditionalTable, condition1Boolean, condition1ThenFn);
const condition2Boolean = true;
const condition2ThenFn = fn(): U { return 'none'; };
condfn(conditionalTable, condition2Boolean, condition2ThenFn);
const conditionTailFn = fn(): U {};
const conditionResult: Maybe<U> = evalcond(conditionalTable, conditionTailFn);
const conditionResultUnwrapped: U = conditionResult.getMaybe();
return conditionResultUnwrapped;
```
3. Add the constraint that `val` has type `A` where there's a function called `isSome` with the signature `(A) -> bool`.
4. Add the constraint that `val` has type `B` where there's a function called `getMaybe` with the signature `(B) -> C`.
5. Add the constraint that type `C` from `getMaybe` has a function called `toString` with the signature `(C) -> U`.
6. Add the constraint that type `U` is the same as the concrete type `string` (this is from the `return 'none'` statement).
7. Reduce type `A` to only have the possibility of being type `Maybe<S>`.
8. Reduce type `B` to only have the possibility of being type `Maybe<R>`.
9. Since `Stringifiable` is an already-existing interface that perfectly matches the constraints on type `C`, replace all instances of type `C` with `Stringifiable`.
10. Since `A` and `B` both are in reference to the same variable, add the constraint that they satisfy each other.
11. Reduce type `Maybe<R>` to be equivalent to the type `Maybe<Stringifiable>`
12. `Maybe<Stringifiable>` is the minimal interface value for `val`, so the signature of the `toString` fn is now rewritten as `(Maybe<Stringifiable>) -> U`.
13. `string` is the concrete type representation for `U`, so the signature of the `toString` fn is now rewritten as `(Maybe<Stringifiable>) -> string`.

(side note: the `getMaybe()` call will also eventually be syntactic sugar, according to [RFC 12][RFC-12]).

### Alternatives Considered

One alternative is to keep the current type system and compiler front-end and adding changes as-needed.
However, the current front-end's design means that it takes relatively substantial work to add each feature, with [new features potentially exposing "load-bearing" bugs][cond-table-PR].
Additionally, there is [a lot of documented tech-debt][tech-debt-issues] that would already result in substantial parts of the compiler being reworked or rewritten.
Lastly, the Hindley-Milner type system has been well-documented and proven both [over time][haskell-type-system-revered] and [in theory][emlti].

On the other hand, another alternative would be to rewrite the entire compiler in Rust.
However, as much of the current compiler already performs well, that would not be very productive and would just be reinventing the wheel, for the most part.

[cond-table-PR]: https://github.com/alantech/alan/pull/430
[emlti]: http://pauillac.inria.fr/~fpottier/publis/emlti-final.pdf
[haskell-type-system-revered]: https://softwareengineering.stackexchange.com/questions/279316/what-exactly-makes-the-haskell-type-system-so-revered-vs-say-java
[tech-debt-issues]: https://github.com/alantech/alan/labels/tech%20debt

## Affected Components

A brief listing of what part(s) of the language ecosystem will be impacted should be written here.

## Expected Timeline

An RFC proposal should define the set of work that needs to be done, in what order, and with an expected level of effort and turnaround time necessary. *No* multi-stage work proposal should leave the language in a non-functioning state.

