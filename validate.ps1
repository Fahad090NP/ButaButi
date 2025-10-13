# Validate Butabuti Project
# Runs all quality checks: build, test, lint, format, docs

Write-Host "`nButabuti Validation" -ForegroundColor Cyan
Write-Host "==================`n" -ForegroundColor Cyan

$failed = 0

# Build
Write-Host "[1/5] Build..." -ForegroundColor Yellow
if (!(cargo build 2>&1 | Select-String "Finished")) { $failed++ }
else { Write-Host "  OK" -ForegroundColor Green }

# Test
Write-Host "[2/5] Tests..." -ForegroundColor Yellow
$result = cargo test --lib 2>&1 | Select-String "test result:"
Write-Host "  $result" -ForegroundColor $(if ($LASTEXITCODE -eq 0) { "Green" } else { $failed++; "Red" })

# Clippy
Write-Host "[3/5] Clippy..." -ForegroundColor Yellow
if (!(cargo clippy -- -D warnings 2>&1 | Select-String "Finished")) { $failed++ }
else { Write-Host "  OK" -ForegroundColor Green }

# Format
Write-Host "[4/5] Format..." -ForegroundColor Yellow
cargo fmt -- --check 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { $failed++; Write-Host "  Run 'cargo fmt'" -ForegroundColor Yellow }
else { Write-Host "  OK" -ForegroundColor Green }

# Docs
Write-Host "[5/5] Docs..." -ForegroundColor Yellow
if (!(cargo doc --no-deps 2>&1 | Select-String "Finished|Documenting")) { $failed++ }
else { Write-Host "  OK" -ForegroundColor Green }

# Summary
Write-Host "`n==================" -ForegroundColor Cyan
if ($failed -eq 0) {
    Write-Host "ALL PASSED`n" -ForegroundColor Green
    exit 0
} else {
    Write-Host "$failed FAILED`n" -ForegroundColor Red
    exit 1
}
