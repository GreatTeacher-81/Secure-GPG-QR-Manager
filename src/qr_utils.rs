use crate::error::{AppError, Result};
use base64::{engine::general_purpose::STANDARD as Base64Engine, Engine as _};
use qrcodegen::{QrCode, QrCodeEcc};

// Generates a QR code as an SVG string
pub fn generate_qr_svg(data: &str) -> Result<String> {
    let ecc = QrCodeEcc::Medium; // Error correction level
    let qr = QrCode::encode_text(data, ecc)
        .map_err(|e| AppError::QrCodeGen(format!("QR encoding failed: {}", e)))?;

    // Simple SVG generation (adjust parameters as needed)
    let border = 4;
    let module_size = 5; // px per module
    let size = qr.size();
    let dim = (size + border * 2) * module_size;
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 {0} {0}" stroke="none">"#,
        dim
    );
    svg.push_str(r#"<rect width="100%" height="100%" fill="#FFFFFF"/>"#); // White background
    svg.push_str(r#"<path d=""#); // Start path for black modules

    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                let module_x = (x + border) * module_size;
                let module_y = (y + border) * module_size;
                // Append SVG move/draw commands for a square module
                svg.push_str(&format!("M{0},{1}h{2}v{2}h-{2}z", module_x, module_y, module_size));
            }
        }
    }
    svg.push_str(r#"" fill="#000000"/>"#); // Fill path black
    svg.push_str(r#"</svg>"#);

    Ok(svg)
}

// Optional: Generate as Base64 PNG (requires an image library like image + png)
/*
pub fn generate_qr_base64_png(data: &str) -> Result<String> {
    // Implementation would involve:
    // 1. qrcodegen::QrCode::encode_text
    // 2. Create an image buffer (e.g., image::GrayImage)
    // 3. Iterate QR modules and set pixels in the buffer
    // 4. Encode the buffer as PNG into a Vec<u8>
    // 5. Base64 encode the Vec<u8>
    // Requires adding `image` and `png` crates
    unimplemented!("PNG generation requires image/png crates");
}
*/