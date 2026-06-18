@echo off
REM Build the Pledge and Crown Tauri desktop shell with the MSVC toolchain.
REM The repo default Rust toolchain is GNU, whose linker cannot emit
REM webview2's huge export table ("export ordinal too large"). Tauri on
REM Windows needs MSVC. This script locates VS via vswhere, loads the x64
REM dev environment (so the MSVC linker finds the CRT + Windows SDK libs),
REM then builds the standalone mobile/src-tauri crate with the MSVC
REM toolchain. Pass-through args go to cargo, e.g. build-desktop.bat --release
setlocal
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" (
  echo [error] vswhere not found at "%VSWHERE%" - is Visual Studio installed?
  exit /b 1
)
for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -property installationPath`) do set "VSPATH=%%i"
if not defined VSPATH (
  echo [error] no Visual Studio installation found via vswhere
  exit /b 1
)
call "%VSPATH%\VC\Auxiliary\Build\vcvars64.bat" || exit /b 1
echo [ok] loaded MSVC x64 environment from "%VSPATH%"
cargo +stable-x86_64-pc-windows-msvc build --manifest-path "%~dp0src-tauri\Cargo.toml" %*
