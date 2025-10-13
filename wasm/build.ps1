# Build script for WASM target (Windows PowerShell)
# Requires wasm-pack: cargo install wasm-pack

Write-Host "Building Butabuti for WebAssembly..." -ForegroundColor Cyan

# Build for web target
# wasm-pack will create the output in pkg/ directory by default
wasm-pack build --target web --features wasm

if ($LASTEXITCODE -eq 0) {
    # Move pkg to wasm/pkg if build succeeded
    if (Test-Path "pkg") {
        if (Test-Path "wasm/pkg") {
            Remove-Item -Recurse -Force "wasm/pkg"
        }
        Move-Item "pkg" "wasm/pkg"
    }

    Write-Host "`nBuild complete! Output in wasm/pkg/" -ForegroundColor Green
    Write-Host "`nTo test locally:" -ForegroundColor Yellow
    Write-Host "  cd wasm"
    Write-Host "  python -m http.server 8000"
    Write-Host "  Open http://localhost:8000 in browser"
} else {
    Write-Host "`nBuild failed!" -ForegroundColor Red
    exit 1
}
