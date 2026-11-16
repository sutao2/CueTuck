# Called with a disposable runner's installed application in the foreground.
param([Parameter(Mandatory=$true)][System.Diagnostics.Process]$App)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'Requires a disposable Windows Actions runner' }
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ClipUi {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int command);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
}
'@
$main = $App.MainWindowHandle
$handle = $main
[void][ClipUi]::ShowWindow($main, 3)
[void][ClipUi]::SetForegroundWindow($main)
function Control([string]$Name, [string]$Type = '*') {
  $root = [System.Windows.Automation.AutomationElement]::FromHandle($handle)
  for ($attempt=0; $attempt -lt 45; $attempt++) {
    $nodes = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants,[System.Windows.Automation.Condition]::TrueCondition)
    foreach ($node in $nodes) {
      if ($node.Current.Name -like $Name -and ($Type -eq '*' -or ($node.Current.ControlType.ProgrammaticName.Split('.')[-1] -in ($Type -split '\|'))) -and !$node.Current.IsOffscreen -and $node.Current.IsEnabled) { return $node }
    }
    Start-Sleep -Milliseconds 200
  }
  $visible = $nodes | Where-Object { !$_.Current.IsOffscreen -and $_.Current.Name } | ForEach-Object { "$($_.Current.ControlType.ProgrammaticName)=$($_.Current.Name)" }
  throw "Missing $Type '$Name'. Visible controls: $($visible -join ' | ')"
}
function Click([string]$Name, [string]$Type = '*') {
  $node = Control $Name $Type
  Write-Host "Selecting $($node.Current.ControlType.ProgrammaticName) '$($node.Current.Name)'"
  $rect = $node.Current.BoundingRectangle
  [void][ClipUi]::SetCursorPos([int]($rect.Left+$rect.Width/2),[int]($rect.Top+$rect.Height/2))
  [ClipUi]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
  [ClipUi]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 250
}
function Type-Text([string]$Name, [string]$Value) {
  $field = Control $Name 'Edit'
  $field.SetFocus()
  [System.Windows.Forms.SendKeys]::SendWait('^a')
  [System.Windows.Forms.SendKeys]::SendWait($Value)
}
# A previously built beta.37 installer retains the original display names.
$legacy = (Control '常用*' 'TabItem').Current.Name -eq '常用片段'
$library = if ($legacy) { '常用片段' } else { '常用资料' }
$add = if ($legacy) { '创建常用片段' } else { '添加资料' }
$item = if ($legacy) { '片段' } else { '资料' }
Click $library 'TabItem|Button'
[void](Control 'Legacy clip' 'Text')
Write-Host 'Existing beta.36 row is visible'
Click '新建分类' 'Button'
Type-Text '新建分类' 'QAgroup'
Click '保存' 'Button'
[void](Control 'QAgroup*0' 'Button')
Click '启动器*' 'Button'
# The launcher is a separate top-level window of the same process.
for ($attempt=0; $attempt -lt 50; $attempt++) {
  $windows = [System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children,[System.Windows.Automation.Condition]::TrueCondition)
  $launcher = $windows | Where-Object { $_.Current.ProcessId -eq $App.Id -and $_.Current.NativeWindowHandle -ne $main.ToInt64() -and $_.Current.Name -like '*启动器*' } | Select-Object -First 1
  if ($launcher) { break }
  Start-Sleep -Milliseconds 200
}
if (!$launcher) { throw 'Installed launcher window did not appear' }
$handle = [IntPtr]$launcher.Current.NativeWindowHandle
[void](Control $add 'Button')
[System.Windows.Forms.SendKeys]::SendWait('https://example.invalid/clip-qa')
Click $add 'ListItem'
Type-Text '名称' 'QASite'
$field = Control '网址' 'Edit'
if ($field.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value -ne 'https://example.invalid/clip-qa') { throw 'Launcher lost the input URL' }
Click "${item}分类" 'Button|ComboBox'
Click 'QAgroup' 'ListItem|Button|Text'
Click '保存' 'Button'
[void](Control 'QASite*' 'ListItem|Text')
[System.Windows.Forms.SendKeys]::SendWait('{ESC}')
$handle = $main
[void][ClipUi]::SetForegroundWindow($main)
[void](Control 'QASite' 'Text')
# WebView2 groups the count in the accessible name of each full-width row.
$ends = @("全部${item}*2",'未分类*1','QAgroup*1') | ForEach-Object { (Control $_ 'Button').Current.BoundingRectangle.Right }
$extent = $ends | Measure-Object -Minimum -Maximum
if ($extent.Maximum - $extent.Minimum -gt 1) { throw 'Sidebar category rows are misaligned' }
Type-Text "搜索${library}" 'clip-qa'
Click "筛选${item}类型" 'Button|ComboBox'
Click '文本' 'ListItem|Button|Text'
[void](Control "没有匹配的${item}" 'Text')
Click "筛选${item}类型" 'Button|ComboBox'
Click '网址' 'ListItem|Button|Text'
[void](Control 'QASite' 'Text')
Write-Host 'Installed NSIS passed legacy clip, category, launcher URL capture, live refresh, counts and combined filters.'
