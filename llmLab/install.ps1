# llmLab One-Command Windows Installer & Launcher
# Usage:
#   irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex
# Optional override:
#   $env:LLMLAB_REPO = "tatarhost/llmLab"; irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = if ($env:LLMLAB_REPO) { $env:LLMLAB_REPO } else { "tatarhost/llmLab" }
$InstallDir = Join-Path $env:LOCALAPPDATA "llmLab"
$ZipUrl = "https://github.com/$Repo/releases/latest/download/llmLab-win64.zip"
$HashUrl = "https://github.com/$Repo/releases/latest/download/SHA256SUMS.txt"

Write-Host "==> Установка llmLab из https://github.com/$Repo ..." -ForegroundColor Cyan

$TempDir = Join-Path $env:TEMP ("llmlab-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $ZipPath = Join-Path $TempDir "llmLab-win64.zip"
    $HashPath = Join-Path $TempDir "SHA256SUMS.txt"

    Write-Host "    Скачивание последнего релиза..."
    Invoke-WebRequest -Uri $ZipUrl -OutFile $ZipPath -UseBasicParsing

    try {
        Invoke-WebRequest -Uri $HashUrl -OutFile $HashPath -UseBasicParsing
        $ExpectedHash = ((Get-Content $HashPath -Raw).Trim() -split "\s+")[0].ToLowerInvariant()
        $ActualHash = (Get-FileHash $ZipPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($ExpectedHash -and ($ExpectedHash -ne $ActualHash)) {
            throw "SHA256 не совпадает! Ожидалось: $ExpectedHash, получено: $ActualHash"
        }
        Write-Host "    SHA256 проверен: $ActualHash" -ForegroundColor Green
    } catch {
        Write-Host "    Предупреждение: не удалось проверить SHA256SUMS.txt ($($_.Exception.Message))" -ForegroundColor Yellow
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force

    $ModelsDir = Join-Path $InstallDir "models"
    New-Item -ItemType Directory -Force -Path $ModelsDir | Out-Null

    $ConfigPath = Join-Path $InstallDir "config.toml"
    $ExampleConfigPath = Join-Path $InstallDir "config.example.toml"
    if ((-not (Test-Path $ConfigPath)) -and (Test-Path $ExampleConfigPath)) {
        Copy-Item $ExampleConfigPath $ConfigPath
    }

    # Add install dir to User PATH if missing
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $PathParts = if ($UserPath) { $UserPath -split ";" } else { @() }
    if ($PathParts -notcontains $InstallDir) {
        $NewUserPath = if ($UserPath) { "$UserPath;$InstallDir" } else { $InstallDir }
        [Environment]::SetEnvironmentVariable("Path", $NewUserPath, "User")
        $env:Path = "$env:Path;$InstallDir"
        Write-Host "    Добавлено в PATH: $InstallDir" -ForegroundColor Green
    }

    # Create Desktop shortcut with embedded llmLab.exe icon
    $ExePath = Join-Path $InstallDir "llmLab.exe"
    $DesktopDir = [Environment]::GetFolderPath("Desktop")
    if ($DesktopDir -and (Test-Path $ExePath)) {
        $WshShell = New-Object -ComObject WScript.Shell
        $Shortcut = $WshShell.CreateShortcut((Join-Path $DesktopDir "llmLab.lnk"))
        $Shortcut.TargetPath = $ExePath
        $Shortcut.WorkingDirectory = $InstallDir
        $Shortcut.IconLocation = "$ExePath,0"
        $Shortcut.Description = "llmLab — Local AI TUI Runtime"
        $Shortcut.Save()
    }

    Write-Host "==> Готово! Запуск llmLab..." -ForegroundColor Cyan
    & $ExePath
}
finally {
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}
