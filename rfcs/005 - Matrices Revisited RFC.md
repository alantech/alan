# 005 - Matrices Revisited RFC

## Current Status

### Proposed

2020-07-27

### Accepted

YYYY-MM-DD

#### Approvers

- Luis de Pombo <luis@alantechnologies.com>

### Implementation

- [ ] Implemented:
  - [@std/matrix](tbd) YYYY-MM-DD
  - [@std/stats](tbd) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <david@alantechnologies.com>

## Summary

The majority of the conclusions in [RFC 003](./003 - Matrices, Vectors, Stats, and Trig RFC.md) are still valid, but matrices cannot simply be an array of arrays, as all of the inner arrays *must* have an equal length or the mathematics will not work out, so Matrices need a special type that enforces that requirement.

## Expected SemBDD Impact

If Alan was already at or above 1.0.0 *and* the original RFC had been implemented, this would have been a **major** change. As the original RFC hadn't yet been implemented, this is a **minor** change.

## Proposal

The original RFC proposed using an Array-of-Arrays syntax for Matrices:

```ln
const matrix = [ [ 1, 2, 3 ],
                 [ 4, 5, 6 ],
                 [ 7, 8, 9 ] ]
```

But the following is totally possible in that approach:

```ln
const matrix = [ [ 1, 2 ],
                 [ 3 ],
                 [ 4, 5, 6, 7, 8, 9 ] ]
```

and neither the compiler or runtime will tell you that anything is wrong.

I would love for the compiler to be able to reject invalid static matrices, but a runtime guard is still necessary for matrices generated on-the-fly, and the problem with static matrices should be quickly picked up by the developer when they get an error back instead of their matrix.

Reviving part of the originally proposed syntax, matrices could be defined this way:

```ln
const matrix = | [ 1, 2, 3 ]
               | [ 4, 5, 6 ]
               | [ 7, 8, 9 ]
```

Where the first `|` is a prefix operator that constructs a 1x3 matrix, then the second `|` is an infix operator that constructs a 2x3 matrix and the final produces a 3x3 matrix. The operators don't directly return an `Array<Array<int64>>`, but instead return a `Result<Array<Array<int64>>>` so if you try to do it incorrectly, you'll get an error condition instead of an array of arrays that you can't do matrix math on.

This doesn't have to be unwrapped in most cases as the matrix math functions can work on the Result-wrapped version. (And their outputs can themselves be result-wrapped if a math error occurs, anyways.)

### Alternatives Considered

Trying to make a special "Matrix" type wouldn't work as the inner contents of the type could be mutated after-the-fact so the matrix math functions still need to perform those checks. Therefore the `|` syntax is optional as you could still manually construct the array-of-arrays and pass them to the matrix math functions.

Not adding the special matrix-constructing operators/functions since the matrix math functions already need to safeguard against issues there was also considered, but being able to do a cheap check that the data you've loaded from a JSON blob (or whatever) is a valid matrix is worthwhile, and the extra work involved is minimal.

## Affected Components

This should only affect the standard library. No changes to the compiler or runtime should be necessary.

## Expected Timeline

The implementation of this RFC should take a few days, with most of the work on the matrix math functions, not the matrix constructor functions.

