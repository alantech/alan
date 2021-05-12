#!/bin/bash

# Be very *vexing* with the output, but good for debugging if something goes wrong
set -vex

# The new version
VERSION=$1

echo Version: ${VERSION}

# Update the package metadata with the specified version
JSRUNTIME="$(jq ".version = \"${VERSION}\"" js-runtime/package.json)" && echo "${JSRUNTIME}" > js-runtime/package.json
COMPILER="$(jq ".version = \"${VERSION}\"" compiler/package.json)" && echo "${COMPILER}" > compiler/package.json
AVM="$(sed "s/^version = .*$/version = \"${VERSION}\"/" avm/Cargo.toml)" && echo "${AVM}" > avm/Cargo.toml
ANYCLOUD_NPM="$(jq ".version = \"${VERSION}\"" anycloud/node/package.json)" && echo "${ANYCLOUD_NPM}" > anycloud/node/package.json
ANYCLOUD="$(sed "s/^version = .*$/version = \"${VERSION}\"/" anycloud/cli/Cargo.toml)" && echo "${ANYCLOUD}" > anycloud/cli/Cargo.toml

# Make sure the lockfiles are updated, too
cd js-runtime
yarn
cd -

cd compiler
yarn
cd -

cd anycloud/cli
cargo build
cd -

cd avm
cargo build
cd -

# Commit and tag the update
git add js-runtime/package.json js-runtime/yarn.lock anycloud/node/package.json compiler/package.json compiler/yarn.lock avm/Cargo.toml avm/Cargo.lock anycloud/cli/Cargo.toml anycloud/cli/Cargo.lock
git commit -m "v${VERSION}"
git push origin main
gh release create v${VERSION} -t v${VERSION} -n v${VERSION} --target main

# Publish the js-runtime to NPM with the new version
cd js-runtime
npm publish
cd -

# Publish the anycloud/node to NPM with the new version
cd anycloud/node
npm publish
cd -
