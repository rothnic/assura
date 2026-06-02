param(
    [string]$AssuraBin = $env:ASSURA_BIN,
    [string]$WorkRoot = $env:ASSURA_SMOKE_DIR
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($AssuraBin)) {
    $AssuraBin = "assura"
}

if ([string]::IsNullOrWhiteSpace($WorkRoot)) {
    $WorkRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("assura-adoption-smoke-" + [System.Guid]::NewGuid())
    $cleanup = $true
} else {
    $cleanup = $false
}

New-Item -ItemType Directory -Force -Path $WorkRoot | Out-Null

function Invoke-Assura {
    & $AssuraBin @args
    if ($LASTEXITCODE -ne 0) {
        throw "assura command failed with exit ${LASTEXITCODE}: $AssuraBin $args"
    }
}

function Assert-JsonField {
    param(
        [string]$Path,
        [string]$Assertion
    )

    $data = Get-Content -Raw -LiteralPath $Path | ConvertFrom-Json
    switch ($Assertion) {
        "success_true" {
            $ok = $data.success -eq $true
        }
        "success_false_with_violations" {
            $ok = $data.success -eq $false -and $data.violations.Count -gt 0
        }
        "status_has_config" {
            $ok = -not [string]::IsNullOrWhiteSpace($data.config_path) -and
                $data.configured_directories -gt 0
        }
        default {
            throw "unknown assertion: $Assertion"
        }
    }

    if (-not $ok) {
        throw "$Path assertion failed: $Assertion"
    }
}

function Invoke-FailingCheck {
    param(
        [string]$Project,
        [string]$Output
    )

    & $AssuraBin check --format json $Project > $Output
    $status = $LASTEXITCODE
    if ($status -eq 0) {
        throw "expected failing check for $Project"
    }
    if ($status -ne 1) {
        throw "expected exit 1 for failing check, got $status"
    }
}

try {
    Write-Host "assura adoption smoke: binary=$AssuraBin"
    Invoke-Assura --version

    $emptyProject = Join-Path $WorkRoot "empty-project"
    New-Item -ItemType Directory -Force -Path $emptyProject | Out-Null
    Set-Content -LiteralPath (Join-Path $emptyProject "README.md") -Value "# Empty Project"

    Invoke-Assura init $emptyProject --no-git-hooks
    if (-not (Test-Path -LiteralPath (Join-Path $emptyProject ".assura/config.yml"))) {
        throw "init did not create .assura/config.yml"
    }

    $emptyStatus = Join-Path $WorkRoot "empty-status.json"
    & $AssuraBin status $emptyProject --format json > $emptyStatus
    if ($LASTEXITCODE -ne 0) { throw "status failed with exit $LASTEXITCODE" }
    Assert-JsonField $emptyStatus status_has_config

    $emptyPass = Join-Path $WorkRoot "empty-check-pass.json"
    & $AssuraBin check --format json $emptyProject > $emptyPass
    if ($LASTEXITCODE -ne 0) { throw "passing check failed with exit $LASTEXITCODE" }
    Assert-JsonField $emptyPass success_true

    Set-Content -LiteralPath (Join-Path $emptyProject "BadName.rs") -Value "fn main() {}"
    $emptyFail = Join-Path $WorkRoot "empty-check-fail.json"
    Invoke-FailingCheck $emptyProject $emptyFail
    Assert-JsonField $emptyFail success_false_with_violations

    $lslintProject = Join-Path $WorkRoot "ls-lint-project"
    New-Item -ItemType Directory -Force -Path $lslintProject | Out-Null
    Set-Content -LiteralPath (Join-Path $lslintProject ".ls-lint.yml") -Value @"
ls:
  .dir: kebab-case
  .rs: snake_case
ignore:
  - target
"@
    Set-Content -LiteralPath (Join-Path $lslintProject "good_name.rs") -Value "fn main() {}"

    Invoke-Assura migrate (Join-Path $lslintProject ".ls-lint.yml") `
        --output (Join-Path $lslintProject ".assura/config.yml")
    if (-not (Test-Path -LiteralPath (Join-Path $lslintProject ".assura/config.yml"))) {
        throw "migrate did not create .assura/config.yml"
    }

    $lslintStatus = Join-Path $WorkRoot "lslint-status.json"
    & $AssuraBin status $lslintProject --format json > $lslintStatus
    if ($LASTEXITCODE -ne 0) { throw "LS-Lint status failed with exit $LASTEXITCODE" }
    Assert-JsonField $lslintStatus status_has_config

    $lslintPass = Join-Path $WorkRoot "lslint-check-pass.json"
    & $AssuraBin check --format json $lslintProject > $lslintPass
    if ($LASTEXITCODE -ne 0) { throw "LS-Lint migrated check failed with exit $LASTEXITCODE" }
    Assert-JsonField $lslintPass success_true

    Write-Host "assura adoption smoke: pass; evidence=$WorkRoot"
} finally {
    if ($cleanup) {
        Remove-Item -Recurse -Force $WorkRoot -ErrorAction SilentlyContinue
    }
}
