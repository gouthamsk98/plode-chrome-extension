use serde::{ de, Deserialize, Serialize };
// Response structures
#[derive(Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub output: String,
    pub output_json: Option<serde_json::Value>,
    pub files: Option<Vec<FileResponse>>,
    pub error: Option<String>,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileResponse {
    pub filename: String,
    pub filetype: String,
    pub path: String,
    pub size: u64,
    pub last_modified: String,
    pub created: String,
    pub is_dir: bool,
    pub is_file: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UsbAddFileRequest {
    pub name: String,
    pub format: String,
    pub path: Option<String>,
    pub data: String,
}

// Request structures
#[derive(Deserialize)]
pub struct ArduinoCommand {
    pub command: String,
    pub args: Vec<String>,
}
