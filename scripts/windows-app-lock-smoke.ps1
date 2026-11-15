# Exercise the signed candidate through WebView2's native accessibility interface.
param([Parameter(Mandatory=$true)][string]$Executable)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'Requires a disposable Windows Actions runner' }
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class CueTuckLockFocus {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
}
'@
$button = [System.Windows.Automation.ControlType]::Button
$edit = [System.Windows.Automation.ControlType]::Edit
$password = 'Qa' + [Guid]::NewGuid().ToString('N')
$app = $null

function Start-Candidate {
  $script:app = Start-Process $Executable -PassThru
  for ($attempt=0; $attempt -lt 100; $attempt++) {
    $app.Refresh()
    if ($app.HasExited) { throw "Candidate exited: $($app.ExitCode)" }
    if ($app.MainWindowHandle -ne [IntPtr]::Zero) {
      [void][CueTuckLockFocus]::SetForegroundWindow($app.MainWindowHandle)
      return
    }
    Start-Sleep -Milliseconds 250
  }
  throw 'Candidate main window did not appear'
}

function Stop-Candidate {
  if ($app -and -not $app.HasExited) { $app.Kill(); $app.WaitForExit() }
}

function Find-Control([string[]]$Names, $Type) {
  $root = [System.Windows.Automation.AutomationElement]::FromHandle($app.MainWindowHandle)
  foreach ($name in $Names) {
    $byName = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,$name)
    $condition = $byName
    if ($Type) {
      $byType = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty,$Type)
      $condition = [System.Windows.Automation.AndCondition]::new($byName,$byType)
    }
    $found = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
    if ($found -and $found.Current.IsEnabled -and -not $found.Current.IsOffscreen) { return $found }
  }
}

function Wait-Control([string[]]$Names, $Type) {
  for ($attempt=0; $attempt -lt 80; $attempt++) {
    $control = Find-Control $Names $Type
    if ($control) { return $control }
    Start-Sleep -Milliseconds 250
  }
  throw "Expected a visible control: $($Names -join ' / ')"
}

function Click-Button([string[]]$Names) {
  $control = Wait-Control $Names $button
  $control.SetFocus()
  $control.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}

function Fill-Password([string]$Name, [string]$Value) {
  $control = Wait-Control $Name $edit
  [void][CueTuckLockFocus]::SetForegroundWindow($app.MainWindowHandle)
  $control.SetFocus()
  if (-not $control.Current.HasKeyboardFocus -or [CueTuckLockFocus]::GetForegroundWindow() -ne $app.MainWindowHandle) {
    throw "Password field did not receive focus: $Name"
  }
  [System.Windows.Forms.SendKeys]::SendWait('^a')
  [System.Windows.Forms.SendKeys]::SendWait($Value)
}

function Unlock([string]$Value) {
  Fill-Password '解锁口令' $Value
  Click-Button '解锁'
}

$settings = @('设置 ›','设置')
try {
  Start-Candidate
  Click-Button $settings
  Click-Button '隐私与安全'
  Click-Button '设置应用锁'
  Fill-Password '新口令（至少 8 个字符）' $password
  Fill-Password '再次输入新口令' $password
  $hello = Find-Control @('启用 Windows Hello 解锁') ([System.Windows.Automation.ControlType]::CheckBox)
  Click-Button '启用应用锁'
  [void](Wait-Control '管理应用锁' $button)
  Click-Button '返回应用'
  Click-Button '立即锁定'
  [void](Wait-Control '解锁口令' $edit)
  if (Find-Control $settings $button) { throw 'Private workspace is still visible' }
  Unlock 'IncorrectTest123'
  [void](Wait-Control '解锁口令不正确' $null)
  Unlock $password
  [void](Wait-Control '立即锁定' $button)
  Stop-Candidate
  Start-Candidate
  [void](Wait-Control '解锁口令' $edit)
  Unlock $password
  [void](Wait-Control '立即锁定' $button)
  Write-Host "Installed NSIS password setup, manual lock, wrong/correct password and restart passed. Hello offered: $([bool]$hello). Live Hello requires an enrolled device."
} finally { Stop-Candidate }
