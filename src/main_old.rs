use std::fs;
use std::path::Path;
use std::process::Command;
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{ channel, Sender };
use regex::Regex;
use std::io::{ BufRead, BufReader };
use std::thread;
use std::process;
use std::io::{ self, Read, Write };
use base64::{ engine::general_purpose::STANDARD, Engine };

#[derive(Debug)]
struct UsbDevice {
    vendor_id: String,
    product_id: String,
    mount_point: Option<String>,
}
fn send_message(message: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    let len = message.len() as u32;
    let len_bytes = len.to_le_bytes();
    stdout.write_all(&len_bytes)?;
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;
    // debug!("Sent message: {}", message);
    Ok(())
}
fn read_thread_func(tx: Sender<Option<String>>) {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        let mut len_bytes = [0u8; 4];
        // Read the 4-byte message length.
        if let Err(e) = handle.read_exact(&mut len_bytes) {
            // error!("Failed to read message length: {}", e);
            let _ = tx.send(None);
            process::exit(1);
        }

        let text_length = u32::from_le_bytes(len_bytes) as usize;
        let mut buf = vec![0u8; text_length];

        // Read the message body.
        if let Err(e) = handle.read_exact(&mut buf) {
            // error!("Failed to read message body: {}", e);
            let _ = tx.send(None);
            process::exit(1);
        }

        let text = match String::from_utf8(buf) {
            Ok(s) => s,
            Err(e) => {
                // error!("Failed to decode message: {}", e);
                continue;
            }
        };

        // debug!("Received message: {}", text);

        // Check for exit condition.
        if text.trim() == r#"{"text":"exit"}"# {
            break;
        }

        if tx.send(Some(text)).is_err() {
            break;
        }
    }
}

fn main() {
    let target_vid = "0xb1b0";
    let pid = "0x8055";
    // match find_mount_point(target_vid, target_pid) {
    //     Some(mount_point) =>
    //         println!("Device VID:{} PID:{} is mounted at: {}", target_vid, target_pid, mount_point),
    //     None => println!("Device VID:{} PID:{} not found or not mounted", target_vid, target_pid),
    // }
    // Send an initial message.
    if let Err(e) = send_message(r#""Running ...""#) {
        // error!("Failed to send initial message: {}", e);
    }
    // Setup a channel to receive messages from the reader thread.
    let (tx, rx) = channel();

    // Start the reading thread.
    let reader = thread::spawn(move || {
        read_thread_func(tx);
    });

    // Since we're in headless mode, we just process messages from the channel.
    for maybe_message in rx {
        match maybe_message {
            Some(message) => {
                match message.as_str() {
                    r#""version""# => {
                        send_message(r#""0.1.3""#).unwrap();
                    }
                    r#""help""# => {
                        send_message(r#""Available commands: list, rm, add""#).unwrap();
                    }
                    r#""mount""# => {
                        match find_mount_point(target_vid) {
                            Some(mount_point) => {
                                let message = format!(
                                    r#""Device VID:{} PID:{} is mounted at: {}""#,
                                    target_vid,
                                    pid,
                                    mount_point
                                );
                                send_message(&message).unwrap();
                                send_message(r#""END""#).unwrap();
                            }
                            None => {
                                let message = format!(
                                    r#""Device VID:{} PID:{} not found or not mounted""#,
                                    target_vid,
                                    pid
                                );
                                send_message(&message).unwrap();
                                send_message(r#""END""#).unwrap();
                            }
                        }
                    }
                    command if command.starts_with(r#""list"#) => {
                        // Check if the message contains a path parameter
                        let re = regex::Regex
                            ::new(r#""list(?:\s+--path\s+\\"([^"]+)\\")?"#)
                            .unwrap();
                        let target_path = if let Some(caps) = re.captures(&message) {
                            caps.get(1).map(|m| m.as_str())
                        } else {
                            None
                        };

                        match find_mount_point(target_vid) {
                            Some(mount_point) => {
                                let message = format!(
                                    r#""Device VID:{} PID:{} is mounted at: {}""#,
                                    target_vid,
                                    pid,
                                    mount_point
                                );

                                send_message(&message).unwrap();

                                // Determine the directory to list
                                let dir_to_list = if let Some(path) = target_path {
                                    Path::new(&mount_point).join(path).to_string_lossy().to_string()
                                } else {
                                    mount_point.clone()
                                };

                                // Send message about which directory we're listing
                                let list_msg = format!(r#""Listing contents of: {}""#, dir_to_list);
                                send_message(&list_msg).unwrap();

                                // Define a recursive function to list files
                                fn list_dir_recursive(path: &str) {
                                    if let Ok(entries) = fs::read_dir(path) {
                                        for entry in entries {
                                            if let Ok(entry) = entry {
                                                let path_buf = entry.path();
                                                let file_path = path_buf.display().to_string();

                                                // If this is a directory, recursively list its contents
                                                if path_buf.is_dir() {
                                                    let message =
                                                        format!(r#""Directory: {}""#, file_path);
                                                    send_message(&message).unwrap();
                                                    list_dir_recursive(&file_path);
                                                } else {
                                                    // Send the file path
                                                    let message =
                                                        format!(r#""File: {}""#, file_path);
                                                    send_message(&message).unwrap();
                                                }
                                            }
                                        }
                                    } else {
                                        let message =
                                            format!(r#""Error reading directory: {}""#, path);
                                        send_message(&message).unwrap();
                                    }
                                }

                                // Check if the directory exists
                                let dir_path = Path::new(&dir_to_list);
                                if dir_path.exists() && dir_path.is_dir() {
                                    // Start the recursive listing
                                    list_dir_recursive(&dir_to_list);
                                } else {
                                    let error_msg =
                                        format!(r#""Directory not found: {}""#, dir_to_list);
                                    send_message(&error_msg).unwrap();
                                }

                                send_message(r#""END""#).unwrap();
                            }
                            None => {
                                let message = format!(
                                    r#""Device VID:{} PID:{} not found or not mounted""#,
                                    target_vid,
                                    pid
                                );
                                send_message(&message).unwrap();
                                send_message(r#""END""#).unwrap();
                            }
                        }
                    }
                    command if command.starts_with(r#""open"#) => {
                        let re = regex::Regex::new(r#""open\s+--loc\s+\\"([^"]+)\\"""#).unwrap();
                        if let Some(caps) = re.captures(&message) {
                            let location = &caps[1];
                            if Path::new(location).exists() {
                                send_message(
                                    &format!(r#""File at location '{}' opened successfully""#, location)
                                ).unwrap();

                                match fs::read(location) {
                                    Ok(file_data) => {
                                        let base64_string = STANDARD.encode(&file_data);
                                        send_message(
                                            &format!(r#""File base64: {}""#, base64_string)
                                        ).unwrap();
                                        send_message(r#""END""#).unwrap();
                                    }
                                    Err(e) => {
                                        send_message(
                                            &format!(r#""Failed to read file data: {}""#, e)
                                        ).unwrap();
                                        send_message(r#""END""#).unwrap();
                                    }
                                }
                            } else {
                                send_message(
                                    r#""File not found at the specified location""#
                                ).unwrap();
                                send_message(r#""END""#).unwrap();
                            }
                        } else {
                            send_message(r#""Invalid open command format""#).unwrap();
                            send_message(r#""END""#).unwrap();
                        }
                    }
                    command if command.starts_with(r#""rm"#) => {
                        let re = regex::Regex::new(r#""rm\s+--path\s+\\"([^"]+)\\"""#).unwrap();
                        if let Some(caps) = re.captures(&message) {
                            let name = &caps[1];
                            match find_mount_point(target_vid) {
                                Some(mount_point) => {
                                    let file_path = Path::new(name);
                                    if file_path.exists() {
                                        fs::remove_file(&file_path).unwrap();
                                        send_message(r#""File removed successfully""#).unwrap();
                                        send_message(r#""END""#).unwrap();
                                    } else {
                                        send_message(r#""File not found""#).unwrap();
                                        send_message(r#""END""#).unwrap();
                                    }
                                }
                                None => {
                                    send_message(r#""Device not found or not mounted""#).unwrap();
                                    send_message(r#""END""#).unwrap();
                                }
                            }
                        } else {
                            send_message(r#""Invalid rm command format""#).unwrap();
                            send_message(r#""END""#).unwrap();
                        }
                    }
                    command if command.starts_with(r#""add"#) => {
                        let re = regex::Regex
                            ::new(
                                r#""add\s+--name\s+\\"([^"]+)\\"\s+--format\s+\\"([^"]+)\\"\s+(?:--path\s+\\"([^"]+)\\"\s+)?--data\s+\\"\\"\\"((?:\\"|.)*?)\\"\\"\\""#
                            )
                            .unwrap();
                        if let Some(caps) = re.captures(&message) {
                            let name = &caps[1];
                            let format = &caps[2];
                            // Path is optional, use empty string if not provided
                            let path = caps.get(3).map_or("", |m| m.as_str());
                            let data = &caps[4].trim_end_matches("\\/");

                            match find_mount_point(target_vid) {
                                Some(mount_point) => {
                                    // Create the directory path
                                    let dir_path = if path.is_empty() {
                                        std::path::PathBuf::from(&mount_point)
                                    } else {
                                        Path::new(&mount_point).join(path)
                                    };

                                    // Create directories if they don't exist
                                    if !dir_path.exists() {
                                        match fs::create_dir_all(&dir_path) {
                                            Ok(_) => {
                                                send_message(
                                                    r#""Created directory path""#
                                                ).unwrap();
                                            }
                                            Err(e) => {
                                                let error_msg =
                                                    format!(r#""Failed to create directory path: {}""#, e);
                                                send_message(&error_msg).unwrap();
                                                send_message(r#""END""#).unwrap();
                                                return;
                                            }
                                        }
                                    }

                                    // Create the full file path
                                    let file_path = dir_path.join(format!("{}.{}", name, format));

                                    // Decode and write the data
                                    let decoded_bytes: Vec<u8> = data
                                        .split(',')
                                        .filter_map(|s| s.parse::<u8>().ok())
                                        .collect();

                                    match fs::write(&file_path, decoded_bytes) {
                                        Ok(_) => {
                                            let success_msg = format!(
                                                r#""File created successfully at {}""#,
                                                file_path.display()
                                            );
                                            send_message(&success_msg).unwrap();
                                        }
                                        Err(e) => {
                                            let error_msg =
                                                format!(r#""Failed to write file: {}""#, e);
                                            send_message(&error_msg).unwrap();
                                        }
                                    }
                                    send_message(r#""END""#).unwrap();
                                }
                                None => {
                                    send_message(r#""Device not found or not mounted""#).unwrap();
                                    send_message(r#""END""#).unwrap();
                                }
                            }
                        } else {
                            send_message(
                                r#""Invalid add command format. Use: add --name \"filename\" --format \"ext\" [--path \"dir/path\"] --data \"\"\"data\"\"\"""#
                            ).unwrap();
                            send_message(r#""END""#).unwrap();
                        }
                    }
                    r#""exit""# => process::exit(0),

                    _ => {
                        send_message(&(message.clone() + "test")).unwrap();
                        send_message(r#""END""#).unwrap();
                    }
                }
            }
            None => {
                break;
            } // exit signal received or error encountered
        }
    }

    // Wait for the reader thread before exiting (optional).
    let _ = reader.join();

    process::exit(0);
}
#[cfg(target_os = "macos")]
fn get_usb_devices() -> Result<Vec<UsbDevice>, Box<dyn Error>> {
    let mut devices = Vec::new();

    // Get USB devices using system_profiler
    let output = Command::new("system_profiler").arg("SPUSBDataType").output()?;

    let usb_output = String::from_utf8(output.stdout)?;
    // Get disk info using diskutil
    // let disk_output = Command::new("diskutil").arg("diskutil").output()?;

    // let disk_info = String::from_utf8(disk_output.stdout)?;

    // Parse system_profiler output to get VID/PID
    // This is a simplified version - macOS output requires more complex parsing
    let mut current_vid = String::new();
    let mut current_pid = String::new();

    for line in usb_output.lines() {
        let line = line.trim();

        if line.contains("Vendor ID:") {
            current_vid = line
                .split(":")
                .nth(1)
                .unwrap_or("")
                .trim()
                .split(" ")
                .next()
                .unwrap_or("")
                .to_string();
        }

        if line.contains("Product ID:") {
            current_pid = line
                .split(":")
                .nth(1)
                .unwrap_or("")
                .trim()
                .split(" ")
                .next()
                .unwrap_or("")
                .to_string();
        }

        // When we have both VID and PID, try to find mount point
        if !current_vid.is_empty() && !current_pid.is_empty() {
            // This is a simplified approach - would need more complex parsing for real macOS implementation
            devices.push(UsbDevice {
                vendor_id: current_vid.clone(),
                product_id: current_pid.clone(),
                mount_point: find_macos_mount_point("0xb1b0", "0x8055", &usb_output),
            });

            current_vid.clear();
            current_pid.clear();
        }
    }

    Ok(devices)
}
#[cfg(target_os = "macos")]
fn find_macos_mount_point(vid: &str, pid: &str, disk_info: &str) -> Option<String> {
    let mut current_mount_point = None;
    let mut found_vid = false;
    let mut found_pid = false;

    for line in disk_info.lines() {
        let line = line.trim();
        if line.contains("Vendor ID:") && line.contains(vid) {
            found_vid = true;
        }

        if line.contains("Product ID:") && line.contains(pid) {
            found_pid = true;
        }
        if found_vid && found_pid && line.starts_with("Mount Point:") {
            current_mount_point = Some(line.split(":").nth(1)?.trim().to_string());
            break;
        }

        if line.is_empty() {
            found_vid = false;
            found_pid = false;
        }
    }

    current_mount_point
}
#[cfg(target_os = "macos")]
fn find_mount_point(target_vid: &str) -> Option<String> {
    // Get USB devices using system_profiler
    let output = Command::new("system_profiler").arg("SPUSBDataType").output();
    let usb_output = String::from_utf8(output.unwrap().stdout).unwrap_or_else(|_| String::new());
    // Get all connected USB devices with their VID and PID
    let usb_devices = get_usb_devices().unwrap_or_default();

    // Find the mount points
    for device in usb_devices {
        if device.vendor_id.eq_ignore_ascii_case(target_vid) {
            return device.mount_point;
        }
    }

    None
}
// #[cfg(target_os = "windows")]
// fn find_mount_point(target_vid: &str) -> Option<String> {
//     None
// }

#[cfg(target_os = "linux")]
fn get_usb_devices() -> Result<Vec<UsbDevice>, Box<dyn Error>> {
    let mut devices = Vec::new();

    // Get all USB devices
    let output = Command::new("lsusb").output()?;

    let lsusb_output = String::from_utf8(output.stdout)?;

    // Get all mount points
    let mount_points = get_mount_points()?;

    // Parse lsusb output and extract VID/PID
    for line in lsusb_output.lines() {
        // Expected format: "Bus 001 Device 002: ID 1234:5678 Device Description"
        if let Some(id_part) = line.split("ID ").nth(1) {
            if let Some(vid_pid) = id_part.split_whitespace().next() {
                if
                    let Some((vid, pid)) = vid_pid
                        .split(':')
                        .collect::<Vec<&str>>()
                        .get(0..2)
                        .map(|s| (s[0], s[1]))
                {
                    let device_path = find_device_path(vid, pid)?;
                    let mount_point = device_path.and_then(|path| mount_points.get(&path).cloned());

                    devices.push(UsbDevice {
                        vendor_id: vid.to_string(),
                        product_id: pid.to_string(),
                        mount_point,
                    });
                }
            }
        }
    }

    Ok(devices)
}

#[cfg(target_os = "linux")]
fn find_device_path(vid: &str, pid: &str) -> Result<Option<String>, Box<dyn Error>> {
    // Look in /sys/bus/usb/devices/ for matching device
    let usb_path = Path::new("/sys/bus/usb/devices");
    if !usb_path.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(usb_path)? {
        let entry = entry?;
        let path = entry.path();

        // Check for VID and PID files
        let vid_path = path.join("idVendor");
        let pid_path = path.join("idProduct");

        if vid_path.exists() && pid_path.exists() {
            let found_vid = fs::read_to_string(vid_path)?.trim().to_string();
            let found_pid = fs::read_to_string(pid_path)?.trim().to_string();

            if found_vid.eq_ignore_ascii_case(vid) && found_pid.eq_ignore_ascii_case(pid) {
                // Find the block device associated with this USB device
                for block_dir in fs::read_dir("/sys/block")? {
                    let block_dir = block_dir?;
                    let block_path = block_dir.path();

                    // Check if this block device is our USB device
                    let device_link = block_path.join("device");
                    if device_link.exists() {
                        if let Ok(real_path) = fs::read_link(device_link) {
                            if
                                real_path
                                    .to_string_lossy()
                                    .contains(&path.file_name().unwrap().to_string_lossy())
                            {
                                return Ok(Some(block_dir.file_name().into_string().unwrap()));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

#[cfg(target_os = "linux")]
fn get_mount_points() -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut mounts = HashMap::new();

    // Read /proc/mounts
    let mount_data = fs::read_to_string("/proc/mounts")?;

    for line in mount_data.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let device = parts[0];
            let mount_point = parts[1];

            // Only include real block devices
            if device.starts_with("/dev/sd") || device.starts_with("/dev/nvme") {
                let device_name = device.trim_start_matches("/dev/");
                mounts.insert(device_name.to_string(), mount_point.to_string());
            }
        }
    }

    Ok(mounts)
}

#[cfg(target_os = "windows")]
fn find_mount_point(target_vid: &str) -> Option<String> {
    let ps_script =
        r#"
    $usbDisk = Get-Disk | Where-Object { $_.FriendlyName -eq 'TinyUSB Flash Storage' }
    if ($usbDisk) {
        $logicalDrive = Get-Partition -DiskNumber $usbDisk.Number | Get-Volume | Select-Object -ExpandProperty DriveLetter
        Write-Output $logicalDrive
    }
    "#;

    let output = Command::new("powershell").args(["-Command", ps_script]).output().ok()?; // Returns None if the command fails

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if result.is_empty() {
        None
    } else {
        Some(result + ":/") // Appends ":" to match standard drive notation
    }
}
