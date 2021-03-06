# 014 - Rework IO Parallelization RFC

## Current Status

### Proposed

2021-03-05

### Accepted

2021-03-05

#### Approvers

- Colton Donnelly <colton@alantechnologies.com>
- Luis de Pombo <luis@alantechnologies.com>

### Implementation

- [x] Implemented: [Implement RFC 014, adding a `syncop` opcode and using it to force-linearize exec and fetch](https://github.com/alantech/alan/pull/437) 2021-03-05
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <david@alantechnologies.com>

## Summary

After [restoring IO parallelization once it was demonstrably correct](https://github.com/alantech/alan/pull/406) according to the semantics of the Alan language, we ran into a bug in [Anycloud](https://github.com/alantech/anycloud) because the code there made assumptions beyond those semantics more like a traditional imperative language. Since it took us, the creators of the language, a few hours to realize this mismatch, we believe that someone not already intimately familiar with Alan would not be able to figure this out and assume Alan is broken.

Considering that the intent of Alan is to unlock optimizations on top of "plainly written" code, this is essentially correct, and so we need to adjust the IO parallelization semantics to match, even if at the cost of losing out on some safe autoparallelization mechanisms.

## Expected SemBDD Impact

If Alan was 1.0.0, this would be a *major* change as the semantic meaning of existing code would change.

## Proposal

There is a large possibility space of potential changes that can be made here, as *everything* about the language syntax and the automatic IO parallelization efforts are on the table. In the interest of unblocking forward progress on Anycloud, there is a "quick fix" proposal and a parallel track to minimize the performance impact that the initial unblocking produces.

Consider the following trivial snippet of code:

```alan
exec('touch temp.txt');
exec('echo foo >> temp.txt');
exec('echo bar >> temp.txt');
exec('cat temp.txt').stdout.print();
```

In a traditional imperative language, this would print `foobar`, but in Alan, these four `exec` calls do not depend on each other, because they don't use any of the returned data from the previous `exec` calls. Therefore Alan will consider them IO operations that can be parallelized, and what will happen is up to the operating system itself. Likely the first will succeed while the remaining three fail to execute because `temp.txt` doesn't exist, yet, and this will print nothing (because it doesn't check `stderr` for text).

Even if we do something like:

```alan
exec('touch temp.txt');
wait(100);
exec('echo foo >> temp.txt');
...
```

will fail because the `wait` call itself is an "io" operation according to the AVM that it can group in parallel with these, so instead of waiting 100ms between each call (and taking a little over 300ms) it will run all 7 calls at once and wait only 100ms before it prints nothing and exits (because the stdout property access and passing it to the print function is a CPU operation and will come after the `exec` and `wait` calls it can group).

Simply put, the automatic parallelization is working against the developer's intuition.

It would work perfectly well if you're accessing independent sources of data, for instance:

```alan
const googleHomepage = get('https://google.com');
const yahooHomepage = get('https://yahoo.com');
```

rightly can be parallelized and reduce the concurrency impact because these two calls really have no hidden state under the hood. But, something like:

```alan
post('https://mywebsite.com/account/1/orders/new', '{"someOrder": "json"}');
const allOrders = get('https://mywebsite.com/account/1/orders');
```

would fail to include the newly added order, because it would start requesting all orders while it is trying to create the new order. There is nothing the compiler can do to determine if there is hidden state in these IO operations, so we **cannot** do anything but linear execution *unless* the developer provides us a hint that it is safe.

The Array methods have a better story on how to determine parallelization (or not). Function purity (whether or not it mutates an outer scope) will eventually automatically cause a `map` to be converted into a `mapLin` call to maintain consistent behavior while "pure" functions that have no external impacts can be executed in parallel.

The absolute simplest solution is to just make all IO operations linear by default and "bolt on" parallelization through arrays and maps, eg:

```
const homepages = ['https://google.com', 'https://yahoo.com'].map(get);
```

but this is only a small amount of syntactic sugar on top of idiomatic Javascript for the same:

```js
const googleHomepage = await (await fetch('https://google.com').text());
const yahooHomepage = await (await fetch('https://yahoo.com').text());
```

vs

```js
const homepages = await Promise.all(['https://google.com', 'https://yahoo.com'].map((url) => (await fetch(url)).json()));
```

There's some syntactic noise in JS, but not that much, and defining a `get` wrapper function eliminates most of it.

```js
const googleHomepage = await get('https://google.com');
const yahooHomepage = await get('https://yahoo.com');
```

```js
const homepages = await Promise.all(['https://google.com', 'https://yahoo.com'].map(get));
```

The third opcode type: `unpred_cpu!` gives us an out; if we had a version of every relevant IO opcode where you could opt in to the current semantics and another version that behaved like `unpred_cpu!` does that is the default, then we get cleaner code in both paths. A few considered suffix names for these: `Par`, like `execPar` based on `reducePar`, but where parallelization makes sense with reduce since there's guaranteed to be many calls, the potential advantages involved with calling `execPar` with only one `exec` usage (but perhaps other IO opcode uses that can batch with it) are unclear. `Async` was considered to potentially confuse people with Rust/C#/JS async/await and people would expect Promises/Futures they need to attach a handler or await on. So while not great, the current suffix under consideration is `Eager`. Eg,

```alan
const googleHomepage = get('https://google.com');
const yahooHomepage = get('https://yahoo.com');
```

vs:

```alan
const googleHomepage = getEager('https://google.com');
const yahooHomepage = getEager('https://yahoo.com');
```

No need to construct an array, push it through a map call, and then destructure the array afterwards (which we don't have syntactic sugar for so you'd have to do that manually) or unwrapping the `Result` type that it produces. Just immediately use the data you got and under the hood it executed those in parallel because you told it that it was safe to do so.

The implementation of these `Eager` functions is literally the existing implementation as-is, the new default that we expose is the alternative under the hood. We can create a higher-order `sync` method that takes the IO opcode as the first argument and the remaining arguments are passed into that IO opcode inside of an inner closure function, and the return of that IO opcode becomes the return of the `sync` method. If we had proper function types, we could even build this on top of `map`, but because we do not have this, a `syncop` opcode with special logic in `opcodes.ts` to take over the type data from the other opcode given to it will have to suffice. At first this will be done with a combination of alan code in the root scope and this opcode, while later this could be optimized into a set of behavior modification flags that the AVM follows to eliminate the closure generation and the extra work the AVM has to do when switching between them.

This `sync` method would be used internally in the standard library only to cause IO operations to behave synchronously by default with their current behavior exposed with the `fooEager` naming scheme.

### Alternatives Considered

*Lots* of alternatives have been considered:

1. Don't change anything, make users depend on prior steps within the code itself. This was rejected as too foreign to most developers, potentially not understandable at all to some, annoying, and potentially impossible in some situations.
2. Create a `sync { }` block syntax, sorta like Rust's `unsafe { }` block syntax that changes compiler behavior within the block. This was rejected as complicating the language, and Rust's complexity is likely the primary reason its adoption is not higher, because of the increased cognitive burden being too much for many. It also keeps the current behavior as the default and the user needs to know they have to opt into a different behavior with a syntax they have never seen before.
3. Create an `async { }` block syntax. This resolves issue of the "wrong" default being selected, but still has the issue of an increased syntactic and cognitive overhead, and lack of discoverability versus seeing a similarly-named method in their autocomplete dropdown.
4. Expose the `.sync()` method directly to users. This would let them turn any auto-parallelizing function into an ordered function that waits until completion before the next statement runs, which increases flexibility, but it is unclear when this would be applicable. It would also potentially confuse people into thinking they could do something like `fn () { asyncFn1(); asyncFn2(); }.sync();` and have those two async functions run synchronously, rather than what would actually happen, which is that they would run asynchronously with each other, but function calls before and after this would happen before and after this statement.
5. Expose an `.async()` method directly to users. This would set the default behavior to what people expect and allow people to opt in to an automatically parallelizable form, but besides the naming might confuse people with Promises/Futures, it is unclear *how* this would work, and in some cases it would be absolutely impossible to parallelize and seem to promise something that it can't deliver.
6. Drop all IO async behavior, only parallel event handlers and array methods can cause parallel IO. This would simplify the language and AVM quite a bit, but eliminates a large swath of advantages to the language versus Javascript. There would be little reason to use Alan at all versus an Erlang-like framework for Node.js, as parallelization would be almost as constrained as that, and the extra Turing completeness constraints on Alan make it a losing proposition in all cases except large array/matrix computations (which themselves are currently better served by GPGPU until the AVM sprouts GPGPU support).

## Affected Components

This will affect the standard library, compiler, and both runtimes to add the new opcode.

## Expected Timeline

The `syncop` version of this should be doable in half a day. Eliminating that with flags emitted by the compiler for the runtime(s) to consider will take a few days to a week, most likely.

