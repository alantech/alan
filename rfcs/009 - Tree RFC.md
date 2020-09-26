# 009 - Tree RFC

## Current Status

### Proposed

2020-09-25

### Accepted

YYYY-MM-DD

#### Approvers

- Luis de Pombo <luis@alantechnologies.com>

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author

- David Ellis <david@alantechnologies.com>

## Summary

The ability to define an reason about data that has a tree-like structure is necessary to handle a host of different problems, from parsers and serializers, to manipulation of an HTML DOM or JSON object.

This is an integral piece of any language. Most choose to allow recursive data types and then deal with the complexities around infinitely-defined types (one way or the other). For Alan, we instead define a new Tree type to handle this and provide a collection of tools to work with it.

## Expected SemBDD Impact

If Alan was post-1.0.0, this would be a minor update.

## Proposal

The `Tree` type is relatively straightforward:

```ln
type Tree<T> {
  vals: Array<T>
  parents: Array<int64>
  children: Array<Array<int64>>
}
```

where the values, parents and children relationships are tracked by three arrays. Each logical node is defined by matching indexes between them, with that being the node's ID, making the `Node` type straightforward:

```ln
type Node<T> {
  id: int64
  tree: Tree<T>
}
```

This inversion of the relationship between Tree and Node avoids recursive representations while still allowing node-based manipulation a clean API.

With basic functionality like tree construction, getting a root node, getting children, getting a node by value, etc, you could combine that with `@std/seq` to perform all of the tree manipulation you care for, but it is more interesting to provide methods similar to the array methods that can potentially perform certain operations in parallel.

An equivalent to `map` for Tree makes sense, letting you transform the values of one tree into a new one, where the callback function could also use the parent and/or children to determine how to create the newly transformed node. The only question is whether or not it uses the same name of `map` or it gets a new name to be clearer?

`reduce` is similarly interesting -- it could be parallelized by "dependency layer", all of the nodes with no children, then all of the nodes in the first layer above those, then the layer above that, etc, until you reach the root node. This means that serializing a data structure built on top of a Tree (like JSON, for instance) would be an automatically parallel operation and could therefore be more performant by default than other languages.

`filter` could have two implementations, one that filters in a way that a failed filter node *and all of its children* are removed, and one where the failed nodes are deleted and their children (that pass the filter) are re-parented on the failed node's parent, where the latter would provide an array of potentially more than one tree if the root node is filtered away. What they should be called is not clear.

`every` and `some` could simply be shorthand for something like `someTree.toNodeArray().every(...)` or just excluded instead.

There are also operations that are purely tree-related operations like a `balance` to rebalance a tree as a binary tree (though this would be part of also creating an array `sort` function and defining an `Orderable` interface type).

It should also be possible to provide syntactic sugar for statically-defined Trees, inspired by XML:

```ln
const myTree = <"foo">
  <"bar">
    <"baz"/>
  </>
  <"bay"/>
</> endTree
```

where the output type in this example is `Tree<string>`. It requires 4 operators, `<`, `>`, `/>`, and `</>`. A prefix `<` causes it to create a new tree with a root node with the value that follows it and has the highest precedence. `>` is an infix operator between two trees or a node and a tree and attaches the second argument as a child of the first argument (a node or the root node of the tree). `/>` and `</>` are prefix operators that create a special intermediate type that causes traversal backwards up the first node before attaching, to get the correct nesting set up except the final one as an infix operator between the two nodes to actually join. That final `endTree` is simply an alias to `void` that tells it to finalize the Tree structure and simply return it and is necessary because Alan does not support postfix operators.

This is the iffiest idea because it's done through an abuse of the operator syntax, but it looks pretty natural for defining tree constants. Fluent style isn't so bad, though:

```ln
const myTree = newTree("foo")
  .addChildTree(newTree("bar")
    .addChildLeaf("baz"))
  .addChildLeaf("bay")
```

With the only downside being keeping track of the parens and being explicit on when a node is a leaf node versus a subtree. (Not strictly necessary, but there would be a lot of extra `newTree` calls.)

### Alternatives Considered

Serious consideration of having recursively defined types. Most languages have them, however most languages also have implicitly nullable fields, so just not defining a valid value when you've hit the end of the recursive nature of the type is fine there and doesn't work here or in Rust. Rust has recursively defined typed and validates that the recursion is safe by automatically picking up that the recursion can terminate, but this doesn't always work. A recursive type in Rust that has a field that is a `Vec<CustomType>` works, but the [seemingly equally valid `Option<CustomType>` fails](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=1e799ce8f319e27fc6907a2ef014c2a6). Recursive typing in a strongly typed language requires a very intelligent compiler, and if even the Rust community can't completely solve this problem, it does not seem like one we should take on at all -- especially when there's an alternative.

A second alternative considered was to make the Tree type a native type instead of built on top of Arrays. This could have a performance advantage, but shifts complexity into the runtimes. Furthermore, improvements to the runtime should make such a penalty nonexistent, eventually and allows the optimization of parallelization strategies to be placed behind a smaller number of opcodes and should allow such optimization to progress faster.

The last alternative is to not have an explicit Tree type and just use the pattern in the type for implementing features that need that sort of recursive relationship, but while that may be very slightly more efficient, it will also encourage reimplementation of the same sorts of ideas over and over again which is more prone to bugs.

## Affected Components

This should only require additions to the root scope file and additional tests.

## Expected Timeline

The base types and most of the methods should only take a day or two. If we decide to implement the XML-like constant declaration that will take another day or two on its own.

