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
    // let matches = clap::Command
    //     ::new("plode")
    //     .version("0.1.0")
    //     .author("Goutham <goutham@yudurobotics.com>")
    //     .about("USB device file manager")
    //     .subcommand(
    //         clap::Command
    //             ::new("list")
    //             .about("Lists all files in the mount point of the specified USB device")
    //             .arg(
    //                 clap::Arg
    //                     ::new("pid")
    //                     .short('p')
    //                     .long("pid")
    //                     .value_name("PID")
    //                     .help("Product ID of the USB device")
    //                     .required(true)
    //             )
    //     )
    //     .subcommand(
    //         clap::Command
    //             ::new("remove")
    //             .about("Removes a file from the mount point of the specified USB device")
    //             .arg(
    //                 clap::Arg
    //                     ::new("pid")
    //                     .short('p')
    //                     .long("pid")
    //                     .value_name("PID")
    //                     .help("Product ID of the USB device")
    //                     .required(true)
    //             )
    //             .arg(
    //                 clap::Arg
    //                     ::new("file")
    //                     .short('f')
    //                     .long("file")
    //                     .value_name("FILE")
    //                     .help("File path to remove from the USB device")
    //                     .required(true)
    //             )
    //     )
    //     .subcommand(
    //         clap::Command
    //             ::new("add")
    //             .about("Adds a file to the USB device")
    //             .arg(clap::Arg::new("pid").short('p').long("pid").required(true).help("Product ID"))
    //             .arg(
    //                 clap::Arg::new("name").short('n').long("name").required(true).help("File name")
    //             )
    //             .arg(
    //                 clap::Arg
    //                     ::new("format")
    //                     .short('f')
    //                     .long("format")
    //                     .required(true)
    //                     .help("File format")
    //             )
    //             .arg(
    //                 clap::Arg
    //                     ::new("data")
    //                     .short('d')
    //                     .long("data")
    //                     .required(true)
    //                     .help("File content")
    //             )
    //     )
    //     .get_matches();
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
                                    }
                                    Err(e) => {
                                        let message = format!(
                                            r#""Error reading directory: {}""#,
                                            e.to_string()
                                        );
                                        send_message(&message).unwrap();
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
                            }
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
                                    } else {
                                        send_message(r#""File not found""#).unwrap();
                                    }
                                }
                                None => {
                                    send_message(r#""Device not found or not mounted""#).unwrap();
                                }
                            }
                        } else {
                            send_message(r#""Invalid rm command format""#).unwrap();
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
                                    // match
                                    //     save_binary_file(
                                    //         data,
                                    //         file_path.as_os_str().to_str().unwrap()
                                    //     )
                                    // {
                                    //     Ok(_) =>
                                    //         send_message(r#""File created successfully""#).unwrap(),
                                    //     Err(e) => {
                                    //         let message = format!(
                                    //             r#""Failed to create file: {}""#,
                                    //             e.to_string()
                                    //         );
                                    //         send_message(&message).unwrap();
                                    //     }
                                    // }
                                }
                                None => {
                                    send_message(r#""Device not found or not mounted""#).unwrap();
                                }
                            }
                        } else {
                            send_message(r#""Invalid add command format""#).unwrap();
                        }
                    }
                    r#""exit""# => process::exit(0),

                    _ => {
                        send_message(&(message.clone() + "test")).unwrap();
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

    // match matches.subcommand() {
    //     Some(("list", sub_m)) => {
    //         if let Some(pid) = sub_m.get_one::<String>("pid") {
    //             match find_mount_point(target_vid, pid) {
    //                 Some(mount_point) => {
    //                     println!(
    //                         "Device VID:{} PID:{} is mounted at: {}",
    //                         target_vid,
    //                         pid,
    //                         mount_point
    //                     );
    //                     let paths = fs::read_dir(mount_point)?;
    //                     for path in paths {
    //                         println!("File: {}", path?.path().display());
    //                     }
    //                 }
    //                 None =>
    //                     println!("Device VID:{} PID:{} not found or not mounted", target_vid, pid),
    //             }
    //         }
    //     }
    //     Some(("remove", sub_m)) => {
    //         if let Some(pid) = sub_m.get_one::<String>("pid") {
    //             if let Some(file) = sub_m.get_one::<String>("file") {
    //                 match find_mount_point(target_vid, pid) {
    //                     Some(mount_point) => {
    //                         let file_path = Path::new(&mount_point).join(file);
    //                         if file_path.exists() {
    //                             fs::remove_file(&file_path)?;
    //                             println!("File {} removed successfully", file_path.display());
    //                         } else {
    //                             println!("File {} not found", file_path.display());
    //                         }
    //                     }
    //                     None =>
    //                         println!(
    //                             "Device VID:{} PID:{} not found or not mounted",
    //                             target_vid,
    //                             pid
    //                         ),
    //                 }
    //             }
    //         }
    //     }
    //     Some(("add", sub_m)) => {
    //         if let Some(pid) = sub_m.get_one::<String>("pid") {
    //             if let Some(name) = sub_m.get_one::<String>("name") {
    //                 if let Some(format) = sub_m.get_one::<String>("format") {
    //                     if let Some(data) = sub_m.get_one::<String>("data") {
    //                         if let Some(mount_point) = find_mount_point(target_vid, pid) {
    //                             let file_path = Path::new(&mount_point).join(
    //                                 format!("{}.{}", name, format)
    //                             );
    //                             fs::write(&file_path, data)?;
    //                             println!("File {} created successfully", file_path.display());
    //                         } else {
    //                             println!("Device not found or not mounted");
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     _ => {}
    // }
    // Ok(())
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
#[cfg(target_os = "windows")]
fn find_mount_point(target_vid: &str) -> Option<String> {
    None
}

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
#[cfg(target_os = "windows")]
fn find_mount_point(target_vid: &str) -> Option<String> {
    let usb_devices = get_usb_devices().unwrap_or_default();

    for device in usb_devices {
        if device.vendor_id.eq_ignore_ascii_case(target_vid) {
            return device.mount_point;
        }
    }

    None
}
#[cfg(target_os = "windows")]
fn get_usb_devices() -> Result<Vec<UsbDevice>, Box<dyn Error>> {
    let mut devices = Vec::new();

    // Get USB devices using wmic
    let output = Command::new("wmic")
        .args(&["path", "Win32_USBControllerDevice", "get", "Dependent"])
        .output()?;

    let usb_output = String::from_utf8(output.stdout)?;

    // Parse the output for VID and PID
    let mut current_vid = String::new();
    let mut current_pid = String::new();

    for line in usb_output.lines() {
        let line = line.trim();
        if line.contains("VID_") && line.contains("PID_") {
            if let Some(start) = line.find("VID_") {
                current_vid = line[start + 4..start + 8].to_string();
            }
            if let Some(start) = line.find("PID_") {
                current_pid = line[start + 4..start + 8].to_string();
            }
        }

        if !current_vid.is_empty() && !current_pid.is_empty() {
            let mount_point = find_windows_mount_point(&current_vid, &current_pid);
            devices.push(UsbDevice {
                vendor_id: current_vid.clone(),
                product_id: current_pid.clone(),
                mount_point,
            });

            current_vid.clear();
            current_pid.clear();
        }
    }

    Ok(devices)
}

#[cfg(target_os = "windows")]
fn find_windows_mount_point(vid: &str, pid: &str) -> Option<String> {
    // Use PowerShell to get mounted USB drives
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(
            "Get-WmiObject Win32_DiskDrive | Where-Object {$_.PNPDeviceID -match 'VID_'} | ForEach-Object { $_.DeviceID }"
        )
        .output()
        .ok()?;

    let drive_output = String::from_utf8(output.stdout).ok()?;

    for line in drive_output.lines() {
        let line = line.trim();
        if line.contains(&format!("VID_{}", vid)) && line.contains(&format!("PID_{}", pid)) {
            let mount_output = Command::new("wmic")
                .args(&["logicaldisk", "get", "DeviceID"])
                .output()
                .ok()?;

            let mount_info = String::from_utf8(mount_output.stdout).ok()?;
            for mount_line in mount_info.lines() {
                let mount_line = mount_line.trim();
                if mount_line.contains(":\\") {
                    return Some(mount_line.to_string());
                }
            }
        }
    }

    None
}
#[cfg(target_os = "windows")]
fn get_usb_devices() -> Result<Vec<UsbDevice>, Box<dyn Error>> {
    let mut devices = Vec::new();

    // On Windows, we would use PowerShell commands or WMI queries
    // to get the USB device information

    // Example PowerShell command (would need to be implemented)
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(
            "Get-PnpDevice -Class 'USB' | Where-Object { $_.Status -eq 'OK' } | Select-Object FriendlyName, DeviceID"
        )
        .output()?;

    let device_output = String::from_utf8(output.stdout)?;

    // Get drive letters with PowerShell
    let mount_output = Command::new("powershell")
        .arg("-Command")
        .arg(
            "Get-WmiObject -Class Win32_DiskDrive | ForEach-Object { $drive = $_; Get-WmiObject -Class Win32_DiskPartition -Filter \"DiskIndex=$($drive.Index)\" | ForEach-Object { Get-WmiObject -Class Win32_LogicalDisk -Filter \"DeviceID='$($_.DeviceID)'\" | Select-Object DeviceID, VolumeName } }"
        )
        .output()?;

    let mount_info = String::from_utf8(mount_output.stdout)?;

    // This is a simplified version - Windows output requires more complex parsing
    // In a real implementation, you would need to parse the DeviceID to extract VID/PID
    // and correlate with the mount points

    // Placeholder for demonstration
    devices.push(UsbDevice {
        vendor_id: "example".to_string(),
        product_id: "example".to_string(),
        mount_point: Some("C:\\".to_string()),
    });

    Ok(devices)
}
