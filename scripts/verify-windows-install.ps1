param([Parameter(Mandatory = $true)][string]$Packages)
$ErrorActionPreference = "Stop"
if ($env:GITHUB_ACTIONS -ne "true") {
  throw "Silent install/runtime/uninstall verification requires a disposable CI runner."
}
$Packages = (Resolve-Path $Packages).Path
$Manifest = Get-Content (Join-Path $Packages "manifest.json") -Raw | ConvertFrom-Json
$Installer = Join-Path $Packages $Manifest.file
if ((Get-FileHash $Installer -Algorithm SHA256).Hash -ne $Manifest.sha256) {
  throw "Installer checksum differs from the staged manifest."
}
$InstallDir = Join-Path $env:RUNNER_TEMP ("alkemyst install " + [guid]::NewGuid())
# NSIS requires /D last and treats the rest of the command line as the path.
$Installed = Start-Process -FilePath $Installer -ArgumentList "/S", "/D=$InstallDir" -Wait -PassThru
if ($Installed.ExitCode -ne 0) { throw "NSIS install failed: $($Installed.ExitCode)" }
$Executable = Join-Path $InstallDir "alkemyst-desktop.exe"
if (!(Test-Path $Executable -PathType Leaf)) { throw "Installed application executable is missing." }
$Bytes = [System.IO.File]::ReadAllBytes($Executable)
$PeOffset = [BitConverter]::ToInt32($Bytes, 0x3c)
if ([BitConverter]::ToUInt16($Bytes, $PeOffset + 4) -ne 0x8664) {
  throw "Installed executable is not Windows x64."
}
$Version = (Get-Item $Executable).VersionInfo
if ($Version.ProductName -ne "Alkemyst") { throw "Installed executable product name differs." }
$ExpectedVersion = [version](($Manifest.version -split "-")[0])
$ActualVersion = [version]$Version.FileVersion
if ($ActualVersion.Major -ne $ExpectedVersion.Major -or
    $ActualVersion.Minor -ne $ExpectedVersion.Minor -or
    $ActualVersion.Build -ne $ExpectedVersion.Build) {
  throw "Installed executable version differs from the package."
}
node (Join-Path $PSScriptRoot "installed-runtime.mjs") $Executable
if ($LASTEXITCODE -ne 0) { throw "Installed runtime verification failed." }
$Uninstaller = Join-Path $InstallDir "uninstall.exe"
if (!(Test-Path $Uninstaller -PathType Leaf)) { throw "NSIS uninstaller is missing." }
$Uninstalled = Start-Process -FilePath $Uninstaller -ArgumentList "/S", "_?=$InstallDir" -Wait -PassThru
if ($Uninstalled.ExitCode -ne 0) { throw "NSIS uninstall failed: $($Uninstalled.ExitCode)" }
if (Test-Path $Executable) { throw "Uninstall left the application executable installed." }
Write-Output "Windows preview silent install, installed runtime and uninstall verification passed."
