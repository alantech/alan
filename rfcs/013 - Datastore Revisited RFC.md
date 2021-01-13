# 013 - Datastore Revisited RFC

## Current Status

### Proposed

2020-01-11

### Accepted

YYYY-MM-DD

#### Approvers

- Full Name <email@example.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- David Ellis <david@alantechnologies.com>

## Summary

The current implementation of `@std/datastore` is type unsound. The underlying opcode definition within the AVM and js-runtime is perfectly fine, though, so if we patch up the way they are exposed to the user in the language / first stage of the compiler, we can resolve this issue.

## Expected SemBDD Impact

This is technically a major update, because while the API behavior was a mistake, fixing it will cause a backwards incompatible change. Fortunately we're still pre-1.0 so we don't have to actually major version bump, yet. :)

## Proposal

The current issue with `@std/datastore` is that it is an unintentional source of unsafe casting between types. Eg

```ln
from @std/app import start, print, exit
from @std/datastore import namespace, set, getOr

on start {
  const foo = namespace('foo');
  foo.set('bar', 5);
  print(foo.getOr('bar', 1));
  print(foo.getOr('bar', 1.0));
  emit exit 0;
}
```

not only works, but it produces the following output

```
5
0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000025
```

The reason why this is a bug and not a feature is that it's an *unsafe* casting between types, so this

```ln
from @std/app import start, print, exit
from @std/datastore import namespace, set, getOr

on start {
  const foo = namespace('foo');
  foo.set('bar', 5);
  print(foo.getOr('bar', 1));
  print(foo.getOr('bar', ''));
  emit exit 0;
}
```

produces this

```
thread 'tokio-runtime-worker' panicked at 'range end index 5 out of range for slice of length 0', src/vm/memory.rs:628:29
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: JoinError::Panic(...)', src/vm/event.rs:260:14
```

a hard crash of the runtime as it tries to access a number as if it was a string. With a stated goal of eliminating as many runtime errors as possible so you can be sure that your backend is going to continue running, having this unsafe casting footgun was absolutely not the intention of the `datastore` library.

There are multiple ways to resolve this such that the actual datastore opcodes are left alone while the compiler guarantees that their usage is safe. For the sake of unblocking datastore usage as fast as possible, we're going to go through a two-step process to get it working, if a bit weird, and then using a new language feature to implement it cleanly. This does mean that the API will be changed twice, but since we are so far away from 1.0, this is considered fine for the adventurous who want to use this standard library that isn't documented publicly, yet.

### Part 1: Minimally-invasive with restored type introspection

We recently removed the ability to get the type of a variable as a string with the `type variableName` syntax, but if we restore that, it should be possible to force type safety by combining the type string with either the namespace or the key string, such that trying to get the same key name with a default value of a different type will find that it doesn't exist and return the provided default.

This will provide zero actual API call changes except to the `has` and `del` functions, which would need to be given an example of the type of data to delete in order to find it. This has the smallest change requirement, though the API is a bit weird.

### Part 2: Generic Functions (in addition to the interface functions that currently exist)

Functions gain the ability to have one or more generic types, eg `fn foo<A>(arg1: int64, arg2: string): A {` such that they can be called with some type that is passed in during invocation but is not passed in as a variable, just the type, so `foo<bool>(1, 'bar')` would do something internally with the type.

In this case, the `namespace` function would take not only the name parameter, but the type info for the kind of variable to store in it, like `namespace<Array<int64>>('variousDigitsOfPi')`, then this type would internally be applied to the namespace using a syntax similar to `type variableName` (perhaps as simple as `type GenericName`), so it can be stored in the datastore opcodes. This would remove the need to provide example values or type information to the `has` and `del` functions since it would already exist on the provided namespace.

It'll make the language much more complex, but the utility should surpass the complexity. This approach could be turned into the foundation for interface functions, such that `fn someFn<A as Orderable>(a: A, b: A): A` could be equivalent to `fn someFn(a: Orderable, b: Orderable): Orderable` with the latter becoming syntactic sugar. This would also allow us to remove duplicated interfaces like `anythingElse` because `fn map<A as Any, B as Any>(arr: Array<A>, mapper: fn(A): B): Array<B>` makes it clear that while `A` and `B` match the same interface, they are not the same actual type.

It would make constructor functions much simpler, eg instead of `newHashMap(firstKey, firstVal)` we could get something like `new<HashMap<KeyType, ValType>>()` without requiring the first key-val pair to be defined.

### Alternatives Considered

Many alternatives were considered and rejected:

### Compiler-level hackery

We could do it with neither of the above via effectively hackery within the compiler; the `ds*` opcodes get a special behavior that always appends the type information to the `namespace` string no matter what and we just have some magic behavior that "just works" but doesn't trust the end user to have access to that power.

I'm including this for completeness, but that is the "Go Way" and in my book "The Go Way Is Stupid."

### Rust-style impl-ish functionality

This is similar to the Generic Functions approach but more constrained. Rust functions within an `impl` block are bound to a specific type. If that function begins with a variable of the same type named `self`, then it can be used as a method off of a variable of that type, if not, it can only be used by invoking it from the type itself with a double-colon, like `HandlerMemory::string_to_hm(...`. While this reduces the regularity of Alan, we could have such type-bound functions and require that each type that wants to be stored in datastore has to re-implement the 5 datastore functions, likely by adding the appropriate metadata to the namespace string.

This has the downside of requiring explicit boilerplate code for new types that want to be stored in datastore (and therefore can be explicitly written incorrectly and not actually do the right thing) and also breaking method syntax for these functions, but it doesn't require the magic of the Generic Functions or even the `type variableName` statement.

This boilerplate could be covered up if/when we add macro functionality to the language, which would also take us further along the path towards Rust.

This was rejected because it adds considerable extra syntax to learn, which will make adoption harder, and for very little gain.

### Namespace *is* Type

We could hide the namespace field from the end users and just make the namespace field the type. This would also require the `type variableName` syntax to be restored, but wouldn't require the special logic for the `set` and `has` functions that the mechanism maintaining the namespace would.

This was rejected because it increases the chance that there are key collisions between libraries that use datastore, since they could be storing on the same types.

### Stop avoiding it; change the ds* opcodes

We could stop avoiding that a mistake was made with the opcode definition and bake in type safety. This would require rethinking it all from scratch so I won't go into too much detail here, but it would likely require similar support as the user-type -> tuple-ish array transformation that the compiler currently does in order to read and write from datastore and would turn it into something as innate as the Array type.

Basically, accepting the "Go Way" here, but then going all-out to do it right. The work there would be effective for also defining an on-disk serialization format, so it might not be absolute madness.

This was ruled out-of-scope for this PR because it requires defining a standardized serialization/deserialization format, presumably interchangeable between AVM and js-runtime, and it would need to be resilient to type mutations between runs (if type `Foo` has a `bar` type but then sprouts a `baz` type in the newly compiled version, that would ideally still load the old serialization, and then if `baz` is dropped in the future, it should also load only the relevant pieces of the type -- but there *also* should be a strict mode where this is a failure.

## Affected Components

This will only affect the compiler and standard library.

## Expected Timeline

The first part of the proposed change should only take 1-2 days. The second part is a multi-week effort reworking large pieces of the parser, function resolution, and type resolution, and shouldn't be tackled until several pieces of technical debt in the first stage of the compiler surrounding functions is cleared out.

