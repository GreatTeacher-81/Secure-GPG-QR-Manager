use crate::error::{AppError, Result};
use crate::gpg_ops;
use crate::qr_utils;
use crate::web_server::AppState; // Import AppState
use askama::Template;
use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use zeroize::Zeroizing;


// --- Templates ---

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    current_keys: Vec<String>,
    secret_keys: Vec<String>,
    last_result: Option<String>,
    last_qr_code: Option<String>, // SVG string
    error_message: Option<String>,
}

// --- Request/Response Structs (examples) ---

#[derive(Deserialize)]
pub struct ExportKeyRequest {
    key_id: String,
    secret: Option<bool>, // Checkbox might send "on" or nothing
}

#[derive(Deserialize)]
pub struct ImportKeyRequest {
    key_data: String,
}

#[derive(Deserialize)]
pub struct EncryptRequest {
    recipients: String, // Comma-separated? Needs parsing
    plaintext: String,
}

#[derive(Deserialize)]
pub struct DecryptRequest {
    ciphertext: String,
    // Passphrase might be needed - how to handle securely? Avoid sending in form.
    // Maybe prompt via JS or rely on agent.
}
#[derive(Deserialize)]
pub struct SignRequest {
     signer_key_id: String,
     plaintext: String,
     sign_mode: String, // "clearsign" or "detach"
}

#[derive(Deserialize)]
pub struct VerifyRequest {
     signed_data: String,
     // Add fields for detached signature if needed
}


#[derive(Deserialize)]
pub struct ProcessQrDataRequest {
    scanned_data: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    qr_code: Option<String>, // Optionally include QR for results
}

// --- Handlers ---

pub async fn root(State(state): State<Arc<AppState>>) -> Result<Html<String>> {
    // Fetch initial data (e.g., keys)
    let public_keys = gpg_ops::list_keys(false).unwrap_or_else(|e| {
        println!("Error listing public keys: {}", e);
        vec![format!("Error listing keys: {}", e)]
    });
     let secret_keys = gpg_ops::list_keys(true).unwrap_or_else(|e| {
         println!("Error listing secret keys: {}", e);
         vec![format!("Error listing secret keys: {}", e)]
     });

    let template = IndexTemplate {
        current_keys: public_keys,
        secret_keys: secret_keys,
        last_result: None,
        last_qr_code: None,
        error_message: None,
    };
    let html = template.render()?;
    Ok(Html(html))
}

pub async fn api_status(State(_state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>> {
     // Return current status, maybe list keys
     let public_keys = gpg_ops::list_keys(false).unwrap_or_default();
     let secret_keys = gpg_ops::list_keys(true).unwrap_or_default();
     Ok(Json(serde_json::json!({
         "success": true,
         "public_keys": public_keys,
         "secret_keys": secret_keys,
         // "current_account": crate::CURRENT_ACCOUNT_NAME // If tracking needed
     })))
}


// Helper to create JSON responses
fn json_response<T: Serialize>(
    data: Option<T>,
    error: Option<String>,
    qr_code: Option<String>,
) -> Response {
    let success = error.is_none();
    (
        if success { StatusCode::OK } else { StatusCode::BAD_REQUEST },
        Json(ApiResponse { success, data, error, qr_code }),
    )
        .into_response()
}


pub async fn api_export_key(
    State(_state): State<Arc<AppState>>,
    Form(payload): Form<ExportKeyRequest>,
) -> Response {
    let result = gpg_ops::export_key(&payload.key_id, payload.secret.unwrap_or(false));
    match result {
        Ok(key_data) => {
            // Generate QR code for the exported key
            let qr_result = qr_utils::generate_qr_svg(&key_data);
            match qr_result {
                 Ok(svg) => json_response(Some(key_data), None, Some(svg)),
                 Err(e) => {
                      println!("QR Generation failed: {}", e);
                      // Still return the key data, but indicate QR failure
                      json_response(Some(key_data), Some("Key exported, but QR generation failed.".to_string()), None)
                 }
            }
        }
        Err(e) => json_response::<String>(None, Some(e.toString()), None),
    }
}

pub async fn api_import_key(
    State(_state): State<Arc<AppState>>,
    Form(payload): Form<ImportKeyRequest>,
) -> Response {
     match gpg_ops::import_key(&payload.key_data) {
          Ok(summary) => json_response(Some(summary), None, None),
          Err(e) => json_response::<String>(None, Some(e.toString()), None),
     }
}

pub async fn api_encrypt(
    State(_state): State<Arc<AppState>>,
    Form(payload): Form<EncryptRequest>,
) -> Response {
     // Basic parsing for comma-separated recipients
     let recipients_vec: Vec<&str> = payload.recipients.split(',')
          .map(str::trim)
          .filter(|s| !s.is_empty())
          .collect();

     if recipients_vec.is_empty() {
          return json_response::<String>(None, Some("No valid recipients provided.".to_string()), None);
     }


     match gpg_ops::encrypt(&payload.plaintext, &recipients_vec) {
         Ok(ciphertext) => {
             let qr_result = qr_utils::generate_qr_svg(&ciphertext);
              match qr_result {
                 Ok(svg) => json_response(Some(ciphertext), None, Some(svg)),
                 Err(e) => {
                      println!("QR Generation failed: {}", e);
                      json_response(Some(ciphertext), Some("Encryption successful, but QR generation failed.".to_string()), None)
                 }
              }
         }
         Err(e) => json_response::<String>(None, Some(e.toString()), None),
     }
}

pub async fn api_decrypt(
    State(_state): State<Arc<AppState>>,
    Form(payload): Form<DecryptRequest>,
) -> Response {
     // Decryption often needs a passphrase. This needs secure handling.
     // For now, assume agent or unprotected key.
     println!("Warning: Decryption initiated. Assumes GPG agent handles passphrase or key is unprotected.");
     match gpg_ops::decrypt(&payload.ciphertext) {
         Ok(plaintext) => {
             // Don't generate QR for plaintext by default unless explicitly requested
             json_response(Some(plaintext), None, None)
         }
          Err(e) => json_response::<String>(None, Some(e.toString()), None),
     }
}

pub async fn api_sign(
    State(_state): State<Arc<AppState>>,
    Form(payload): Form<SignRequest>,
) -> Response {
     // Passphrase handling needed for signing protected keys.
     // Relying on agent or unprotected key for this example.
     println!("Warning: Signing initiated. Assumes GPG agent handles passphrase or key is unprotected.");

     let mode = match payload.sign_mode.to_lowercase().as_str() {
         "clearsign" => gpgme::SignMode::Clear,
         "detach" => gpgme::SignMode::Detach,
         _ => gpgme::SignMode::Normal, // Default or Clear? Clear is safer for text.
     };

     // Pass None for passphrase for now
     match gpg_ops::sign(&payload.plaintext, &payload.signer_key_id, mode, None) {
         Ok(signed_data) => {
              let qr_result = qr_utils::generate_qr_svg(&signed_data);
              match qr_result {
                 Ok(svg) => json_response(Some(signed_data), None, Some(svg)),
                 Err(e) => {
                      println!("QR Generation failed: {}", e);
                      json_response(Some(signed_data), Some("Signing successful, but QR generation failed.".to_string()), None)
                 }
              }
         }
         Err(e) => json_response::<String>(None, Some(e.toString()), None),
     }
}

pub async fn api_verify(
    State(_state): State<Arc<AppState>>,
    Form(payload): Form<VerifyRequest>,
) -> Response {
      match gpg_ops::verify(&payload.signed_data) {
         // Verification function now returns a summary string on success
         Ok(summary) => json_response(Some(summary), None, None),
         Err(e) => json_response::<String>(None, Some(e.toString()), None),
     }
}


// Handler to process data received from client-side QR scan
pub async fn api_process_qr_data(
    State(_state): State<Arc<AppState>>,
    Form(payload): Form<ProcessQrDataRequest>,
) -> Response {
     println!("Received data from QR Scan: {} bytes", payload.scanned_data.len());
     // Here, decide what to do with the scanned data.
     // For simplicity, just return it to the client, maybe identifying its type.
     let data_type = if payload.scanned_data.contains("-----BEGIN PGP PUBLIC KEY BLOCK-----") {
         "PGP Public Key"
     } else if payload.scanned_data.contains("-----BEGIN PGP MESSAGE-----") {
         "PGP Encrypted Message"
     } else if payload.scanned_data.contains("-----BEGIN PGP SIGNED MESSAGE-----") {
         "PGP Signed Message"
     } else if payload.scanned_data.contains("-----BEGIN PGP SIGNATURE-----") {
         "PGP Detached Signature"
     } else {
         "Unknown / Plain Text"
     };

     json_response(Some(serde_json::json!({
         "received_data": payload.scanned_data,
         "data_type": data_type,
         "message": "Data received. Choose next action (e.g., Import Key, Decrypt Message)."
     })), None, None)
}