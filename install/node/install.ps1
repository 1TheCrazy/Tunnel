$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# Run from a directory containing node.toml. The node executable is installed
# into that same directory as .\node.
if ($args.Count -ne 0) {
    throw 'This installer does not accept arguments.'
}

$repository = '1TheCrazy/Tunnel'
$configSource = Join-Path (Get-Location) 'node.toml'
$destination = Join-Path (Get-Location) 'node.exe'

if (-not (Test-Path -LiteralPath $configSource -PathType Leaf)) {
    throw "Expected configuration file at $configSource"
}

if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    throw 'winget is required to install WireGuard.'
}

$machineArchitecture = $env:PROCESSOR_ARCHITEW6432
if ([string]::IsNullOrWhiteSpace($machineArchitecture)) {
    $machineArchitecture = $env:PROCESSOR_ARCHITECTURE
}

switch ($machineArchitecture.ToUpperInvariant()) {
    'AMD64' {
        $architecture = 'x86_64'
        break
    }
    'ARM64' {
        $architecture = 'aarch64'
        break
    }
    'ARM' {
        $architecture = 'armv7'
        break
    }
    default {
        throw "Unsupported CPU architecture: $machineArchitecture. Supported: x86_64, aarch64, armv7."
    }
}

$downloadUrl = "https://github.com/$repository/releases/latest/download/tunnel_node_$architecture.exe"

& winget install --id WireGuard.WireGuard --exact --source winget `
    --silent `
    --accept-package-agreements `
    --accept-source-agreements | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "WireGuard installation failed with exit code $LASTEXITCODE."
}

$temporaryBinary = "$destination.download"
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $temporaryBinary
    if ((Get-Item -LiteralPath $temporaryBinary).Length -eq 0) {
        throw 'Downloaded node binary is empty.'
    }

    Move-Item -LiteralPath $temporaryBinary -Destination $destination -Force
}
finally {
    if (Test-Path -LiteralPath $temporaryBinary) {
        Remove-Item -LiteralPath $temporaryBinary -Force
    }
}

$configDirectory = Join-Path $env:APPDATA '1thecrazy\tunnel'
New-Item -ItemType Directory -Path $configDirectory -Force | Out-Null
Copy-Item -LiteralPath $configSource -Destination (Join-Path $configDirectory 'node.toml') -Force
Remove-Item -LiteralPath $configSource -Force

Write-Output 'Tunnel was successfully installed'
