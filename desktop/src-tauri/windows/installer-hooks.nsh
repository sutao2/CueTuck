!define CUETUCK_WINDOWS_SOURCE "${__FILEDIR__}"

!macro NSIS_HOOK_POSTINSTALL
  SetOutPath "$INSTDIR"
  File /oname=CueTuck.icon.ico "${CUETUCK_WINDOWS_SOURCE}\..\icons\icon.ico"
  File /oname=CueTuck.RefreshIcons.ps1 "${CUETUCK_WINDOWS_SOURCE}\..\..\..\scripts\windows-refresh-icons.ps1"
  Push $0
  ExecWait '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -WindowStyle Hidden -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "$INSTDIR\CueTuck.RefreshIcons.ps1" -ApplicationPath "$INSTDIR\cuetuck.exe" -IconPath "$INSTDIR\CueTuck.icon.ico"' $0
  DetailPrint "CueTuck shortcut icon refresh returned $0"
  Pop $0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\CueTuck.icon.ico"
  Delete "$INSTDIR\CueTuck.RefreshIcons.ps1"
!macroend
