<#
.SYNOPSIS
    Installs my-service-bus and my-service-bus-persistence on one Windows machine as two native
    Windows services, reachable from this machine only - or, run again, replaces their exes.

.DESCRIPTION
    Run it from an ELEVATED PowerShell. What it makes:

      <Root>\persistence\bin\     my-sb-persistence.exe + wwwroot\       service: read
      <Root>\persistence\config\  settings.yaml, logs\                   service: read; logs\ write
      <Root>\bus\bin\             my-service-bus-main-node.exe + wwwroot\
      <Root>\bus\config\          settings.yaml, logs\
      <Data>\                     the messages                           persistence: write

    <Root> and <Data> are closed: owned by the administrators, open to the system and the
    administrators only. Each service runs as its own virtual account (NT SERVICE\<name>), which
    its own `--install` gives exactly the folders above. Both listen on 127.0.0.1 only
    (`listen_ip` in their settings): 6421 (bus TCP), 6123 (bus HTTP + UI), 7124 (persistence
    gRPC), 7123 (persistence HTTP + UI). Neither asks who is calling, so the loopback is the
    boundary; an inbound firewall rule blocks the four ports as well, should a setting ever be
    widened. The bus depends on the persistence: Windows starts it after, and stops it before.

    First run: makes the folders, writes both settings files (a random delete key for the
    persistence), installs and starts both services.
    Later runs: the settings files are kept as they are; the services are stopped, their exes and
    wwwroot replaced, and started again.

.PARAMETER Root
    Where the services live. A folder of its own in the root of a drive: `--install` refuses a
    folder whose parents plain users can rename.

.PARAMETER Data
    Where the persistence keeps the messages.

.PARAMETER BusBuild / PersistenceBuild
    The folders of the two repositories, built with `cargo build --release`.
#>
[CmdletBinding()]
param(
    [string]$Root = 'C:\my-sb',
    [string]$Data = 'D:\my-sb-data',
    [string]$BusBuild = (Resolve-Path "$PSScriptRoot\..\..").Path,
    [string]$PersistenceBuild = (Join-Path (Split-Path (Resolve-Path "$PSScriptRoot\..\..").Path -Parent) 'my-service-bus-persistence')
)

$ErrorActionPreference = 'Stop'

$BusService = 'my-service-bus'
$PersistenceService = 'my-service-bus-persistence'
$Ports = 6421, 6123, 7123, 7124

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) { throw 'Run this from an elevated PowerShell (Run as administrator).' }

$busExe = Join-Path $BusBuild 'target\release\my-service-bus-main-node.exe'
$persistenceExe = Join-Path $PersistenceBuild 'target\release\my-sb-persistence.exe'
foreach ($file in $busExe, $persistenceExe) {
    if (-not (Test-Path $file)) { throw "$file is not built: cargo build --release in its repository first." }
}

function Close-Folder([string]$folder) {
    # Owned by the administrators, not inheriting, open to the system and the administrators only;
    # what is inside takes its access from the folder. SIDs, as group names differ by language.
    New-Item -ItemType Directory -Force $folder | Out-Null
    & icacls $folder /setowner '*S-1-5-32-544' /T /C /Q | Out-Null
    & icacls $folder /inheritance:r /grant:r '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-544:(OI)(CI)F' /Q | Out-Null
    if (Get-ChildItem -Force $folder) { & icacls "$folder\*" /reset /T /C /Q | Out-Null }
    if ($LASTEXITCODE -ne 0) { throw "icacls failed on $folder" }
}

function Copy-Build([string]$exe, [string]$repo, [string]$bin) {
    New-Item -ItemType Directory -Force $bin | Out-Null
    Copy-Item -Force $exe $bin
    if (Test-Path "$bin\wwwroot") { Remove-Item -Recurse -Force "$bin\wwwroot" }
    Copy-Item -Recurse (Join-Path $repo 'wwwroot') "$bin\wwwroot"
}

function Test-Service([string]$name) { [bool](Get-Service -Name $name -ErrorAction SilentlyContinue) }

function Wait-Listening([int[]]$ports) {
    for ($i = 0; $i -lt 60; $i++) {
        $listening = @(Get-NetTCPConnection -State Listen -LocalPort $ports -ErrorAction SilentlyContinue)
        if (($listening.LocalPort | Sort-Object -Unique).Count -eq $ports.Count) { return $listening }
        Start-Sleep -Milliseconds 500
    }
    throw "ports $($ports -join ', ') do not listen after 30 s - see the logs under $Root"
}

$persistenceSettings = "$Root\persistence\config\settings.yaml"
$busSettings = "$Root\bus\config\settings.yaml"

if ((Test-Service $BusService) -or (Test-Service $PersistenceService)) {
    if (-not ((Test-Service $BusService) -and (Test-Service $PersistenceService))) {
        throw "Only one of $BusService and $PersistenceService is installed: uninstall it (tools\windows\uninstall.ps1) and run this again."
    }

    "== update: stopping $BusService, then $PersistenceService"
    Stop-Service $BusService
    Stop-Service $PersistenceService

    Copy-Build $persistenceExe $PersistenceBuild "$Root\persistence\bin"
    Copy-Build $busExe $BusBuild "$Root\bus\bin"
}
else {
    "== install: closing $Root and $Data"
    Close-Folder $Root
    Close-Folder $Data

    New-Item -ItemType Directory -Force "$Root\persistence\config", "$Root\bus\config" | Out-Null
    Copy-Build $persistenceExe $PersistenceBuild "$Root\persistence\bin"
    Copy-Build $busExe $BusBuild "$Root\bus\bin"

    if (-not (Test-Path $persistenceSettings)) {
        $bytes = New-Object byte[] 32
        [Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
        $deleteKey = [Convert]::ToBase64String($bytes)
        $dataYaml = $Data.Replace('\', '/')
        [IO.File]::WriteAllText($persistenceSettings, @"
# my-service-bus-persistence on Windows - tools\windows\install.ps1 of my-service-bus.
data: "$dataYaml"
max_response_records_amount: 500
delete_topic_secret_key: "$deleteKey"
# gRPC 7124 and HTTP 7123: this machine only. Neither asks who is calling.
listen_ip: "127.0.0.1"
"@)
    }

    if (-not (Test-Path $busSettings)) {
        [IO.File]::WriteAllText($busSettings, @'
# my-service-bus on Windows - tools\windows\install.ps1 of my-service-bus.
persistence_grpc_url: "http://127.0.0.1:7124"
queue_gc_timeout: "00:00:20"
debug_mode: false
max_delivery_size: 4194304
auto_create_topic_on_publish: true
auto_create_topic_on_subscribe: true
# TCP 6421 and HTTP 6123: this machine only. Neither asks who is calling; whoever is to reach the
# bus from elsewhere comes through a tunnel to 127.0.0.1.
listen_ip: "127.0.0.1"
'@)
    }

    # What was written above inherits the closed folder; reset makes sure of it.
    & icacls "$Root\*" /reset /T /C /Q | Out-Null

    "== install: $PersistenceService"
    & "$Root\persistence\bin\my-sb-persistence.exe" --install --settings $persistenceSettings
    if ($LASTEXITCODE -ne 0) { throw "--install of $PersistenceService failed" }

    "== install: $BusService"
    & "$Root\bus\bin\my-service-bus-main-node.exe" --install --settings $busSettings
    if ($LASTEXITCODE -ne 0) { throw "--install of $BusService failed" }

    # The bus hands its messages to the persistence: started after it, stopped before it.
    & sc.exe config $BusService depend= $PersistenceService | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "sc.exe config $BusService depend= $PersistenceService failed" }

    $rule = 'my-service-bus: no inbound connections'
    if (-not (Get-NetFirewallRule -DisplayName $rule -ErrorAction SilentlyContinue)) {
        New-NetFirewallRule -DisplayName $rule -Direction Inbound -Protocol TCP -LocalPort $Ports -Action Block `
            -Description 'my-service-bus and its persistence listen on 127.0.0.1 only; this blocks their ports from the network should a setting ever be widened. Loopback is not filtered.' | Out-Null
    }
}

"== start: $PersistenceService, then $BusService"
Start-Service $PersistenceService
Wait-Listening 7123, 7124 | Out-Null
Start-Service $BusService
$listening = Wait-Listening $Ports

"== listening (every address must be 127.0.0.1)"
$listening | Sort-Object LocalPort | Format-Table LocalAddress, LocalPort, OwningProcess -AutoSize | Out-Host
$open = @($listening | Where-Object { $_.LocalAddress -ne '127.0.0.1' })
if ($open.Count -gt 0) { Write-Warning "Listening beyond this machine: $($open.LocalPort -join ', ') - check listen_ip in the settings." }

"== services"
Get-CimInstance Win32_Service -Filter "Name='$PersistenceService' OR Name='$BusService'" |
    Format-Table Name, State, StartMode, StartName -AutoSize | Out-Host

"UI: http://127.0.0.1:6123/ (bus), http://127.0.0.1:7123/ (persistence) - from this machine."
"Logs: $Root\bus\config\logs\, $Root\persistence\config\logs\"
