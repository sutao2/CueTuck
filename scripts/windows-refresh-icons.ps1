param(
  [string]$ApplicationPath,
  [string]$IconPath = (Join-Path $PSScriptRoot 'CueTuck.ico')
)
$ErrorActionPreference = 'Stop'
$shell = New-Object -ComObject WScript.Shell
if (!$ApplicationPath) {
  foreach ($location in @([Environment]::GetFolderPath('Desktop'),[Environment]::GetFolderPath('StartMenu'))) {
    $shortcut = Join-Path $location 'CueTuck.lnk'
    if (!(Test-Path -LiteralPath $shortcut)) { continue }
    $target = $shell.CreateShortcut($shortcut).TargetPath
    if (Test-Path -LiteralPath $target) { $ApplicationPath = $target; break }
  }
}
if (!$ApplicationPath) { throw 'CueTuck shortcut not found. Pass -ApplicationPath with your installed cuetuck.exe path.' }
$application = (Get-Item -LiteralPath $ApplicationPath).FullName
if ([IO.Path]::GetFileName($application) -ine 'cuetuck.exe') { throw 'Expected the installed CueTuck executable' }
$source = Get-Item -LiteralPath $IconPath
# A content-specific icon path avoids reusing the executable's old shell image.
$hash = (Get-FileHash -LiteralPath $source.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$icons = Join-Path $env:LOCALAPPDATA 'CueTuck/icons'
New-Item -ItemType Directory -Force $icons | Out-Null
$icon = Join-Path $icons "$hash.ico"
if (!(Test-Path -LiteralPath $icon)) { Copy-Item -LiteralPath $source.FullName -Destination $icon }
if (-not ('CueTuckShortcutRefresh' -as [type])) {
  Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class CueTuckShortcutRefresh {
  [DllImport("shell32.dll", CharSet=CharSet.Unicode)]
  public static extern void SHChangeNotify(int change,uint flags,string item,IntPtr other);
}
'@
}
$locations = @(
  [Environment]::GetFolderPath('Desktop'),
  [Environment]::GetFolderPath('StartMenu'),
  (Join-Path $env:APPDATA 'Microsoft/Internet Explorer/Quick Launch/User Pinned/TaskBar')
)
$changed = 0
foreach ($location in $locations) {
  if (!(Test-Path -LiteralPath $location)) { continue }
  foreach ($file in (Get-ChildItem -LiteralPath $location -Filter '*.lnk' -Recurse -File)) {
    $link = $shell.CreateShortcut($file.FullName)
    if (![string]::Equals($link.TargetPath,$application,[StringComparison]::OrdinalIgnoreCase)) { continue }
    $link.IconLocation = "$icon,0"
    $link.Save()
    $changed++
    [CueTuckShortcutRefresh]::SHChangeNotify(0x2000,0x1005,$file.FullName,[IntPtr]::Zero)
  }
}
if ($changed -eq 0) { throw 'No shortcuts point to the selected CueTuck executable' }
Write-Host "Updated $changed CueTuck shortcut icons."
