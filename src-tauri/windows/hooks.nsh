!macro NSIS_HOOK_PREINSTALL

    DetailPrint "Checking for FFmpeg..."

    ; === 1. Check if ffmpeg is already available ===
    nsExec::ExecToStack 'ffmpeg -version'
    Pop $0

    ${If} $0 == 0
        DetailPrint "FFmpeg is already installed and accessible."
        Goto EndHook
    ${EndIf}

    ; === 2. Install via winget (with admin rights) ===
    DetailPrint "FFmpeg not found → installing via winget (Gyan.FFmpeg)..."

    ; We use RequestExecutionLevel admin in the main script, so this should work
    nsExec::ExecToStack 'winget install --id Gyan.FFmpeg -e --silent --accept-source-agreements --accept-package-agreements --scope machine'
    Pop $1

    ${If} $1 == 0
        DetailPrint "FFmpeg installed successfully via winget."
    ${Else}
        DetailPrint "Failed to install FFmpeg via winget (code $1). Trying user scope..."

        ; Fallback : install for current user only (no admin required)
        nsExec::ExecToStack 'winget install --id Gyan.FFmpeg -e --silent --accept-source-agreements --accept-package-agreements --scope user'
        Pop $1

        ${If} $1 == 0
            DetailPrint "FFmpeg installed for current user."
        ${Else}
            DetailPrint "Failed to install FFmpeg (code $1)."
            MessageBox MB_OK|MB_ICONEXCLAMATION "Could not install FFmpeg automatically.$\nPlease install it manually:$\nwinget install Gyan.FFmpeg"
            Goto EndHook
        ${EndIf}
    ${EndIf}

    ; === 3. Verify after installation ===
    DetailPrint "Verifying after installation..."

    nsExec::ExecToStack 'ffmpeg -version'
    Pop $2

    ${If} $2 == 0
        DetailPrint "FFmpeg verified successfully after installation."
    ${Else}
        DetailPrint "FFmpeg was installed but may not be in the PATH yet."
        DetailPrint "A computer restart is recommended."
        MessageBox MB_OK|MB_ICONINFORMATION "FFmpeg has been installed.$\nA restart is recommended for it to be fully usable."
    ${EndIf}

EndHook:

!macroend