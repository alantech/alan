# 015 - TCP RFC

## Current Status

### Proposed

2021-03-24

### Accepted

YYYY-MM-DD

#### Approvers

- Luis De Pombo <luis@alantechnologies.com>
- Colton Donnely <colton@alantechnologies.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <david@alantechnologies.com>

## Summary

The current `@std/http` in Alan is perfectly serviceable for "normal" web servers that consume a request in its entirety, then provide a response in its entirety, and then close the connection. But if the server was to drip-feed data back to the client, or wanted to upgrade into a two-way websocket connection, this is impossible. Adding support for arbitrary TCP messaging to Alan solves this for pass-through and provides a foundation for a fully-Alan HTTP implementation. It also stretches the language with its static event system, but not to the breaking point.

## Expected SemBDD Impact

This would be a minor update if Alan was post-1.0.0 as it adds new functionality without regressing on existing functionality.

## Proposal

A TCP connection has four essential states: open, closed-on-purpose, closed-on-error, and closed-by-timeout. While in the open state, data can be read from its output, written to its input, or explicitly closed as triggers by the user, while it can also unexpectedly transition to the other two closed states. The amount of data received at any particular point in time is completely arbitrary and up to both the originating machine and the local kernel, which makes it a good fit for the event system so the users' code is prompted when each chunk is ready to process.

The main issue is that Alan's event system is intentionally a static event system determined at compile time, so it is impossible to add new event handlers for each new socket that is opened by the users' code or has been received by the TCP server. The secondary issue is that the AVM's internal memory model would require an expensive conversion of that chunk from packed bytes to an internal `Array<int8>` type which, to simplify the memory model, is actually really an `Array<int64>` under the hood, which is very wasteful for a true byte array, so being able to avoid that conversion when possible would be good. Finally, the ergonomics of the API need to be both easy-to-understand and mesh well with the existing syntax while adding stream processing support at the same time.

While changes to the language are on the table (and some of the alternatives will cover a few of them), the best overall solution (considering implementation complexity and cognitive complexity) doesn't need it. :)

The clearest way to introduce the new syntax is to simply use it in an example similar to how it will be used by Anycloud:

```ln
from @std/tcpserver import connection
from @std/tcp import Connection, connect, addContext, chunk, Chunk, Context, read, write, connClose, close, connError, connTimeout

on connection fn (conn: Connection) {
  const tunnel = connect('localhost', 8088);
  conn.addContext(tunnel);
  tunnel.addContext(conn);
}

on chunk fn (ctx: Context<Connection>) {
  const c: Chunk = ctx.conn.read();
  ctx.context.write(c);
}

on connClose fn (ctx: Context<Connection>) {
  ctx.context.close();
}

on connError fn (ctx: Context<Connection>) {
  ctx.context.close();
}

on connTimeout fn (ctx: Context<Connection>) {
  ctx.context.close();
}
```

The TCP server is a separate module for the same reason as the http server. The presence of a TCP listener would trigger the AVM to automatically start listening to an appropriate port. (Whether or not that autoselection sticks around long-term is still up in the air.)

The connection handler in this case triggers the construction of a TCP client connection to the specified host and port. Once constructed, the server's connection and the client connection are added to each other's contexts.

And that's it. Once constructed, the AVM will keep it resident and emit events involving that connection. Those events get the `Context<T>` type, where the `T` is the type that was added to the context. One known vulnerability with this as-is is that the client and server context types need to match each other and the compiler will not be able to keep this completely safe. This could potentially be avoided by having completely separate events for client and server, but that also nearly doubles the size of the codebase so it is being avoided for now. This can also be worked around with an `Either<ServerType, ClientType>` that is checked by the handlers which is why this is not considered a huge problem.

The `Context<T>` type has two properties, the `conn: Connection` that corresponds to the connection that the triggered the event, and the `context: T` property that is whatever value was attached to the connection, which in this case is another connection.

The `chunk` event is triggered when a "chunk" of data has arrived. The implementation of the `Chunk` type is an opaque type that is essentially a pointer to the actual chunk of data that has arrived. The `read` function returns that chunk, which is guaranteed to exist when the event is triggered. A follow-up read or a read out of the chunk event should return an empty chunk. For normal Alan applications that want to actually manipulate the Chunk data, `toInt8Array` and `toAsciiString` methods would exist to provide parseable data. ASCII instead of UTF-8 because a chunk could split a multi-byte UTF-8 char. There should be an `Array<int8>` `toResultString` implementation that converts UTF-8, if possible, though, so repeatedly-appended chunks could be stream processed with a second layer of event logic (or all-at-once on connection close).

The chunk handler in our case just takes the `Chunk` and immediately writes it to the connection stored in the `context` field. This nice and tidily joins the two streams to each other in 4 lines of code.

The remaining three events deal with event closure. I am tempted to make it just one event with a helper function to get the reason for the closing, but that might encourage ignoring errors, so I'm not sure.

All of these do the same thing: close out the other end when the other closes. This looks like it would technically infinitely loop on `connClose` events as they both call each others' `close` method over and over again, but it won't be a loop if trying to close an already closed event does nothing. This makes it impossible to write "crashy" or "loopy" code involving these streams, so doing so even if it makes the AVM implementation more complicated is the way to go here.

The `Context<T>` type makes this structure possible, as the various events can "share" state between one another. That state is copied to each event, not actually shared, but it can be atomically updated by re-calling `addContext` on the `conn` property, allowing actual mutable state to be passed between the handlers, though it is a synchronization point.

To re-emphasize, the biggest flaw of this approach is that the `T` in `Context<T>` is enforced poorly by the compiler. The `addContext` function will take `any` value, and it's up to the user to make sure the `T` they add to their listeners matches the `T` they set it with.

### Alternatives Considered

Many different alternatives were considered from a wide variety of angles.

#### Tunnel-only TCP

Just allow tunneling a TCP connection to a specified server:

```ln
from @std/tcp import connection, Connection, tunnel

on connection fn (conn: Connection) {
  conn.tunnel('localhost', 8088);
}
```

Super simple and solves the Anycloud problem, but absolutely useless outside of that context. Not worth it, doesn't build towards the future.

#### Pythonic TCP

The first considered approach was to keep the TCP connection very similar to the existing HTTP connection -- apparently blocking, even if implemented with futures under the hood. Something like this:

```ln
from @std/seq import seq, while
from @std/tcpserver import connection, Connection
from @std/tcp import connect, chunk, Chunk, read, write, close, isOpen

on connection fn (conn: Connection) {
  const tunnel = connect('localhost', 8088);
  seq(1000000000).while(fn = conn.isOpen() && tunnel.isOpen(), fn {
    const connChunk = conn.read();
    tunnel.write(connChunk);
    const tunnelChunk = tunnel.read();
    conn.write(tunnelChunk);
  });
  if conn.isOpen() {
    conn.close();
  }
  if tunnel.isOpen() {
    tunnel.close();
  }
}
```

This approach simply enters a quasi-infinite loop, reads a chunk of data from one socket to write into the other, then reads a chunk of data from the second socket to write into the first, and repeats until one or both of the connections close and then closes the other.

It is actually less lines of code, but it has several drawbacks:

1. `@std/seq` is required. If not configured "correctly" by the end user, it could terminate the loop even while the connection is validly going because the `Seq` object has run out of iterations allowed.
2. Latency is introduced depending on the behavior of the `read` function here. If it waits until something has arrived, it could be stuck forever if logically that connection is waiting for data from the other connection, so it has to eventually time out and return an empty "chunk". But if the wait until it times out is too short, the loop will burn a lot of unnecessary CPU polling.
3. The "ceremony" around the core loop logic can be easily messed up, but it cannot be auto-generated by the compiler. That you need to check the connection state of both connections and exit if either of the two has closed is required, and if you forget the conditionals at the end to close out the other connection still open you'll have lots of dangling sockets on the process that could eventually choke the server of file descriptors.

#### Node-like TCP

The next considered approach was to work similarly to how the TCP event system in Node.js works:

```ln
from @std/tcpserver import connection, Connection
from @std/tcp import connect, onChunk, Chunk, write, onClose, close, onError, onTimeout

on connection fn (conn: Connection) {
  const tunnel = connect('localhost', 8088);
  conn.onChunk(fn (c: Chunk) {
    tunnel.write(c);
  });
  tunnel.onChunk(fn (c: Chunk) {
    conn.write(c);
  });
  conn.onClose(fn = tunnel.close());
  tunnel.onClose(fn = conn.close());
  conn.onError(fn = tunnel.close());
  tunnel.onError(fn = conn.close());
  conn.onTimeout(fn = tunnel.close());
  tunnel.onTimeout(fn = conn.close());
}
```

This appears to solve the issues with the Python-style, but a few things arise from it:

1. We have two event systems effectively bolted on top of each other now. It's weird and doesn't mesh well with the language.
2. Even ignoring that, there's some ugly trade-offs that have to be made. These closure functions either actually share the same `HandlerMemory` so they still block each other, which is *alright* but puts some upper limits on throughput, or they behave differently from every other closure in that they get their own copy of the `connection` event's `HandlerMemory` for better throughput but an unexpected "immutable" outer scope that doesn't exist anywhere else.

This is what made me realize that what I wanted was to be able to pass the relevant *context* around between the events in question.

#### Handlers-in-Handlers TCP

This has the same trade-offs as the Node-like TCP above, so similarly ruled out, but it was just the idea of syntactically allowing `on <event> <handlerFunction>` within another handler function. The big issue is the shared mutable state. Either it serializes these events, or it's not actually mutable. Same reason why module scope only allows `const`, not `let`.

#### Erlangish TCP

The most drastic possibility considered was to augment or switch Alan to an Actor-based system. The actors themselves would be declared statically but constructed dynamically, with message passing in and out, and a way to reference each other to direct said messages around. These would work similar to classes syntactically, but they would be top-level, and internally they'd be like a `handler` that has an infinite loop polling its `inbox` for messages to do something and then put one or more things into its `outbox` and/or terminate itself.

It could potentially work well, but it would require a significant undertaking to build, be a "foreign" concept to most developers, and potentially introduce memory management problems if an actor's "address" is lost to all other actors and it did not clean itself up.

Finally, since events and actors are analogous, it should be possible to introduce an actor library built on top of static events and the `Context` concept to maintain the relevant state. So the language isn't locked out of this approach in the future.

## Affected Components

This will affect the standard library to expose the new opcodes and opaque types, the compiler to make it aware of the new opcodes and opaque types, and the two runtimes to implement the relevant opcodes.

## Expected Timeline

Implementation of a proof-of-concept in the js-runtime should take 1-2 days, and getting it working and performant in the AVM likely 1-3 more days (greater variability due to the handwaviness on how the context storage works under the hood. It'll be a `HandlerMemory` similar to how datastore works, but what locking mechanism is needed is not yet clear. Also how to not crash on double-calling `close` on a connection will require some finesse.)

