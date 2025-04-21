use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("GPGME Error: {0}")]
    GpgME(#[from] gpgme::Error),

    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Web Server Error: {0}")]
    WebHyper(#[from] hyper::Error),

    #[error("Address Parsing Error: {0}")]
    AddrParse(#[from] std::net::AddrParseError),

    #[error("Template Error: {0}")]
    Askama(#[from] askama::Error),

    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Base64 Decode Error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("QR Code Generation Error: {0}")]
    QrCodeGen(String), // qrcodegen uses integers typically

    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("Operation Error: {0}")]
    Operation(String),

    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Port unavailable: {0}")]
    PortUnavailable(u16),

    #[error("Unknown Error: {0}")]
    Unknown(#[from] anyhow::Error),
}

// Implement necessary conversions for Axum error handling
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::GpgME(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("GPG Error: {}", e),
            ),
            AppError::Io(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO Error: {}", e),
            ),
            AppError::InvalidInput(msg) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Invalid Input: {}", msg),
            ),
             AppError::Operation(msg) => (
                 axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                 format!("Operation Failed: {}", msg),
             ),
            // Add other error types
            _ => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("An unexpected error occurred: {}", self),
            ),
        };

        (status, axum::Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;