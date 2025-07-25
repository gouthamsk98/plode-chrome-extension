use serde::{ Deserialize, Serialize };
#[cfg(target_os = "windows")]
const WCH_ASSETS_LINK: &str = "https://example.com/assets/";
#[cfg(target_os = "linux")]
const WCH_ASSETS_LINK: &str = "https://example.com/assets/";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const WCH_ASSETS_LINK: &str = "http://localhost:3000/macos/arm64/compiler.zip";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
const WCH_ASSETS_LINK: &str = "http://localhost:3000/macos/x86_64/compiler.zip";
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

#[derive(Serialize, Deserialize)]
pub struct LibraryUploadResponse {
    pub success: bool,
    pub message: String,
    pub library_name: String,
    pub file_path: Option<String>,
}
#[derive(Debug)]
pub enum DownloadError {
    NetworkError(reqwest::Error),
    IoError(std::io::Error),
    HttpError(u16),
    ZipError(zip::result::ZipError),
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::NetworkError(err)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError::IoError(err)
    }
}

impl From<zip::result::ZipError> for DownloadError {
    fn from(err: zip::result::ZipError) -> Self {
        DownloadError::ZipError(err)
    }
}
