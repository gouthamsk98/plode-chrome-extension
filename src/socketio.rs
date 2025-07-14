use serde_json::Value;
use socketioxide::{ extract::{ AckSender, Data, SocketRef } };
use tracing::{ info, error };
use crate::usb::{ find_mount_point, list_directory_recursive, read_file_as_base64, write_file };
use crate::models::*;
use crate::compiler::run_arduino_command;
use std::path::Path;
use std::fs;
use chrono;
use std::sync::{ Arc, Mutex };
// Target USB device VID (vendor ID)
#[cfg(target_os = "macos")]
const TARGET_VID: &str = "0xb1b0";
#[cfg(target_os = "windows")]
const TARGET_VID: &str = "0xB1B0";
pub fn on_connect(socket: SocketRef, Data(data): Data<Value>) {
    info!(ns = socket.ns(), ?socket.id, "Socket.IO connected");
    socket.emit("auth", &data).ok();
    socket.on("version", |ack: AckSender| {
        let version = env!("CARGO_PKG_VERSION");
        info!(?version, "Version requested");
        ack.send(&version).ok();
    });
    let port_address = Arc::new(Mutex::new(Option::<String>::None));
    // Specific commands for common Arduino CLI operations
    register_arduino_handlers(&socket, Arc::clone(&port_address));
    register_usb_handlers(&socket);
    check_port_connection(socket.clone(), Arc::clone(&port_address));
    // Start automatic log monitoring
    start_log_monitoring(socket.clone());
}
fn is_device_connected(port: &str) -> bool {
    let ports = serialport::available_ports().unwrap_or_else(|_| { vec![] });
    for available_port in ports {
        if available_port.port_name == port {
            return true;
        }
    }
    false
}
fn check_port_connection(socket: SocketRef, port_address: Arc<Mutex<Option<String>>>) {
    tokio::spawn(async move {
        let mut last_status = None;
        // let port = port_address.lock();
        loop {
            match port_address.lock() {
                Ok(port) => {
                    if let Some(port) = port.as_ref() {
                        // Check if the device is connected
                        let connected = is_device_connected(port);
                        if last_status != Some(connected) {
                            socket.emit("device-connected", &connected).ok();
                            last_status = Some(connected);
                        }
                    }
                }
                Err(e) => {
                    error!(?e, "Failed to lock port address");
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
    });
}
fn register_usb_handlers(socket: &SocketRef) {
    socket.on("list-mount", |ack: AckSender| {
        tokio::spawn(async move {
            let mount_point = find_mount_point(TARGET_VID);
            match mount_point {
                Some(mount_point) => {
                    info!(?mount_point, "Mount point found");
                    let response = CommandResponse {
                        success: true,
                        output: mount_point.clone(),
                        output_json: None,
                        files: None,
                        error: None,
                        command: "list-mount".to_string(),
                        args: vec![],
                    };
                    ack.send(&response).ok();
                }
                None => {
                    info!("Mount point not found");
                    let response = CommandResponse {
                        success: false,
                        output: "".to_string(),
                        output_json: None,
                        files: None,
                        error: Some("Mount point not found".to_string()),
                        command: "list-mount".to_string(),
                        args: vec![],
                    };
                    ack.send(&response).ok();
                }
            }
        });
    });
    socket.on("list-files", |Data::<Value>(data), ack: AckSender| {
        tokio::spawn(async move {
            let mount_point = find_mount_point(TARGET_VID);
            match mount_point {
                Some(mount_point) => {
                    let file_path = match data.get("file_path").and_then(|v| v.as_str()) {
                        Some(path) => path,
                        None => mount_point.as_str(),
                    };
                    info!(?file_path, "File path to list");
                    match list_directory_recursive(&file_path) {
                        Ok(file_list) => {
                            info!(?file_list, "Files found");
                            // Update the response to use the JSON string
                            let response = CommandResponse {
                                success: true,
                                output: "".to_string(),
                                output_json: None,
                                files: Some(file_list),
                                error: None,
                                command: "list-files".to_string(),
                                args: vec![file_path.to_string()],
                            };
                            ack.send(&response).ok();
                        }
                        Err(e) => {
                            info!(?e, "Error listing files");
                            let response = CommandResponse {
                                success: false,
                                output: "".to_string(),
                                output_json: None,
                                files: None,
                                error: Some(format!("Error listing files: {}", e)),
                                command: "list-files".to_string(),
                                args: vec![file_path.to_string()],
                            };
                            ack.send(&response).ok();
                        }
                    }
                }
                None => {
                    info!("Mount point not found");
                    let response = CommandResponse {
                        success: false,
                        output: "".to_string(),
                        output_json: None,
                        files: None,
                        error: Some("Mount point not found".to_string()),
                        command: "list-files".to_string(),
                        args: vec![],
                    };
                    ack.send(&response).ok();
                }
            }
        });
    });
    socket.on("add-file", |Data::<UsbAddFileRequest>(data), ack: AckSender| {
        let name = data.name;
        let format = data.format;
        let path = data.path.unwrap_or_default();
        let data_str = data.data;
        info!("Handling usb-add-file request for {}.{} in {}", name, format, path);
        match find_mount_point(TARGET_VID) {
            Some(mount_point) => {
                info!(?path, "File path to add");
                match write_file(&path, &name, &format, &data_str) {
                    Ok(_) => {
                        info!("File written successfully");
                        let response = CommandResponse {
                            success: true,
                            output: path.clone(),
                            output_json: None,
                            files: None,
                            error: None,
                            command: "add-file".to_string(),
                            args: vec![path],
                        };
                        ack.send(&response).ok();
                    }
                    Err(e) => {
                        info!(?e, "Error writing file");
                        let response = CommandResponse {
                            success: false,
                            output: "".to_string(),
                            output_json: None,
                            files: None,
                            error: Some(format!("Error writing file: {}", e)),
                            command: "add-file".to_string(),
                            args: vec![path],
                        };
                        ack.send(&response).ok();
                    }
                }
            }
            None => {
                info!("Mount point not found");
                let response = CommandResponse {
                    success: false,
                    output: "".to_string(),
                    output_json: None,
                    files: None,
                    error: Some("Mount point not found".to_string()),
                    command: "add-file".to_string(),
                    args: vec![],
                };
                ack.send(&response).ok();
            }
        }
    });
    socket.on("remove-file", |Data::<Value>(data), ack: AckSender| {
        tokio::spawn(async move {
            let mount_point = find_mount_point(TARGET_VID);
            match mount_point {
                Some(mount_point) => {
                    let file_path = match data.get("file_path").and_then(|v| v.as_str()) {
                        Some(path) => path,
                        None => mount_point.as_str(),
                    };
                    info!(?file_path, "File path to remove");
                    match std::fs::remove_file(&file_path) {
                        Ok(_) => {
                            info!("File removed successfully");
                            let response = CommandResponse {
                                success: true,
                                output: file_path.to_string(),
                                output_json: None,
                                files: None,
                                error: None,
                                command: "remove-file".to_string(),
                                args: vec![file_path.to_string()],
                            };
                            ack.send(&response).ok();
                        }
                        Err(e) => {
                            info!(?e, "Error removing file");
                            let response = CommandResponse {
                                success: false,
                                output: "".to_string(),
                                output_json: None,
                                files: None,
                                error: Some(format!("Error removing file: {}", e)),
                                command: "remove-file".to_string(),
                                args: vec![file_path.to_string()],
                            };
                            ack.send(&response).ok();
                        }
                    }
                }
                None => {
                    info!("Mount point not found");
                    let response = CommandResponse {
                        success: false,
                        output: "".to_string(),
                        output_json: None,
                        files: None,
                        error: Some("Mount point not found".to_string()),
                        command: "remove-file".to_string(),
                        args: vec![],
                    };
                    ack.send(&response).ok();
                }
            }
        });
    });
    socket.on("remove-dir", |Data::<Value>(data), ack: AckSender| {
        tokio::spawn(async move {
            let mount_point = find_mount_point(TARGET_VID);
            match mount_point {
                Some(mount_point) => {
                    let dir_path = match data.get("dir_path").and_then(|v| v.as_str()) {
                        Some(path) => path,
                        None => mount_point.as_str(),
                    };
                    info!(?dir_path, "Directory path to remove");
                    match std::fs::remove_dir_all(&dir_path) {
                        Ok(_) => {
                            info!("Directory removed successfully");
                            let response = CommandResponse {
                                success: true,
                                output: dir_path.to_string(),
                                output_json: None,
                                files: None,
                                error: None,
                                command: "remove-dir".to_string(),
                                args: vec![dir_path.to_string()],
                            };
                            ack.send(&response).ok();
                        }
                        Err(e) => {
                            info!(?e, "Error removing directory");
                            let response = CommandResponse {
                                success: false,
                                output: "".to_string(),
                                output_json: None,
                                files: None,
                                error: Some(format!("Error removing directory: {}", e)),
                                command: "remove-dir".to_string(),
                                args: vec![dir_path.to_string()],
                            };
                            ack.send(&response).ok();
                        }
                    }
                }
                None => {
                    info!("Mount point not found");
                    let response = CommandResponse {
                        success: false,
                        output: "".to_string(),
                        files: None,
                        error: Some("Mount point not found".to_string()),
                        command: "remove-dir".to_string(),
                        args: vec![],
                        output_json: None,
                    };
                    ack.send(&response).ok();
                }
            }
        });
    });
    socket.on("read-file", |Data::<Value>(data), ack: AckSender| {
        tokio::spawn(async move {
            let mount_point = find_mount_point(TARGET_VID);
            match mount_point {
                Some(mount_point) => {
                    let file_path = match data.get("file_path").and_then(|v| v.as_str()) {
                        Some(path) => path,
                        None => mount_point.as_str(),
                    };
                    info!(?file_path, "File path to read");
                    match read_file_as_base64(file_path) {
                        Ok(base64_data) => {
                            info!("File read successfully");
                            let response = CommandResponse {
                                success: true,
                                output: base64_data,
                                output_json: None,
                                files: None,
                                error: None,
                                command: "read-file".to_string(),
                                args: vec![file_path.to_string()],
                            };

                            ack.send(&response).ok();
                        }
                        Err(e) => {
                            info!(?e, "Error reading file");
                            let response = CommandResponse {
                                success: false,
                                output: "".to_string(),
                                output_json: None,
                                files: None,
                                error: Some(format!("Error reading file: {}", e)),
                                command: "read-file".to_string(),
                                args: vec![file_path.to_string()],
                            };
                            ack.send(&response).ok();
                        }
                    }
                }
                None => {
                    info!("Mount point not found");
                    let response = CommandResponse {
                        success: false,
                        output: "".to_string(),
                        output_json: None,
                        files: None,
                        error: Some("Mount point not found".to_string()),
                        command: "read-file".to_string(),
                        args: vec![],
                    };
                    ack.send(&response).ok();
                }
            }
        });
    });
}
fn get_sketches_list() -> Value {
    todo!("Implement get_sketches_list function to return a list of sketches in JSON format");
}
// Register specific handlers for common Arduino CLI operations
// Helper function to create error responses
fn create_error_response(error_msg: &str, command: &str, args: Vec<String>) -> CommandResponse {
    CommandResponse {
        success: false,
        output: String::new(),
        output_json: None,
        files: None,
        error: Some(error_msg.to_string()),
        command: command.to_string(),
        args,
    }
}

// Helper function to create success responses
fn create_success_response(
    output: String,
    command: &str,
    args: Vec<String>,
    output_json: Option<Value>
) -> CommandResponse {
    CommandResponse {
        success: true,
        output,
        output_json,
        files: None,
        error: None,
        command: command.to_string(),
        args,
    }
}

// Helper function to extract string field from JSON data
fn extract_string_field(data: &Value, field: &str) -> Option<String> {
    data.get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// Helper function to get sketch directory
fn get_sketch_directory() -> Result<std::path::PathBuf, String> {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let sketch_dir = current_dir.join("sketches");

    if !sketch_dir.exists() {
        return Err("Sketch directory does not exist".to_string());
    }

    Ok(sketch_dir)
}

// Helper function to read file content
async fn read_file_content(file_path: &std::path::Path) -> Result<String, String> {
    match std::fs::read_to_string(file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

// Helper function to extract multiple required fields from JSON data
fn extract_required_fields(data: &Value, fields: &[&str]) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    for field in fields {
        match extract_string_field(data, field) {
            Some(value) => values.push(value),
            None => {
                return Err(format!("Missing {}", field));
            }
        }
    }
    Ok(values)
}

// Helper function to run Arduino command asynchronously
async fn run_arduino_command_async(command: &str, args: Vec<String>, ack: AckSender) {
    let arduino_command = ArduinoCommand {
        command: command.to_string(),
        args,
    };
    let response = run_arduino_command(&arduino_command).await;
    ack.send(&response).ok();
}

fn register_arduino_handlers(socket: &SocketRef, port_address: Arc<Mutex<Option<String>>>) {
    socket.on("connect-device", |Data::<Value>(data), ack: AckSender| {
        tokio::spawn(async move {
            let port_address_clone = Arc::clone(&port_address);
            info!("Handling connect event");

            let port_address = match extract_string_field(&data, "port_address") {
                Some(address) => address,
                None => {
                    let error_response = create_error_response(
                        "Missing mount address",
                        "connect-device",
                        vec![]
                    );
                    ack.send(&error_response).ok();
                    return;
                }
            };

            info!(?port_address, "USB mount address received");
            match port_address_clone.lock() {
                Ok(mut port_lock) => {
                    *port_lock = Some(port_address.clone());
                }
                Err(e) => {
                    error!(?e, "Failed to lock port address");
                }
            }

            let connected = is_device_connected(port_address.as_str());
            let response = create_success_response(
                port_address.clone(),
                "connect",
                vec![port_address],
                None
            );
            let response = CommandResponse {
                success: connected,
                ..response
            };
            ack.send(&response).ok();
        });
    });
    // List all available boards
    socket.on("list-boards", |ack: AckSender| {
        tokio::spawn(async move {
            run_arduino_command_async(
                "board",
                vec!["listall".to_string(), "--format".to_string(), "json".to_string()],
                ack
            ).await;
        });
    });

    // List connected boards
    socket.on("list-connected", |ack: AckSender| {
        tokio::spawn(async move {
            run_arduino_command_async(
                "board",
                vec!["list".to_string(), "--format".to_string(), "json".to_string()],
                ack
            ).await;
        });
    });

    // List installed cores
    socket.on("list-cores", |ack: AckSender| {
        tokio::spawn(async move {
            run_arduino_command_async(
                "core",
                vec!["list".to_string(), "--format".to_string(), "json".to_string()],
                ack
            ).await;
        });
    });
    // Install a core
    socket.on("install-core", |Data::<Value>(data), ack: AckSender| {
        let core_name = match extract_string_field(&data, "core") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing core name",
                    "core",
                    vec!["install".to_string()]
                );
                ack.send(&error_response).ok();
                return;
            }
        };

        tokio::spawn(async move {
            let args = vec![
                "install".to_string(),
                core_name,
                "--log".to_string(),
                "--log-file".to_string(),
                "log.txt".to_string()
            ];
            run_arduino_command_async("core", args, ack).await;
        });
    });
    //create a new sketch
    socket.on("create-sketch", |Data::<Value>(data), ack: AckSender| {
        let sketch_name = match extract_string_field(&data, "sketch_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing sketch name",
                    "create-sketch",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        let sketch_path = format!("sketches/{}", sketch_name);
        tokio::spawn(async move {
            let args = vec![
                "new".to_string(),
                sketch_path,
                "--format".to_string(),
                "json".to_string()
            ];
            run_arduino_command_async("sketch", args, ack).await;
        });
    });
    socket.on("read-sketch-file", |Data::<Value>(data), ack: AckSender| {
        let fields = match extract_required_fields(&data, &["sketch_name", "file_name"]) {
            Ok(values) => values,
            Err(error_msg) => {
                let error_response = create_error_response(&error_msg, "read-file", vec![]);
                ack.send(&error_response).ok();
                return;
            }
        };

        let _sketch_name = fields[0].clone();
        let file_name = fields[1].clone();

        tokio::spawn(async move {
            let sketch_dir = match get_sketch_directory() {
                Ok(dir) => dir,
                Err(error_msg) => {
                    let error_response = create_error_response(&error_msg, "read-file", vec![]);
                    ack.send(&error_response).ok();
                    return;
                }
            };

            let sketch_path = sketch_dir.join(&file_name);
            match read_file_content(&sketch_path).await {
                Ok(content) => {
                    let success_response = create_success_response(
                        content,
                        "read-sketch-file",
                        vec![sketch_path.to_string_lossy().to_string()],
                        None
                    );
                    ack.send(&success_response).ok();
                }
                Err(error_msg) => {
                    let error_response = create_error_response(&error_msg, "read-file", vec![]);
                    ack.send(&error_response).ok();
                }
            }
        });
    });
    socket.on("write-sketch-file", |Data::<Value>(data), ack: AckSender| {
        let fields = match
            extract_required_fields(&data, &["sketch_name", "file_name", "file_value"])
        {
            Ok(values) => values,
            Err(error_msg) => {
                let error_response = create_error_response(&error_msg, "write-file", vec![]);
                ack.send(&error_response).ok();
                return;
            }
        };

        let sketch_name = fields[0].clone();
        let file_name = fields[1].clone();
        let file_value = fields[2].clone();

        tokio::spawn(async move {
            //get the sketch directory
            let mut sketch_dir = match get_sketch_directory() {
                Ok(dir) => dir,
                Err(error_msg) => {
                    let error_response = create_error_response(&error_msg, "write-file", vec![]);
                    ack.send(&error_response).ok();
                    return;
                }
            };
            sketch_dir.push(&sketch_name);
            sketch_dir.push(&file_name);
            //write the file
            match std::fs::write(&sketch_dir, file_value) {
                Ok(_) => {
                    let success_response = create_success_response(
                        sketch_dir.to_string_lossy().to_string(),
                        "write-sketch-file",
                        vec![sketch_dir.to_string_lossy().to_string()],
                        None
                    );
                    ack.send(&success_response).ok();
                }
                Err(e) => {
                    let error_response = create_error_response(
                        &format!("Failed to write sketch file: {}", e),
                        "write-file",
                        vec![]
                    );
                    ack.send(&error_response).ok();
                }
            }
        });
    });
    socket.on("delete-sketch-file", |Data::<Value>(data), ack: AckSender| {
        let sketch_name = match extract_string_field(&data, "sketch_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing sketch name",
                    "delete=sketch-file",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        let file_name = match extract_string_field(&data, "file_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing file name",
                    "delete-file",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        let mut sketch_path = match get_sketch_directory() {
            Ok(dir) => dir,
            Err(error_msg) => {
                let error_response = create_error_response(&error_msg, "delete-file", vec![]);
                ack.send(&error_response).ok();
                return;
            }
        };
        sketch_path.push(&sketch_name);
        sketch_path.push(&file_name);
        tokio::spawn(async move {
            match std::fs::remove_file(&sketch_path) {
                Ok(_) => {
                    let success_response = create_success_response(
                        sketch_path.to_string_lossy().to_string(),
                        "delete-file",
                        vec![sketch_path.to_string_lossy().to_string()],
                        None
                    );
                    ack.send(&success_response).ok();
                }
                Err(e) => {
                    let error_response = create_error_response(
                        &format!("Failed to delete sketch file: {}", e),
                        "delete-file",
                        vec![]
                    );
                    ack.send(&error_response).ok();
                }
            }
        });
    });
    socket.on("list-sketches", |ack: AckSender| {
        let mut projects = vec![];
        tokio::spawn(async move {
            let sketch_dir = match get_sketch_directory() {
                Ok(dir) => dir,
                Err(error_msg) => {
                    let error_response = create_error_response(
                        &format!("Failed to read sketches directory: {}", error_msg),
                        "list-sketches",
                        vec![]
                    );
                    ack.send(&error_response).ok();
                    return;
                }
            };
            let sketches = match std::fs::read_dir(&sketch_dir) {
                Ok(entries) => {
                    entries
                        .filter_map(|entry| {
                            entry.ok().and_then(|e| {
                                let path = e.path();
                                if path.is_dir() {
                                    Some(path.file_name()?.to_string_lossy().to_string())
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<_>>()
                }
                Err(error_msg) => {
                    let error_response = create_error_response(
                        &error_msg.to_string(),
                        "list-sketches",
                        vec![]
                    );
                    ack.send(&error_response).ok();
                    return;
                }
            };
            projects.extend(sketches);
            let response = CommandResponse {
                success: true,
                output: "Sketches listed successfully".to_string(),
                output_json: Some(serde_json::to_value(projects.clone()).unwrap_or(Value::Null)),
                files: None,
                error: None,
                command: "list-sketches".to_string(),
                args: vec![],
            };
            ack.send(&response).ok();
        });
    });
    socket.on("remove-sketch", |Data::<Value>(_data), ack: AckSender| {
        let sketch_name = match extract_string_field(&_data, "sketch_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing sketch name",
                    "remove-sketch",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        let mut sketch_path = match get_sketch_directory() {
            Ok(dir) => dir,
            Err(error_msg) => {
                let error_response = create_error_response(&error_msg, "remove-sketch", vec![]);
                ack.send(&error_response).ok();
                return;
            }
        };
        sketch_path.push(&sketch_name);

        tokio::spawn(async move {
            if std::fs::remove_dir_all(&sketch_path).is_ok() {
                let success_response = create_success_response(
                    sketch_path.to_string_lossy().to_string(),
                    "remove-sketch",
                    vec![sketch_path.to_string_lossy().to_string()],
                    None
                );
                ack.send(&success_response).ok();
            } else {
                let error_response = create_error_response(
                    &format!("Failed to remove sketch: {}", sketch_name),
                    "remove-sketch",
                    vec![]
                );
                ack.send(&error_response).ok();
            }
        });
    });
    //lists files inisde a sketch
    socket.on("list-sketch-files", |Data::<Value>(data), ack: AckSender| {
        let sketch_name = match extract_string_field(&data, "sketch_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing sketch name",
                    "list-sketch-files",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        let mut sketch_path = match get_sketch_directory() {
            Ok(dir) => dir,
            Err(error_msg) => {
                let error_response = create_error_response(&error_msg, "list-sketch-files", vec![]);
                ack.send(&error_response).ok();
                return;
            }
        };
        sketch_path.push(&sketch_name);
        if !sketch_path.exists() {
            let error_response = create_error_response(
                &format!("Sketch {} does not exist", sketch_name),
                "list-sketch-files",
                vec![]
            );
            ack.send(&error_response).ok();
            return;
        }
        tokio::spawn(async move {
            match list_directory_recursive(&sketch_path.to_string_lossy()) {
                Ok(file_list) => {
                    let response = CommandResponse {
                        success: true,
                        output: "Sketch files listed successfully".to_string(),
                        output_json: None,
                        files: Some(file_list),
                        error: None,
                        command: "list-sketch-files".to_string(),
                        args: vec![sketch_path.to_string_lossy().to_string()],
                    };
                    ack.send(&response).ok();
                }
                Err(e) => {
                    let error_response = create_error_response(
                        &format!("Error listing sketch files: {}", e),
                        "list-sketch-files",
                        vec![]
                    );
                    ack.send(&error_response).ok();
                }
            }
        });
    });
    // Compile a sketch
    socket.on("compile-sketch", |Data::<Value>(data), ack: AckSender| {
        let sketch_name = match extract_string_field(&data, "sketch_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing sketch name",
                    "compile",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        let mut sketch_path = match get_sketch_directory() {
            Ok(dir) => dir,
            Err(error_msg) => {
                let error_response = create_error_response(&error_msg, "compile", vec![]);
                ack.send(&error_response).ok();
                return;
            }
        };
        sketch_path.push(&sketch_name);

        let mut args = vec![];

        // Add FQBN if provided
        if let Some(fqbn) = extract_string_field(&data, "fqbn") {
            args.push("--fqbn".to_string());
            args.push(fqbn);
        }

        args.push(sketch_name);
        args.extend(vec!["--log".to_string(), "--log-file".to_string(), "log.txt".to_string()]);

        tokio::spawn(async move {
            run_arduino_command_async("compile", args, ack).await;
        });
    });
    // Upload a sketch
    socket.on("upload-sketch", |Data::<Value>(data), ack: AckSender| {
        let fields = match extract_required_fields(&data, &["sketch_path", "port", "fqbn"]) {
            Ok(values) => values,
            Err(error_msg) => {
                let error_response = create_error_response(&error_msg, "upload", vec![]);
                ack.send(&error_response).ok();
                return;
            }
        };

        let sketch_path = fields[0].clone();
        let port = fields[1].clone();
        let fqbn = fields[2].clone();

        let args = vec![
            "--port".to_string(),
            port,
            "--fqbn".to_string(),
            fqbn,
            sketch_path,
            "--log".to_string(),
            "--log-file".to_string(),
            "log.txt".to_string()
        ];

        tokio::spawn(async move {
            run_arduino_command_async("upload", args, ack).await;
        });
    });
    // libary commands
    socket.on("list-libraries", |ack: AckSender| {
        tokio::spawn(async move {
            run_arduino_command_async(
                "lib",
                vec!["list".to_string(), "--format".to_string(), "json".to_string()],
                ack
            ).await;
        });
    });
    // serach for a library
    socket.on("search-library", |Data::<Value>(data), ack: AckSender| {
        let library_name = match extract_string_field(&data, "library_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing library name",
                    "search-library",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        tokio::spawn(async move {
            let args = vec![
                "lib".to_string(),
                "search".to_string(),
                library_name,
                "--format".to_string(),
                "json".to_string()
            ];
            run_arduino_command_async("lib", args, ack).await;
        });
    });
    socket.on("install-library", |Data::<Value>(data), ack: AckSender| {
        let library_name = match extract_string_field(&data, "library_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing library name",
                    "install-library",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        tokio::spawn(async move {
            let args = vec![
                "lib".to_string(),
                "install".to_string(),
                library_name,
                "--log".to_string(),
                "--log-file".to_string(),
                "log.txt".to_string()
            ];
            run_arduino_command_async("lib", args, ack).await;
        });
    });
    socket.on("uninstall-library", |Data::<Value>(data), ack: AckSender| {
        let library_name = match extract_string_field(&data, "library_name") {
            Some(name) => name,
            None => {
                let error_response = create_error_response(
                    "Missing library name",
                    "uninstall-library",
                    vec![]
                );
                ack.send(&error_response).ok();
                return;
            }
        };
        tokio::spawn(async move {
            let args = vec![
                "lib".to_string(),
                "uninstall".to_string(),
                library_name,
                "--log".to_string(),
                "--log-file".to_string(),
                "log.txt".to_string()
            ];
            run_arduino_command_async("lib", args, ack).await;
        });
    });
}

fn start_log_monitoring(socket: SocketRef) {
    tokio::spawn(async move {
        let log_file_path = "log.txt";

        // Create the log file if it doesn't exist
        if !std::path::Path::new(log_file_path).exists() {
            if let Err(e) = std::fs::write(log_file_path, "") {
                error!(?e, "Failed to create log file");
                return;
            }
        }

        info!("Started automatic log monitoring for file: {}", log_file_path);

        // Start file monitoring
        start_log_file_monitoring(socket, log_file_path.to_string()).await;
    });
}

async fn start_log_file_monitoring(socket: SocketRef, log_file_path: String) {
    use std::time::Duration;
    use tokio::time::sleep;

    let mut last_position = 0;
    let path = Path::new(&log_file_path);

    loop {
        if path.exists() {
            match fs::metadata(&path) {
                Ok(metadata) => {
                    let current_size = metadata.len();
                    if current_size > last_position {
                        // Read new content
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                let new_content = if last_position > 0 {
                                    content
                                        .chars()
                                        .skip(last_position as usize)
                                        .collect::<String>()
                                } else {
                                    content
                                };

                                if !new_content.is_empty() {
                                    // Try to parse as JSON lines
                                    let lines: Vec<&str> = new_content.lines().collect();
                                    for line in lines {
                                        if !line.trim().is_empty() {
                                            let log_data = match
                                                serde_json::from_str::<Value>(line)
                                            {
                                                Ok(json) => json,
                                                Err(_) =>
                                                    serde_json::json!({
                                                    "message": line,
                                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                                }),
                                            };

                                            socket.emit("logs", &log_data).ok();
                                        }
                                    }
                                }
                                last_position = current_size;
                            }
                            Err(e) => {
                                info!("Error reading log file: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    info!("Error getting file metadata: {}", e);
                }
            }
        } else {
            info!("Log file does not exist: {}", log_file_path);
        }

        sleep(Duration::from_millis(100)).await;
    }
}
