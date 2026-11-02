param([switch]$System)
$ErrorActionPreference = 'Stop'
if ($System) {
    $principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw 'System install requires an elevated PowerShell. Re-run as Administrator.' }
    $InstallRoot = Join-Path $env:ProgramFiles 'CyberClip'
    $PathScope = 'Machine'
} else {
    $InstallRoot = Join-Path $env:LOCALAPPDATA 'CyberClip'
    $PathScope = 'User'
}
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$BinDir = Join-Path $InstallRoot 'bin'
$StateDir = Join-Path $InstallRoot 'state'
$StateFile = Join-Path $StateDir 'installed-commands.json'

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'Rust/Cargo is required for a source install. A prebuilt Windows release will not require this.'
}
Write-Host 'Detected Windows: CyberClip uses clipboard-rs with the native Win32 clipboard API.'
Write-Host 'Building CyberClip for Windows...'
cargo build --locked --release --manifest-path (Join-Path $ScriptDir 'Cargo.toml')
$Built = Join-Path $ScriptDir 'target\release\cyberclip.exe'
if (-not (Test-Path $Built)) { throw "Release binary not found: $Built" }

$Old = @{ cyberclip = $false; cpl = $false; backup = @{} }
if (Test-Path $StateFile) {
    $Saved = Get-Content $StateFile -Raw | ConvertFrom-Json
    foreach ($n in @('cyberclip','cpl')) { $Old[$n] = [bool]$Saved.installed.$n; $Old.backup[$n] = [bool]$Saved.backup.$n }
}
Write-Host 'Existing commands:'
foreach ($n in @('cyberclip','cpl')) {
    $cmd = Get-Command $n -ErrorAction SilentlyContinue
    if ($cmd) { Write-Host "  $n : $($cmd.Source)" } else { Write-Host "  $n : not found" }
}
Write-Host "`nChoose command name(s):`n  1) cyberclip only`n  2) cpl only`n  3) both`n  4) cancel"
$choice = Read-Host 'Choose [1-4]'
switch ($choice) { '1' { $Want = @{ cyberclip=$true; cpl=$false } } '2' { $Want = @{ cyberclip=$false; cpl=$true } } '3' { $Want = @{ cyberclip=$true; cpl=$true } } default { Write-Host 'Cancelled.'; exit 0 } }

New-Item -ItemType Directory -Force -Path $BinDir,$StateDir | Out-Null
foreach ($n in @('cyberclip','cpl')) {
    if (-not $Want[$n] -and -not $Old[$n]) {
        $target = Join-Path $BinDir "$n.exe"
        if (Test-Path $target) {
            if ((Read-Host "Untracked '$n' exists in the CyberClip install folder. Remove it as an older CyberClip command? [y/N]") -match '^(y|yes)$') {
                Remove-Item $target -Force
            }
        }
    }
}
$Backups = @{ cyberclip=$Old.backup.cyberclip; cpl=$Old.backup.cpl }
foreach ($n in @('cyberclip','cpl')) {
    $target = Join-Path $BinDir "$n.exe"
    if ($Want[$n]) {
        $existing = Get-Command $n -ErrorAction SilentlyContinue
        if ($existing -and -not $Old[$n] -and $existing.Source -ne $target) {
            Write-Warning "$n already exists at $($existing.Source); CyberClip's user PATH entry may take priority."
            if ((Read-Host 'Continue? [y/N]') -notmatch '^(y|yes)$') { throw 'Installation cancelled; existing commands were not changed.' }
        }
    }
}
# Remove names managed by the previous CyberClip install but no longer selected.
foreach ($n in @('cyberclip','cpl')) {
    $target = Join-Path $BinDir "$n.exe"
    if ($Old[$n] -and -not $Want[$n]) {
        Remove-Item $target -Force -ErrorAction SilentlyContinue
        if ($Old.backup[$n]) { $backup = Join-Path $StateDir "$n.previous.exe"; if (Test-Path $backup) { Move-Item $backup $target -Force } }
    }
}
foreach ($n in @('cyberclip','cpl')) {
    if (-not $Want[$n]) { continue }
    $target = Join-Path $BinDir "$n.exe"
    $backup = Join-Path $StateDir "$n.previous.exe"
    if ($Old[$n]) { Remove-Item $target -Force -ErrorAction SilentlyContinue }
    elseif (Test-Path $target) {
        if (Test-Path $backup) { throw "Backup already exists at $backup; refusing to overwrite it." }
        Move-Item $target $backup; $Backups[$n]=$true
    }
    Copy-Item $Built $target -Force
}
$record = @{ installed=$Want; backup=$Backups }
$record | ConvertTo-Json -Depth 4 | Set-Content $StateFile
$existingPath = [Environment]::GetEnvironmentVariable('Path',$PathScope)
if (($existingPath -split ';') -notcontains $BinDir) { [Environment]::SetEnvironmentVariable('Path', (($existingPath.TrimEnd(';') + ';' + $BinDir).Trim(';')), $PathScope) }
Write-Host "Installed: $((@('cyberclip','cpl') | Where-Object { $Want[$_] }) -join ', ')"
if ($System) { Write-Host 'Installed for all Windows users. Open a new terminal to use the selected command(s).' } else { Write-Host 'Installed for this Windows user. Open a new terminal to use the selected command(s).' }
