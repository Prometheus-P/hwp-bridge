// crates/hwp-wasm/src/lib.rs

//! WebAssembly bindings for HWP document parsing
//!
//! This crate provides JavaScript-callable functions for parsing HWP documents
//! directly in the browser using WebAssembly.
//!
//! # Usage (JavaScript)
//!
//! ```javascript
//! import init, { parse_hwp_content, extract_hwp_text, get_hwp_info } from 'hwp-wasm';
//!
//! await init();
//!
//! // From file input
//! const file = document.getElementById('file-input').files[0];
//! const arrayBuffer = await file.arrayBuffer();
//! const uint8Array = new Uint8Array(arrayBuffer);
//!
//! // Get document info (header only - fast)
//! const info = get_hwp_info(uint8Array);
//! console.log(info.version, info.properties);
//!
//! // Extract all text content
//! const text = extract_hwp_text(uint8Array);
//! console.log(text);
//! ```

use std::io::Cursor;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use hwp_core::{HwpOleFile, HwpTextExtractor};
use hwp_types::{FileHeader, HwpError};

// ═══════════════════════════════════════════════════════════════════════════
// Initialization
// ═══════════════════════════════════════════════════════════════════════════

/// Initialize panic hook for better error messages in console
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ═══════════════════════════════════════════════════════════════════════════
// JS-friendly types
// ═══════════════════════════════════════════════════════════════════════════

/// HWP document information (JS-friendly version of FileHeader)
#[derive(Serialize)]
pub struct HwpInfo {
    /// Version string (e.g., "5.1.0.0")
    pub version: String,
    /// Major version number
    pub version_major: u8,
    /// Minor version number
    pub version_minor: u8,
    /// Whether the document is compressed
    pub is_compressed: bool,
    /// Whether the document is encrypted (will fail to parse)
    pub is_encrypted: bool,
    /// Whether this is a distribution-only document (will fail to parse)
    pub is_distribution: bool,
    /// Whether the document contains scripts
    pub has_script: bool,
    /// Whether the document has DRM protection
    pub has_drm: bool,
}

impl From<&FileHeader> for HwpInfo {
    fn from(header: &FileHeader) -> Self {
        Self {
            version: header.version.to_string(),
            version_major: header.version.major,
            version_minor: header.version.minor,
            is_compressed: header.properties.is_compressed(),
            is_encrypted: header.properties.is_encrypted(),
            is_distribution: header.properties.is_distribution(),
            has_script: header.properties.has_script(),
            has_drm: header.properties.has_drm(),
        }
    }
}

/// Parse result containing extracted content
#[derive(Serialize)]
pub struct ParseResult {
    /// Document information
    pub info: HwpInfo,
    /// Extracted text content
    pub text: String,
    /// Number of sections found
    pub section_count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// Error conversion
// ═══════════════════════════════════════════════════════════════════════════

fn to_js_error(e: HwpError) -> JsError {
    JsError::new(&e.to_string())
}

// ═══════════════════════════════════════════════════════════════════════════
// Public WASM API
// ═══════════════════════════════════════════════════════════════════════════

/// Get HWP document information without full parsing
///
/// This is a fast operation that only reads the FileHeader.
/// Use this to check document properties before attempting full extraction.
///
/// # Arguments
/// * `data` - The raw HWP file bytes (Uint8Array in JavaScript)
///
/// # Returns
/// A JavaScript object containing document information
///
/// # Errors
/// Returns an error if:
/// - The file is not a valid HWP document
/// - The file is encrypted
/// - The file is a distribution-only document
/// - The HWP version is not supported (< 5.0)
#[wasm_bindgen]
pub fn get_hwp_info(data: &[u8]) -> Result<JsValue, JsError> {
    let cursor = Cursor::new(data);
    let ole = HwpOleFile::open(cursor).map_err(to_js_error)?;

    let info = HwpInfo::from(ole.header());
    serde_wasm_bindgen::to_value(&info).map_err(|e| JsError::new(&e.to_string()))
}

/// Extract all text from an HWP document
///
/// # Arguments
/// * `data` - The raw HWP file bytes (Uint8Array in JavaScript)
///
/// # Returns
/// The extracted text content as a string
///
/// # Errors
/// Returns an error if:
/// - The file is not a valid HWP document
/// - The file is encrypted or distribution-only
/// - Text extraction fails
#[wasm_bindgen]
pub fn extract_hwp_text(data: &[u8]) -> Result<String, JsError> {
    let cursor = Cursor::new(data);
    let mut extractor = HwpTextExtractor::open(cursor).map_err(to_js_error)?;

    extractor.extract_all_text().map_err(to_js_error)
}

/// Parse HWP document and extract all content
///
/// This is the main parsing function that returns both document info and text.
///
/// # Arguments
/// * `data` - The raw HWP file bytes (Uint8Array in JavaScript)
///
/// # Returns
/// A JavaScript object containing:
/// - `info`: Document information (version, properties)
/// - `text`: Extracted text content
/// - `section_count`: Number of sections in the document
///
/// # Errors
/// Returns an error if parsing or extraction fails
#[wasm_bindgen]
pub fn parse_hwp_content(data: &[u8]) -> Result<JsValue, JsError> {
    let cursor = Cursor::new(data);
    let mut extractor = HwpTextExtractor::open(cursor).map_err(to_js_error)?;

    // Re-open to get header info (HwpTextExtractor doesn't expose it directly)
    let cursor2 = Cursor::new(data);
    let ole = HwpOleFile::open(cursor2).map_err(to_js_error)?;
    let section_count = ole.list_sections().len();

    let text = extractor.extract_all_text().map_err(to_js_error)?;
    let info = HwpInfo::from(ole.header());

    let result = ParseResult {
        info,
        text,
        section_count,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Check if the given bytes represent a valid HWP file
///
/// This only checks the file signature, not the full structure.
///
/// # Arguments
/// * `data` - The raw file bytes (at least 32 bytes needed)
///
/// # Returns
/// `true` if the file starts with a valid HWP signature
#[wasm_bindgen]
pub fn is_hwp_file(data: &[u8]) -> bool {
    if data.len() < 32 {
        return false;
    }

    &data[0..32] == hwp_types::HWP_SIGNATURE
}

/// Get the library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hwp_file_with_valid_signature() {
        let mut data = vec![0u8; 256];
        data[0..32].copy_from_slice(hwp_types::HWP_SIGNATURE);

        assert!(is_hwp_file(&data));
    }

    #[test]
    fn test_is_hwp_file_with_invalid_signature() {
        let data = vec![0u8; 256];
        assert!(!is_hwp_file(&data));
    }

    #[test]
    fn test_is_hwp_file_with_short_data() {
        let data = vec![0u8; 10];
        assert!(!is_hwp_file(&data));
    }

    #[test]
    fn test_version_returns_package_version() {
        let v = version();
        assert!(!v.is_empty());
        assert!(v.contains('.'));
    }
}
