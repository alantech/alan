#!/usr/bin/env pwsh
# Script to check and set the correct Rust version based on Cargo.toml
# This ensures that the version of rustc matches the version declared in the root Cargo.toml

Write-Host "Checking Rust version compatibility..."

# Check if rustup is available
try {
    $null = rustup --version
} catch {
    Write-Error "Error: rustup is not available. Please install rustup first."
    exit 1
}

# Function to extract rust-version from Cargo.toml
function Get-RustVersion {
    $cargoToml = "Cargo.toml"
    
    # Look for rust-version field in the [workspace.package] section
    $content = Get-Content $cargoToml -Raw
    $lines = $content -split "`n"
    $rustVersionLine = $null
    $inWorkspacePackage = $false
    
    foreach ($line in $lines) {
        if ($line -match '\[workspace\.package\]') {
            $inWorkspacePackage = $true
            continue
        }
        if ($inWorkspacePackage -and $line -match 'rust-version') {
            $rustVersionLine = $line
            break
        }
        if ($inWorkspacePackage -and $line -match '^\[') {
            # Hit another section, stop looking
            break
        }
    }
    
    Write-Host "Debug: Found rust-version line: '$rustVersionLine'" -ForegroundColor Yellow
    
    # Extract version using regex
    if ($rustVersionLine -match 'rust-version\s*=\s*["\x27]?([0-9]+\.[0-9]+\.[0-9]+)["\x27]?') {
        return $matches[1]
    } else {
        Write-Error "Error: Could not extract version from line: '$rustVersionLine'"
        exit 1
    }
}

# Function to get current rustc version
function Get-CurrentRustcVersion {
    $versionOutput = rustc --version
    Write-Host "Debug: Raw rustc --version output: $versionOutput" -ForegroundColor Yellow
    
    if ($versionOutput -match 'rustc ([0-9]+\.[0-9]+\.[0-9]+)') {
        return $matches[1]
    } else {
        Write-Error "Error: Could not extract version from rustc --version output: $versionOutput"
        exit 1
    }
}

# Get the required Rust version from Cargo.toml
$REQUIRED_VERSION = Get-RustVersion
Write-Host "Required Rust version: $REQUIRED_VERSION"

# Get current rustc version
$CURRENT_VERSION = Get-CurrentRustcVersion
Write-Host "Current rustc version: $CURRENT_VERSION"

# Compare versions
if ($CURRENT_VERSION -eq $REQUIRED_VERSION) {
    Write-Host "✓ Rust version matches required version ($REQUIRED_VERSION)" -ForegroundColor Green
    exit 0
}
else {
    Write-Host "⚠ Rust version mismatch. Current: $CURRENT_VERSION, Required: $REQUIRED_VERSION" -ForegroundColor Yellow
    Write-Host "Installing and switching to Rust $REQUIRED_VERSION..."
    
    # Install the required version using rustup
    Write-Host "Debug: Running 'rustup install $REQUIRED_VERSION'..." -ForegroundColor Yellow
    rustup install $REQUIRED_VERSION
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Error: Failed to install Rust $REQUIRED_VERSION"
        exit 1
    }
    
    Write-Host "Debug: Running 'rustup default $REQUIRED_VERSION'..." -ForegroundColor Yellow
    rustup default $REQUIRED_VERSION
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Error: Failed to set default Rust version to $REQUIRED_VERSION"
        exit 1
    }
    
    # Verify the installation
    Write-Host "Debug: Verifying installation..." -ForegroundColor Yellow
    $NEW_VERSION = Get-CurrentRustcVersion
    Write-Host "Debug: New rustc version: $NEW_VERSION" -ForegroundColor Yellow
    
    if ($NEW_VERSION -eq $REQUIRED_VERSION) {
        Write-Host "✓ Successfully switched to Rust $REQUIRED_VERSION" -ForegroundColor Green
    } else {
        Write-Error "Error: Failed to switch to Rust $REQUIRED_VERSION. Current version is still $NEW_VERSION"
        Write-Host "Debug: rustup show output:" -ForegroundColor Yellow
        rustup show
        exit 1
    }
}
