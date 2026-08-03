$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

if ($args.Count -ne 0) {
    throw 'This installer does not accept arguments.'
}

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this installer from an elevated PowerShell session.'
}

$repository = '1TheCrazy/Tunnel'
$installDirectory = 'C:\Program Files\Tunnel'
$destination = Join-Path $installDirectory 'tunnel.exe'

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

& winget install --id WireGuard.WireGuard --exact --source winget `
    --silent `
    --accept-package-agreements `
    --accept-source-agreements | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "WireGuard installation failed with exit code $LASTEXITCODE."
}

$downloadUrl = "https://github.com/$repository/releases/latest/download/tunel_client_cli_$architecture.exe"
$temporaryBinary = [System.IO.Path]::GetTempFileName()
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $temporaryBinary
    if ((Get-Item -LiteralPath $temporaryBinary).Length -eq 0) {
        throw 'Downloaded CLI binary is empty.'
    }

    New-Item -ItemType Directory -Path $installDirectory -Force | Out-Null
    Move-Item -LiteralPath $temporaryBinary -Destination $destination -Force
}
finally {
    if (Test-Path -LiteralPath $temporaryBinary) {
        Remove-Item -LiteralPath $temporaryBinary -Force
    }
}

$machinePath = [Environment]::GetEnvironmentVariable('Path', 'Machine')
$pathEntries = @($machinePath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
$directoryInPath = $pathEntries | Where-Object {
    $_.TrimEnd('\') -ieq $installDirectory.TrimEnd('\')
}
if (@($directoryInPath).Count -eq 0) {
    [Environment]::SetEnvironmentVariable('Path', (($pathEntries + $installDirectory) -join ';'), 'Machine')
}

Write-Output 'Tunnel was successfully installed'
