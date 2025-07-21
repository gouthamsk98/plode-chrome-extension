use std::path::PathBuf;
use socketioxide::socket;
use tracing::info;
use tokio::process::Command as TokioCommand;
use axum::{ extract::{ Multipart, Query }, http::StatusCode, response::Json as ResponseJson };
use std::collections::HashMap;
use tokio::fs;
use crate::{ models::*, socketio::get_sketch_directory };
// Path to the arduino-cli binary
#[cfg(target_os = "linux")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/linux/arduino-cli"); // Change this if needed
#[cfg(target_os = "windows")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/windows/arduino-cli.exe"); // Change this if needed
#[cfg(target_os = "macos")]
static ARDUINO_CLI_BINARY: &[u8] = include_bytes!("../resource/macOS_x86_64/arduino-cli"); // Change this if needed
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

            // Try to parse stdout as JSON, fallback to string if parsing fails
            let parsed_output = match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(json) => Some(json),
                Err(_) => None,
            };

            CommandResponse {
                success: output.status.success(),
                output: stdout,
                output_json: parsed_output,
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
                output_json: None,
                files: None,
                error: Some(format!("Failed to execute command: {}", e)),
                command: cmd_name.clone(),
                args: args.clone(),
            },
    }
}

// Handler for library upload endpoint
pub async fn upload_library(
    Query(params): Query<HashMap<String, String>>,
    mut multipart: Multipart
) -> Result<ResponseJson<LibraryUploadResponse>, StatusCode> {
    let mut library_name = String::new();
    let mut file_path: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let field_name = field.name().unwrap_or("").to_string();
        info!("Processing field: {}", field_name);

        // Log content type for debugging
        if let Some(content_type) = field.content_type() {
            info!("Field content type: {}", content_type);
        }

        // Use the form field name as the library name
        library_name = field_name.clone();

        // Get file name - use field name with .zip extension
        let file_name = format!("{}.zip", field_name);
        info!("Using file name: {}", file_name);

        // Read the data as bytes directly
        let data = field.bytes().await.map_err(|e| {
            info!("Error reading bytes from field: {:?}", e);
            StatusCode::BAD_REQUEST
        })?;

        info!("Successfully read {} bytes from field '{}'", data.len(), field_name);
        // Create a temporary directory for the library
        let temp_dir = match get_sketch_directory() {
            Ok(dir) => dir.join("libraries"),
            Err(e) => {
                info!("Failed to get sketch directory: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        fs::create_dir_all(&temp_dir).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let file_path_buf = temp_dir.join(&file_name);

        // Write the uploaded file
        fs::write(&file_path_buf, &data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        file_path = Some(file_path_buf.to_string_lossy().to_string());
    }

    if library_name.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = LibraryUploadResponse {
        success: true,
        message: format!("Library '{}' uploaded successfully", library_name),
        library_name,
        file_path,
    };

    Ok(ResponseJson(response))
}
