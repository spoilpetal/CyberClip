param([switch]$System, [switch]$PurgeData)
$ErrorActionPreference = 'Stop'
if ($System) {
    $principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw 'System uninstall requires an elevated PowerShell.'
    }
    $InstallRoot = Join-Path $env:ProgramFiles 'CyberClip'
    $PathScope = 'Machine'
} else {
    $InstallRoot = Join-Path $env:LOCALAPPDATA 'CyberClip'
    $PathScope = 'User'
}
$BinDir = Join-Path $InstallRoot 'bin'
$StateDir = Join-Path $InstallRoot 'state'
$StateFile = Join-Path $StateDir 'installed-commands.json'

if (Test-Path $StateFile) {
    $Saved = Get-Content $StateFile -Raw | ConvertFrom-Json
    foreach ($n in @('cyberclip','cpl')) {
        if ($Saved.installed.$n) {
            Remove-Item (Join-Path $BinDir "$n.exe") -Force -ErrorAction SilentlyContinue
        }
    }
    foreach ($n in @('cyberclip','cpl')) {
        if ($Saved.backup.$n) {
            $backup = Join-Path $StateDir "$n.previous.exe"
            if (Test-Path $backup) {
                New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
                Move-Item $backup (Join-Path $BinDir "$n.exe") -Force
            }
        }
    }
    Remove-Item $StateFile -Force
} else {
    Write-Host "No CyberClip install record found at $StateFile."
}

# Remove only the PATH entry created by this installer.
$pathValue = [Environment]::GetEnvironmentVariable('Path',$PathScope)
if ($pathValue) {
    $cleanPath = (($pathValue -split ';') | Where-Object {
        $_ -and $_.TrimEnd('\') -ne $BinDir.TrimEnd('\')
    }) -join ';'
    [Environment]::SetEnvironmentVariable('Path',$cleanPath,$PathScope)
}

# Remove empty installer-owned folders, never the shared Program Files root.
foreach ($dir in @($StateDir,$BinDir,$InstallRoot)) {
    if (Test-Path $dir) {
        $children = Get-ChildItem -Force -LiteralPath $dir
        if (-not $children) { Remove-Item -LiteralPath $dir -Force }
    }
}

if ($PurgeData) {
    if ($System) {
        throw 'System-wide data purge is not supported: each Windows user has separate CyberClip data.'
    }
    $dataDir = Join-Path $env:LOCALAPPDATA 'cyberclip'
    if (Test-Path $dataDir) {
        $confirm = Read-Host "Permanently delete this user's CyberClip history and data at '$dataDir'? [y/N]"
        if ($confirm -match '^(y|yes)$') { Remove-Item $dataDir -Recurse -Force }
        else { Write-Host 'User data preserved.' }
    }
} else {
    Write-Host 'User history and data were preserved. Use -PurgeData to remove them too.'
}
Write-Host 'CyberClip commands and installer metadata removed; previous commands restored where available.'
