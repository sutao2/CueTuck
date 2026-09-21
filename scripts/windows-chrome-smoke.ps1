param([Parameter(Mandatory=$true)][System.Diagnostics.Process]$App)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ChromeSmoke {
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
  [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out Rect r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref Point p);
  [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int command);
}
'@
$App.Refresh()
$handle = $App.MainWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Main window handle missing' }
$rect = New-Object ChromeSmoke+Rect
$origin = New-Object ChromeSmoke+Point
if (-not [ChromeSmoke]::GetWindowRect($handle,[ref]$rect) -or -not [ChromeSmoke]::ClientToScreen($handle,[ref]$origin)) { throw 'Window geometry unavailable' }
if ($origin.Y - $rect.Top -gt 10) { throw 'Unexpected separate native titlebar' }
function Invoke-WindowButton([string]$name) {
  $condition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,$name)
  for ($attempt=0; $attempt -lt 30; $attempt++) {
    $root = [System.Windows.Automation.AutomationElement]::FromHandle($handle)
    $button = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
    if ($null -ne $button) {
      $pattern = $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
      $pattern.Invoke()
      return
    }
    Start-Sleep -Milliseconds 200
  }
  throw "Window button missing: $name"
}
function Wait-State([scriptblock]$condition, [string]$message) {
  for ($attempt=0; $attempt -lt 30; $attempt++) {
    if (& $condition) { return }
    Start-Sleep -Milliseconds 200
  }
  throw $message
}
Invoke-WindowButton '最大化'
Wait-State { [ChromeSmoke]::IsZoomed($handle) } 'Maximize button did not maximize'
Invoke-WindowButton '还原窗口'
Wait-State { -not [ChromeSmoke]::IsZoomed($handle) } 'Restore button did not restore'
Invoke-WindowButton '最小化'
Wait-State { [ChromeSmoke]::IsIconic($handle) } 'Minimize button did not minimize'
[void][ChromeSmoke]::ShowWindow($handle,9)
Wait-State { -not [ChromeSmoke]::IsIconic($handle) } 'Could not restore minimized window'
Invoke-WindowButton '关闭窗口'
Wait-State { -not [ChromeSmoke]::IsWindowVisible($handle) } 'Close button did not close or hide the window'
Write-Host 'Windows integrated titlebar and actual maximize/restore/minimize/close buttons passed.'
