# Generates mpv.lib (MSVC import library) from libmpv-2.dll.
# shinchiro mpv-dev archives ship libmpv.dll.a (MinGW), not mpv.lib.
param(
    [string]$Dest = ""
)

$ErrorActionPreference = "Stop"

if ($Dest -eq "") {
    $Dest = Join-Path (Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)) "third_party\mpv\win"
}

$Dll = Join-Path $Dest "libmpv-2.dll"
$Lib = Join-Path $Dest "mpv.lib"

if (-not (Test-Path $Dll)) {
    throw ('libmpv-2.dll not found at ' + $Dest + '; run npm run fetch-mpv first')
}

if (Test-Path $Lib) {
    Write-Host "mpv.lib already exists at $Dest"
    exit 0
}

function Find-MsvcTool {
    param([string]$ToolName)

    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        foreach ($hostDir in @("Hostx64", "HostX64")) {
            $pattern = "VC\Tools\MSVC\*\bin\$hostDir\x64\$ToolName"
            $found = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find $pattern 2>$null |
                Select-Object -First 1
            if ($found -and (Test-Path $found)) {
                return $found
            }
        }
    }

    $searchRoots = @(
        "$env:ProgramFiles\Microsoft Visual Studio",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio"
    )

    foreach ($root in $searchRoots) {
        if (-not (Test-Path $root)) { continue }

        $years = Get-ChildItem -Path $root -Directory -ErrorAction SilentlyContinue
        foreach ($year in $years) {
            $editions = Get-ChildItem -Path $year.FullName -Directory -ErrorAction SilentlyContinue
            foreach ($edition in $editions) {
                $msvcRoot = Join-Path $edition.FullName "VC\Tools\MSVC"
                if (-not (Test-Path $msvcRoot)) { continue }

                $toolsets = Get-ChildItem -Path $msvcRoot -Directory -ErrorAction SilentlyContinue
                foreach ($toolset in $toolsets) {
                    foreach ($hostDir in @("Hostx64", "HostX64")) {
                        $candidate = Join-Path $toolset.FullName "bin\$hostDir\x64\$ToolName"
                        if (Test-Path $candidate) {
                            return $candidate
                        }
                    }
                }
            }
        }
    }

    return $null
}

$dumpbin = Find-MsvcTool "dumpbin.exe"
$libExe = Find-MsvcTool "lib.exe"

if (-not $dumpbin -or -not $libExe) {
    throw @'
MSVC tools (dumpbin.exe, lib.exe) not found.

Install Visual Studio 2022 with the "Desktop development with C++" workload,
or open "x64 Native Tools Command Prompt for VS 2022" and run:

  npm run generate-mpv-lib
'@
}

Write-Host "Using dumpbin: $dumpbin"
Write-Host "Using lib: $libExe"
Write-Host "Generating mpv.lib from libmpv-2.dll..."

$defPath = Join-Path $Dest "mpv.def"
$exports = & $dumpbin /exports $Dll 2>&1 |
    Select-String "^\s+\d+\s+[0-9A-Fa-f]+\s+[0-9A-Fa-f]+\s+(\S+)" |
    ForEach-Object { $_.Matches[0].Groups[1].Value }

if (-not $exports) {
    throw "No exports found in libmpv-2.dll"
}

$defContent = "LIBRARY libmpv-2`r`nEXPORTS`r`n"
$defContent += ($exports | ForEach-Object { " $_" }) -join "`r`n"
Set-Content -Path $defPath -Value $defContent -Encoding ASCII

& $libExe "/def:$defPath" "/name:libmpv-2.dll" "/machine:x64" "/out:$Lib" | Out-Null

if (-not (Test-Path $Lib)) {
    throw "Failed to generate mpv.lib"
}

Write-Host "Created $Lib"
