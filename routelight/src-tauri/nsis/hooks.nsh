!macro RouteLight_UpdateShortcutIcon LINK_PATH
  IfFileExists "${LINK_PATH}" 0 +3
    CreateShortcut "${LINK_PATH}" "$INSTDIR\${MAINBINARYNAME}.exe" "" "$INSTDIR\${MAINBINARYNAME}.exe" 0
    !insertmacro SetLnkAppUserModelId "${LINK_PATH}"
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro RouteLight_UpdateShortcutIcon "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
  !insertmacro RouteLight_UpdateShortcutIcon "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !insertmacro RouteLight_UpdateShortcutIcon "$DESKTOP\${PRODUCTNAME}.lnk"
!macroend
