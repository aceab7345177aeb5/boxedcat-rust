# BoxedCat Build Script
# Compiles Rust core + C++ GUI into a single executable
# Requires: MinGW 11.2.0 (ABI-compatible with Qt 6.7.3)

param(
    [switch]$Run,
    [switch]$Clean,
    [switch]$RustOnly
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Target = Join-Path $Root "target"

$MinGW11 = "C:\Users\Velocity\Documents\Default Project\mingw11\mingw64\bin"
$Qt = "C:\Qt\6.7.3\mingw_64"
$Gpp = Join-Path $MinGW11 "g++.exe"

if (!(Test-Path $Gpp)) { throw "MinGW 11.2.0 not found at $MinGW11" }
if (!(Test-Path $Qt)) { throw "Qt 6.7.3 not found at $Qt" }

$env:PATH = "$MinGW11;$Qt\bin;$env:PATH"

function Invoke-Cleanup {
    Write-Host "Cleaning..." -ForegroundColor Yellow
    if (Test-Path $Target) { Remove-Item $Target -Recurse -Force }
    Write-Host "Clean." -ForegroundColor Green
}

function Build-Rust {
    Write-Host "Building Rust core modules..." -ForegroundColor Cyan
    $env:RUSTFLAGS = "-C target-feature=+crt-static"
    Push-Location $Root
    cargo build --release --target x86_64-pc-windows-gnu 2>&1 | ForEach-Object { Write-Host $_ }
    if ($LASTEXITCODE -ne 0) { throw "Rust build failed" }
    Pop-Location

    $lib = Join-Path $Target "x86_64-pc-windows-gnu\release\libboxedcat.a"
    if (!(Test-Path $lib)) { throw "Rust staticlib not found" }
    Write-Host "Rust staticlib: $lib" -ForegroundColor Green
}

function Build-GUI {
    Write-Host "Linking C++ GUI with Rust core..." -ForegroundColor Cyan

    $cppFile = Join-Path $Root "cpp\main.cpp"
    $exeFile = Join-Path $Target "boxedcat.exe"
    $releaseDir = Join-Path $Target "x86_64-pc-windows-gnu\release"
    $qrcFile = Join-Path $Root "app.qrc"
    $resCpp = Join-Path $Target "app_generated.cpp"
    $resObj = Join-Path $Target "app_generated.o"
    $rcc = Join-Path $Qt "bin\rcc.exe"

    if (!(Test-Path $cppFile)) { throw "C++ source not found: $cppFile" }
    New-Item -ItemType Directory -Force -Path $Target | Out-Null

    # Compile Qt resources
    if (Test-Path $qrcFile) {
        Write-Host "Compiling Qt resources..." -ForegroundColor Cyan
        & $rcc $qrcFile -o $resCpp 2>&1 | ForEach-Object { Write-Host $_ }
        if ($LASTEXITCODE -ne 0) { throw "rcc failed" }
        & $Gpp -c $resCpp -o $resObj -I"$Qt\include" -I"$Qt\include\QtCore" 2>&1 | ForEach-Object { Write-Host $_ }
        if ($LASTEXITCODE -ne 0) { throw "Resource compile failed" }
    }

    $linkFiles = @($cppFile)
    if (Test-Path $resObj) { $linkFiles += $resObj }

    & $Gpp -std=c++17 -O2 -mwindows -o $exeFile @linkFiles `
        -I"$Qt\include" -I"$Qt\include\QtCore" -I"$Qt\include\QtGui" -I"$Qt\include\QtWidgets" `
        -L"$releaseDir" -L"$Qt\lib" `
        -l:libboxedcat.a `
        -lQt6Core -lQt6Gui -lQt6Widgets `
        -lws2_32 -liphlpapi -luser32 -ladvapi32 -lkernel32 -lshell32 -lole32 -luuid -lcomdlg32 -lgdi32 -lntdll -lbcrypt -lwinhttp 2>&1

    if ($LASTEXITCODE -ne 0) { throw "C++ link failed" }

    $dllDir = Join-Path $Root "dlls"
    Copy-Item "$dllDir\libgcc_s_seh-1.dll" $Target -Force -ErrorAction SilentlyContinue
    Copy-Item "$dllDir\libwinpthread-1.dll" $Target -Force -ErrorAction SilentlyContinue
    Copy-Item "$dllDir\libstdc++-6.dll" $Target -Force -ErrorAction SilentlyContinue

    $assetsDir = Join-Path $Root "assets"
    $targetAssets = Join-Path $Target "assets"
    if (Test-Path $assetsDir) {
        if (!(Test-Path $targetAssets)) { New-Item -ItemType Directory -Force -Path $targetAssets | Out-Null }
        Copy-Item "$assetsDir\*" $targetAssets -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "Build complete: $exeFile" -ForegroundColor Green
}

function Invoke-Run {
    $exe = Join-Path $Target "boxedcat.exe"
    if (!(Test-Path $exe)) { throw "boxedcat.exe not found. Build first." }
    Write-Host "Launching BoxedCat..." -ForegroundColor Cyan
    $proc = Start-Process $exe -PassThru -WorkingDirectory $Target
    Write-Host "Running (PID: $($proc.Id))" -ForegroundColor Green
}

try {
    if ($Clean) { Invoke-Cleanup; exit 0 }
    if (!$RustOnly) {
        Build-Rust
        Build-GUI
    } else {
        Build-Rust
    }
    if ($Run) { Invoke-Run }
} catch {
    Write-Host "ERROR: $_" -ForegroundColor Red
    exit 1
}
