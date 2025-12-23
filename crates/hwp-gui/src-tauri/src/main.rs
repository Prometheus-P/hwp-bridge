#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fs::File;
use std::io::BufReader;

use hwp_core::{export::parse_structured_document_lenient, parser::SectionLimits};
use hwp_types::StructuredDocument;

#[tauri::command]
fn parse_hwp_file(path: String) -> Result<StructuredDocument, String> {
    let file = File::open(&path).map_err(|e| format!("파일을 열 수 없습니다: {}", e))?;
    let reader = BufReader::new(file);

    let limits = SectionLimits::default();

    let title = std::path::Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    let doc = parse_structured_document_lenient(reader, title, limits)
        .map_err(|e| format!("파싱 실패: {}", e))?;

    Ok(doc)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![parse_hwp_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
