// crates/hwp-core/src/parser/mod.rs

mod header;
mod ole;
mod record;
mod section;

pub use header::parse_file_header;
pub use ole::HwpOleFile;
pub use record::{tags, Record, RecordHeader, RecordIterator};
pub use section::{decompress_section, extract_text_from_para_text, parse_section_records};
