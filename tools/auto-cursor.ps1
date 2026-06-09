# auto-cursor.ps1
# Monitors AIMemory/current.md, auto-drives Cursor when a new handoff is ready.
# Usage: powershell -ExecutionPolicy Bypass -File tools/auto-cursor.ps1

Add-Type -AssemblyName System.Windows.Forms

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32 {
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")]
    public static extern bool IsIconic(IntPtr hWnd);
}
"@

$repo = "E:\desktop\bevy-poc"
$triggerFile = Join-Path $repo "AIMemory\current.md"
$pollSeconds = 30
$lastStatus = ""

Write-Host "=== Auto-Cursor started ===" -ForegroundColor Green
Write-Host "Watching: $triggerFile" -ForegroundColor Gray
Write-Host ""

function Find-CursorWindow {
    $p = Get-Process -Name "Cursor" -ErrorAction SilentlyContinue
    return ($null -ne $p)
}

function Activate-Cursor {
    $p = Get-Process -Name "Cursor" -ErrorAction SilentlyContinue
    if (-not $p) { return $false }
    $hwnd = $p.MainWindowHandle
    if ($hwnd -eq [IntPtr]::Zero) { return $false }
    if ([Win32]::IsIconic($hwnd)) {
        [Win32]::ShowWindow($hwnd, 9) | Out-Null
    }
    Start-Sleep -Milliseconds 300
    [Win32]::SetForegroundWindow($hwnd) | Out-Null
    Start-Sleep -Milliseconds 300
    return $true
}

function Send-ToCursor {
    param([string]$text)
    [System.Windows.Forms.Clipboard]::SetText($text)
    Start-Sleep -Milliseconds 100
    [System.Windows.Forms.SendKeys]::SendWait("^v")
    Start-Sleep -Milliseconds 200
    [System.Windows.Forms.SendKeys]::SendWait("{ENTER}")
    Start-Sleep -Milliseconds 100
}

function Parse-CurrentFile {
    param([string]$content)

    # Parse key-value lines: "- key: value"
    $info = @{}
    foreach ($line in ($content -split "`n")) {
        if ($line -match '^-\s*([^:]+):\s*(.+)$') {
            $key = $Matches[1].Trim()
            $val = $Matches[2].Trim()
            $info[$key] = $val
        }
    }
    return $info
}

function Process-Trigger {
    if (-not (Test-Path $triggerFile)) { return }

    $content = Get-Content $triggerFile -Raw -Encoding UTF8
    $info = Parse-CurrentFile -content $content

    $status = $info["status"]
    $handoffFile = $info["file"]
    $mode = $info["mode"]
    if (-not $mode) { $mode = "Standard" }

    # Only trigger on status change to pending
    if ($status -ne "pending") {
        $script:lastStatus = $status
        return
    }
    if ($script:lastStatus -eq "pending") {
        # Already processing this one
        return
    }
    if (-not $handoffFile) {
        Write-Host "[$(Get-Date -Format HH:mm:ss)] current.md missing file path" -ForegroundColor Red
        return
    }

    $handoffPath = Join-Path $repo $handoffFile
    if (-not (Test-Path $handoffPath)) {
        Write-Host "[$(Get-Date -Format HH:mm:ss)] handoff file not found: $handoffPath" -ForegroundColor Red
        return
    }

    Write-Host "[$(Get-Date -Format HH:mm:ss)] NEW HANDOFF: $handoffFile" -ForegroundColor Yellow

    if (-not (Find-CursorWindow)) {
        Write-Host "  Cursor not running, will retry..." -ForegroundColor DarkYellow
        return
    }
    if (-not (Activate-Cursor)) {
        Write-Host "  Cannot activate Cursor window" -ForegroundColor Red
        return
    }

    Write-Host "  Cursor activated, sending command..." -ForegroundColor Cyan

    if ($mode -eq "Max") {
        $msg = "Switch to Max Mode. Read $handoffFile, execute."
    } else {
        $msg = "Read $handoffFile, execute."
    }

    Send-ToCursor -text $msg
    Write-Host "  Sent: $msg" -ForegroundColor Green
    $script:lastStatus = "pending"
}

# Main loop
Write-Host "Waiting for DeepSeek handoffs..." -ForegroundColor Gray
while ($true) {
    try {
        $null = git -C $repo pull 2>&1
        Process-Trigger
    }
    catch {
        # Silently retry
    }
    Start-Sleep -Seconds $pollSeconds
}
