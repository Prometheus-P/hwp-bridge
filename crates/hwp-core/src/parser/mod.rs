// crates/hwp-core/src/parser/mod.rs

pub mod bodytext;
pub mod docinfo;
mod document;
mod header;
mod ole;
pub mod primitives;
mod record;
pub mod record_nom;
mod section;

pub use bodytext::{parse_table, parse_table_cell};
pub use docinfo::{DocInfo, parse_docinfo};
pub use document::HwpTextExtractor;
pub use header::parse_file_header;
pub use ole::HwpOleFile;
pub use record::{Record, RecordHeader, RecordIterator, tags};
pub use record_nom::{
    FilteredRecordIterator, RecordHeaderNom, RecordIteratorNom, RecordNom, extract_records_by_tag,
    find_first_record, parse_record, parse_record_header, parse_records,
};
pub use section::{decompress_section, extract_text_from_para_text, parse_section_records};
