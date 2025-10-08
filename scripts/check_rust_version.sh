#!/bin/bash

# Script to check and set the correct Rust version based on Cargo.toml
# This ensures that the version of rustc matches the version declared in the root Cargo.toml

set -e

# Function to extract rust-version from Cargo.toml
extract_rust_version() {
    local cargo_toml="$1"
    
    # Look for rust-version field in the [workspace.package] section
    # Use a more robust approach that works with both BSD and GNU sed
    local version_line=$(grep -A 20 "\[workspace\.package\]" "$cargo_toml" | grep "rust-version" | head -1)
    echo "Debug: Found rust-version line: '$version_line'" >&2
    
    # Extract version using a more portable approach
    # First try with basic sed (works on both BSD and GNU)
    local version=$(echo "$version_line" | sed 's/.*rust-version[[:space:]]*=[[:space:]]*["'"'"']*\([0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*\)["'"'"']*.*/\1/')
    
    # If that didn't work, try a more aggressive approach
    if [ -z "$version" ] || [ "$version" = "$version_line" ]; then
        # Use awk as a fallback - it's more portable
        version=$(echo "$version_line" | awk '{
            gsub(/.*rust-version[[:space:]]*=[[:space:]]*["'"'"']*/, "")
            gsub(/["'"'"'].*/, "")
            gsub(/[[:space:]]*$/, "")
            print
        }')
    fi
    
    # Final validation - make sure we got a version number
    if [ -z "$version" ] || ! echo "$version" | grep -q '^[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*$'; then
        echo "Error: Could not extract valid version from line: '$version_line'"
        echo "Debug: Extracted version was: '$version'"
        exit 1
    fi
    
    echo "$version"
}

# Function to get current rustc version
get_current_rustc_version() {
    rustc --version | sed 's/rustc \([0-9]\+\.[0-9]\+\.[0-9]\+\).*/\1/'
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
        echo "Debug: Running 'rustup install $REQUIRED_VERSION'..." >&2
        rustup install "$REQUIRED_VERSION"
        
        echo "Debug: Running 'rustup default $REQUIRED_VERSION'..." >&2
        rustup default "$REQUIRED_VERSION"
        
        # Verify the installation
        echo "Debug: Verifying installation..." >&2
        NEW_VERSION=$(get_current_rustc_version)
        echo "Debug: New rustc version: $NEW_VERSION" >&2
        
        if [ "$NEW_VERSION" = "$REQUIRED_VERSION" ]; then
            echo "✓ Successfully switched to Rust $REQUIRED_VERSION"
        else
            echo "Error: Failed to switch to Rust $REQUIRED_VERSION. Current version is still $NEW_VERSION"
            echo "Debug: rustup show output:" >&2
            rustup show >&2
            exit 1
        fi
    fi