# 018 - Control Port Extensions RFC

## Current Status

### Proposed

2021-05-11

### Accepted

2021-05-12

#### Approvers

- Luis De Pombo <luis@alantechnologies.com>
- Colton Donnelly <colton@alantechnologies.com>
- Alejandro Guillen <alejandro@alantechnologies.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <david@alantechnologies.com>

## Summary

A control port is a useful pattern for backend applications to provide operational controls while it is running. Alan's AVM in daemon mode creates a control port that is used for initialization, health and metric monitoring, and cluster coordination, so far. These things are necessary for any Alan or Anycloud application, but there are things that may be application-specific that would be useful to expose through the control port instead of the primary HTTPS server.

A relatively simple addition to the standard library and AVM will make arbitrary extensions of the control port available to end users if they so desire, and can be immediately used by Anycloud to make the distributed datastore more easily accessible to any language.

## Expected SemBDD Impact

If we were post-1.0.0, this would be a minor update; no breakage of existing code is possible.

## Proposal

The `@std/httpserver` would be modified with a new event: `event controlPort: Connection` that an Alan application can optionally register an event handler for. This would work almost identically to the `connection` event, with two major exceptions:

1. The built-in control port functionality the AVM requires is evaluated *first*. If it matches, it will immediately handle the control port request, while if it doesn't match it will fall through to this event handler.
2. The default response defined for the control port if not set by the event handler will be a 403 Forbidden error. As the secret token authentication portion will already have passed before reaching the handler 401 doesn't make sense, and since the acceptable paths are statically defined and only modifiable by upgrading the service, 404 also does not make sense.

The choice of evaluating built-in control port functionality first was for three reasons:

1. Several of the control port endpoints are required to behave in a well-defined way so the cluster management will work and `alan deploy` and `anycloud` can properly inspect, update, or tear down a cluster. If the user accidentally overrides any of these endpoints, these guarantees are gone and likely the cluster will fail to function properly.
2. There is a performance impact of handing off from the internals of the AVM to the Alan application and then back again, which can negatively impact the speed of Alan applications themselves, since things like the distributed datastore would run slower if they effectively jump back and forth twice on every datastore operation, so avoiding that hand-off when possible is desireable.
3. The security guards on the control port could be bypassed or overridden if they are not handled first. The Alan application code itself has no access to the cluster secret that guards control port accesses (at least right now) so there's no way to even implement that secret token authentication even if attempted.

The biggest disadvantage of this evaluation order is that adding new paths to the control port for the AVM would produce a breaking change for code, as a path that worked before in the user's application logic no longer works as expected, as it is being used for some purpose for the AVM, and calling it would likely fail or even worse have some unintended behavior consequences.

However, this issue can be eliminated by recognizing that the endpoints are paths, and a path *prefix* can be used to namespace the two. There are three approaches:

1. Namespace the AVM control port endpoints with `/avm/*`
2. Namespace the Alan application control port endpoints with `/app/*`
3. Namespace both with `/avm/*` and `/app/*`

The absolute safest path is option 3, as we can also add more namespaces in the future for other sources of control port functionality, however the split between "control port endpoints maintained by the AVM developers" and "control port endpoints maintained by AVM users" is a pretty solid split that is *unlikely* to run into issues in the future.

Option 1 provides the least annoyance to application developers as they don't have `/app/` prefixed on every custom endpoint, while Option 2 is backwards compatible with how the AVM cluster is managed and coordinates with itself right now. Since this functionality is only for true power users of the language, and is not intended to be a publicly-facing endpoint, but glue for advanced integration needs with their own backend services, I am leaning towards Option 2.

In the end, a service that establishes an HTTPS server and custom control port functionality would simply have two handlers, one for each:

```ln
from @std/httpserver import connection, Connection, status, body, send
from @std/avmdaemon import controlPort

on connection fn (conn: Connection) {
  const res = conn.res;
  res.status(200).body('Hello, World!').send();
}

on controlPort fn (conn: Connection) {
  const res = conn.res;
  res.status(200).body('Secret access!').send();
}
```

No new concepts to learn, just a slight deviation in the default response if not set explicitly. The built-in control port endpoints would be "out of sight, out of mind" and the times your handler is called would be if the path begins with `/app/`.

### Alternatives Considered

Because of the hardwired behaviors of the control port, having a separate control port registration path was considered, with a separate standard library involved, something like:

```ln
from @std/app import start
from @std/controlPort import handle
from @std/httpserver import connection, Connection, status, body, send

on connection fn (conn: Connection) {
  const res = conn.res;
  res.status(200).body('Hello, World!').send();
}

on start {
  handle('/app/something', fn (conn: Connection) {
    const res = conn.res;
    res.status(200).body('Secret access!').send();
  });
  handle('/app/somethingElse', fn (conn: Connection) {
    const res = conn.res;
    res.status(401).body('Get outta here!').send();
  })
}
```

This was rejected for several reasons: requiring `@std/app` which is geared towards CLI applications, creating a new way to handle http requests different from the normal way that you have to learn separately, and the forced linearization by these handler closure functions all sharing the same start event `HandlerMemory` (or violation of the language design by duplicating the `HandlerMemory` for each endpoint and not allowing them to share a `let` variable between them).

Not adding custom control port endpoints at all and hardwiring the endpoints we want for Anycloud was rejected as it further pollutes and complicates the mental model of the AVM for a privileged application (and the need Anycloud has for this is proof that user applications would want it, as well).

Adding a second control port for user control port behavior was considered but dropped because either it wouldn't be accessible outside of the machine itself and have limited utility, or it would, but be a very easy way to accidentally backdoor your own application due to a lack of security measures, while having it on the same control port means it inherits the secret-based access authorization and doesn't require any special logic in the cluster management (how the VM security group is configured, and determining whether or not to make that other port open would require inspecting the compiled code, which would be difficult and potentially brittle).

Generalizing the `@std/httpserver` to allow multiple ports was rejected for similar reasons as the second control port, but also because determining "which one" was the right port to expose as an HTTPS server and which as a control port is convoluted, and further mixing the main server and control port logic into the same event handler is likely to cause unintended security bugs if any branch of the handler fails to confirm the `port` number. (And that inspection is further complicated by the "main" port being any of `8000`, `80`, or `443` depending on how the AVM is started.)

## Affected Components

This would affect the standard library, lightly touch the compiler to add a new built-in event, and affect the AVM and js-runtime. As the js-runtime does not actually coordinate with anything and is not used by `alan deploy` / `anycloud`, it would simply be a second HTTP server.

## Expected Timeline

This should take just 1-2 days of work to implement in a single PR and as proposed is completely backwards compatible with the current cluster management approach.
