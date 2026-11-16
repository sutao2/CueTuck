param(
  [Parameter(Mandatory=$true)][string]$Executable,
  [Parameter(Mandatory=$true)][ValidateSet('before','after')][string]$Stage
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'Requires a disposable Windows Actions runner' }
Add-Type -AssemblyName System.Drawing
if (-not ('CueTuckIconProbe' -as [type])) {
  Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class CueTuckIconProbe {
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
  public struct FileInfo {
    public IntPtr Icon; public int Index; public uint Attributes;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=260)] public string DisplayName;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=80)] public string TypeName;
  }
  [DllImport("shell32.dll", CharSet=CharSet.Unicode)]
  public static extern IntPtr SHGetFileInfo(string path,uint attributes,ref FileInfo info,uint size,uint flags);
  [DllImport("user32.dll")] public static extern bool DestroyIcon(IntPtr icon);
  [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr window,uint message,IntPtr w,IntPtr l);
  [DllImport("user32.dll", EntryPoint="GetClassLongPtrW")] public static extern IntPtr GetClassLongPtr(IntPtr window,int index);
}
'@
}
$folder = Join-Path $env:RUNNER_TEMP "cuetuck-icon-verification/$Stage"
New-Item -ItemType Directory -Force $folder | Out-Null
function Save-Icon([IntPtr]$Handle,[string]$Name) {
  if ($Handle -eq [IntPtr]::Zero) { return }
  $copy = [System.Drawing.Icon]::FromHandle($Handle).Clone()
  $bitmap = $copy.ToBitmap()
  try { $bitmap.Save((Join-Path $folder "$Name.png"),[System.Drawing.Imaging.ImageFormat]::Png) }
  finally { $bitmap.Dispose(); $copy.Dispose() }
}
function Save-ShellIcon([string]$Path,[string]$Name) {
  $info = [CueTuckIconProbe+FileInfo]::new()
  [void][CueTuckIconProbe]::SHGetFileInfo($Path,0,[ref]$info,[Runtime.InteropServices.Marshal]::SizeOf($info),0x100)
  try { Save-Icon $info.Icon $Name } finally { if ($info.Icon -ne [IntPtr]::Zero) { [void][CueTuckIconProbe]::DestroyIcon($info.Icon) } }
}
$report = @{ executable=$Executable; version=(Get-Item $Executable).VersionInfo.ProductVersion; sha256=(Get-FileHash $Executable -Algorithm SHA256).Hash; shortcuts=@() }
$app = Start-Process $Executable -PassThru
try {
  for ($i=0;$i -lt 80;$i++) {
    $app.Refresh()
    if ($app.HasExited) { throw 'Application exited during icon inspection' }
    if ($app.MainWindowHandle -ne [IntPtr]::Zero) { break }
    Start-Sleep -Milliseconds 250
  }
  if ($app.MainWindowHandle -eq [IntPtr]::Zero) { throw 'Application window missing during icon inspection' }
  Start-Sleep -Seconds 2
  Save-ShellIcon $Executable 'shell-executable'
  foreach ($kind in @(0,1,2)) {
    $icon = [CueTuckIconProbe]::SendMessage($app.MainWindowHandle,0x7F,[IntPtr]$kind,[IntPtr]::Zero)
    Save-Icon $icon "window-$kind"
  }
  Save-Icon ([CueTuckIconProbe]::GetClassLongPtr($app.MainWindowHandle,-14)) 'class-large'
  $shell = New-Object -ComObject WScript.Shell
  $locations = @([Environment]::GetFolderPath('Desktop'),[Environment]::GetFolderPath('CommonDesktopDirectory'),[Environment]::GetFolderPath('StartMenu'),(Join-Path $env:APPDATA 'Microsoft/Internet Explorer/Quick Launch/User Pinned/TaskBar'))
  $index = 0
  foreach ($location in $locations) {
    if (!(Test-Path $location)) { continue }
    foreach ($file in (Get-ChildItem $location -Filter '*CueTuck*.lnk' -Recurse -File)) {
      $link = $shell.CreateShortcut($file.FullName)
      $report.shortcuts += @{path=$file.FullName;target=$link.TargetPath;icon=$link.IconLocation}
      Save-ShellIcon $file.FullName "shortcut-$index"
      $index++
    }
  }
  $report | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 (Join-Path $folder 'report.json')
  Write-Host "Recorded $Stage executable, window, shell and shortcut icons."
  if ($Stage -eq 'after') {
    $compiler = "${env:ProgramFiles(x86)}/NSIS/makensis.exe"
    $temporary = Join-Path $env:RUNNER_TEMP 'cuetuck-icon-hook'
    New-Item -ItemType Directory -Force $temporary | Out-Null
    $installer = Join-Path $temporary 'refresh-icons.exe'
    $hook = Join-Path (Split-Path $PSScriptRoot) 'desktop/src-tauri/windows/installer-hooks.nsh'
    $recipe = @'
Unicode true
RequestExecutionLevel user
SilentInstall silent
OutFile "OUTPUT_PATH"
InstallDir "INSTALL_DIRECTORY"
!include "HOOK_PATH"
Section
  !insertmacro NSIS_HOOK_POSTINSTALL
SectionEnd
'@
    $recipe.Replace('OUTPUT_PATH',$installer).Replace('INSTALL_DIRECTORY',(Split-Path $Executable)).Replace('HOOK_PATH',$hook) | Set-Content -Encoding utf8 (Join-Path $temporary 'test.nsi')
    & $compiler (Join-Path $temporary 'test.nsi')
    if ($LASTEXITCODE -ne 0) { throw 'Icon refresh installer hook compilation failed' }
    if ((Start-Process $installer -Wait -PassThru).ExitCode -ne 0) { throw 'Icon refresh installer hook failed' }

    $folder = Join-Path $env:RUNNER_TEMP 'cuetuck-icon-verification/repaired'
    New-Item -ItemType Directory -Force $folder | Out-Null
    $index = 0
    foreach ($item in $report.shortcuts) {
      $link = $shell.CreateShortcut($item.path)
      $iconFile = $link.IconLocation -replace ',0$',''
      if (!(Test-Path -LiteralPath $iconFile) -or (Get-FileHash -LiteralPath $iconFile).Hash -ne (Get-FileHash (Join-Path (Split-Path $PSScriptRoot) 'desktop/src-tauri/icons/icon.ico')).Hash) {
        $legacyPowerShell = Join-Path $env:SystemRoot 'SysWOW64/WindowsPowerShell/v1.0/powershell.exe'
        $installedScript = Join-Path (Split-Path $Executable) 'CueTuck.RefreshIcons.ps1'
        $installedIcon = Join-Path (Split-Path $Executable) 'CueTuck.icon.ico'
        Write-Host "Installed refresh files: script=$(Test-Path $installedScript), icon=$(Test-Path $installedIcon)"
        & $legacyPowerShell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File $installedScript -ApplicationPath $Executable -IconPath $installedIcon
        Write-Host "Direct Windows PowerShell result: $LASTEXITCODE"
        throw "Shortcut did not receive the current icon: $($link.IconLocation)"
      }
      Save-ShellIcon $item.path "shortcut-$index"
      $index++
    }
    Write-Host 'Recorded icons repaired with the current content-specific icon resource.'
  }
} finally {
  if (!$app.HasExited) { $app.Kill(); $app.WaitForExit() }
}
