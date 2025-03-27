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
                        send_message(r#""0.1.0""#).unwrap();
                    }
                    r#""help""# => {
                        send_message(r#""Available commands: list, rm, add""#).unwrap();
                    }
                    r#""list""# => {
                        // send_message(r#""Running ...""#).unwrap();
                        match find_mount_point(target_vid) {
                            Some(mount_point) => {
                                let message = format!(
                                    r#""Device VID:{} PID:{} is mounted at: {}""#,
                                    target_vid,
                                    pid,
                                    mount_point
                                );

                                send_message(&message).unwrap();

                                let paths = fs::read_dir(mount_point);
                                match paths {
                                    Ok(paths) => {
                                        for path in paths {
                                            let message = format!(
                                                r#""File: {}""#,
                                                path.unwrap().path().display().to_string()
                                            );
                                            send_message(&message).unwrap();
                                        }
                                        send_message(r#""END""#).unwrap();
                                    }
                                    Err(e) => {
                                        let message = format!(
                                            r#""Error reading directory: {}""#,
                                            e.to_string()
                                        );
                                        send_message(&message).unwrap();
                                        send_message(r#""END""#).unwrap();
                                    }
                                }
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
                        let re = regex::Regex::new(r#""rm\s+--name\s+\\"([^"]+)\\"""#).unwrap();
                        if let Some(caps) = re.captures(&message) {
                            let name = &caps[1];
                            match find_mount_point(target_vid) {
                                Some(mount_point) => {
                                    let file_path = Path::new(&mount_point).join(name);
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
                                r#""add\s+--name\s+\\"([^"]+)\\"\s+--format\s+\\"([^"]+)\\"\s+--data\s+\\"\\"\\"((?:\\"|.)*?)\\"\\"\\""#
                            )
                            .unwrap();
                        if let Some(caps) = re.captures(&message) {
                            let name = &caps[1];
                            let format = &caps[2];
                            let data = &caps[3].trim_end_matches("\\/");
                            match find_mount_point(target_vid) {
                                Some(mount_point) => {
                                    let file_path = Path::new(&mount_point).join(
                                        format!("{}.{}", name, format)
                                    );
                                    let decoded_bytes: Vec<u8> = data
                                        .split(',')
                                        .filter_map(|s| s.parse::<u8>().ok())
                                        .collect();
                                    fs::write(&file_path, decoded_bytes).unwrap();
                                    send_message(r#""File created successfully""#).unwrap();
                                    send_message(r#""END""#).unwrap();
                                }
                                None => {
                                    send_message(r#""Device not found or not mounted""#).unwrap();
                                    send_message(r#""END""#).unwrap();
                                }
                            }
                        } else {
                            send_message(r#""Invalid add command format""#).unwrap();
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
