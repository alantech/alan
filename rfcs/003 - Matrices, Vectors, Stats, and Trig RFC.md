# 003 - Matrices, Vectors, Stats, and Trig RFC

## Current Status

### Proposed

2020-07-01

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

The goals of clarity of expression and minimizing the downsides of modern language package managers while keeping their upsides pushes us towards a Python or Java style "Batteries Included" approach to the standard library. Therefore, we would want to have standard libraries for disciplines of math beyond basic arithmetic and boolean algebra, with the first to tackle being trigonometry, statistics, and matrix math (with vectors tagging along there). Different fields of mathematics use different annotations and operators, and sometimes they borrow operators and overload their meaning, which reduces code clarity for those not familiar with this. Separating these fields of mathematics out into their own standard libraries with their own annotations should give us the best of both worlds.

## Expected SemBDD Impact

If we were past the 1.0.0 mark, this would be a minor version update to the language.

## Proposal

This RFC came about because of the desire to be able to concisely represent a matrix in the source code seemed desirable, as Alan's automatic parallelization should be a good fit for matrix-based computation. Through an overloading of the `ternary` "operator"'s `:` operator, we found that we could concisely define a 1-D array simply by putting `:` in between the values of the array. Eg, the following Array definition:

```ln
const fibonacci = new Array<int64> [ 1, 1, 2, 3, 5, 8, 13 ]
```

could be represented more concisely as:

```ln
const fibonacci = 1:1:2:3:5:8:13
```

and the two could be combined for "Matrix-like" Array-of-Arrays:

```ln
const matrix = new Array<Array<int64>> [
  1 : 2 : 3,
  4 : 5 : 6,
  7 : 8 : 9,
]
```

which is much easier to type than the equivalent:

```ln
const matrix = new Array<Array<int64>> [
  new Array<int64> [ 1, 2, 3 ],
  new Array<int64> [ 4, 5, 6 ],
  new Array<int64> [ 7, 8, 9 ],
]
```

and has the added benefit of visually looking similar to column separators.

This did meet our purpose, but the vast majority of people would not understand that `value : value : value` is going to produce an array of 3 elements, and we don't like the idea of that behavior being in the root scope, as the root scope is considered the set of functions and operators that should be "obvious" to the developer (that they would be expected to memorize along with the grammar) and this did not feel like such a thing. It is undeniable, however, the advantage it brings to clearly and concisely defining matrices when that is what you want to do.

So, we want to isolate this behavior as an opt-in behavior. Placing it inside of a `@std/matrix` standard library with a `Matrix` interface that pulls in the `:` operator overloading and a `matrix` type defined as `type matrix<T> = Array<Array<T>>` and a `type vector<T> = Array<T>` alias would work, but at that point, it felt like we should also include some operator overloading for matrix addition, scalar multiplying, matrix and vector multiplying, and matrix and matrix multiplying, transposing, and etc.

And from there similar `@std/stats` for statistical methods and operators, and `@std/trig` for trigonometric methods would be similarly justifiable.

Other domains of mathematics, like Calculus, Diff Eq, etc, could come later with time following a similar pattern (but will each take much more effort, especially if symbolic manipulation is desired features of these implementations). These three seem most relevant to Alan's initial problem space, as tracking statistics on performance is often done in many large scale backends and reinventing that wheel is pointless, many "pedestrian" geospatial needs depend on trigonometry (and with it being isolated, we can go beyond classic sine, cosine, and tangent, and include full on haversine and other lesser-known trigonometric functions), and matrix-style computation is well-suited to our design.

### Alternatives Considered

We could take the Node approach of keeping a minimal core that is easy to learn and let the community build up these tools, but this was rejected as the primary reason why the Node ecosystem is currently disparaged despite the runtime being faster than any other interpreted language and having the easiest-to-use package manager application of any of the interpreted languages. When large outages can be caused by a single developer unpublishing a trivial one-liner package making up for a deficiency in the language itself, that makes the whole language and ecosystem less trustworthy, and only Javascript's preferred position in web browsers didn't cause a complete collapse in usage of Node.js after that debacle.

We could also have a narrower selection of math libraries, perhaps more closely matching the math libraries available in something like Rust or Java, and available at all times, and keep the syntactic sugar to a minimum, but that would be less welcoming to these other disciplines and would be unwelcoming towards them.

We could also have gone with a broader notion and include all of this in the root scope, but this makes the initial learning of the language much steeper than it has to be.

## Affected Components

The standard library will need new source files, and some of them will require new opcodes to be implemented (particularly for the trigonometric functions) in the runtimes and therefore compiler support for them, as well, but no architectural changes to the compiler or runtimes should be necessary.

## Expected Timeline

More precisely defining the contents of the three standard libraries should be done (we should be open to extending them over time with more useful features, but being reasonably feature complete is valuable. This will probably take a few days to nail down, though it can be done at the same time as the implementation, which should be straightforward and take a few hours more than the decision on what to include.
