!macro NSIS_HOOK_PREINSTALL
  ; Business data is stored beside the executable, so installation requires a writable directory.
  CreateDirectory "$INSTDIR"
  ClearErrors
  GetTempFileName $0 "$INSTDIR"
  ${If} ${Errors}
    MessageBox MB_OK|MB_ICONSTOP "无法写入安装目录。请选择当前用户可写的目录，例如 D:\Program\Lumo。笔记和设置将保存在所选安装目录中。"
    Abort
  ${EndIf}
  Delete "$0"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  CreateDirectory "$SMPROGRAMS\Lumo-Hub"
  CreateShortCut "$SMPROGRAMS\Lumo-Hub\卸载并清理数据.lnk" "$INSTDIR\lumo-hub.exe" "--clean-uninstall"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$SMPROGRAMS\Lumo-Hub\卸载并清理数据.lnk"
!macroend
