// crates/hwp-core/src/parser/mod.rs

mod document;
mod header;
mod ole;
mod record;
mod section;

pub use document::HwpTextExtractor;
pub use header::parse_file_header;
pub use ole::HwpOleFile;
pub use record::{Record, RecordHeader, RecordIterator, tags};
pub use section::{decompress_section, extract_text_from_para_text, parse_section_records};
