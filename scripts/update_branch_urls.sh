#!/bin/bash

# Script to update the Git URLs in alan_compiler/src/std/root.ln to point to the current branch
# This is used during CI to ensure tests use the current branch code instead of main

set -e

# Get the current branch name
BRANCH_NAME="${GITHUB_REF_NAME:-$(git rev-parse --abbrev-ref HEAD)}"

# Only proceed if we're not on main branch (to avoid accidentally modifying main)
if [ "$BRANCH_NAME" = "main" ]; then
    echo "On main branch, skipping URL update"
    exit 0
fi

echo "Updating Git URLs to point to branch: $BRANCH_NAME"

# Update the RootBacking definitions to include the branch name
# Lines 130-131 in alan_compiler/src/std/root.ln
sed -i "s|@ \"https://github.com/alantech/alan.git\"|@ \"https://github.com/alantech/alan.git#$BRANCH_NAME\"|g" alan_compiler/src/std/root.ln

echo "Successfully updated Git URLs to point to branch: $BRANCH_NAME"
