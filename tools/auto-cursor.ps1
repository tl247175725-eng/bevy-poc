# auto-cursor.ps1 — 监控 AIMemory/current.md，自动驱动 Cursor 执行 handoff
# 用法: 右键此文件 → "使用 PowerShell 运行"
#       或终端执行: powershell -ExecutionPolicy Bypass -File tools/auto-cursor.ps1

Add-Type -AssemblyName System.Windows.Forms

# Win32 — 窗口控制
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32 {
    [DllImport("user32.dll", SetLastError=true)]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")]
    public static extern bool IsIconic(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();
}
"@

$repo = "E:\桌面\bevy-poc"
$triggerFile = "$repo\AIMemory\current.md"
$pollSeconds = 30
$lastHash = ""

Write-Host "=== Auto-Cursor 已启动 ===" -ForegroundColor Green
Write-Host "监控: $triggerFile" -ForegroundColor Gray
Write-Host "每 ${pollSeconds}s 检查一次..." -ForegroundColor Gray
Write-Host ""

function Find-CursorWindow {
    # Cursor 窗口标题格式: "filename — Cursor" 或 "Cursor"
    $titles = @()
    # 枚举可能的标题格式
    $proc = Get-Process -Name "Cursor" -ErrorAction SilentlyContinue
    if ($proc) {
        return $true
    }
    return $false
}

function Activate-Cursor {
    $proc = Get-Process -Name "Cursor" -ErrorAction SilentlyContinue
    if (-not $proc) { return $false }
    $hwnd = $proc.MainWindowHandle
    if ($hwnd -eq [IntPtr]::Zero) { return $false }

    # 如果最小化，恢复
    if ([Win32]::IsIconic($hwnd)) {
        [Win32]::ShowWindow($hwnd, 9)  # SW_RESTORE
    }
    Start-Sleep -Milliseconds 200
    [Win32]::SetForegroundWindow($hwnd) | Out-Null
    Start-Sleep -Milliseconds 300
    return $true
}

function Send-ToCursor {
    param([string]$text)

    # 逐字发送，中文需要用模拟键盘输入
    # SendKeys 不直接支持中文 → 用剪贴板
    $oldClip = [System.Windows.Forms.Clipboard]::GetText()
    [System.Windows.Forms.Clipboard]::SetText($text)
    Start-Sleep -Milliseconds 50
    [System.Windows.Forms.SendKeys]::SendWait("^v")  # Ctrl+V
    Start-Sleep -Milliseconds 200
    [System.Windows.Forms.SendKeys]::SendWait("{ENTER}")
    Start-Sleep -Milliseconds 100
    # 恢复剪贴板（可选，但可能干扰用户，先注释掉）
    # [System.Windows.Forms.Clipboard]::SetText($oldClip)
}

function Mark-Executing {
    $content = Get-Content $triggerFile -Raw -Encoding UTF8
    $content = $content -replace "状态: 待执行", "状态: 执行中"
    Set-Content $triggerFile -Value $content -Encoding UTF8 -NoNewline
    git -C $repo add AIMemory/current.md 2>$null
    git -C $repo commit -m "Auto: mark handoff executing" 2>$null
    git -C $repo push 2>$null
}

function Process-Trigger {
    if (-not (Test-Path $triggerFile)) { return }

    $content = Get-Content $triggerFile -Raw -Encoding UTF8

    # 检测 "待执行"
    if ($content -notmatch "状态:\s*待执行") { return }

    # 解析 handoff 文件路径
    $handoffMatch = [regex]::Match($content, "文件:\s*(.+)")
    if (-not $handoffMatch.Success) {
        Write-Host "[$(Get-Date -Format HH:mm:ss)] current.md 格式错误：缺少文件路径" -ForegroundColor Red
        return
    }
    $handoffFile = $handoffMatch.Groups[1].Value.Trim()
    $handoffPath = Join-Path $repo $handoffFile

    # 解析模式
    $modeMatch = [regex]::Match($content, "模式:\s*(.+)")
    $mode = if ($modeMatch.Success) { $modeMatch.Groups[1].Value.Trim() } else { "Standard" }

    Write-Host "[$(Get-Date -Format HH:mm:ss)] 检测到待执行 handoff: $handoffFile" -ForegroundColor Yellow

    # 确认文件存在
    if (-not (Test-Path $handoffPath)) {
        Write-Host "  handoff 文件不存在: $handoffPath" -ForegroundColor Red
        return
    }

    # 激活 Cursor
    if (-not (Find-CursorWindow)) {
        Write-Host "  Cursor 未运行，等待下次检查..." -ForegroundColor DarkYellow
        return
    }
    if (-not (Activate-Cursor)) {
        Write-Host "  无法激活 Cursor 窗口" -ForegroundColor Red
        return
    }

    Write-Host "  Cursor 窗口已激活，发送指令..." -ForegroundColor Cyan

    # 构造发送文本
    if ($mode -eq "Max") {
        $msg = "切换到 Max Mode。读取 $handoffFile，执行。"
    } else {
        $msg = "读取 $handoffFile，执行。"
    }

    Send-ToCursor -text $msg
    Write-Host "  指令已发送: $msg" -ForegroundColor Green

    # 标记执行中
    Mark-Executing
    Write-Host "  current.md 已标记为执行中" -ForegroundColor Gray
    Write-Host ""
}

# 主循环
Write-Host "等待 DeepSeek 的 handoff..." -ForegroundColor Gray
while ($true) {
    try {
        # git pull 更新
        $pullResult = git -C $repo pull 2>&1
        if ($pullResult -notmatch "Already up to date") {
            Write-Host "[$(Get-Date -Format HH:mm:ss)] 检测到远程更新" -ForegroundColor DarkCyan
        }

        Process-Trigger
    }
    catch {
        Write-Host "[$(Get-Date -Format HH:mm:ss)] 检查出错: $_" -ForegroundColor DarkRed
    }

    Start-Sleep -Seconds $pollSeconds
}
