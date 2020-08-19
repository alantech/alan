#!/usr/bin/env node

const semver = require('semver')

const arglen = process.argv.length

const expected = process.argv[arglen - 2]
const actual = process.argv[arglen - 1]

if (semver.gte(actual, expected)) {
  process.exit(0)
} else {
  process.exit(1)
}