use axum::routing::{ get, post };
use socketioxide::SocketIo;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use tower_http::cors::{ CorsLayer, Any };
use plode_web_agent::socketio::on_connect;
use plode_web_agent::compiler::{ health_check, upload_library };
use include_dir::{ include_dir, Dir };
use plode_web_agent::models::{ LibraryUploadResponse, DownloadError };
use std::path::Path;
use std::fs;
// Embed entire directory at compile time
static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");
const ASSETS_DIR_PATH: &str = "assets_test";
fn extract_assets_to_temp() -> Result<String, Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("my_app_assets");

    // Create temp directory
    fs::create_dir_all(&temp_dir)?;

    // Extract all files from embedded directory
    ASSETS_DIR.extract(&temp_dir)?;

    Ok(temp_dir.to_string_lossy().to_string())
}
fn use_embedded_assets() -> Result<(), Box<dyn std::error::Error>> {
    // Method 1: Extract to temporary directory
    let assets_path = extract_assets_to_temp()?;
    info!("Assets extracted to: {}", assets_path);

    // Method 3: Access files from embedded directory
    if let Some(file) = ASSETS_DIR.get_file("README.md") {
        let content = file.contents_utf8().unwrap_or("Invalid UTF-8");
        info!("File content: {}", content);
    }

    // Method 4: List all embedded files
    for file in ASSETS_DIR.files() {
        info!("Embedded file: {}", file.path().display());
    }
    // list all directories
    for dir in ASSETS_DIR.dirs() {
        info!("Embedded directory: {}", dir.path().display());
    }

    Ok(())
}
// Asynchronous version
pub async fn download_asset_async(url: &str, local_path: &str) -> Result<(), DownloadError> {
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(local_path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    // print absolute path
    let absolute_path = std::fs
        ::canonicalize(local_path)
        .unwrap_or_else(|_| Path::new(local_path).to_path_buf());
    println!("Absolute path: {}", absolute_path.display());

    // Download the file
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(DownloadError::HttpError(response.status().as_u16()));
    }

    let bytes = response.bytes().await?;

    // Write to file
    tokio::fs::write(local_path, &bytes).await?;

    println!("Downloaded: {} -> {}", url, local_path);
    Ok(())
}
// Extract ZIP file to directory

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
    // download_and_extract_zip(
    //     "http://localhost:3000/macos/compiler.zip",
    //     ASSETS_DIR_PATH,
    //     false
    // ).await.expect("Failed to download asset");
    use_embedded_assets()?;
    // Health check for arduino-cli
    match health_check() {
        true => info!("arduino-cli initialized successfully"),
        false => {
            info!("arduino-cli test failed");
            std::process::exit(1);
        }
    }

    let (socketio_layer, io) = SocketIo::new_layer();
    io.ns("/", on_connect);
    // Configure CORS to allow all origins
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = axum::Router
        ::new()
        .route(
            "/",
            get(|| async { "alive" })
        )
        .route("/upload-library", post(upload_library))
        .layer(socketio_layer)
        .layer(cors);
    #[cfg(debug_assertions)]
    info!("Starting server with CORS for all origins");
    #[cfg(not(debug_assertions))]
    info!("Starting server with CORS for all origins");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8536").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
