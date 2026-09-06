param(
    [string]$Repo = $env:ASSURA_REPO,
    [string]$Version = $env:ASSURA_VERSION,
    [string]$BinDir = $env:BIN_DIR,
    [string]$AssetUrl = $env:ASSURA_ASSET_URL,
    [string]$ChecksumUrl = $env:ASSURA_CHECKSUM_URL
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($Repo)) {
    $Repo = "rothnic/assura"
}

function Copy-OrDownload {
    param([string]$Source, [string]$Destination)
    if ($Source.StartsWith("file://")) {
        Copy-Item -Force -LiteralPath ([System.Uri]$Source).LocalPath -Destination $Destination
    } elseif (Test-Path -LiteralPath $Source) {
        Copy-Item -Force -LiteralPath $Source -Destination $Destination
    } else { Invoke-WebRequest -Uri $Source -OutFile $Destination }
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = "latest"
}

if ([string]::IsNullOrWhiteSpace($BinDir)) {
    $BinDir = Join-Path $env:LOCALAPPDATA "Programs\Assura\bin"
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
if ($arch -ne [System.Runtime.InteropServices.Architecture]::X64) {
    throw "assura installer: unsupported Windows architecture: $arch"
}

$asset = "assura-windows-amd64.zip"
if ([string]::IsNullOrWhiteSpace($AssetUrl)) {
    if ($Version -eq "latest") {
        $AssetUrl = "https://github.com/$Repo/releases/latest/download/$asset"
    } else {
        $AssetUrl = "https://github.com/$Repo/releases/download/$Version/$asset"
    }
}
if ([string]::IsNullOrWhiteSpace($ChecksumUrl)) { $ChecksumUrl = "$AssetUrl.sha256" }

$tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("assura-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

try {
    $archive = Join-Path $tmpDir $asset
    $checksum = Join-Path $tmpDir "$asset.sha256"
    Write-Host "Downloading $AssetUrl"
    Copy-OrDownload $AssetUrl $archive
    Write-Host "Verifying $asset"
    Copy-OrDownload $ChecksumUrl $checksum
    $match = [regex]::Match((Get-Content -Raw -LiteralPath $checksum), '(?im)^\s*([0-9a-f]{64})\b')
    if (-not $match.Success) { throw "assura installer: invalid SHA-256 checksum for $asset" }
    $expected = $match.Groups[1].Value.ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expected -ne $actual) { throw "assura installer: checksum mismatch for $asset" }

    Expand-Archive -Path $archive -DestinationPath $tmpDir -Force
    $newAssura = Join-Path $tmpDir "assura.exe"
    $newFull = Join-Path $tmpDir "assura-full.exe"
    if (-not (Test-Path -LiteralPath $newAssura) -or -not (Test-Path -LiteralPath $newFull)) {
        throw "assura installer: archive must contain assura.exe and assura-full.exe"
    }

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    $stageDir = Join-Path $BinDir (".assura-stage-" + [System.Guid]::NewGuid())
    $backupDir = Join-Path $BinDir (".assura-backup-" + [System.Guid]::NewGuid())
    New-Item -ItemType Directory -Force -Path $stageDir, $backupDir | Out-Null
    Copy-Item -Force $newAssura (Join-Path $stageDir "assura.exe")
    Copy-Item -Force $newFull (Join-Path $stageDir "assura-full.exe")
    $destAssura = Join-Path $BinDir "assura.exe"
    $destFull = Join-Path $BinDir "assura-full.exe"
    $backupAssura = $false; $backupFull = $false; $newAssuraInstalled = $false; $newFullInstalled = $false
    try {
        if (Test-Path -LiteralPath $destAssura) { Move-Item -Force $destAssura (Join-Path $backupDir "assura.exe"); $backupAssura = $true }
        if ($env:ASSURA_TEST_FAIL_DURING_SECOND_BACKUP -eq "1") { throw "injected second backup failure" }
        if (Test-Path -LiteralPath $destFull) { Move-Item -Force $destFull (Join-Path $backupDir "assura-full.exe"); $backupFull = $true }
        Move-Item -Force (Join-Path $stageDir "assura.exe") $destAssura
        $newAssuraInstalled = $true
        if ($env:ASSURA_TEST_FAIL_AFTER_FIRST_REPLACE -eq "1") {
            throw "injected replacement failure"
        }
        Move-Item -Force (Join-Path $stageDir "assura-full.exe") $destFull
        $newFullInstalled = $true
    } catch {
        if ($newAssuraInstalled) { Remove-Item -Force $destAssura -ErrorAction SilentlyContinue }
        if ($newFullInstalled) { Remove-Item -Force $destFull -ErrorAction SilentlyContinue }
        if ($backupAssura) { Move-Item -Force (Join-Path $backupDir "assura.exe") $destAssura }
        if ($backupFull) { Move-Item -Force (Join-Path $backupDir "assura-full.exe") $destFull }
        throw "assura installer: replacement failed; restored previous installation: $($_.Exception.Message)"
    } finally { Remove-Item -Recurse -Force $stageDir, $backupDir -ErrorAction SilentlyContinue }

    Write-Host "Installed assura to $(Join-Path $BinDir 'assura.exe')"
    $pathEntries = @([Environment]::GetEnvironmentVariable("PATH", "User") -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($pathEntries -notcontains $BinDir) {
        [Environment]::SetEnvironmentVariable("PATH", ($pathEntries + $BinDir) -join ";", "User")
        $env:PATH = "$BinDir;$env:PATH"
        Write-Host "Added $BinDir to your user PATH. Restart other terminals to use assura."
    }
} finally {
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}
