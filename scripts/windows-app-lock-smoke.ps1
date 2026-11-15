# Exercise the signed candidate through WebView2's native accessibility interface.
param([Parameter(Mandatory=$true)][string]$Executable, [switch]$WithQuickClips)
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
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int command);
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
      [void][CueTuckLockFocus]::ShowWindow($app.MainWindowHandle,9)
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

function Find-Control([string[]]$Names, $Type, [switch]$ScrollIntoView) {
  $root = [System.Windows.Automation.AutomationElement]::FromHandle($app.MainWindowHandle)
  foreach ($name in $Names) {
    $byName = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,$name)
    $condition = $byName
    if ($Type) {
      $byType = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty,$Type)
      $condition = [System.Windows.Automation.AndCondition]::new($byName,$byType)
    }
    $found = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
    if ($ScrollIntoView -and $found -and $found.Current.IsOffscreen) {
      $scroll = $null
      if ($found.TryGetCurrentPattern([System.Windows.Automation.ScrollItemPattern]::Pattern, [ref]$scroll)) { $scroll.ScrollIntoView() }
    }
    if ($found -and $found.Current.IsEnabled -and -not $found.Current.IsOffscreen) { return $found }
  }
}

function Wait-Control([string[]]$Names, $Type) {
  for ($attempt=0; $attempt -lt 80; $attempt++) {
    $control = Find-Control $Names $Type -ScrollIntoView
    if ($control) { return $control }
    Start-Sleep -Milliseconds 250
  }
  $root = [System.Windows.Automation.AutomationElement]::FromHandle($app.MainWindowHandle)
  $buttons = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty,$button))
  $visible = $buttons | Where-Object { -not $_.Current.IsOffscreen } | ForEach-Object { $_.Current.Name } | Select-Object -First 35
  throw "Expected a visible control: $($Names -join ' / '). Visible buttons: $($visible -join ', ')"
}

function Click-Button([string[]]$Names) {
  $control = Wait-Control $Names $button
  $control.SetFocus()
  $control.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}

function Fill-Password([string]$Name, [string]$Value) {
  $control = Wait-Control $Name $edit
  if (-not $control.Current.IsPassword) { throw "Expected a protected input: $Name" }
  $valuePattern = $null
  if ($control.TryGetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern, [ref]$valuePattern) -and -not $valuePattern.Current.IsReadOnly) {
    try {
      $valuePattern.SetValue($Value)
      return
    } catch {
      Write-Host "This protected input requires keyboard entry: $Name"
    }
  }
  [void][CueTuckLockFocus]::SetForegroundWindow($app.MainWindowHandle)
  $control.SetFocus()
  for ($attempt=0; $attempt -lt 20; $attempt++) {
    if ($control.Current.HasKeyboardFocus -and [CueTuckLockFocus]::GetForegroundWindow() -eq $app.MainWindowHandle) {
      [System.Windows.Forms.SendKeys]::SendWait('^a')
      [System.Windows.Forms.SendKeys]::SendWait($Value)
      return
    }
    Start-Sleep -Milliseconds 100
  }
  throw "Password field did not receive keyboard focus: $Name"
}

function Unlock([string]$Value) {
  Fill-Password '解锁口令' $Value
  Click-Button '解锁'
}

$settings = @('设置 ›','设置')
try {
  Start-Candidate
  if ($WithQuickClips) { & "$PSScriptRoot/windows-quick-clips-smoke.ps1" -App $app }
  Click-Button $settings
  # An early WebView2 invoke can be dropped during the first foreground activation.
  for ($attempt=0; $attempt -lt 12; $attempt++) {
    if (Find-Control @('返回应用') $button) { break }
    Start-Sleep -Milliseconds 250
    if ($attempt -eq 5 -and (Find-Control $settings $button)) { Click-Button $settings }
  }
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
