use serde_json::Value;
use socketioxide::{ extract::{ AckSender, Data, SocketRef }, SocketIo };
use tracing::info;
use crate::usb::{ find_mount_point, list_directory_recursive, read_file_as_base64, write_file };
use crate::models::*;
use crate::compiler::run_arduino_command;
// Target USB device VID (vendor ID)
const TARGET_VID: &str = "0xb1b0";
pub fn on_connect(socket: SocketRef, Data(data): Data<Value>) {
    info!(ns = socket.ns(), ?socket.id, "Socket.IO connected");
    socket.emit("auth", &data).ok();

    socket.on("message", |Data::<Value>(data), socket: SocketRef| {
        info!(?data, "Received event:");
        socket.emit("message-back", &data).ok();
    });

    socket.on("message-with-ack", |Data::<Value>(data), ack: AckSender| {
        info!(?data, "Received event");
        ack.send(&data).ok();
    });
    // Specific commands for common Arduino CLI operations
    register_arduino_handlers(&socket);
    register_usb_handlers(&socket);
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

// Register specific handlers for common Arduino CLI operations
fn register_arduino_handlers(socket: &SocketRef) {
    // List all available boards
    socket.on("list-boards", |ack: AckSender| {
        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "board".to_string(),
                args: vec!["listall".to_string(), "--format".to_string(), "json".to_string()],
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // List connected boards
    socket.on("list-connected", |ack: AckSender| {
        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "board".to_string(),
                args: vec!["list".to_string(), "--format".to_string(), "json".to_string()],
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // List installed cores
    socket.on("list-cores", |ack: AckSender| {
        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "core".to_string(),
                args: vec!["list".to_string(), "--format".to_string(), "json".to_string()],
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // Install a core
    socket.on("install-core", |Data::<Value>(data), ack: AckSender| {
        let core_name = match data.get("core").and_then(|v| v.as_str()) {
            Some(name) => name.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    files: None,
                    error: Some("Missing core name".to_string()),
                    command: "core".to_string(),
                    args: vec!["install".to_string()],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "core".to_string(),
                args: vec!["install".to_string(), core_name],
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // Compile a sketch
    socket.on("compile-sketch", |Data::<Value>(data), ack: AckSender| {
        // Extract sketch path and optional FQBN
        let sketch_path = match data.get("sketch_path").and_then(|v| v.as_str()) {
            Some(path) => path.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    files: None,
                    error: Some("Missing sketch path".to_string()),
                    command: "compile".to_string(),
                    args: vec![],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        let mut args = vec![];

        // Add FQBN if provided
        if let Some(fqbn) = data.get("fqbn").and_then(|v| v.as_str()) {
            args.push("--fqbn".to_string());
            args.push(fqbn.to_string());
        }

        args.push(sketch_path);

        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "compile".to_string(),
                args,
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });

    // Upload a sketch
    socket.on("upload-sketch", |Data::<Value>(data), ack: AckSender| {
        let sketch_path = match data.get("sketch_path").and_then(|v| v.as_str()) {
            Some(path) => path.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    files: None,
                    error: Some("Missing sketch path".to_string()),
                    command: "upload".to_string(),
                    args: vec![],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        let port = match data.get("port").and_then(|v| v.as_str()) {
            Some(port) => port.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    files: None,
                    error: Some("Missing port".to_string()),
                    command: "upload".to_string(),
                    args: vec![],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        let fqbn = match data.get("fqbn").and_then(|v| v.as_str()) {
            Some(fqbn) => fqbn.to_string(),
            None => {
                let error_response = CommandResponse {
                    success: false,
                    output: String::new(),
                    files: None,
                    error: Some("Missing FQBN".to_string()),
                    command: "upload".to_string(),
                    args: vec![],
                };
                ack.send(&error_response).ok();
                return;
            }
        };

        let args = vec!["--port".to_string(), port, "--fqbn".to_string(), fqbn, sketch_path];

        tokio::spawn(async move {
            let command = ArduinoCommand {
                command: "upload".to_string(),
                args,
            };

            let response = run_arduino_command(&command).await;
            ack.send(&response).ok();
        });
    });
}
