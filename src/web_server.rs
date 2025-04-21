use crate::cli::CliArgs; // Assuming CliArgs is defined elsewhere
use crate::error::{AppError, Result};
use crate::web_handlers; // Define handlers in a separate file
use axum::{
    routing::{get, post},
    Router, Server,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::Level;


// Shared state for the web server
#[derive(Clone)]
pub struct AppState {
    pub gpg_dir: Option<String>, // Pass gpg dir if overridden
                                  // Add other shared state if needed, e.g., Arc<Mutex<GpgContext>>
                                  // Be cautious with mutable shared state across requests.
}

pub async fn run_web_server(bind_ip: String, port: Option<u16>, gpg_dir: Option<String>) -> Result<()> {
    let bind_addr: IpAddr = bind_ip
        .parse()
        .map_err(|e| AppError::AddrParse(e))?;

    let actual_port = port.map_or_else(find_available_port, |p| Ok(p))?;

    let addr = SocketAddr::new(bind_addr, actual_port);

    // Initialize tracing (logging)
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO) // Adjust log level as needed
        .init();

    tracing::info!("Starting web server on http://{}", addr);
    if bind_addr == Ipv4Addr::LOCALHOST || bind_addr == IpAddr::V6(std::net::Ipv6Addr::LOCALHOST) {
        tracing::info!("Server is bound to localhost - accessible only from this machine.");
    } else {
        tracing::warn!(
            "Server is bound to {} - potentially accessible from other devices on the network!",
            bind_addr
        );
    }
    tracing::info!("GPG operations will use home directory: {}",
        gpg_dir.as_deref().unwrap_or("Default (~/.gnupg or system default)")
    );

    // Set GPG homedir if provided
    if let Some(ref dir) = gpg_dir {
         crate::gpg_ops::set_gpg_homedir(Some(dir.clone()))?;
    }


    let shared_state = Arc::new(AppState { gpg_dir });

    // Define routes
    let app = Router::new()
        .route("/", get(web_handlers::root))
        .route("/api/status", get(web_handlers::api_status))
        .route("/api/export_key", post(web_handlers::api_export_key))
        .route("/api/import_key", post(web_handlers::api_import_key))
        .route("/api/encrypt", post(web_handlers::api_encrypt))
        .route("/api/decrypt", post(web_handlers::api_decrypt))
        .route("/api/sign", post(web_handlers::api_sign))
        .route("/api/verify", post(web_handlers::api_verify))
        .route("/api/process_qr_data", post(web_handlers::api_process_qr_data))
        // Serve static files (CSS, JS)
        .nest_service("/static", ServeDir::new("static"))
        .with_state(shared_state)
        .layer(TraceLayer::new_for_http()); // Add request logging

    // Run the server
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| AppError::WebHyper(e))?; // Use hyper::Error conversion

    Ok(())
}

// Simple function to find an available high port
fn find_available_port() -> Result<u16> {
    // Start search from the ephemeral port range
    for port in 49152..=65535 {
        if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
            // Port is available, return it
            return Ok(port);
            // The listener is automatically closed when it goes out of scope.
        }
        // If bind fails, port is likely in use, try the next one.
    }
    Err(AppError::Operation("Could not find an available port.".to_string()))
}