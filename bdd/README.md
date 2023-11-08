# bdd

The BDD test suite for the Alan Programming Language.

As the tools for Alan are written in a few different languages, the test suite is a collection of shell scripts using [shellspec](https://shellspec.info/).

Tests are placed in the `/bdd/spec/` directory and run in sorting order, so all tests are prefixed with a number to guarantee a predictable ordering.

Once the Alan Programming Language reaches feature complete on all planned features and ticks over to 1.0.0, Alan will be versioned according to a system we have defined as "Semantic BDD," described below:

## Semantic BDD

Semantic versioning is meant to tell the developer who is using it which versions could break their existing code if they update, but users eventually stop trusting it because they library developer makes a mistake in versioning, or "doesn't like" the rapidly rising major version numbers and just doesn't increment it every time they should. So, developers shouldn't be in charge of the semantic versioning. Automated analysis should be. We want to move from Semantic Versioning to Semantic BDD. 

Semantic BDD will require BDD tests to describe the supported features. If a change makes no change to the tests, then it is a patch. If a change adds new tests then it's a minor update. If it changes the results of prior testing it's a major update. That last part is determined by running the BDD test currently registered for the library and marking it as major if the old one fails and the new one succeeds. You can't publish if these special BDD tests fail. The version of your library is not up to you, anymore.

The publishing flow will expect a `sembdd` directory or `sembdd.ln` file. The publishing tool would perform the updating of major/minor/patch based on the results of this file and the file from the previously published version. If there is no such file, the major version is 0, and the minor version increments each time (so patch is also always 0). Once the package transitions from not having sembdd to having it, it runs it and will block publishing if sembdd fails or returns zero successes (to avoid a potential workaround of an empty sembdd file), otherwise it performs the major version bump to 1.0.0.

After that, we a history of sembdd files through checksums and it will use the following algorithm to compares the two sembdd declarations:

1. If they are identical (the sembdd.ln file or all files in the sembdd directory have the same checksums) then it does a patch increment after the test succeeds.
2. If they are different, then it runs both versions. If they both succeed, the minor version is bumped. If the old one fails and the new one succeeds, the major version is bumped.

With a centralized package management flow this can be enforced because you can't get into the registry without running these tests, we could potentially run them ourselves instead of letting them run locally (where a weirdly devious developer could try to fake it), and we can compare our record of expected versions in the publish branch versus what's in their repository and freeze things if it doesn't match the expected value (or have tarballs with bad signatures).

## License

MIT