!macro NSIS_HOOK_POSTINSTALL
  CreateDirectory "$SMPROGRAMS\Lumo-Hub"
  CreateShortCut "$SMPROGRAMS\Lumo-Hub\卸载并清理数据.lnk" "$INSTDIR\lumo-hub.exe" "--clean-uninstall"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  Delete "$SMPROGRAMS\Lumo-Hub\卸载并清理数据.lnk"
!macroend
