; Steps of Crest's own that Tauri's NSIS installer runs (bundle.windows.nsis.installerHooks in
; tauri.conf.json; the hook names are Tauri's).

; Before the files go: take Crest's hooks out of Claude Code's settings, so Claude Code isn't
; left starting a crest.exe that no longer exists. Not during an update (/UPDATE): the same
; crest.exe comes right back. An upgrade that uninstalls the old version first (without
; /UPDATE) removes them too; the new Crest adds them back at its next start while the
; integration is on.
!macro NSIS_HOOK_PREUNINSTALL
  ${If} $UpdateMode <> 1
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --remove-claude-code-hooks'
  ${EndIf}
!macroend
