use serde_json::Value;
use socketioxide::{ extract::{ AckSender, Data, SocketRef }, SocketIo };
use tracing::info;
use crate::usb::{ find_mount_point, list_directory_recursive };
use crate::models::*;
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
    // register_arduino_handlers(&socket);
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
    })
}
