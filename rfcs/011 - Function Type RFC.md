# 011 - Function Type RFC

## Current Status

### Proposed

2020-12-15

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

Function types in Alan being just `function` was a shortcut that was taken because the way the language and compiler works it would successfully compile your code when you provided the right kind of function to a user-defined function, and it was a finite number of opcodes that had to have manual tuning in the compiler to support passing user-defined functions to them, with similar compilation behavior.

But a combination of wanting better error messages for users and smaller, less error prone code for type inference, as well as more legible type signatures for users of higher-order functions, means we need to address this with properly defined function types.

## Expected SemBDD Impact

If Alan was 1.0.0 or later, this would be a major update that breaks all code that involves higher-order functions (which in Alan is all non-trivial code).

## Proposal

From an implementation perspective, all of the work done for type matching generic and interface types can be easily reused, so under the hood the implementation would be swapping from the `function` type to a `Function<A, R>` generic type, with `A` being the arguments and `R` the return type.

Then we can have `A` be `void` for functions without arguments, `Arg1<type>` for single-argument functions, `Arg2<typeA, typeB>` for 2 argument functions, etc. We could either auto-generate the `Arg[n]` types as needed or just pre-add `Arg1` to `Arg9` (or higher) and leave it at that for the first pass.

This could be directly exposed to the end user and it would keep the type syntax very regular, but it wouldn't match at all with how you define an actual function or reference it in an interface. Keeping these as similar to each other as possible will reduce the cognitive load on the developer, and keeping the functions (verbs) syntactically different from the types (nouns) is a good place to split.

Here's an example function to consider:

```ln
fn isSmall(s: string, threshold: int64): bool {
  return s.length() < threshold;
}
```

The declaration header of the function consists of the following parts:

```"fn" [name] ["("<argname>":" <argtype>["," ...]")"][":" <returnType>] <body>```

The elements in quotes are exact character matches, in brackets `[]` are optional and the elements in `<>` are required.

Function names aren't required if the function is being assigned to a variable or defined directly inside of a higher-order function call, otherwise they are required. If the function takes no arguments and returns no value (it is a 100% side effect function) then nothing but the function body is necessary. If there are arguments, they are defined inside of parenthesis, with each variable followed by a colon and its argument type declaration, then optionally a comma for the next argument. After this the return type is optional, declaring what kind of return value it produces. If not defined, it will be inferred, but it could also mean that it is `void` - nothing is returned.

Within an interface, it (currently) looks something like this:

```ln
interface MaybeSmallfry {
  isSmall(MaybeSmallfry, int64): bool,
}
```

Here the function name is *required* because you can't access the function as a method if you can't name it. Inside of the parenthesis are only the argument types because the argument names don't matter to those calling the function, just their location, and finally the return type is defined the same way at the end.

When defining a function type, the name of the function *and* its arguments are irrelevant, only the types are relevant, so it would be possible to define the function type of the function above as simply:

```ln
(string, int64): bool
```

However, for clarity and keeping the potential to add tuples to the language in the future, we're going with:

```ln
fn (string, int64): bool
```

And it's also proposed to add that to interfaces:

```ln
interface MaybeSmallfry {
  fn isSmall(MaybeSmallfry, int64): bool,
}
```

This is to keep the three function declarations as similar as possible and keep the syntactic confusion low.

### Alternatives Considered

Since internally function types will be using the Generics logic to work, we could have just exposed that directly with the function type for the example function being `Function<Arg2<string, int64>, bool>`. This was rejected because it would clash hard with the function signature and interface function type syntax, and there's no way to replace either syntax with something similar due to the rigidity of the generic type syntax (where would the function name and argument names go)?

Similar ideas to simplify the generic signature to `Function<Args<string, int64>, bool>` where `Args` is a special generic type that has an undefined number of subtypes was rejected because of the added complexity and extra behavior to generic types that only works on a singular built-in type that cannot be used by the user for their own types (limiting special carve-outs for the developers of the language helps maintain regularity and learning-by-example).

An even simpler generic form of `Function<string, int64, bool>` was rejected for similar reasons *and* being ambiguous in certain scenarios (if a function takes only one argument and returns nothing, or takes no arguments and returns a type, they would naturally be written the same way by the developer).

Smaller alternatives to the function type signature were also considered. Mentioned above is skipping the `fn` text, but rejected by boxing in our ability to add tuples to the language in the future.

Also considered was separating the return type with an equal sign `=` instead of a colon `:` so having a function type as an argument doesn't have a weird double-colon like this:

```ln
fn map(arr: Array<any>, mapper: fn(any, int64): anythingElse): Array<anythingElse>
```

and instead look like this:

```ln
fn map(arr: Array<any>, mapper: fn(any, int64) = anythingElse): Array<anythingElse>
```

but beyond just immediately having a different symbol for the return type immediately after the symbol for the inner argument's return type, and beyond being irregular, it also becomes ambiguous when defining a variable that's going to house a function that's also defined inline:

```ln
const someFunction: fn (int64) = fn ...
```

Is that `= fn` the beginning of the return type of the function, or is that the beginning of the function definition for a function that doesn't return anything?

It was also considered to leave the function type inside of interfaces alone, but this was rejected to minimize the number of differences between the three function signature syntaxes in the language, making the function type syntax a perfect subset of the interfaces' function type syntax, instead.

## Affected Components

The first stage of the compiler and the standard library must be updated. Most code written in Alan would also need an update, but since that's almost nothing, it's not so bad to do this right now. :)

## Expected Timeline

Once the RFC is approved, the first part of the work would be to convert the `function` type to a `Function<A, R>` generic type and create the `Arg[n]` generic types, and rework all of the code to use that, which is mostly ripping out band-aids throughout the first stage of the compiler to deal with the lack of solid function types. After that, updating the ANTLR grammar to handle the new type format and updating the standard library to use the new format is required.

These will likely be done in one PR to not churn the standard library twice, but may not be done if the band-aid deletion makes it too hard to follow what's going on with the new syntax.

In either case, these two pieces will rapidly follow one another and should only take 1-3 days to complete.

