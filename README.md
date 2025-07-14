# Plode Web Agent

A native application that provides communication between web extensions and hardware devices, particularly focusing on Arduino boards and USB mass storage devices.

## Socket.IO Commands

This document outlines all available Socket.IO commands that can be used to interact with the Plode application.

### Core Events

#### `version`

Requests the version of the Plode application.

**Request:**

```javascript
socket.emit("version", (response) => {
  console.log(response);
});
```

**Response:**

```javascript
"1.0.0"; // Current version string
```

#### `device-connected`

Automatically emitted when a device connection status changes. This is a server-to-client event that monitors device connectivity.

**Event Data:**

```javascript
{
  connected: true; // or false
}
```

#### `connect-device`

Connects to a specific device using its port address.

**Request:**

```javascript
socket.emit("connect-device", { port_address: "/dev/ttyUSB0" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "/dev/ttyUSB0",
  output_json: null,
  files: null,
  error: null,
  command: "connect",
  args: ["/dev/ttyUSB0"]
}
```

#### `logs`

Automatically emitted log events from the Arduino CLI operations. This is a server-to-client event that provides real-time logging.

**Event Data:**

```javascript
{
  message: "Log message content",
  timestamp: "2025-01-01T00:00:00Z"
}
// or parsed JSON log data from Arduino CLI
```

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
  output_json: { /* Parsed JSON object with board information */ },
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
  output_json: { /* Parsed JSON object with connected board information */ },
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
  output_json: { /* Parsed JSON object with core information */ },
  files: null,
  error: null,
  command: "core",
  args: ["list", "--format", "json"]
}
```

#### `install-core`

Installs an Arduino core with logging enabled.

**Request:**

```javascript
socket.emit("install-core", { core: "arduino:avr" }, (response) => {
  console.log(response);
});
```

**Common core packages:**

- `arduino:avr` - Arduino AVR Boards (Uno, Nano, Pro Mini, etc.)
- `arduino:megaavr` - Arduino megaAVR Boards (Uno WiFi Rev2, Nano Every)
- `arduino:sam` - Arduino ARM (32-bits) Boards (Due)
- `arduino:samd` - Arduino SAMD Boards (Zero, MKR series)
- `esp32:esp32` - ESP32 boards
- `esp8266:esp8266` - ESP8266 boards

**Response:**

```javascript
{
  success: true,
  output: "Output from Arduino CLI",
  output_json: null,
  files: null,
  error: null,
  command: "core",
  args: ["install", "arduino:avr", "--log", "--log-file", "log.txt"]
}
```

#### `create-sketch`

Creates a new Arduino sketch.

**Request:**

```javascript
socket.emit("create-sketch", { sketch_name: "MySketch" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "JSON output from Arduino CLI",
  output_json: { /* Parsed JSON object with sketch creation details */ },
  files: null,
  error: null,
  command: "sketch",
  args: ["new", "MySketch", "--format", "json"]
}
```

#### `read-sketch-file` (Sketch Files)

Reads a file from the sketches directory.

**Request:**

```javascript
socket.emit(
  "read-sketch-file",
  {
    sketch_name: "MySketch",
    file_name: "MySketch.ino",
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
  output: "// File content here",
  output_json: null,
  files: null,
  error: null,
  command: "read-sketch-file",
  args: ["/path/to/sketches/MySketch.ino"]
}
```

#### `write=sketch-file` (Sketch Files)

Writes content to a file in the sketches directory.

**Request:**

```javascript
socket.emit(
  "write-sketch-file",
  {
    sketch_name: "MySketch",
    file_name: "MySketch.ino",
    file_value: "void setup() {\n  // code\n}\n\nvoid loop() {\n  // code\n}",
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
  output: "Write operation is not implemented",
  output_json: null,
  files: null,
  error: null,
  command: "write-sketch-file",
  args: []
}
```

#### `delete-sketch-file` (Sketch Files)

Deletes a file from the sketches directory.

**Request:**

```javascript
socket.emit(
  "delete-sketch-file",
  {
    sketch_name: "MySketch",
    file_name: "MySketch.ino",
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
  output: "/path/to/sketches/MySketch/MySketch.ino",
  output_json: null,
  files: null,
  error: null,
  command: "delete-sketch-file",
  args: ["/path/to/sketches/MySketch/MySketch.ino"]
}
```

#### `list-sketches`

Lists all available sketches in the sketches directory.

**Request:**

```javascript
socket.emit("list-sketches", (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "Sketches listed successfully",
  output_json: ["MySketch", "AnotherSketch", "TestProject"],
  files: null,
  error: null,
  command: "list-sketches",
  args: []
}
```

#### `remove-sketch`

Removes an entire sketch directory and all its contents.

**Request:**

```javascript
socket.emit("remove-sketch", { sketch_name: "MySketch" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "/path/to/sketches/MySketch",
  output_json: null,
  files: null,
  error: null,
  command: "remove-sketch",
  args: ["/path/to/sketches/MySketch"]
}
```

#### `list-sketch-files`

Lists all files inside a specific sketch directory.

**Request:**

```javascript
socket.emit("list-sketch-files", { sketch_name: "MySketch" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "Sketch files listed successfully",
  output_json: null,
  files: [
    {
      filename: "MySketch.ino",
      filetype: "ino",
      path: "/path/to/sketches/MySketch/MySketch.ino",
      size: 1024,
      last_modified: "2025-07-14T10:30:00Z",
      created: "2025-07-14T10:00:00Z",
      is_dir: false,
      is_file: true
    }
  ],
  error: null,
  command: "list-sketch-files",
  args: ["/path/to/sketches/MySketch"]
}
```

#### `compile-sketch`

Compiles an Arduino sketch with logging enabled.

**Request:**

```javascript
socket.emit(
  "compile-sketch",
  {
    sketch_name: "MySketch",
    fqbn: "arduino:avr:uno", // Optional
  },
  (response) => {
    console.log(response);
  }
);
```

**Common FQBN (Fully Qualified Board Name) examples:**

- `arduino:avr:uno` - Arduino Uno
- `arduino:avr:nano` - Arduino Nano
- `arduino:avr:mega` - Arduino Mega 2560
- `arduino:avr:leonardo` - Arduino Leonardo
- `arduino:samd:zero` - Arduino Zero
- `esp32:esp32:esp32` - ESP32 Dev Module
- `esp8266:esp8266:nodemcuv2` - NodeMCU 1.0 (ESP-12E Module)

**Response:**

```javascript
{
  success: true,
  output: "Compilation output",
  output_json: null,
  files: null,
  error: null,
  command: "compile",
  args: ["--fqbn", "arduino:avr:uno", "MySketch", "--log", "--log-file", "log.txt"]
}
```

#### `upload-sketch`

Uploads a compiled sketch to an Arduino board with logging enabled.

**Request:**

```javascript
socket.emit(
  "upload-sketch",
  {
    sketch_path: "/path/to/sketch.ino",
    port: "/dev/ttyUSB0", // Use appropriate port for your system (e.g., "COM3" on Windows)
    fqbn: "arduino:avr:uno",
  },
  (response) => {
    console.log(response);
  }
);
```

**Common port examples:**

- **Linux/macOS:** `/dev/ttyUSB0`, `/dev/ttyACM0`, `/dev/cu.usbmodem14101`
- **Windows:** `COM1`, `COM3`, `COM4`, etc.

**Response:**

```javascript
{
  success: true,
  output: "Upload output",
  output_json: null,
  files: null,
  error: null,
  command: "upload",
  args: ["--port", "/dev/ttyUSB0", "--fqbn", "arduino:avr:uno", "/path/to/sketch.ino", "--log", "--log-file", "log.txt"]
}
```

### Library Management

Commands for managing Arduino libraries:

#### `list-libraries`

Lists all installed Arduino libraries.

**Request:**

```javascript
socket.emit("list-libraries", (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "JSON output from Arduino CLI",
  output_json: { /* Parsed JSON object with library information */ },
  files: null,
  error: null,
  command: "lib",
  args: ["list", "--format", "json"]
}
```

#### `search-library`

Searches for Arduino libraries by name.

**Request:**

```javascript
socket.emit("search-library", { library_name: "WiFi" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "JSON output from Arduino CLI",
  output_json: { /* Parsed JSON object with search results */ },
  files: null,
  error: null,
  command: "lib",
  args: ["lib", "search", "WiFi", "--format", "json"]
}
```

#### `install-library`

Installs an Arduino library with logging enabled.

**Request:**

```javascript
socket.emit("install-library", { library_name: "WiFi" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "Installation output",
  output_json: null,
  files: null,
  error: null,
  command: "lib",
  args: ["lib", "install", "WiFi", "--log", "--log-file", "log.txt"]
}
```

#### `uninstall-library`

Uninstalls an Arduino library with logging enabled.

**Request:**

```javascript
socket.emit("uninstall-library", { library_name: "WiFi" }, (response) => {
  console.log(response);
});
```

**Response:**

```javascript
{
  success: true,
  output: "Uninstallation output",
  output_json: null,
  files: null,
  error: null,
  command: "lib",
  args: ["lib", "uninstall", "WiFi", "--log", "--log-file", "log.txt"]
}
```

### Real-time Logging

The application provides real-time logging through Socket.IO events. Most Arduino commands automatically generate log entries that are monitored and sent to connected clients.

#### Automatic Log Monitoring

The application monitors the `log.txt` file for changes and automatically emits `logs` events when new content is added. This provides real-time feedback during compilation, upload, and other operations.

**Log Event Structure:**

```javascript
// For JSON log entries from Arduino CLI
{
  level: "info",
  message: "Compilation completed successfully",
  timestamp: "2025-01-01T00:00:00Z"
}

// For plain text log entries
{
  message: "Plain text log message",
  timestamp: "2025-01-01T00:00:00Z"
}
```

### Device Connection Monitoring

The application continuously monitors device connections and automatically emits `device-connected` events when the connection status changes. This allows web applications to react to device plugging/unplugging in real-time.

**Usage:**

```javascript
socket.on("device-connected", (connected) => {
  console.log("Device connected:", connected);
  // Update UI based on connection status
});
```

## Error Handling

All commands return a standard response object with the following structure:

```javascript
{
  success: boolean,           // true if the command was successful, false otherwise
  output: string,            // command output as a string
  output_json: object | null, // parsed JSON object if output is valid JSON, null otherwise
  files: array | null,       // array of file objects for list-files command, null otherwise
  error: string | null,      // error message if success is false, null otherwise
  command: string,           // the command that was executed
  args: array                // array of arguments passed to the command
}
```

### Logging

Most Arduino commands include automatic logging to `log.txt`. The log file is monitored in real-time and updates are sent to connected clients via Socket.IO `logs` events. This provides detailed information about compilation, upload processes, and any errors that occur during command execution.

The logging system supports both JSON-formatted log entries from Arduino CLI and plain text messages. All log entries are automatically timestamped and sent to connected clients immediately when new content is detected.

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
