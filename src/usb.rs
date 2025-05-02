use std::process::Command;
use tracing::info;
use std::error::Error;
use std::fs;
use std::path::Path;
use chrono::{ DateTime, Local };
use serde::{ Deserialize, Serialize };
use std::time::SystemTime;
use crate::models::*;
use base64::{ engine::general_purpose::STANDARD, Engine };

//Eject a USB device
#[cfg(target_os = "macos")]
pub fn eject_usb(path: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("diskutil").arg("eject").arg(path).output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Error ejecting USB device: {}", error).into())
    }
}
#[cfg(target_os = "windows")]
pub fn eject_usb(path: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("powershell")
        .args(["-Command", "Get-Volume", "-DriveLetter", path])
        .output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Error ejecting USB device: {}", error).into())
    }
}
#[cfg(target_os = "linux")]
pub fn eject_usb(path: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("eject").arg(path).output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Error ejecting USB device: {}", error).into())
    }
}
//mount a USB device with the specified VID
#[cfg(target_os = "macos")]
pub fn mount_usb(vid: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("diskutil").arg("mount").arg(vid).output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Error mounting USB device: {}", error).into())
    }
}

//Unmount a USB device
#[cfg(target_os = "macos")]
pub fn unmount_usb(path: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("diskutil").arg("unmount").arg(path).output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Error unmounting USB device: {}", error).into())
    }
}
#[cfg(target_os = "windows")]
pub fn unmount_usb(path: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("powershell")
        .args(["-Command", "Get-Volume", "-DriveLetter", path])
        .output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Error unmounting USB device: {}", error).into())
    }
}
#[cfg(target_os = "linux")]
pub fn unmount_usb(path: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("umount").arg(path).output()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Error unmounting USB device: {}", error).into())
    }
}

// Read file content as base64 string
pub fn read_file_as_base64(file_path: &str) -> Result<String, Box<dyn Error>> {
    let file_data = fs::read(file_path)?;
    Ok(STANDARD.encode(&file_data))
}
// Write data to a file on the USB device
pub fn write_file(
    path: &str,
    name: &str,
    format: &str,
    data: &str
) -> Result<String, Box<dyn Error>> {
    // Create the directory path
    let dir_path = Path::new(path);

    // Create directories if they don't exist
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path)?;
    }

    // Create the full file path
    let file_path = dir_path.join(format!("{}.{}", name, format));

    // Decode the base64 data
    let decoded_data = STANDARD.decode(data)?;

    // Write the decoded data as binary
    fs::write(&file_path, decoded_data)?;

    Ok(file_path.display().to_string())
}

// List directory contents recursively
pub fn list_directory_recursive(path: &str) -> Result<Vec<FileResponse>, Box<dyn Error>> {
    let mut file_list = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path_buf = entry.path();
                let file_path = path_buf.display().to_string();

                let metadata = match fs::metadata(&path_buf) {
                    Ok(m) => m,
                    Err(_) => {
                        continue;
                    } // Skip files we can't read metadata for
                };

                let is_dir = metadata.is_dir();
                let is_file = metadata.is_file();

                // Get file size
                let size = metadata.len();

                // Get file name and extension
                let filename = path_buf
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();

                let filetype = path_buf
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();

                // Format timestamps
                let last_modified = metadata.modified().unwrap_or_else(|_| SystemTime::now());
                let created = metadata.created().unwrap_or_else(|_| SystemTime::now());

                let last_modified_str = DateTime::<Local>
                    ::from(last_modified)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                let created_str = DateTime::<Local>
                    ::from(created)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();

                // Add current file/directory
                file_list.push(FileResponse {
                    filename,
                    filetype,
                    path: file_path.clone(),
                    size,
                    last_modified: last_modified_str,
                    created: created_str,
                    is_dir,
                    is_file,
                });

                // Recursively get contents of subdirectory
                if is_dir {
                    match list_directory_recursive(&file_path) {
                        Ok(sub_files) => file_list.extend(sub_files),
                        Err(_) => {
                            continue;
                        } // Skip directories we can't access
                    }
                }
            }
        }
    } else {
        return Err(format!("Error reading directory: {}", path).into());
    }

    Ok(file_list)
}

// Find mount point for the specified VID
pub fn find_mount_point(vid: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        find_macos_mount_point(vid)
    }

    #[cfg(target_os = "windows")]
    {
        find_windows_mount_point(vid)
    }

    #[cfg(target_os = "linux")]
    {
        find_linux_mount_point(vid)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}
// Platform-specific implementations
#[cfg(target_os = "macos")]
fn find_macos_mount_point(vid: &str) -> Option<String> {
    info!("Looking for USB device with VID: {}", vid);

    // Get USB devices using system_profiler
    let output = Command::new("system_profiler").arg("SPUSBDataType").output();
    if let Ok(output) = output {
        let usb_output = String::from_utf8_lossy(&output.stdout).to_string();

        // Parse system_profiler output
        let mut current_vid = String::new();
        let mut current_pid = String::new();
        let mut current_mount_point = None;

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

            if line.starts_with("Mount Point:") {
                current_mount_point = Some(line.split(":").nth(1).unwrap_or("").trim().to_string());
            }

            // If we found VID, PID and mount point, check if it matches our target
            if
                !current_vid.is_empty() &&
                !current_pid.is_empty() &&
                current_mount_point.is_some() &&
                current_vid.eq_ignore_ascii_case(vid)
            {
                info!("Found USB device with mount point: {:?}", current_mount_point);
                return current_mount_point;
            }

            // Reset values at end of device entry
            if line.is_empty() {
                current_vid = String::new();
                current_pid = String::new();
                current_mount_point = None;
            }
        }
    }

    info!("No matching USB device found");
    None
}

#[cfg(target_os = "windows")]
fn find_windows_mount_point(vid: &str) -> Option<String> {
    info!("Looking for USB device with VID: {}", vid);

    let ps_script = format!(
        r#"
    $usbDevices = Get-PnpDevice -Class USB | Where-Object {{ $_.DeviceID -match "VID_{}" }}
    if ($usbDevices) {{
        foreach ($device in $usbDevices) {{
            $devicePath = $device.DeviceID
            $drive = Get-WmiObject Win32_DiskDrive | Where-Object {{ $_.PNPDeviceID -eq $devicePath }} | 
                     Get-WmiObject -Query "ASSOCIATORS OF {{Win32_DiskDrive.DeviceID='$($_.DeviceID)'}} WHERE AssocClass = Win32_DiskDriveToDiskPartition" |
                     Get-WmiObject -Query "ASSOCIATORS OF {{Win32_DiskPartition.DeviceID='$($_.DeviceID)'}} WHERE AssocClass = Win32_LogicalDiskToPartition" |
                     Select-Object DeviceID
            
            if ($drive) {{
                Write-Output $drive.DeviceID
            }}
        }}
    }}
    "#,
        vid.trim_start_matches("0x")
    );

    let output = Command::new("powershell").args(["-Command", &ps_script]).output().ok()?;

    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if result.is_empty() {
        info!("No matching USB device found");
        None
    } else {
        info!("Found USB device with mount point: {}", result);
        Some(result)
    }
}

#[cfg(target_os = "linux")]
fn find_linux_mount_point(vid: &str) -> Option<String> {
    info!("Looking for USB device with VID: {}", vid);

    // Get all USB devices
    if let Ok(output) = Command::new("lsusb").output() {
        let lsusb_output = String::from_utf8_lossy(&output.stdout).to_string();

        // Get all mount points
        if let Ok(mount_points) = get_linux_mount_points() {
            // Parse lsusb output and extract VID/PID
            for line in lsusb_output.lines() {
                if let Some(id_part) = line.split("ID ").nth(1) {
                    if let Some(vid_pid) = id_part.split_whitespace().next() {
                        if
                            let Some((device_vid, pid)) = vid_pid
                                .split(':')
                                .collect::<Vec<&str>>()
                                .get(0..2)
                                .map(|s| (s[0], s[1]))
                        {
                            if device_vid.eq_ignore_ascii_case(vid.trim_start_matches("0x")) {
                                // Try to find the device path
                                if
                                    let Ok(Some(device_path)) = find_linux_device_path(
                                        device_vid,
                                        pid
                                    )
                                {
                                    if let Some(mount_point) = mount_points.get(&device_path) {
                                        info!("Found USB device with mount point: {}", mount_point);
                                        return Some(mount_point.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info!("No matching USB device found");
    None
}

#[cfg(target_os = "linux")]
fn find_linux_device_path(vid: &str, pid: &str) -> Result<Option<String>, Box<dyn Error>> {
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
fn get_linux_mount_points() -> Result<HashMap<String, String>, Box<dyn Error>> {
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
