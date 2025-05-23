use axum::routing::get;
use socketioxide::SocketIo;
use tracing::info;
use tracing_subscriber::FmtSubscriber;
use tower_http::cors::{ CorsLayer, Any };
use plode_web_agent::socketio::on_connect;
use plode_web_agent::compiler::health_check;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
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
    #[cfg(debug_assertions)]
    let allowed_origin = "http://localhost:3000";
    #[cfg(not(debug_assertions))]
    let allowed_origin = "https://plode.org";
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(allowed_origin.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);
    let app = axum::Router
        ::new()
        .route(
            "/",
            get(|| async { "alive" })
        )
        .layer(socketio_layer)
        .layer(cors);
    #[cfg(debug_assertions)]
    info!("Starting server with CORS for localhost:3000");
    #[cfg(not(debug_assertions))]
    info!("Starting server with CORS for plode.org");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8536").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
