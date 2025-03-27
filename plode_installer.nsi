!define APP_NAME "plode"
!define INSTALL_DIR "C:\Program Files\${APP_NAME}"
!define BIN_PATH "${INSTALL_DIR}\${APP_NAME}.exe"
!define REG_KEY "Software\Google\Chrome\NativeMessagingHosts\com.${APP_NAME}.native"

Outfile "${APP_NAME}_Installer.exe"
InstallDir "${INSTALL_DIR}"

RequestExecutionLevel admin

Section "Install"
    SetOutPath "${INSTALL_DIR}"
    CreateDirectory "${INSTALL_DIR}"

    ; Copy the executable
    File "target\x86_64-pc-windows-gnu\release\plode.exe"

    ; Create JSON for Native Messaging
    SetOutPath "$APPDATA\Google\Chrome\NativeMessagingHosts"
    
    FileOpen $0 "$APPDATA\Google\Chrome\NativeMessagingHosts\com.${APP_NAME}.native.json" w
    FileWrite $0 '{$\r$\n'
    FileWrite $0 ' "name": "com.${APP_NAME}.native",$\r$\n'
    FileWrite $0 ' "description": "Native messaging host for ${APP_NAME}",$\r$\n'
    FileWrite $0 ' "path$\": "C:\\Program Files\\${APP_NAME}\\${APP_NAME}.exe",$\r$\n'
    FileWrite $0 ' "type": "stdio",$\r$\n'
    FileWrite $0 ' "allowed_origins": [$\r$\n'
    FileWrite $0 '    "chrome-extension://knldjmfmopnpolahpmmgbagdohdnhkik/$\"$\r$\n'
    FileWrite $0 '  ]$\r$\n'
    FileWrite $0 '}$\r$\n'
    FileClose $0

    ; Add registry entry using REG ADD
    nsExec::Exec 'REG ADD "HKCU\Software\Google\Chrome\NativeMessagingHosts\com.${APP_NAME}.native" /ve /t REG_SZ /d "$APPDATA\Google\Chrome\NativeMessagingHosts\com.${APP_NAME}.native.json" /f'

    ; Create a scheduled task to run the app on startup
    nsExec::Exec 'schtasks /Create /SC ONLOGON /TN "${APP_NAME}Startup" /TR "$\"${BIN_PATH}$\"" /RL HIGHEST /F'

    ; Write the uninstaller inside the install section
    WriteUninstaller "${INSTALL_DIR}\Uninstall.exe"

    MessageBox MB_OK "Installation Complete! The application is now installed."
SectionEnd

Section "Uninstall"
    ; Remove executable
    Delete "${BIN_PATH}"

    ; Remove JSON configuration
    Delete "$APPDATA\Google\Chrome\NativeMessagingHosts\com.${APP_NAME}.native.json"

    ; Remove registry entry
    nsExec::Exec 'REG DELETE "HKCU\Software\Google\Chrome\NativeMessagingHosts\com.${APP_NAME}.native" /f'

    ; Remove scheduled task
    nsExec::Exec 'schtasks /Delete /TN "${APP_NAME}Startup" /F'

    ; Remove installation directory
    RMDir /r "${INSTALL_DIR}"

    MessageBox MB_OK "Uninstallation Complete!"
SectionEnd
