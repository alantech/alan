#!/bin/bash

# Script to check and set the correct Rust version based on Cargo.toml
# This ensures that the version of rustc matches the version declared in the root Cargo.toml

set -e

# Function to extract rust-version from Cargo.toml
extract_rust_version() {
    local cargo_toml="$1"
    
    # Look for rust-version field in the [workspace.package] section
    # Handle both quoted and unquoted versions
    local version=$(grep -A 20 "\[workspace\.package\]" "$cargo_toml" | grep "rust-version" | head -1 | sed -E 's/.*rust-version\s*=\s*["\x27]?([^"\x27\s]+)["\x27]?.*/\1/')
    
    if [ -z "$version" ]; then
        echo "Error: Could not find rust-version in $cargo_toml"
        exit 1
    fi
    
    echo "$version"
}

# Function to get current rustc version
get_current_rustc_version() {
    rustc --version | sed -E 's/rustc ([0-9]+\.[0-9]+\.[0-9]+).*/\1/'
}

# Function to check if rustup is available
check_rustup() {
    if ! command -v rustup &> /dev/null; then
        echo "Error: rustup is not available. Please install rustup first."
        exit 1
    fi
}

# Main execution
echo "Checking Rust version compatibility..."

# Check if rustup is available
check_rustup

# Get the required Rust version from Cargo.toml
REQUIRED_VERSION=$(extract_rust_version "Cargo.toml")
echo "Required Rust version: $REQUIRED_VERSION"

# Get current rustc version
CURRENT_VERSION=$(get_current_rustc_version)
echo "Current rustc version: $CURRENT_VERSION"

# Compare versions
if [ "$CURRENT_VERSION" = "$REQUIRED_VERSION" ]; then
    echo "✓ Rust version matches required version ($REQUIRED_VERSION)"
    exit 0
else
    echo "⚠ Rust version mismatch. Current: $CURRENT_VERSION, Required: $REQUIRED_VERSION"
    echo "Installing and switching to Rust $REQUIRED_VERSION..."
    
    # Install the required version using rustup
    rustup install "$REQUIRED_VERSION"
    rustup default "$REQUIRED_VERSION"
    
    # Verify the installation
    NEW_VERSION=$(get_current_rustc_version)
    if [ "$NEW_VERSION" = "$REQUIRED_VERSION" ]; then
        echo "✓ Successfully switched to Rust $REQUIRED_VERSION"
    else
        echo "Error: Failed to switch to Rust $REQUIRED_VERSION. Current version is still $NEW_VERSION"
        exit 1
    fi
fi