param(
    [switch]$Publish
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Title,
        [Parameter(Mandatory = $true)]
        [string[]]$Command
    )

    Write-Host ""
    Write-Host "==> $Title" -ForegroundColor Cyan
    Write-Host ($Command -join " ")
    & $Command[0] $Command[1..($Command.Length - 1)]
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $($Command -join ' ')"
    }
}

$cargoToml = Join-Path $repoRoot "crates\sanos\Cargo.toml"
$versionLine = Select-String -Path $cargoToml -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1
if (-not $versionLine) {
    throw "Unable to determine sanos version from $cargoToml"
}
$version = $versionLine.Matches[0].Groups[1].Value

Write-Host "Preparing sanos release v$version from $repoRoot" -ForegroundColor Green

Invoke-Step -Title "Run default-feature tests" -Command @("cargo", "test", "-p", "sanos")
Invoke-Step -Title "Run no-default-feature tests" -Command @("cargo", "test", "-p", "sanos", "--no-default-features")

$previousRustdocFlags = $env:RUSTDOCFLAGS
try {
    $env:RUSTDOCFLAGS = "-D warnings"
    Invoke-Step -Title "Build docs with warnings denied" -Command @("cargo", "doc", "-p", "sanos", "--no-deps")
}
finally {
    $env:RUSTDOCFLAGS = $previousRustdocFlags
}

Invoke-Step -Title "Package sanos" -Command @("cargo", "package", "-p", "sanos")

if ($Publish) {
    Invoke-Step -Title "Publish sanos to crates.io" -Command @("cargo", "publish", "-p", "sanos")
}
else {
    Write-Host ""
    Write-Host "Validation complete. Re-run with -Publish to publish sanos v$version." -ForegroundColor Yellow
}
