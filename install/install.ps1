$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

if ($args.Count -ne 1) {
    throw 'Usage: .\install.ps1 --node|--server|--cli'
}

$installKind = $args[0]
$repository = '1TheCrazy/Tunnel'

switch ($installKind) {
    '--node' {
        $assetName = 'tunnel_node'
        $binaryName = 'node.exe'
        $configName = 'node.toml'
        break
    }
    '--server' {
        $assetName = 'tunnel_server'
        $binaryName = 'server.exe'
        $configName = 'server.toml'
        break
    }
    '--cli' {
        $assetName = 'tunnel_client_cli'
        $binaryName = 'tunnel.exe'
        $configName = $null
        break
    }
    default {
        throw 'Usage: .\install.ps1 --node|--server|--cli'
    }
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

if ($installKind -eq '--cli') {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw 'Run the CLI installer from an elevated PowerShell session.'
    }
}

if ($null -ne $configName) {
    $configSource = Join-Path (Get-Location) $configName
    if (-not (Test-Path -LiteralPath $configSource -PathType Leaf)) {
        throw "Expected configuration file at $configSource"
    }
}

if ($installKind -ne '--server') {
    if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
        throw 'winget is required to install WireGuard.'
    }

    & winget install --id WireGuard.WireGuard --exact --source winget `
        --silent `
        --accept-package-agreements `
        --accept-source-agreements | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "WireGuard installation failed with exit code $LASTEXITCODE."
    }
}

if ($installKind -eq '--cli') {
    $installDirectory = 'C:\Program Files\Tunnel'
    $destination = Join-Path $installDirectory $binaryName
}
else {
    $destination = Join-Path (Get-Location) $binaryName
}

$downloadUrl = "https://github.com/$repository/releases/latest/download/$assetName`_$architecture.exe"
$temporaryBinary = [System.IO.Path]::GetTempFileName()
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $temporaryBinary
    if ((Get-Item -LiteralPath $temporaryBinary).Length -eq 0) {
        throw "Downloaded $binaryName is empty."
    }

    if ($installKind -eq '--cli') {
        New-Item -ItemType Directory -Path $installDirectory -Force | Out-Null
    }
    Move-Item -LiteralPath $temporaryBinary -Destination $destination -Force
}
finally {
    if (Test-Path -LiteralPath $temporaryBinary) {
        Remove-Item -LiteralPath $temporaryBinary -Force
    }
}

if ($null -ne $configName) {
    $configDirectory = Join-Path $env:APPDATA '1thecrazy\tunnel'
    New-Item -ItemType Directory -Path $configDirectory -Force | Out-Null
    Copy-Item -LiteralPath $configSource -Destination (Join-Path $configDirectory $configName) -Force
    Remove-Item -LiteralPath $configSource -Force
}

if ($installKind -eq '--cli') {
    $machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $pathEntries = @($machinePath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    $directoryInPath = $pathEntries | Where-Object {
        $_.TrimEnd('\') -ieq $installDirectory.TrimEnd('\')
    }
    if (@($directoryInPath).Count -eq 0) {
        [Environment]::SetEnvironmentVariable('Path', (($pathEntries + $installDirectory) -join ';'), 'Machine')
    }
}

Write-Output 'Tunnel was successfully installed'
