# 004 - Runtime Error Elimination RFC

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

There are already many high-level advantages to programming in `alan`: IO concurrency and compute parallelism are handled for you automatically, every function is guaranteed to halt, automatic memory management without a GC, and all of this with a syntax that is close to existing dynamically-typed languages. However, while we guarantee halting, sometimes your code may halt before you'd want it to.

We can't do anything about OOM errors (if you try to allocate a terabyte-sized array, I hope you have a terabyte of RAM...), but other errors due to things like array out-of-bounds errors, division-by-zero or overflow/underflow errors, etc we could prevent from occuring and force the developer to handle these cases in the language itself through the type system. This could be done with minimal actual effort in the user's code with the right opcode design.

## Expected SemBDD Impact

If `alan` was already `1.0.0` this would be a `major` update as it would break almost all existing code with changes to many opcodes and syntactic features in the language.

## Proposal

First, we need to list all of the possible runtime errors and what parts of the language they affect.

* Out of Memory - A simple reality of running anything on a computer that is only an approximation of a Turing machine. We can improve things here by finding more efficient ways to represent data in memory, but there's nothing we can do to ever fully fix this one. It's up to the developer to not write code that will exceed the limits of their computer.
* [Memory Leaks](https://en.wikipedia.org/wiki/Memory_leak) - Implementation issue with the runtimes. Shouldn't have an impact on running code except eventually causing an OOM if left unchecked for too long. Nothing for the developer to handle here.
* [Null pointer](https://en.wikipedia.org/wiki/Null_pointer) - There are no pointers in `alan` so running into one is a compiler or runtime error and should be treated as a bug. Nothing for the developer to handle here.
* [Stack Overflow](https://en.wikipedia.org/wiki/Stack_overflow) - Should not be possible due to `alan`'s aggressive inlining and the runtime implementation not using a stack to represent handler calls at all, but there could be implementation issues within the runtime itself that could cause this, but there's nothing for the developer to do here except report the issue in the runtime itself.
* [Buffer Overflow](https://en.wikipedia.org/wiki/Buffer_overflow) - Accessing beyond the valid range of a string, array, or user type. Would be considered a compiler error for strings and user types, but possible for users to trigger this with the array accessor syntax.
* [Integer Overflow](https://en.wikipedia.org/wiki/Integer_overflow) and [Division by Zero](https://en.wikipedia.org/wiki/Division_by_zero) - Numeric calculation issues that can cause crashes on integer operations (floating point has `NaN` to represent these failure cases). This can be triggered by basic math operations and would require work to be done by the user to correct for it.
* IO Errors - "IO" is an operating system construct for many different things, including filesystem access, tcp/udp access, etc. For `stdin`, `stdout`, and `stderr` any errors involving these are to be handled by the runtime as they are exposed as events to the developer and there is nothing for them to do. For other forms, the APIs in `alan` must expose these errors to the user for them to handle appropriately.

Of these runtime errors, the last half are ones that affect the end users *and* there's something that can be done about them.

The proposed solution at a high level is simple: return `Result` types for all three of them (Buffer Overflow, Integer Overflow/Div-by-Zero, and IO Errors) which will cause the compiler to force the user to unwrap the results to use them.

Let's start with the simplest: IO Errors. All that is necessary here is to make sure any fallible API returns its value wrapped in a `Result` and that's it. The compiler will force the developer to check for the potential for an error and unwrap it, or replace the value with a default and drop the error otherwise. Sometimes the latter is fine, sometimes not, but now instead of being a runtime error, it would be a [Logic Error](https://en.wikipedia.org/wiki/Logic_error) and there's nothing we can really do to prevent that from occurring -- that's up to the developer to understand what their program is required to do and to accomplish it.

Next, for array accessor syntax (involving arrays and hashmaps) from the user's perspective that's simply going from:

```ln
const someValue = someArray[index]
```

to:

```ln
const someValue = someArray[index] || defaultValue
```

Or in a situation where method chaining is occuring:

```ln
someArray[index].getOr(defaultValue).doSomething()
```

The compiler would treat array accesses as returning `Result<T>` instead of `T` from an `Array<T>`, and then complain if the right thing isn't done.

The complication starts here, though: Currently `Result<T>` is a user type built on top of `Maybe<T>` and `Maybe<T>` is a user type built on top of `Array<T>`, so how could that `Result<T>` unwrap its value if that itself is done with an array access that would need unwrapping?

The answer is you can't. So `Result` and `Maybe` need to be turned into special built-in types the compiler and runtimes are aware of and aren't dependent on the `Array` type in any way. Once that's done, though, array accessor syntax is safe (and with `Result` being a built-in type, the performance impact may be much smaller than expected, though the runtime error guarding *will* introduce a performance impact).

Finally let's look at Integer Overflow and Divide by Zero errors. All of the integer arithmetic operations (excepting modulus) are impacted by one of the two (adding, subtracting, multiplying, and exponentiating can Integer Overflow/Underflow, while divide can Divide-by-Zero), which means that all of these should wrap their integer results into `Result` objects, but needing to unwrap after each operation would make integer math equations ridiculously convoluted!

```ln
1 + 2 * 3 ** 4
```

would need to be written something like:

```ln
(1 + (2 * ((3 ** 4) || -1) || -1)) || -1
```

However, if the integer arithmetic opcodes could take `Result` wrapped integers as well as raw integers, then you could write a simpler:

```ln
1 + 2 * 3 ** 4 || -1
```

This also makes the end-result easier to be sure of, as a failure anywhere in the chain would "bubble up" and cause the `-1` result, while if the failure happened deep in the earlier example (say at `2 * (3 ** 4)`) but not anywhere else, instead of `-1` you'd get `0` as `1 + -1 == 0`.

With respect to the implementation of this, the `add` function for `int64`, for instance, would become 4 functions:

```ln
fn add(a: int64, b: int64) = addi64(some(a), some(b))
fn add(a: Result<int64>, b: int64) = addi64(a, some(b))
fn add(a: int64, b: Result<int64>) = addi64(some(a), b))
fn add(a: Result<int64>, b: Result<int64>) = addi64(a, b)
```

Not super-complicated, though breaking existing behavior (due to the new return type of `Result<int64>`).

However, there's a question about how to handle the floating point calculations. Floating point representation already includes roughly this behavior -- overflows get turned into `Infinity`, underflows into `-Infinity`, and divide-by-zero becomes one of the two (depending on the signs of the two numbers), with the special case `0.0 / 0.0` becoming `NaN`. Should floating-point calculations get an easier and faster representation than integers because of this built-in error handling representation in the standard?

And what about following this "saturating" approach to under/overflows and divide-by-zero for integers instead? Then `1 + INT_MAX == INT_MAX` and `1 / 0 == INT_MAX` while `INT_MIN - 1 == INT_MIN` and `-1 / 0 == INT_MIN`. No need for wrapping things in results if there's a check for these issues (which would have to be done for the `Result` approach, as well).

The issue with that is `INT_MAX` is just a number with integers, not a special `Infinity` representation that's separate from the other numeric representations in floating point numbers, and it can't solve `0 / 0` without picking an `INT_MAX` or `INT_MIN` arbitrarily. You also can't tell if you're *legitimately* at `INT_MAX` or `INT_MIN` or if you hit saturation and stayed there, while the `Result` approach is very explicit about what kind of error you ran into, which can help you actually track down the logic error that got you there in the first place.

In the other direction, if we have this behavior of returning a `Result` with a clear `Error` in integer math, why don't we have that with floating point math? Detecting the error and returning a meaningful error message on how you got there instead of a bare "NaN" can also help with tracking down *what* part of the entire chain of computation had the unexpected source of the "corrupting" floating point value.

But besides the potential performance impact in that direction, as well, sometimes you really *do* need to work with those special floating point values in your own code, such as parsing them from a JSON blob.

Due to all of this, while it is more stuff to memorize, I propose having `integer-style` and `floating-point-style` arithmetic functions and operators for both integers and floating point numbers. This way, the user can decide if they want `integer-style` wrapped results that may have an error message, or `floating-point-style` saturated results with no error messaging. The question now is which one gets the "normal" operators and which gets "abnormal" operators.

I would propose that the `floating-point-style` is less correct -- it's exceedingly unlikely that the numbers you multiplied together (for instance) *actually* would result in the output being `Infinity` and far more likely that `Infinity` just means some finite number that simply can't be represented in 64-bits. Because of that, I would give `+`, `-`, `*`, `/` to the `integer-style` functions that return `Result`-wrapped outputs, and I would give new operators to the `floating-point-style`. I propose these new operators borrow the `.` from the `floating-point` itself, producing the operators `+.`, `-.`, `*.`, and [`/.`](https://slashdot.org/).

The final piece to consider is how these two sets of operators interact with each other. I propose that no special work is done to make them intercompatible -- the `integer-style` operators can deterministically wrap the numeric types into `Result` objects and work with them, but the `floating-point-style` operators cannot do the reverse correctly, so trying to use `/.` on a `Result`-typed value is a compile error and it would need to be unwrapped by the developer with a default value (or have the error handled appropriately).

### Alternatives Considered

We could simply have the alternative arithmetic opcodes work as the current ones to and cause a runtime error if there is an integer overflow/underflow or divide-by-zero instead of imposing any sort of runtime performance penalty with the "saturation" model. We could have also just not done any of the changes proposed to array accesses, too, since we're already bringing a lot to the table. But, [we do what we must, because we can](https://images-wixmp-ed30a86b8c4ca887773594c2.wixmp.com/f/47a4b452-c867-4d13-99e0-c9971120f09b/d8bvmla-58a714aa-4547-494d-a766-36e8187c4fca.png/v1/fill/w_1024,h_768,q_75,strp/we_do_what_we_must____because_we_can_by_princesstwilight1-d8bvmla.png?token=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJ1cm46YXBwOjdlMGQxODg5ODIyNjQzNzNhNWYwZDQxNWVhMGQyNmUwIiwic3ViIjoidXJuOmFwcDo3ZTBkMTg4OTgyMjY0MzczYTVmMGQ0MTVlYTBkMjZlMCIsImF1ZCI6WyJ1cm46c2VydmljZTppbWFnZS5vcGVyYXRpb25zIl0sIm9iaiI6W1t7InBhdGgiOiIvZi80N2E0YjQ1Mi1jODY3LTRkMTMtOTllMC1jOTk3MTEyMGYwOWIvZDhidm1sYS01OGE3MTRhYS00NTQ3LTQ5NGQtYTc2Ni0zNmU4MTg3YzRmY2EucG5nIiwid2lkdGgiOiI8PTEwMjQiLCJoZWlnaHQiOiI8PTc2OCJ9XV19.1kXuChTJw6OoM8XAhLk-lH6Div9oPh-bUBXJRlAgO7U).

Both of these were rejected because developers that need to worry about that level of performance optimization are not likely to relinquish control over multithreading and underlying data structures to `alan`, but developers that are looking to develop backend services that scale pretty well, are easy to understand, and can't tolerate bugs are more likely to appreciate these trade-offs in the language design. Code can be written in a simple imperative style, it'll take advantage of all of the cores on the server, and it'll prevent you from writing buggy code without too much cognitive overhead (especially if most of the built-in functions that take integers also sprout support for working with `Result`-wrapped integers and they rarely have to explicitly unwrap them).

## Affected Components

This will affect every layer of the compiler and runtime stack and would affect anything written in the language. It's a huge change, so we should take care of it quickly.

## Expected Timeline

This is a pretty beefy change. First we need several days (up to a week, but likely not) creating native versions of `Maybe`, `Result`, and `Error` (and probably `Either` though that one is not strictly necessary for this). After that is several more days implementing all of the new opcodes, writing the new standard library root scope to bind them, and updating the tests to cover them and the new corner cases they guard against.

The total time for this is likely half a month since we're often multiplexing other changes at the same time.

