# 019 - Datastore Compute RFC

## Current Status

### Proposed

2021-07-01

### Accepted

YYYY-MM-DD

#### Approvers

- Luis De Pombo <luis@alantechnologies.com>
- Alejandro Guillen <alejandro@alantechnologies.com>
- Colton Donnelly <colton@alantechnologies.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <david@alantechnologies.com>

## Summary

Often we want to extract small amounts of data out of a larger pool of data. Pushing that work to the data rather than transferring the data to where the answer is desired and then computing it is more efficient from both a time and bandwidth perspective in many situations. This is also a parallelization strategy if the relevant data is spread across multiple nodes, and complements the event-level and array-level parallelization already present in Alan.

Adding the right remote compute primitive on top of datastore will allow us to build lots of data primitives on top efficiently. We should be able to build queues, B-Tree indexes, tables, MapReduce, etc with code that *looks like* single-threaded, single-process algorithmic tutorial code that is actually multi-server. Entire classes of external tooling simply fall away when you can express what you need from them cleanly in a few lines of code and don't have to deploy anything extra to support them! :)

## Expected SemBDD Impact

This would be a minor update if we were post-1.0 as it should have zero breaking impact on existing code.

## Proposal

When Alan has user-defined code passed into a built-in opcode, such as the `Array<T>`'s `map`, the inner function is compiled into AGA as a `closure for <uuid>` that temporarily takes over the outer scope's `HandlerMemory` and executes the inner function with the relevant arguments passed in on each run, then the output is placed into the `HandlerMemory` and it continues on. If `mapLin` was used, it runs sequentially and the inner function can mutate values in the outer scope safely (and is allowed to do so).

There is no theoretical reason why `map` must do its work on the same machine in the cluster. It could copy the `HandlerMemory` to any other node, do its work there, and then copy it back at the end. This simply doesn't make any sense from a performance perspective since you must spend the time to serialize, transmit, deserialize, compute, serialize, transmit, and finally deserialize, while staying on the same machine only requires creating a special child `HandlerMemory` associated with the original one, almost no overhead versus the compute step.

Keeping the data local reduces overhead and makes a lot of sense. But some data needs to be shared so decisions are consistent regardless of where it is asked to be computed. In that case, a set of fetch, serialize, deserialize, and compute are performed on each piece of data that needs to be brought from one node to another, with the reverse happening if the computation needs to be stored again. But if the end-result data after the computation is much smaller than the input data, then a lot of the data transmission overhead can be eliminated if the compute is moved to where the data is stored and only the result is transmitted back.

So, taking the `map` example from earlier, if there was an opcode that could be told:

1. the data to do computation on and
2. the code to execute

It could determine the location of the data and tell that node to perform the specified computation, and receive back the results to continue on from there.

Sometimes this computation would also need to mutate the remotely-stored data, and sometimes it would need to reference and/or mutate the data defined outside of the closure. That means the most general purpose opcode would be a `mutate_and_map` opcode that takes a datastore namespace-key pair and a closure function that is given a mutable reference to the data and can both return a value and mutate outer scope values.

But while the most general, this would have poor performance in several circumstances:

1. If it is given a mutable reference to the datastore key, the only way to safely trigger multiple of these operations at the same time is if it locks the key for the duration of this remote code execution.
2. Current closure scope handling does not know (or care) which outer-scope values are being mutated by the closure (or closure's closure, etc), so if the outer scope has a massive amount of data, even if not by this closure, it would have to copy back and forth.
3. If the remote execution is a "fire-and-forget" mutation on the remote data, the original node will pause execution until it is completed on the other side anyways. (This can be partially worked around through an intermediate data structure that can be more quickly updated, but not eliminated with the proposed opcode.

The first point can be avoided if you can specify whether or not you want a mutable reference to the remote data, the second point if you can specify whether or not you want to use closure semantics or "pure function" semantics (or something in-between), and the third can be avoided if you can specify whether or not you want a return value. With a sufficiently-advanced compiler you could have that all automatically inferred from the way it is being used (is the output assigned to a variable, used in a method chain, operator, etc, or not; is the input argument to the function ever mutated, are any outer-scope variables accessed) but for the first pass, it is proposed that we tackle this with separate explicit opcodes. This is useful even if/when automatic performance improvements to the most-general opcode are handled by the compiler as:

1. It could just emit those well-defined, tighter-bound opcodes and this logic doesn't need to live in the AVM at runtime.
2. Developers could explicitly opt-in to tighter constraints in "hotter" paths of distributed computing to prevent other developers (or their future self) from accidentally ratcheting down the runtime performance.

Stepping up to the higher-level API design:

```ln
const baz = ns('someNamespace').ref('foo').run(fn (foo: string) = foo.length());
```

```ln
let bar = 'bar';
const baz = ns('someNamespace').ref('foo').closure(fn (foo: string) {
  bar = 'foobar: ' + foo + bar;
  return foo.length();
});
```

```ln
const bar = 'bar';
const baz = ns('someNamespace').ref('foo').with(bar).run(fn (bar: string, foo: string) = #bar + #foo);
```

```ln
const baz = ns('someNamespace').mut('foo').run(fn (foo: string) {
  foo = foo + 'foo';
  return foo.length();
});
```

```ln
let bar = 'bar';
const baz = ns('someNamespace').mut('foo').closure(fn (foo: string) {
  foo = foo + bar;
  bar = bar * foo.length();
  return bar.length();
});
```

```ln
const bar = 'bar';
const baz = ns('someNamespace').mut('foo').with(bar).run(fn (bar: string, foo: string) {
  foo = foo * #bar;
  return foo.length();
});
```

```ln
ns('someNamespace').mut('foo').mutOnly(fn (foo: string) {
  foo = foo + foo;
});
```

```ln
const bar = 'bar';
ns('someNamespace').mut('foo').with(bar).mutOnly(fn (bar: string, foo: string) {
  foo = foo + bar;
});
```

All eight potential remote compute possibilities are handled with this API. (Technically the ninth where it's "mutation only" but also mutates the outer scope could exist, but it's identical in performance to `ns().mut().closure()` and so it was elided.)

From a given namespace you grab either a read-only `ref`erence or a read-write `mut`able reference, then you can either `run` a pure function that operates on the data and returns some sort of summary from it, or you can execute a full `closure` that has access to all outer-scope variables defined, or you can go in between by `run`ning it `with` one variable passed only. If you got a `mut`able reference, you could say it's an external side-effect by declaring it `mutOnly`, that may or may not be `with` one variable passed to it.

For the types of execution that return a value, the functions all return the specified type `T`, but the actual variable that comes back is `Result<T>` in case there is an issue with the execution (such as the data doesn't exist, or it gives up trying to execute the specified function for some reason).

With these primitives, we can easily build a queue API:

```ln
fn createQueue(queueName: string) {
  // Taking advantage of the current type unsafety with datastore
  ns('queue').set(queueName, new Array<int64> []);
}

fn enqueue(queueName: string, payload: any) {
  ns('queue').mut(queueName).with(payload).mutOnly(fn (payload: any, queue: Array<any>) {
    queue = [payload] + queue;
  });
}

fn dequeue(queueName: string, default: any): any = ns('queue')
  .mut(queueName)
  .with(default)
  .run(fn (default: any, queue: Array<any>): any = queue.pop() || default) || default;
```

If enqueuing performance isn't a problem, you could also merge the `createQueue` function into the `enqueue` function behind a conditional on the datastore `has` method.

This could then be trivially used in a work processing pool that accepts jobs by http server:

```ln
from @std/app import start
from @std/httpserver import connection, Connection, body, send
import myQueue

event checkForWork: void

on start {
  emit checkForWork;
}

on connection fn (conn: Connection) {
  myQueue.enqueue('work', conn.req.body);
  const res = conn.res;
  res.body('ok').send();
}

on checkForWork {
  const work = myQueue.dequeue('work', 'no work');
  if work != 'no work' {
    // Do the work
  }
  wait(100); // Don't spin *too* hard if no work is available
  emit checkForWork;
}
```

We just created Celery!

With remote mutations of arrays like this, we could also insert in a sorted order if the type is `Orderable`. Couple that with a `KeyVal` such that the `Orderable` is the `Key` and the `Val` is the name of another key in datastore, and we have a B-Tree to a row of data (a struct) stored in the cluster. If we want to get all records in a range, we could do a `ref` call on the index, split the range on the sorted array and then `map` that array to the specified inner keys and return that new array. That gives us a `select * from <table> where <column> in <range>`, where the datastore key for each row is like a UUID index. We now have a SQL-like table that can scale near-infinitely (as large as you can scale up the cluster, at least).

You can also do any relevant transform on this data in that function without needing to bring it back to map, so if you're computing something somewhat complex (say, mean and standard deviation of the haversine distance of a collection of points from the bounding-box center) you don't have to load that data into Postgres, install the PostGIS extension, and then wrangle with implementing that imperative algorithm in the declarative SQL syntax, you can just perform that computation right there and simply return the resulting pair of values. Now you have made it even more efficient and kept the transform clean and clear, no need for MapReduce.

Streaming some keys out to disk when they aren't recently used can further expand the quasi-infinite size of the table, and etc. These things can be built up over time as libraries in the language and be quite efficient while also clearly written with explicit performance trade-offs.

### Alternatives Considered

The primary alternative considered is to keep data and remote computing outsourced to databases, but then all of the problems of scaling and interfacing with them are left up to you, and the advantages of the language can be mostly lost if the auto-parallelization 4x's the performance of the part of your service call that takes only 1% of the time.

Adding more "complete" database/queue/etc functionality as separate tools not built on datastore was also considered, as they may be slightly more performant, but then the complexity of the virtual machine goes up much higher, and changes/improvements to it would likely cause breaking changes to the language. They would also be less extensible/mergeable as explicitly separated concepts, which would reduce the amount of experimentation that can be performed when the boundaries are blurred.

Finally, it was considered to only implement one opcode that's equivalent to `ns().mut().closure()` and try to automatically deduce what optimizations could be performed, but that was eliminated for practical reasons (it will take a while for the compiler to get that smart, and users will want to be able to know what those optimizations will be so they don't accidentally regress on performance).

## Affected Components

This will mostly affect the standard library and runtimes, with a little bit of plumbing in the compiler to support the new opcodes.

## Expected Timeline

This should probably only take about a week. :D
