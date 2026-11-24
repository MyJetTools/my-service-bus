<#
.SYNOPSIS
    Stops and removes the two Windows services install.ps1 made - the bus first, then the
    persistence - and the firewall rule. The folders and the messages stay; the commands that
    remove them are printed, not run.
#>
[CmdletBinding()]
param(
    [string]$Root = 'C:\my-sb',
    [string]$Data = 'D:\my-sb-data'
)

$ErrorActionPreference = 'Stop'

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) { throw 'Run this from an elevated PowerShell (Run as administrator).' }

foreach ($service in @(
        @{ Name = 'my-service-bus'; Exe = "$Root\bus\bin\my-service-bus-main-node.exe" },
        @{ Name = 'my-service-bus-persistence'; Exe = "$Root\persistence\bin\my-sb-persistence.exe" })) {
    if (-not (Get-Service -Name $service.Name -ErrorAction SilentlyContinue)) { "$($service.Name): not installed"; continue }

    # A graceful stop: the bus hands its messages to the persistence, the persistence writes them.
    Stop-Service $service.Name
    & $service.Exe --uninstall --name $service.Name
    "$($service.Name): removed"
}

Get-NetFirewallRule -DisplayName 'my-service-bus: no inbound connections' -ErrorAction SilentlyContinue | Remove-NetFirewallRule

"The folders are kept. To remove them, with every message:"
"  Remove-Item -Recurse -Force '$Root', '$Data'"
