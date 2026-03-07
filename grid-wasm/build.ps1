# build.ps1 — Build WASM for Windows
Write-Host "=== Building WASM with wasm-pack ===" -ForegroundColor Cyan

# Ensure wasm-pack is installed
if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "wasm-pack not found. Installing..." -ForegroundColor Yellow
    cargo install wasm-pack
}

# Build
wasm-pack build --target web --out-dir ../grid-app/src/app/wasm --release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "=== Build successful ===" -ForegroundColor Green
    Write-Host "Output: grid-app/src/app/wasm/" -ForegroundColor Gray
} else {
    Write-Host "=== Build FAILED ===" -ForegroundColor Red
    exit 1
}
