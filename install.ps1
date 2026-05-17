$ErrorActionPreference = "Stop"

$Repo = "hyperq/jav"
$Binary = "jav"
$InstallDir = if ($env:JAV_INSTALL_DIR) { $env:JAV_INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

# detect arch
$Arch = if ([Environment]::Is64BitOperatingSystem) {
    if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64" } else { "x86_64" }
} else { "x86_64" }

$Target = "$Binary-$Arch-pc-windows-msvc"

# get latest version
if ($env:JAV_VERSION) {
    $Version = $env:JAV_VERSION
} else {
    $Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $Release.tag_name
}

Write-Host "Installing $Binary $Version ($Arch-windows)..." -ForegroundColor Cyan

$Url = "https://github.com/$Repo/releases/download/$Version/$Target.zip"
$TmpDir = Join-Path $env:TEMP "jav-install"
$ZipFile = Join-Path $TmpDir "$Target.zip"

New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
Invoke-WebRequest -Uri $Url -OutFile $ZipFile
Expand-Archive -Path $ZipFile -DestinationPath $TmpDir -Force

# install
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Move-Item -Path (Join-Path $TmpDir "$Binary.exe") -Destination (Join-Path $InstallDir "$Binary.exe") -Force

# cleanup
Remove-Item -Recurse -Force $TmpDir

# check PATH
if ($env:PATH -notlike "*$InstallDir*") {
    Write-Host ""
    Write-Host "Add to PATH: $InstallDir" -ForegroundColor Yellow
    Write-Host 'Run: [Environment]::SetEnvironmentVariable("PATH", $env:PATH + ";' + $InstallDir + '", "User")' -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "Installed $Binary to $InstallDir\$Binary.exe" -ForegroundColor Green
Write-Host "Run 'jav --help' to get started"
