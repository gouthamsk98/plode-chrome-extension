use std::path::{ Path, PathBuf };
use tracing::info;
use tokio::process::Command as TokioCommand;
use axum::{ extract::Json, routing::{ get, post }, http::StatusCode, Router };
use crate::models::*;
// Path to the arduino-cli binary
#[cfg(target_os = "linux")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/linux/arduino-cli"); // Change this if needed
#[cfg(target_os = "windows")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/windows/arduino-cli.exe"); // Change this if needed
#[cfg(target_os = "macos")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/macos/arduino-cli"); // Change this if needed
static ARDUINO_CLI_PATH: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

// Function to initialize the arduino-cli binary
fn initialize_arduino_cli() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let arduino_cli_path = temp_dir.join("arduino-cli-embedded");

    // Write the binary to a temporary location
    std::fs
        ::write(&arduino_cli_path, ARDUINO_CLI_BINARY)
        .expect("Failed to write arduino-cli binary to disk");

    // Make it executable on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs
            ::metadata(&arduino_cli_path)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&arduino_cli_path, perms).expect("Failed to set permissions");
    }

    arduino_cli_path
}
// Get the path to the arduino-cli binary
pub fn get_arduino_cli_path() -> &'static PathBuf {
    ARDUINO_CLI_PATH.get_or_init(initialize_arduino_cli)
}
pub fn health_check() -> bool {
    let arduino_cli_path = get_arduino_cli_path();
    let test_result = std::process::Command::new(arduino_cli_path).arg("version").output();
    match test_result {
        Ok(output) => {
            if output.status.success() {
                info!("{}", String::from_utf8_lossy(&output.stdout));
                true
            } else {
                info!("arduino-cli test failed: {}", String::from_utf8_lossy(&output.stderr));
                false
            }
        }
        Err(e) => {
            info!("Failed to execute arduino-cli: {}", e);
            false
        }
    }
}
// Helper function to run Arduino CLI commands
pub async fn run_arduino_command(command: &ArduinoCommand) -> CommandResponse {
    let arduino_cli_path = get_arduino_cli_path();

    let cmd_name = &command.command;
    let args = &command.args;

    info!("Running Arduino CLI command: {} {:?}", cmd_name, args);

    let output = TokioCommand::new(arduino_cli_path).arg(cmd_name).args(args).output().await;

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            CommandResponse {
                success: output.status.success(),
                output: stdout,
                files: None,
                error: if stderr.is_empty() {
                    None
                } else {
                    Some(stderr)
                },
                command: cmd_name.clone(),
                args: args.clone(),
            }
        }
        Err(e) =>
            CommandResponse {
                success: false,
                output: String::new(),
                files: None,
                error: Some(format!("Failed to execute command: {}", e)),
                command: cmd_name.clone(),
                args: args.clone(),
            },
    }
}
