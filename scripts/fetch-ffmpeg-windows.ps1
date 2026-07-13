# Downloads ffmpeg.exe for Windows x64 into third_party\ffmpeg\win\
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Dest = Join-Path $Root "third_party\ffmpeg\win"
$Tmp = Join-Path $env:TEMP "iptv-ffmpeg-fetch"
$Url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
$Archive = Join-Path $Tmp "ffmpeg-release-essentials.zip"

New-Item -ItemType Directory -Force -Path $Dest | Out-Null
if (Test-Path $Tmp) { Remove-Item -Recurse -Force $Tmp }
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null

Write-Host "Downloading ffmpeg-release-essentials.zip..."
Invoke-WebRequest -Uri $Url -OutFile $Archive

Expand-Archive -Path $Archive -DestinationPath $Tmp -Force

$Exe = Get-ChildItem -Path $Tmp -Recurse -Filter ffmpeg.exe | Select-Object -First 1
if (-not $Exe) {
  throw "ffmpeg.exe not found inside archive"
}

Copy-Item -Force $Exe.FullName -Destination (Join-Path $Dest "ffmpeg.exe")
Set-Content -Path (Join-Path $Dest "VERSION") -Value "ffmpeg-release-essentials"

Write-Host "Installed:"
Get-ChildItem $Dest -Include ffmpeg.exe,VERSION | Format-Table Name, Length
