!define APP_NAME "plode_web_agent"
!define INSTALL_DIR "C:\Program Files\${APP_NAME}"
!define BIN_PATH "${INSTALL_DIR}\${APP_NAME}.exe"
!define VBS_PATH "${INSTALL_DIR}\RunBackground.vbs"

Outfile "${APP_NAME}_Installer.exe"
InstallDir "${INSTALL_DIR}"

RequestExecutionLevel admin

Section "Install"
    SetOutPath "${INSTALL_DIR}"
    CreateDirectory "${INSTALL_DIR}"

    ; Copy the executable
    File "plode_web_agent.exe"
    
    ; Create VBS script to run in background
    FileOpen $0 "${VBS_PATH}" w
    FileWrite $0 'Option Explicit$\r$\n$\r$\n'
    FileWrite $0 'Dim exePath$\r$\n'
    FileWrite $0 'exePath = "${BIN_PATH}"$\r$\n$\r$\n'
    FileWrite $0 'Dim arguments$\r$\n'
    FileWrite $0 'arguments = "--background"$\r$\n$\r$\n'
    FileWrite $0 'RunInBackground exePath, arguments$\r$\n$\r$\n'
    FileWrite $0 'Sub RunInBackground(path, args)$\r$\n'
    FileWrite $0 '    Dim shell, command$\r$\n'
    FileWrite $0 '    $\r$\n'
    FileWrite $0 '    Set shell = CreateObject("WScript.Shell")$\r$\n'
    FileWrite $0 '    $\r$\n'
    FileWrite $0 '    command = """" & path & """"$\r$\n'
    FileWrite $0 '    $\r$\n'
    FileWrite $0 '    If Len(args) > 0 Then$\r$\n'
    FileWrite $0 '        command = command & " " & args$\r$\n'
    FileWrite $0 '    End If$\r$\n'
    FileWrite $0 '    $\r$\n'
    FileWrite $0 '    shell.Run command, 0, False$\r$\n'
    FileWrite $0 '    $\r$\n'
    FileWrite $0 '    Set shell = Nothing$\r$\n'
    FileWrite $0 'End Sub$\r$\n'
    FileClose $0

    ; Add to Windows startup registry (for current user) - pointing to VBS script
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${APP_NAME}" '"${VBS_PATH}"'
    
    ; Add shortcut to Startup folder (alternative method)
    CreateDirectory "$SMPROGRAMS\Startup"
    CreateShortCut "$SMPROGRAMS\Startup\${APP_NAME}.lnk" "${VBS_PATH}"
    
    ; Write the uninstaller
    WriteUninstaller "${INSTALL_DIR}\Uninstall.exe"

    ; Start application immediately in background mode using VBS
    Exec '"cscript.exe" "${VBS_PATH}"'

    MessageBox MB_OK "Installation Complete! ${APP_NAME} is now installed and will run automatically at startup."
SectionEnd

Section "Uninstall"
    ; Kill any running instances before uninstalling
    nsExec::Exec 'taskkill /F /IM ${APP_NAME}.exe'
    
    ; Remove files
    Delete "${BIN_PATH}"
    Delete "${VBS_PATH}"
    
    ; Remove startup registry entry
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${APP_NAME}"

    ; Remove startup shortcut
    Delete "$SMPROGRAMS\Startup\${APP_NAME}.lnk"

    ; Remove installation directory
    RMDir /r "${INSTALL_DIR}"

    MessageBox MB_OK "Uninstallation Complete!"
SectionEnd