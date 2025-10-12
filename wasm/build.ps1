# Build script for WASM target (Windows PowerShell)
# Requires wasm-pack: cargo install wasm-pack

Write-Host "Building ButaButi for WebAssembly..." -ForegroundColor Cyan

# Build for web target
wasm-pack build --target web --features wasm --out-dir wasm/pkg ..

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBuild complete! Output in wasm/pkg/" -ForegroundColor Green
    Write-Host "`nTo test locally:" -ForegroundColor Yellow
    Write-Host "  cd wasm"
    Write-Host "  python -m http.server 8000"
    Write-Host "  Open http://localhost:8000 in browser"
} else {
    Write-Host "`nBuild failed!" -ForegroundColor Red
    exit 1
}
