# Installs Wrangler on Windows, from the newest release (alphas included):
#
#   irm https://raw.githubusercontent.com/GenuineMoron/img-wrangler/main/install.ps1 | iex
#
# It only downloads from this repository's releases, then runs the installer.
$ErrorActionPreference = 'Stop'
$repo = 'GenuineMoron/img-wrangler'
$release = (Invoke-RestMethod "https://api.github.com/repos/$repo/releases?per_page=1")[0]
$asset = $release.assets | Where-Object { $_.name -like '*_x64-setup.exe' } | Select-Object -First 1
if (-not $asset) { throw "The newest release ($($release.tag_name)) has no Windows installer." }
$file = Join-Path $env:TEMP $asset.name
Write-Host "Downloading $($asset.name)"
Invoke-WebRequest $asset.browser_download_url -OutFile $file -UseBasicParsing
Start-Process $file -Wait
Remove-Item $file
Write-Host 'Wrangler is installed. Find it in the Start menu.'
