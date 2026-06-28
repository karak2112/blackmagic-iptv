# Downloads libmpv-2.dll for Windows x64 into third_party\mpv\win\
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dest = Join-Path $Root "third_party\mpv\win"
$Tmp = Join-Path $env:TEMP "iptv-mpv-fetch"
$GenerateScript = Join-Path $Root "scripts\generate-mpv-lib.ps1"

New-Item -ItemType Directory -Force -Path $Dest | Out-Null
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null

$Release = Invoke-RestMethod "https://api.github.com/repos/shinchiro/mpv-winbuild-cmake/releases/latest"
$Asset = $Release.assets | Where-Object {
  $_.name -like "mpv-dev-x86_64-*.7z" -and $_.name -notlike "*v3*"
} | Select-Object -First 1

if (-not $Asset) { throw "Could not find mpv-dev-x86_64 asset in latest release" }

$Archive = Join-Path $Tmp $Asset.name
Write-Host "Downloading $($Asset.name) (release $($Release.tag_name))..."
Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $Archive

$SevenZip = @(
  "${env:ProgramFiles}\7-Zip\7z.exe",
  "${env:ProgramFiles(x86)}\7-Zip\7z.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $SevenZip) {
  throw "7-Zip not found. Install from https://www.7-zip.org/ or use WSL: npm run fetch-mpv"
}

& $SevenZip x -y "-o$Dest" $Archive "*.dll" "*.lib" "*.dll.a" | Out-Null

Get-ChildItem -Path $Dest -Recurse -Include *.dll,*.lib,*.dll.a | ForEach-Object {
  Move-Item -Force $_.FullName -Destination $Dest
}

if (-not (Test-Path (Join-Path $Dest "libmpv-2.dll"))) {
  throw "libmpv-2.dll not found after extract"
}

& $GenerateScript -Dest $Dest

Set-Content -Path (Join-Path $Dest "VERSION") -Value $Release.tag_name
Write-Host "Installed:"
Get-ChildItem $Dest -Include *.dll,*.lib | Format-Table Name, Length
