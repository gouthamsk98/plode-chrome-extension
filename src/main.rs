use axum::routing::get;
use socketioxide::SocketIo;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use tower_http::cors::{ CorsLayer, Any };
use plode_web_agent::socketio::on_connect;
use plode_web_agent::compiler::health_check;
use include_dir::{ include_dir, Dir };
use std::fs;
// Embed entire directory at compile time
static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
    // use_embedded_assets()?;
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
