$ErrorActionPreference = 'Stop'
$reportDirectory = Join-Path $PWD 'work/windows-loader'
New-Item -ItemType Directory -Force -Path $reportDirectory | Out-Null
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$dumpbin = & $vswhere -latest -products '*' -find 'VC/Tools/MSVC/**/bin/Hostx64/x64/dumpbin.exe' | Select-Object -First 1
if (-not $dumpbin) { throw 'MSVC dumpbin was not found.' }
$executables = Get-ChildItem 'target/debug/deps/commands-*.exe' -ErrorAction SilentlyContinue
if (-not $executables) { throw 'No native IPC test executable was built.' }
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class NativeLoaderProbe {
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, ExactSpelling=true, SetLastError=true)]
    public static extern IntPtr LoadLibraryW(string name);
    [DllImport("kernel32.dll", CharSet=CharSet.Ansi, ExactSpelling=true, SetLastError=true)]
    public static extern IntPtr GetProcAddress(IntPtr module, string name);
}
'@
$results = @()
foreach ($executable in $executables) {
    $imports = & $dumpbin /imports $executable.FullName
    if ($LASTEXITCODE -ne 0) { throw 'dumpbin could not inspect native test imports.' }
    $imports | Out-File (Join-Path $reportDirectory "$($executable.BaseName)-imports.txt")
    $module = [IntPtr]::Zero
    $library = ''
    foreach ($line in $imports) {
        if ($line -match '^\s+(\S+\.dll)\s*$') {
            $library = $Matches[1]
            $module = [NativeLoaderProbe]::LoadLibraryW($library)
            $errorCode = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
            if ($module -eq [IntPtr]::Zero) {
                $results += [PSCustomObject]@{ library=$library; function='LoadLibraryW'; error=$errorCode }
            }
        } elseif ($module -ne [IntPtr]::Zero -and $line -match '^\s+[0-9A-Fa-f]+\s+([A-Za-z_?][^\s]+)\s*$') {
            $symbol = $Matches[1]
            $address = [NativeLoaderProbe]::GetProcAddress($module, $symbol)
            $errorCode = [Runtime.InteropServices.Marshal]::GetLastWin32Error()
            if ($address -eq [IntPtr]::Zero) {
                $results += [PSCustomObject]@{ library=$library; function=$symbol; error=$errorCode }
            }
        }
    }
}
$results | ConvertTo-Json -Depth 4 | Out-File (Join-Path $reportDirectory 'unresolved-imports.json')
$results | Format-Table -AutoSize | Out-String | Write-Output
Get-ComputerInfo -Property WindowsProductName, WindowsVersion, OsBuildNumber |
    ConvertTo-Json | Out-File (Join-Path $reportDirectory 'windows-version.json')
