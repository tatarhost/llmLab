# Helper script to publish llmLab to your GitHub and trigger the first v0.1.0 release.
# Run inside llmLab/ directory on your PC:
#   .\publish.ps1 -Repo "your-username/llmLab"

param(
    [Parameter(Mandatory = $true)]
    [string]$Repo,
    [string]$Version = "v0.1.0"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    throw "Git не установлен. Установите Git для Windows."
}

# Replace OWNER/llmLab placeholder in install.ps1 and README.md with actual repo
(Get-Content "install.ps1" -Raw).Replace("OWNER/llmLab", $Repo) | Set-Content "install.ps1" -Encoding utf8
(Get-Content "README.md" -Raw).Replace("OWNER/llmLab", $Repo) | Set-Content "README.md" -Encoding utf8

if (-not (Test-Path ".git")) {
    git init -b main
}

git add .
git commit -m "feat: initial llmLab Windows TUI runtime ($Version)"

$RemoteUrl = "https://github.com/$Repo.git"
$ExistingRemote = git remote 2>$null
if ($ExistingRemote -contains "origin") {
    git remote set-url origin $RemoteUrl
} else {
    git remote add origin $RemoteUrl
}

# If GitHub CLI is logged in, automatically create the repo if it doesn't exist yet
if (Get-Command gh -ErrorAction SilentlyContinue) {
    gh repo view $Repo 2>$null
    if ($LASTEXITCODE -ne 0) {
        gh repo create $Repo --public --source=. --remote=origin
    }
}

git push -u origin main
git tag -f $Version
git push -f origin $Version

Write-Host ""
Write-Host "==> Отправлено в https://github.com/$Repo (тег $Version)!" -ForegroundColor Green
Write-Host "    Дождитесь завершения GitHub Actions (~3 мин), затем запускайте одной командой:" -ForegroundColor Cyan
Write-Host "    irm https://raw.githubusercontent.com/$Repo/main/install.ps1 | iex" -ForegroundColor White
