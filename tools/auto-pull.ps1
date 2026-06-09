# auto-pull.ps1
# Periodically pulls latest code from GitHub so local files stay in sync.
# Usage: powershell -ExecutionPolicy Bypass -File tools/auto-pull.ps1

$repo = "E:\桌面\bevy-poc"
$interval = 60

Write-Host "=== Auto-Pull started ===" -ForegroundColor Green
Write-Host "Repo: $repo"
Write-Host "Checking every ${interval}s..."
Write-Host ""

while ($true) {
    try {
        Push-Location $repo
        $result = git pull 2>&1
        if ($result -notmatch "Already up to date") {
            Write-Host "[$(Get-Date -Format HH:mm:ss)] Updated:" -ForegroundColor Cyan
            Write-Host $result
        }
        Pop-Location
    }
    catch {
        Write-Host "[$(Get-Date -Format HH:mm:ss)] Error: $_" -ForegroundColor DarkRed
        Pop-Location -ErrorAction SilentlyContinue
    }
    Start-Sleep -Seconds $interval
}
