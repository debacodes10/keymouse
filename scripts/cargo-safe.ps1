param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$repoRoot = Split-Path -Parent $PSScriptRoot
$explicitTargetDir = $env:KEYMOUSE_CARGO_TARGET_DIR
$isWindowsHost = $env:OS -eq "Windows_NT"

if (-not $explicitTargetDir -and $isWindowsHost) {
    $downloadsRoot = Join-Path $env:USERPROFILE "Downloads"
    $repoPath = [System.IO.Path]::GetFullPath($repoRoot)
    $downloadsPath = [System.IO.Path]::GetFullPath($downloadsRoot)

    if ($repoPath.StartsWith($downloadsPath, [System.StringComparison]::OrdinalIgnoreCase)) {
        $explicitTargetDir = Join-Path $env:LOCALAPPDATA "keymouse\cargo-target"
        Write-Host "Using safe Cargo target dir: $explicitTargetDir"
    }
}

if ($explicitTargetDir) {
    New-Item -ItemType Directory -Force -Path $explicitTargetDir | Out-Null
    $env:CARGO_TARGET_DIR = $explicitTargetDir
}

& cargo @CargoArgs
$exitCode = $LASTEXITCODE

if ($null -eq $exitCode) {
    $exitCode = 0
}

exit $exitCode
