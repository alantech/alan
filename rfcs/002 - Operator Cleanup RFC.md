# 002 - Operator Cleanup RFC

## Current Status

### Proposed

2020-06-03

### Accepted

YYYY-MM-DD

#### Approvers

- Luis de Pombo <luis@alantechnologies.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author

- David Ellis <david@alantechnologies.com>

## Summary

The operator declaration syntax is awkward, hard to understand, and referencing functionality that ended up not being used at all. This should be fixed before launch.

## Expected SemBDD Impact

If we were 1.0.0 or greater, this would be a major version bump since it would break all code, including the standard library, that declares operators. However, since we are pre-release and pre-1.0.0, breaking changes are fine. (We don't even have the code to auto-version based on the Semantic BDD tests, yet.)

## Proposal

The current syntax is defined as follows:

```
infix commutative associative + 2 add

infix / 3 div

prefix - 2 negate
```

First you specify if the new operator is going to be an infix or a prefix operator. After this, if it is infix, you can optionally add `commutative` or `associative` keywords to indicate the infix operator may be commutative and/or associative, or none of the above. Following that in both paths is the symbol to be used for the operator, then the operator precedence level, and finally the name of the function this operator is aliasing.

There can also be apparently duplicative lines allowing multiple functions to be bound to the same operator symbol, the following is valid (and done by the root scope):

```
export infix commutative associative + 2 add
export infix associative + 2 concat
```

Here we bind two functions to `+`, `add` and `concat`. They both have the same precedence so the resolution of the functions themselves is the first function that matches the arguments surrounding the infix operator, with the `add` functions checked before the `concat` functions.

The `concat` function does not declare that it is commutative, only associative (which is true, `"foo" + "bar" != "bar" + "foo"` while `"foo" + "bar" + "baz" == "foo" + ("bar" + "baz")`) but the compiler does absolutely nothing with this information. It was intended to eventually use metadata like this to determine safe transforms of code to produce faster equivalents for the user, particularly in conjunction with array `map`, `reduce`, etc operations where the potential boost could matter.

However, there are several problems with this:

1. The annotation is only on the operators, not the functions, when it is really a propery of the function the operator is an alias of.
2. It is very easy to make a mistake with the commutative and associative declarations and cause the compiler to make unsafe transforms of your code.
3. Many optimizations also require defining a distributive property between multiple functions, which would need another syntax layered on top.
4. If the distributive rule could work, but the output type changes in one of the two functions, an alternative form that flips the function changing the type is necessary and would need to be written by the developer. If both functions change the output type, a 2x2 matrix of related functions is required of the developer to allow automatic transforms.

The first and primary issue is the annotation was placed on the wrong layer of abstraction. It should be eliminated from the operators. The question is whether or not it is moved to the two-argument functions defining a special syntax for them.

Because it cannot be proven that the declarations from the developer are actually true without either exhaustive testing or [defining a formal proof of the behavior](https://coq.inria.fr/), where the former would cause compilation times to skyrocket and require all such functions to have no (consequential) side effects, while the latter would place a significant burden on the developer, it does not seem reasonable to maintain these annotations if they cannot be acted upon correctly.

For our initial launch, the only parallelization will be done in the array operators, and the only array operator that this theory applies to is `reduce`. Impure functions that mutate the outer state would be automatically run sequentially to make sure those mutations occur in the proper order. If the provided function is pure, it would be possible to potentially parallelize it, with the associative-style parallelization being effective for both associative and commutative+associative functions as the number of items in an array to be parallelized will far outweigh the number of CPU cores the array is distributed across.

The simplest solution here is to create a `reducePar` operator that uses the parallel algorithm while the default `reduce` uses the sequential algorithm, but this requires the developer to pay attention to this optimization problem. The `reduce` function could also "spot check" parallelization when possible by executing the reduce in parallel any time the underlying function is pure, but also testing the result by computing the second chunk twice, from both the second thread independently as well as continuing off of the first chunk in the first thread, then comparing the results of combining the first chunk with the second chunk versus the sequential computation of the first chunk through the second chunk -- if they are equal, it is "truly" pure and can be flagged that the test is not necessary anymore, while if they are not equal, it can be flagged "impure" and fall back to the sequential algorithm. This would produce a small cost only on the first run of a reduce that triggered parallelization (by being run on an array deemed "worth" parallelizing), but could produce a false positive of being safely parallelizable when it is not if by chance the test passes.

In any case, this implies that the `commutative` and `associative` keywords should be eliminated from the operators, reducing the syntax to:

```
infix + 2 add
prefix - 2 negate
```

This is better, but it is still very unclear what is going on. It may as well be a special "function" that the compiler uses:

```
infix("+", 2, add)
prefix("-", 2, negate)
```

But functions were avoided because it is not possible to allow the creation of new operators at runtime, similar to the `on` and `event` syntaxes being added to force event loop declarations to be compile-time only constructs that the runtime can rely on, making operators compile-time only constructs allows the compiler to decompose the operators back into the original function calls and serialize out a fixed dependency graph of opcode calls.

The syntax for these various compile-time module-level constructs has tended to be "pythonic" (particularly the import syntax) and readable, but the operator declaration syntax, even with the simplification, is not.

The final part of this proposal is to "python-ify" the syntax with use (and reuse) of "glue" keywords. This would clarify the statements and allow a swappable order.

```
infix add as + precedence 2
infix precedence 2 concat as +
prefix negate as - precendence 2
```

With this proposal `infix` and `prefix` remain the root keywords that indicate an operator is being declared. The `precedence` keyword is added to indicate the following number is the precedence level and can either immediately follow the `infix`/`prefix` keyword or come at the end. The `as` keyword is reused in this context to indicate the "aliasing" of the function specified as the operator specified, mimicking the use of the `as` keyword for renaming things imported from another scope.

### Alternatives Considered

1. Simply removing only the `commutative` and `associative` keywords and otherwise leaving it alone. It would work, but clarity would be low and you would *have* to read the documentation to have any idea what they are doing.
2. Convert them to functions and compile-time error their use inside of functions. It would work and reduce the set of keywords in the language, but it would also special-case certain behaviors and what should the language even *do* if you define a function yourself named "infix" or "prefix"?
3. Drop custom operators, bake the mapping into the compiler. Many languages do this, but it does limit the expressive power given to the user and the idea that everything in the language is just a noun (type) or verb (function) with all alternative representations tools to make it easier for the user to express themselves and therefore clearer when reading the language later.

## Affected Components

This affects just the compiler. The LN grammar file must be updated and regenerated, and the lntoamm compiler stage must be updated to use that new grammar, but that's it.

## Expected Timeline

Implementation should be done in just a day.

