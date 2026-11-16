!define CUETUCK_WINDOWS_SOURCE "${__FILEDIR__}"

!macro NSIS_HOOK_POSTINSTALL
  SetOutPath "$INSTDIR"
  File /oname=CueTuck.icon.ico "${CUETUCK_WINDOWS_SOURCE}\..\icons\icon.ico"
  File /oname=CueTuck.RefreshIcons.ps1 "${CUETUCK_WINDOWS_SOURCE}\..\..\..\scripts\windows-refresh-icons.ps1"
  Push $0
  Push $1
  Push $2
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$INSTDIR\CueTuck.RefreshIcons.ps1" -ApplicationPath "$INSTDIR\cuetuck.exe" -IconPath "$INSTDIR\CueTuck.icon.ico"'
  Pop $0
  Pop $1
  DetailPrint "CueTuck shortcut icon refresh returned $0"
  DetailPrint "$1"
  FileOpen $2 "$TEMP\CueTuck-icon-refresh.log" w
  FileWrite $2 "$0$\r$\n$1"
  FileClose $2
  Pop $2
  Pop $1
  Pop $0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\CueTuck.icon.ico"
  Delete "$INSTDIR\CueTuck.RefreshIcons.ps1"
!macroend
