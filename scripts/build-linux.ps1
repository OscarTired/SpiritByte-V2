$ErrorActionPreference = 'Stop'
$projectPath = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (-not (Test-Path -LiteralPath (Join-Path $projectPath 'core/Cargo.toml'))) {
    throw 'core/Cargo.toml was not found in the desktop repository.'
}

$dockerPlatform = docker info --format '{{.OSType}}'
if ($LASTEXITCODE -ne 0 -or $dockerPlatform -ne 'linux') {
    throw 'Start Docker Desktop with Linux containers, then run this script again.'
}

docker build --tag spiritbyte-linux-builder (Join-Path $PSScriptRoot 'linux')
if ($LASTEXITCODE -ne 0) { throw 'The Linux builder image could not be built.' }

$bundlePath = Join-Path $projectPath 'src-tauri/target/release/bundle'
New-Item -ItemType Directory -Force -Path $bundlePath | Out-Null
docker run --rm `
    --mount "type=bind,source=$projectPath,target=/source/desktop,readonly" `
    --mount "type=bind,source=$bundlePath,target=/output" `
    --mount 'type=volume,source=spiritbyte-linux-cargo-registry,target=/root/.cargo/registry' `
    --mount 'type=volume,source=spiritbyte-linux-target,target=/linux-target' `
    spiritbyte-linux-builder
if ($LASTEXITCODE -ne 0) { throw 'The Linux build failed.' }

Write-Host "Linux packages are available in $bundlePath\appimage and $bundlePath\deb."
