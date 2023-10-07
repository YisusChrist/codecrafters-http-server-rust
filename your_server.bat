@echo off
setlocal

set TARGET_DIR=%TEMP%\codecrafters-http-server-target
set MANIFEST_PATH=%~dp0Cargo.toml

echo Running your server...

REM Check if the target directory exists; create it if not
if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"

REM Run the server with the cargo command
cargo run --quiet --release --target-dir="%TARGET_DIR%" --manifest-path="%MANIFEST_PATH%" -- %*

endlocal
