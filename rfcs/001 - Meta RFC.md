# 001 - Meta RFC

## Current Status

### Proposed

2020-05-31

### Accepted

YYYY-MM-DD

### Implementation

- [ ] Implemented: [One or more PRs](https://github.com/alantech/alan/some-pr-link-here) YYYY-MM-DD
- [ ] Revoked/Superceded by: [RFC ###](./000 - RFC Template.md) YYYY-MM-DD

## Author

- David Ellis <david@alantechnologies.com> (Can I please have that? :) )

## Summary

Keeping track of decisions made around the language and why can be tricky. An RFC process by PR allows detailed version history and review processes to occur entirely through GitHub, easing the process and keeping it close to the codebases that it affects.

## Expected SemBDD Impact

This Meta RFC should have no impact on the language ecosystem.

## Proposal

The `rfcs` directory will house all RFCs that have been accepted, with the RFC number incrementing atomically according to acceptance order, not creation order, so the last step of an RFC would be to rebase on latest master, update the RFC number appropriately, and then merge it.

The RFCs should use [this template](./000 - RFC Template.md) as a guide on the sections that are necessary to include in their own proposal. Adding or removing sections from the template should be called out in the PR submission -- it should not be done lightly, but may indicate an issue with the RFC template itself, which can be revised.

The template includes guidance to both the RFC author and to reviewers of RFCs, and should be read by all who submit and review RFCs. Some of the guidance falls into the kind of things that go into a Code of Conduct document, but repetition here can be useful.

The review process itself is not documented in the template, as that is likely to change over time as there are more contributors. This should be documented at some point, but this process is already heavyweight for such a small team. The RFCs at the moment are more a way to help drive clarity in thinking and document our decision-making to our future selves, at the moment, and formalizing the review process itself can come later.

Once it is approved, the RFCs should be landed in a different way. Since for RFCs we value the change history more than with code PRs (which are ideally small enough that a single commit is intelligible when reviewed), they should be rebased on master with master head fast-forwarded, instead of squashed and rebased. This will allow RFC PRs to be fully usable through the GitHub UI potentially years in the future.

### Alternatives Considered

1. RFC as Github issue: Github issues allow templating, so the template could simply be included there. However, Github's Issue tracking is much more primitive. Issues are simply closed (though they could be tagged) and there is no version history available (when the tags were changed, when the proposal was changed, etc, we only know it was "edited") so seeing how it evolved over time is not possible.
2. Google Docs: Collaborative review and editing can also happen in Google Docs, and there is a version history there, as well. But it would be separated from the code. It is also another paid service to maintain and the version history is not exportable.
3. 100% Git-based RFC process: The RFCs could instead be in a detached "rfc" branch, where RFC proposals are immediately committed and all comments are PRs that annotate the RFC and eventually the acceptance is done in a separate PR. This guarantees the *entire* RFC history is portable out of Github, as well, but it makes the process awkward and imposing to most due to how foreign it is, and how much has to be learned before you can contribute.
4. Mailing list-based RFC process: A public mailing list server could be established for RFC proposals, with comments and adjustments done by email and the final RFC committed with a link to the mailing list tree of messages. Very old school and would work, but also intimidating and foreign to most outsiders.

## Affected Components

This meta RFC would not immediately affect anything, but it will affect everything indirectly. ;)

## Expected Timeline

Once it is approved, it is immediately in effect and no work would be necessary.
