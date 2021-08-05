# 020 - Dependencies permissions RFC

## Current Status

### Proposed

2021-08-05

### Accepted

YYYY-MM-DD

#### Approvers

- David Ellis <david@alantechnologies.com>
- Luis De Pombo <luis@alantechnologies.com>
- Colton Donnelly <colton@alantechnologies.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author(s)

- Alejandro Guillen <alejandro@alantechnologies.com>

## Summary

Alan's third-party module permission system is one of the value propositions of the language. It adds a layer of security no other project approaches. The idea is to allow users to prevent specific third-party dependencies from having access to specific standard libraries that they should not have access to. This can be achieve with current mocking built-in and some updates to the `@std/deps` standard library.

## Expected SemBDD Impact

This would be a minor update if we were post-1.0 as it should have zero breaking impact on existing code.

## Proposal

The idea is to take advantage of alan's module resolution and build-in mock systems to prevent libraries to access specific standard libraries. We could also block specific standard libraries at application level, meaning that we would not want that any of our third party libraries use them. If we do not specify anything and just use the library will use everything.

The priorities of the module resoulution system would go as follow starting with the highest priority:

1. Actual dependency level `modules` directory
2. Global dependencies `modules` directory
3. Application `modules` directory

Meaning that inner `modules` > outer `modules`.

### Block at library level

This is the basic case when we want to use `new_dep_A` in our project but we do not want this library to have access to `std/cmd` for example. So, we we add this library we will specify the standard libraries blacklisted. This block would need to work recursively due to module resolution priorities. The flow will be:

![Block at library level](./lib-lvl-v1.png)

The `.dependencies.ln` file could look something like:

Block `@std/cmd` but do not override if any mock exists:
```ln
add('https://github.com/org/new_dep_A', some(['cmd']), false);
```

Block `@std/cmd` and override mocks if any:
```ln
add('https://github.com/org/new_dep_A', some(['cmd']), true);
```

Do not block any standrad library:
```ln
add('https://github.com/org/new_dep_A', none(), false);
```

### Block at application level

We might want to block any standard library we decide for every third party dependency or maybe have a custom behaviour for every third party library. This block would also need to work recursively due to module resolution priorities. The flow will be:

![Block at app level](./app-lvl-v1.png)


The `.dependencies.ln` file could look something like:

Block `@std/cmd` but do not override if any mock exists:
```ln
block('cmd', false);
```

Block `@std/cmd` and override mocks if any:
```ln
block('cmd', true);
```

### Alternatives Considered

- The fisrt option is leave it as is and do not provide any built-in feature, letting users do it manually. This is painful.

- The same as the proposed solution but do not look recursively trough nested dependencies. The downside of this is that we still will need to trust third party libraries' authors to ensure their dependencies does not intend malicious activities.

- Kind of system where every standard library is blocked and users should enable all the time the necessary standard libraries for each dependency.

## Affected Components

This will mostly affect the standard library.

## Expected Timeline

This should probably only take about a week.
