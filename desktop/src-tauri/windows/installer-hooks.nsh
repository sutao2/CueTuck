!define CUETUCK_WINDOWS_SOURCE "${__FILEDIR__}"

!macro NSIS_HOOK_POSTINSTALL
  SetOutPath "$INSTDIR"
  File /oname=CueTuck.icon.ico "${CUETUCK_WINDOWS_SOURCE}\..\icons\icon.ico"
  File /oname=CueTuck.RefreshIcons.ps1 "${CUETUCK_WINDOWS_SOURCE}\..\..\..\scripts\windows-refresh-icons.ps1"
  Push $0
  nsExec::ExecToLog '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$INSTDIR\CueTuck.RefreshIcons.ps1" -ApplicationPath "$INSTDIR\cuetuck.exe" -IconPath "$INSTDIR\CueTuck.icon.ico"'
  Pop $0
  DetailPrint "CueTuck shortcut icon refresh returned $0"
  Pop $0
!macroend
