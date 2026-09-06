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

if ([string]::IsNullOrWhiteSpace($ChecksumUrl)) {
    $ChecksumUrl = "$AssetUrl.sha256"
}

function Copy-OrDownload {
    param(
        [string]$Source,
        [string]$Destination
    )

    if ($Source.StartsWith("file://")) {
        Copy-Item -Force -LiteralPath ([System.Uri]$Source).LocalPath -Destination $Destination
    } elseif (Test-Path -LiteralPath $Source) {
        Copy-Item -Force -LiteralPath $Source -Destination $Destination
    } else {
        Invoke-WebRequest -Uri $Source -OutFile $Destination
    }
}

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
    if (-not $match.Success) {
        throw "assura installer: invalid SHA-256 checksum for $asset"
    }
    $expected = $match.Groups[1].Value.ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expected -ne $actual) {
        throw "assura installer: checksum mismatch for $asset"
    }

    Expand-Archive -Path $archive -DestinationPath $tmpDir -Force

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    Copy-Item -Force (Join-Path $tmpDir "assura.exe") (Join-Path $BinDir "assura.exe")
    Copy-Item -Force (Join-Path $tmpDir "assura-full.exe") (Join-Path $BinDir "assura-full.exe")

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
