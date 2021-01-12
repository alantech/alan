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

It's a bit more difficult to judge the SemBDD impact here. It depends both on the particular solution chosen (to be updated once we've debated this point) and whether or not we consider the prior behavior a bug that is being fixed or a (mis-)feature being removed.

That means it's either a patch update, or a major update, depending on that point of view. Though I'm leaning towards classifying it as a major update even though it wasn't intentional; taking a page from the Linux Kernel view on this, the users' code is more important than the cleanliness of the language.

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

There are multiple ways to resolve this such that the actual datastore opcodes are left alone while the compiler guarantees that their usage is safe. I'm not actually sure which one(s) I like best, so I'm listing them all in here and we can debate the pros/cons.

### Minimally-invasive with restored type introspection

We recently removed the ability to get the type of a variable as a string with the `type variableName` syntax, but if we restore that, it should be possible to force type safety by combining the type string with either the namespace or the key string, such that trying to get the same key name with a default value of a different type will find that it doesn't exist and return the provided default.

This could provide zero actual API call changes except to the `del` function, which would need to be given an example of the type of data to delete in order to find it. This has the smallest change requirement, though it does require bringing back the string-based type manipulation code.

### Generic Functions (in addition to the interface functions that currently exist)

The maximum contrast to the prior is this particularly invasive change, where functions gain the ability to have one or more generic types, eg `fn foo<A>(arg1: int64, arg2: string): A {` such that they can be called with some type that is passed in during invocation but is not passed in as a variable, just the type, so `foo<bool>(1, 'bar')` would do something internally with the type.

In this case, the `namespace` function would take not only the name parameter, but the type info for the kind of variable to store in it, like `namespace<Array<int64>>('variousDigitsOfPi')`, then this type would internally be applied to the namespace through some syntax similar to the `type variableName` before so it can be stored in the datastore opcodes, but it would also be used to make sure writes and reads are sound, always providing values of the correct type.

The biggest issue here is: it adds a new "color" of function, and there will be pressure to add something like an `fn foo<A as Orderable>(...` syntax so the generics defined for the function are also interface constrained so the function knows what to do with it besides pass it around or poke at it in some way or the other.

It'll make the language much more complex, but will the utility gain be worth the complexity? It would make constructor functions much simpler, eg instead of `newHashMap(firstKey, firstVal)` we get `new<HashMap<KeyType, ValType>>()` without requiring the first key-val pair to be defined.

### Compiler-level hackery

We could do it with neither of the above via effectively hackery within the compiler; the `ds*` opcodes get a special behavior that always appends the type information to the `namespace` string no matter what and we just have some magic behavior that "just works" but doesn't trust the end user to have access to that power.

I'm including this for completeness, but that is the "Go Way" and in my book "The Go Way Is Stupid."

### Rust-style impl-ish functionality

This is similar to the Generic Functions approach but more constrained. Rust functions within an `impl` block are bound to a specific type. If that function begins with a variable of the same type named `self`, then it can be used as a method off of a variable of that type, if not, it can only be used by invoking it from the type itself with a double-colon, like `HandlerMemory::string_to_hm(...`. While this reduces the regularity of Alan, we could have such type-bound functions and require that each type that wants to be stored in datastore has to re-implement the 5 datastore functions, likely by adding the appropriate metadata to the namespace string.

This has the downside of requiring explicit boilerplate code for new types that want to be stored in datastore (and therefore can be explicitly written incorrectly and not actually do the right thing) and also breaking method syntax for these functions, but it doesn't require the magic of the Generic Functions or even the `type variableName` statement.

This boilerplate could be covered up if/when we add macro functionality to the language, which would also take us further along the path towards Rust.

### Namespace *is* Type

We could hide the namespace field from the end users and just make the namespace field the type. This would also require the `type variableName` syntax to be restored, but wouldn't require the special logic for the `set` and `has` functions that the mechanism maintaining the namespace would.

### Stop avoiding it; change the ds* opcodes

Finally, we could stop avoiding that a mistake was made with the opcode definition and bake in type safety. This would require rethinking it all from scratch so I won't go into too much detail here, but it would likely require similar support as the user-type -> tuple-ish array transformation that the compiler currently does in order to read and write from datastore and would turn it into something as innate as the Array type.

Basically, accepting the "Go Way" here, but then going all-out to do it right. The work there would be effective for also defining an on-disk serialization format, so it might not be absolute madness.

### Alternatives Considered

Move some of the alternatives above into here once we've decided which is the right approach!

## Affected Components

This will only affect the compiler and standard library.

## Expected Timeline

The exact timeline will depend on which version we choose, but only 1-3 days for most of them.
