@echo off
REM Script to check and set the correct Rust version based on Cargo.toml
REM This ensures that the version of rustc matches the version declared in the root Cargo.toml

echo Checking Rust version compatibility...

REM Check if rustup is available
rustup --version >nul 2>&1
if errorlevel 1 (
    echo Error: rustup is not available. Please install rustup first.
    exit /b 1
)

REM Extract rust-version from Cargo.toml
REM Look for rust-version field in the [workspace.package] section
for /f "tokens=2 delims==" %%i in ('findstr /r "rust-version" Cargo.toml') do (
    set "version_line=%%i"
    goto :found_version
)
echo Error: Could not find rust-version in Cargo.toml
exit /b 1

:found_version
REM Clean up the version string (remove quotes and spaces)
set "version_line=%version_line: =%"
set "version_line=%version_line:"=%"
set "version_line=%version_line:'=%"
set "REQUIRED_VERSION=%version_line%"

echo Required Rust version: %REQUIRED_VERSION%

REM Get current rustc version
for /f "tokens=2" %%i in ('rustc --version') do set "CURRENT_VERSION=%%i"
echo Current rustc version: %CURRENT_VERSION%

REM Compare versions
if "%CURRENT_VERSION%"=="%REQUIRED_VERSION%" (
    echo ✓ Rust version matches required version (%REQUIRED_VERSION%)
    exit /b 0
) else (
    echo ⚠ Rust version mismatch. Current: %CURRENT_VERSION%, Required: %REQUIRED_VERSION%
    echo Installing and switching to Rust %REQUIRED_VERSION%...
    
    REM Install the required version using rustup
    echo Debug: Running 'rustup install %REQUIRED_VERSION%'...
    rustup install %REQUIRED_VERSION%
    if errorlevel 1 (
        echo Error: Failed to install Rust %REQUIRED_VERSION%
        exit /b 1
    )
    
    echo Debug: Running 'rustup default %REQUIRED_VERSION%'...
    rustup default %REQUIRED_VERSION%
    if errorlevel 1 (
        echo Error: Failed to set default Rust version to %REQUIRED_VERSION%
        exit /b 1
    )
    
    REM Verify the installation
    echo Debug: Verifying installation...
    for /f "tokens=2" %%i in ('rustc --version') do set "NEW_VERSION=%%i"
    echo Debug: New rustc version: %NEW_VERSION%
    
    if "%NEW_VERSION%"=="%REQUIRED_VERSION%" (
        echo ✓ Successfully switched to Rust %REQUIRED_VERSION%
    ) else (
        echo Error: Failed to switch to Rust %REQUIRED_VERSION%. Current version is still %NEW_VERSION%
        echo Debug: rustup show output:
        rustup show
        exit /b 1
    )
)