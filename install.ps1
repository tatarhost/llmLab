# llmLab One-Command Windows Installer & Launcher
# Usage:
#   irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex
# Optional override:
#   $env:LLMLAB_REPO = "tatarhost/llmLab"; irm https://raw.githubusercontent.com/tatarhost/llmLab/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = if ($env:LLMLAB_REPO) { $env:LLMLAB_REPO } else { "tatarhost/llmLab" }
$InstallDir = Join-Path $env:LOCALAPPDATA "llmLab"

function Resolve-ReleaseAssets {
    param([string]$RepoSlug)

    $candidates = @(
        @{
            Zip  = "https://github.com/$RepoSlug/releases/latest/download/llmLab-win64.zip"
            Hash = "https://github.com/$RepoSlug/releases/latest/download/SHA256SUMS.txt"
        },
        @{
            Zip  = "https://github.com/$RepoSlug/releases/download/latest/llmLab-win64.zip"
            Hash = "https://github.com/$RepoSlug/releases/download/latest/SHA256SUMS.txt"
        }
    )

    try {
        $headers = @{ "User-Agent" = "llmLab-Installer" }
        $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$RepoSlug/releases?per_page=5" -Headers $headers -UseBasicParsing -ErrorAction SilentlyContinue
        if ($releases) {
            foreach ($rel in $releases) {
                $zipAsset = $rel.assets | Where-Object { $_.name -eq "llmLab-win64.zip" } | Select-Object -First 1
                if ($zipAsset) {
                    $hashAsset = $rel.assets | Where-Object { $_.name -eq "SHA256SUMS.txt" } | Select-Object -First 1
                    return @{
                        Zip  = $zipAsset.browser_download_url
                        Hash = if ($hashAsset) { $hashAsset.browser_download_url } else { $null }
                    }
                }
            }
        }
    } catch {
        # Ignore API errors and fall back
    }

    foreach ($c in $candidates) {
        try {
            $resp = Invoke-WebRequest -Uri $c.Zip -Method Head -UseBasicParsing -ErrorAction SilentlyContinue
            if ($resp -and $resp.StatusCode -ge 200 -and $resp.StatusCode -lt 400) {
                return $c
            }
        } catch {
            continue
        }
    }

    return $null
}

Write-Host "==> Установка llmLab из https://github.com/$Repo ..." -ForegroundColor Cyan

$TempDir = Join-Path $env:TEMP ("llmlab-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

    # Stop running llmLab instance & delete old version artifacts before upgrading
    Get-Process -Name "llmLab" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 250
    foreach ($oldPattern in @("llmLab.exe", "*.exe.old", "*.exe.bak", "models\*.part")) {
        $target = Join-Path $InstallDir $oldPattern
        Remove-Item $target -Force -ErrorAction SilentlyContinue
    }

    $Assets = Resolve-ReleaseAssets -RepoSlug $Repo
    if ($Assets) {
        $ZipPath = Join-Path $TempDir "llmLab-win64.zip"
        $HashPath = Join-Path $TempDir "SHA256SUMS.txt"

        Write-Host "    Скачивание последнего релиза (старая версия удалена)..."
        Invoke-WebRequest -Uri $Assets.Zip -OutFile $ZipPath -UseBasicParsing

        if ($Assets.Hash) {
            try {
                Invoke-WebRequest -Uri $Assets.Hash -OutFile $HashPath -UseBasicParsing
                $ExpectedHash = ((Get-Content $HashPath -Raw).Trim() -split "\s+")[0].ToLowerInvariant()
                $ActualHash = (Get-FileHash $ZipPath -Algorithm SHA256).Hash.ToLowerInvariant()
                if ($ExpectedHash -and ($ExpectedHash -ne $ActualHash)) {
                    throw "SHA256 не совпадает! Ожидалось: $ExpectedHash, получено: $ActualHash"
                }
                Write-Host "    SHA256 проверен: $ActualHash" -ForegroundColor Green
            } catch {
                Write-Host "    Предупреждение: не удалось проверить SHA256SUMS.txt ($($_.Exception.Message))" -ForegroundColor Yellow
            }
        }

        Expand-Archive -Path $ZipPath -DestinationPath $InstallDir -Force
    } else {
        Write-Host "    На GitHub Releases пока нет готового архива llmLab-win64.zip." -ForegroundColor Yellow

        $HasGit = [bool](Get-Command git -ErrorAction SilentlyContinue)
        $HasCargo = [bool](Get-Command cargo -ErrorAction SilentlyContinue)

        if ($HasGit -and $HasCargo) {
            Write-Host "    Найдены git и cargo — выполняю локальную release-сборку из main..." -ForegroundColor Cyan
            $RepoDir = Join-Path $TempDir "src"
            & git clone --depth 1 "https://github.com/$Repo.git" $RepoDir
            Push-Location $RepoDir
            try {
                if (Get-Command python -ErrorAction SilentlyContinue) {
                    try {
                        & python scripts/make_icon.py
                    } catch {
                        Write-Host "    Pillow не найден — сборка продолжится без обновления icon.ico" -ForegroundColor Yellow
                    }
                }
                & cargo build --release --target x86_64-pc-windows-msvc
                $BuiltExe = Join-Path $RepoDir "target\x86_64-pc-windows-msvc\release\llmLab.exe"
                if (-not (Test-Path $BuiltExe)) {
                    throw "Не удалось найти собранный $BuiltExe"
                }
                Copy-Item $BuiltExe (Join-Path $InstallDir "llmLab.exe") -Force
                if (Test-Path "README.md") { Copy-Item "README.md" (Join-Path $InstallDir "README.md") -Force }
                if (Test-Path "config.example.toml") { Copy-Item "config.example.toml" (Join-Path $InstallDir "config.example.toml") -Force }
            } finally {
                Pop-Location
            }
        } else {
            Write-Host ""
            Write-Host "Ошибка: релиз llmLab-win64.zip ещё не опубликован в https://github.com/$Repo/releases." -ForegroundColor Red
            Write-Host "1) Откройте https://github.com/$Repo/actions и дождитесь зелёного завершения 'CI & Windows Release' (или запустите Workflow manually)." -ForegroundColor Yellow
            Write-Host "2) Либо установите Rust (winget install Rustlang.Rustup) и Git, затем повторите команду — скрипт соберёт llmLab.exe сам." -ForegroundColor Yellow
            return
        }
    }

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

    if (Test-Path $ExePath) {
        Write-Host "==> Готово! Запуск llmLab..." -ForegroundColor Cyan
        & $ExePath
    }
}
finally {
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}
