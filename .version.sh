#!/bin/bash

# Be very *vexing* with the output, but good for debugging if something goes wrong
set -vex

# The new version
VERSION=$1

# Update the package metadata with the specified version
JSRUNTIME="$(jq '.version = "${VERSION}"' js-runtime/package.json)" && echo "${JSRUNTIME}" > js-runtime/package.json
COMPILER="$(jq '.version = "${VERSION}"' compiler/package.json)" && echo "${COMPILER}" > compiler/package.json
AVM="$(sed 's/version = .*$/version = "0.1.7"/' avm/Cargo.toml)" && echo "${AVM}" > avm/Cargo.toml

# Make sure the lockfiles are updated, too
cd js-runtime
yarn
cd -

cd compiler
yarn
cd -

cd avm
cargo build
cd -

# Commit and tag the update
git add js-runtime/* compiler/* avm/*
git commit -m "v${VERSION}"
git tag v${VERSION}
git push origin main --tags

# Publish the js-runtime to NPM with the new version
cd js-runtime
npm publish
cd -

