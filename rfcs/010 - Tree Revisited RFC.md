# 009 - Tree RFC

## Current Status

### Proposed

2020-11-30

### Accepted

2020-11-30

#### Approvers

- David Ellis <david@alantechnologies.com>

### Implementation

- [ ] Implemented
  - [x] [Remove `addChild` methods that can accept nodes in different trees](https://github.com/alantech/alan/pull/333)
  - [x] [Modify remaining `addChild` to return the newly created node](https://github.com/alantech/alan/pull/333)
  - [ ] Remaining advanced Tree methods-- `filter` and `reducePar` YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author

- Luis De Pombo <luis@alantechnologies.com>

## Summary

The majority of the conclusions in [RFC 009](./009 - Tree RFC.md) are still valid, but the originally API proposed for Tree construction allowed nodes from different trees to be merged without properly reconciling the original trees into a single one. Additionally, the `addChild` API does not return the newly created node which makes the originally proposed Tree creation harder to follow.

## Expected SemBDD Impact

If Alan was post-1.0.0, this would be a major update because it changes existing API visible to the end user.

## Proposal

The original RFC proposed the following API for tree construction:

```ln
const myTree = newTree("foo")
  .addChild(newTree("bar")
    .addChild("baz"))
  .addChild("bay")
```

There are three complications here. First the two created `Tree`s need to be properly merged into a single one. Secondly, it is not entirely obvious if `bay` is a child of `bar` or `foo`. Finally, `addChild(n: Node<any>, val: any): Node<any>` needs to be modified to return the newly created node, as opposed the `n`. 

If we remove `addChild(t: Tree<any>, val: any): Node<any>` and `addChild(n: Node<any>, val: Node<any>): Node<any>` we sidestep merging over `Tree`s for now and a future RFC will come to figure out subtree merging. With the proposed changes to `addChild(n: Node<any>, val: any): Node<any>`, the above tree construction can be rewritten using a more clear syntax:

```ln
const myTree = newTree('foo')
const barNode = myTree.addChild('bar')
const bazNode = myTree.addChild('baz')
const bayNode = barNode.addChild('bay')
```

### Alternatives Considered

Instead of having `addChild` method, we have an `addNode` method that takes the parent node ID or value as a parameter. However, then you either need to always be extracting the parent ID into a local temporary variable to pass back in, or you block trees that have two nodes with the same value because you can't choose which node to attach to by value.

## Affected Components

This should only require additions to the root scope file and additional tests.

## Expected Timeline

A `filter` and parallel version of `reduce` as proposed in the original RFC should take around two days.

