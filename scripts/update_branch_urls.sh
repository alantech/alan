#!/bin/bash

# Script to update the Git URLs in alan_compiler/src/std/root.ln to point to the current branch
# This is used during CI to ensure tests use the current branch code instead of main

set -e

# Get the current branch name
# For pull requests, use GITHUB_HEAD_REF (source branch)
# For pushes, use GITHUB_REF_NAME (current branch)
# Fall back to git command for local development
BRANCH_NAME="${GITHUB_HEAD_REF:-${GITHUB_REF_NAME:-$(git rev-parse --abbrev-ref HEAD)}}"

# Only proceed if we're not on main branch (to avoid accidentally modifying main)
if [ "$BRANCH_NAME" = "main" ]; then
    echo "On main branch, skipping URL update"
    exit 0
fi

echo "Updating Git URLs to point to branch: $BRANCH_NAME"

# Update the RootBacking definitions to include the branch name
# Only update lines that don't already have a commit SHA (lines 133 and 135)
# Skip line 134 which has a specific commit SHA for x86 Mac wgpu workaround
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS (BSD sed)
    # Only replace URLs that end with .git" followed by closing brace (no hash/commit SHA)
    sed -i '' "s|@ \"https://github.com/alantech/alan.git\"}|@ \"https://github.com/alantech/alan.git#$BRANCH_NAME\"}|g" alan_compiler/src/std/root.ln
else
    # Linux and Windows (GNU sed)
    # Only replace URLs that end with .git" followed by closing brace (no hash/commit SHA)
    sed -i "s|@ \"https://github.com/alantech/alan.git\"}|@ \"https://github.com/alantech/alan.git#$BRANCH_NAME\"}|g" alan_compiler/src/std/root.ln
fi

echo "Successfully updated Git URLs to point to branch: $BRANCH_NAME"
