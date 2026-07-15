[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$target = 'x86_64-pc-windows-msvc'

Push-Location $repositoryRoot
try {
    $metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) {
        throw 'Unable to read package metadata.'
    }
    $version = ($metadata.packages | Where-Object name -eq 'winget-automation').version

    cargo build --release --target $target
    if ($LASTEXITCODE -ne 0) {
        throw 'The release build failed.'
    }

    $compiler = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    $compilerPath = if ($compiler) { $compiler.Source } else { $null }
    if (-not $compilerPath) {
        $candidate = Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'
        if (Test-Path -LiteralPath $candidate) {
            $compilerPath = $candidate
        }
    }
    if (-not $compilerPath) {
        throw 'Inno Setup 6 is required. Install it, then rerun installer\build.ps1.'
    }

    & $compilerPath "/DAppVersion=$version" (Join-Path $PSScriptRoot 'winget-automation.iss')
    if ($LASTEXITCODE -ne 0) {
        throw 'The installer build failed.'
    }
}
finally {
    Pop-Location
}
