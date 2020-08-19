#!/bin/bash

echo "Checking build commands..."

# First check the needed commands exist
if ! command -v node &> /dev/null; then
  echo "Node.js is required to build this project. Please install Node >=10.20.1"
  exit 1
fi

if ! command -v yarn &> /dev/null; then
  echo "yarn is required to build this project.Please install yarn >=1.22.4"
  exit 1
fi

if ! command -v rustc &> /dev/null; then
  echo "Rust is required to build this project. Please install Rust >=1.41.1"
  exit 1
fi

if ! command -v cargo &> /dev/null; then
  echo "cargo is required to build this project. Please install cargo >=1.41.0"
  exit 1
fi

# Next, confirm the command versions are up-to-date
yarn add semver

if ! ./.semver.js 10.20.1 $(node --version); then
  echo "Node.js is out of date. Please use Node >=10.20.1"
  exit 1
fi

if ! ./.semver.js 1.22.4 $(yarn --version); then
  echo "yarn is out of date. Please use yarn >=1.22.4"
  exit 1
fi

if ! ./.semver.js 1.41.1 $(rustc --version | sed 's/rustc //g;s/(.*$//g'); then
  echo "Rust is out of date. Please use Rust >=1.41.1"
  exit 1
fi

if ! ./.semver.js 1.41.0 $(cargo --version | sed 's/cargo //g;s/(.*$//g'); then
  echo "cargo is out of date. Please use cargo >=1.40.0"
  exit 1
fi

echo "Done!"
exit 0