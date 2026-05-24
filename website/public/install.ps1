param(
    [string]$Repo = $env:ASSURA_REPO,
    [string]$Version = $env:ASSURA_VERSION,
    [string]$BinDir = $env:BIN_DIR,
    [string]$AssetUrl = $env:ASSURA_ASSET_URL
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

$tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("assura-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

try {
    $archive = Join-Path $tmpDir $asset
    Write-Host "Downloading $AssetUrl"
    if ($AssetUrl.StartsWith("file://")) {
        $localPath = ([System.Uri]$AssetUrl).LocalPath
        Copy-Item -Force -LiteralPath $localPath $archive
    } elseif (Test-Path -LiteralPath $AssetUrl) {
        Copy-Item -Force -LiteralPath $AssetUrl $archive
    } else {
        Invoke-WebRequest -Uri $AssetUrl -OutFile $archive
    }

    Expand-Archive -Path $archive -DestinationPath $tmpDir -Force

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    Copy-Item -Force (Join-Path $tmpDir "assura.exe") (Join-Path $BinDir "assura.exe")
    Copy-Item -Force (Join-Path $tmpDir "assura-full.exe") (Join-Path $BinDir "assura-full.exe")

    Write-Host "Installed assura to $(Join-Path $BinDir 'assura.exe')"
    $pathEntries = [Environment]::GetEnvironmentVariable("PATH", "User") -split ";"
    if ($pathEntries -notcontains $BinDir) {
        Write-Host "Add $BinDir to your user PATH before running assura."
    }
} finally {
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}
