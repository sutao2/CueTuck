# Run after tauri build, before installing the NSIS package.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$config = Get-Content (Join-Path $root 'desktop/src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
$binary = Join-Path $root 'desktop/src-tauri/target/release/cuetuck.exe'
$output = Join-Path $root 'desktop/windows-artifact'
$temp = Join-Path ([IO.Path]::GetTempPath()) ('cuetuck-portable-' + [guid]::NewGuid())
$stage = Join-Path $temp 'package'
$extract = Join-Path $temp 'extracted portable'
$name = "CueTuck_$($config.version)_x64-portable.zip"
$app = $null
try {
  New-Item -ItemType Directory -Force $stage, $output | Out-Null
  Copy-Item $binary (Join-Path $stage 'cuetuck.exe')
  Copy-Item (Join-Path $root 'LICENSE') $stage
  @'
CueTuck — Windows x64 免安装版 / Portable

解压整个 ZIP 后运行 cuetuck.exe。请勿直接在压缩包内启动。
需要 Microsoft Edge WebView2 Runtime；缺少时请从微软官网安装：
https://developer.microsoft.com/microsoft-edge/webview2/

本版本免安装，但数据不会保存在程序旁边：提示词和设置仍使用
Windows 用户数据目录（%APPDATA%\app.promptark.desktop），
WebView 缓存使用用户本地数据目录，密钥保存在系统凭据库。
与同一 Windows 账号的安装版共用数据，请勿同时运行两个版本。
复制 ZIP 或程序文件夹到另一台电脑不会迁移数据或登录凭据。

保持免安装使用：退出 CueTuck，下载新版 portable ZIP，解压替换程序。
应用内“安装更新”会运行安装版更新程序；免安装用户请手动更新 ZIP。
下载与校验和：https://github.com/sutao2/CueTuck/releases

Extract the entire ZIP and run cuetuck.exe. Requires Microsoft Edge WebView2.
This is an installation-free distribution, not a self-contained data folder.
Data is stored in the Windows user profile and secrets in the credential store,
and is shared with the installed version. Do not run both at the same time.
To remain portable, quit the app and replace it with the latest ZIP manually.
The in-app updater launches the installer. Copying this folder does not migrate data.
'@ | Set-Content -Encoding utf8NoBOM (Join-Path $stage 'README.txt')
  $zip = Join-Path $output $name
  Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -Force
  Expand-Archive -Path $zip -DestinationPath $extract
  $files = @(Get-ChildItem $extract -File | Select-Object -ExpandProperty Name | Sort-Object)
  if (Compare-Object @('cuetuck.exe', 'LICENSE', 'README.txt') $files) {
    throw "Unexpected portable contents: $files"
  }
  $executable = Join-Path $extract 'cuetuck.exe'
  if ((Get-FileHash $binary).Hash -ne (Get-FileHash $executable).Hash) { throw 'Portable binary differs from build' }
  $app = Start-Process $executable -WorkingDirectory $extract -PassThru
  Start-Sleep -Seconds 10
  $app.Refresh()
  if ($app.HasExited) { throw "Portable app exited early: $($app.ExitCode)" }
  if ($app.MainWindowHandle -eq [IntPtr]::Zero) { throw 'Portable main window missing' }
  $hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant()
  "$hash  $name" | Set-Content -Encoding utf8NoBOM (Join-Path $output 'SHA256SUMS-windows')
  @{
    file = $name
    sha256 = $hash
    smoke = 'extract ZIP, verify binary, launch 10s with main window before NSIS install'
  } | ConvertTo-Json | Set-Content -Encoding utf8NoBOM (Join-Path $output 'portable-info.json')
  Write-Host "Portable package verified: $name"
} finally {
  if ($app -and -not $app.HasExited) { Stop-Process -Id $app.Id -Force }
  Remove-Item $temp -Recurse -Force -ErrorAction SilentlyContinue
}
