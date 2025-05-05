# Plode Web Agent

A native application that provides communication between web extensions and hardware devices, particularly focusing on Arduino boards and USB mass storage devices.

## Socket.IO Commands

This document outlines all available Socket.IO commands that can be used to interact with the Plode application.

### USB Operations

Commands for interacting with USB mass storage devices:

#### `list-mount`

Lists the mount point for the target USB device (VID: 0xb1b0).

**Request:**

```javascript
socket.emit("list-mount", (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "/path/to/mounted/device",
  files: null,
  error: null,
  command: "list-mount",
  args: []
}
```

#### `list-files`

Lists all files in a directory on the mounted USB device.

**Request:**

```javascript
socket.emit("list-files", { file_path: "/optional/path" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "",
  files: [
    {
      name: "file1.txt",
      path: "/path/to/file1.txt",
      is_dir: false,
      size: 1024
    },
    {
      name: "directory1",
      path: "/path/to/directory1",
      is_dir: true,
      size: 0
    }
  ],
  error: null,
  command: "list-files",
  args: ["/optional/path"]
}
```

#### `add-file`

Writes a file to the USB device.

**Request:**

```javascript
socket.emit(
  "add-file",
  {
    name: "example",
    format: "txt",
    path: "/optional/path",
    data: "base64_encoded_data",
  },
  (response) => {
    console.log(response);
  }
);
```

**Response:**

```javascript
{
  success: true,
  output: "/path/to/file",
  files: null,
  error: null,
  command: "add-file",
  args: ["/path/to/file"]
}
```

#### `remove-file`

Removes a file from the USB device.

**Request:**

```javascript
socket.emit("remove-file", { file_path: "/path/to/file.txt" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "/path/to/file.txt",
  files: null,
  error: null,
  command: "remove-file",
  args: ["/path/to/file.txt"]
}
```

#### `remove-dir`

Removes a directory and all its contents from the USB device.

**Request:**

```javascript
socket.emit("remove-dir", { dir_path: "/path/to/directory" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "/path/to/directory",
  files: null,
  error: null,
  command: "remove-dir",
  args: ["/path/to/directory"]
}
```

#### `read-file`

Reads a file from the USB device and returns its contents as base64 encoded data.

**Request:**

```javascript
socket.emit("read-file", { file_path: "/path/to/file.txt" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "base64_encoded_file_contents",
  files: null,
  error: null,
  command: "read-file",
  args: ["/path/to/file.txt"]
}
```

### Arduino CLI Operations

Commands for interacting with Arduino boards using Arduino CLI:

#### `list-boards`

Lists all available Arduino boards.

**Request:**

```javascript
socket.emit("list-boards", (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "JSON output from Arduino CLI",
  files: null,
  error: null,
  command: "board",
  args: ["listall", "--format", "json"]
}
```

#### `list-connected`

Lists all connected Arduino boards.

**Request:**

```javascript
socket.emit("list-connected", (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "JSON output from Arduino CLI",
  files: null,
  error: null,
  command: "board",
  args: ["list", "--format", "json"]
}
```

#### `list-cores`

Lists all installed Arduino cores.

**Request:**

```javascript
socket.emit("list-cores", (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "JSON output from Arduino CLI",
  files: null,
  error: null,
  command: "core",
  args: ["list", "--format", "json"]
}
```

#### `install-core`

Installs an Arduino core.

**Request:**

```javascript
socket.emit("install-core", { core: "arduino:avr" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "Output from Arduino CLI",
  files: null,
  error: null,
  command: "core",
  args: ["install", "arduino:avr"]
}
```

#### `compile-sketch`

Compiles an Arduino sketch.

**Request:**

```javascript
socket.emit(
  "compile-sketch",
  {
    sketch_path: "/path/to/sketch.ino",
    fqbn: "arduino:avr:uno", // Optional
  },
  (response) => {
    console.log(response);
  }
);
```

**Response:**

```javascript
{
  success: true,
  output: "Compilation output",
  files: null,
  error: null,
  command: "compile",
  args: ["--fqbn", "arduino:avr:uno", "/path/to/sketch.ino"]
}
```

#### `upload-sketch`

Uploads a compiled sketch to an Arduino board.

**Request:**

```javascript
socket.emit(
  "upload-sketch",
  {
    sketch_path: "/path/to/sketch.ino",
    port: "/dev/ttyUSB0",
    fqbn: "arduino:avr:uno",
  },
  (response) => {
    console.log(response);
  }
);
```

**Response:**

```javascript
{
  success: true,
  output: "Upload output",
  files: null,
  error: null,
  command: "upload",
  args: ["--port", "/dev/ttyUSB0", "--fqbn", "arduino:avr:uno", "/path/to/sketch.ino"]
}
```

## Error Handling

All commands return a standard response object with the following structure:

```javascript
{
  success: boolean,      // true if the command was successful, false otherwise
  output: string,        // command output if any
  files: array | null,   // array of file objects for list-files command, null otherwise
  error: string | null,  // error message if success is false, null otherwise
  command: string,       // the command that was executed
  args: array            // array of arguments passed to the command
}
```

## Installation

### macOS

To install the application on macOS, run:

```bash
sudo installer -pkg plode_mass_storage.pkg -target /
```

### Windows

Run the `plode_mass_storage_Installer.exe` file and follow the installation instructions.

## Building from Source

Refer to the build scripts in the project for building from source code.
