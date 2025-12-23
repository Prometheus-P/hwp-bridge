// crates/hwp-core/src/parser/mod.rs

pub mod bodytext;
pub mod chart;
mod chart_schema;
mod chart_types;
pub mod docinfo;
mod document;
mod header;
mod ole;
pub mod primitives;
mod record;
pub mod record_nom;
mod section;
pub mod summary;

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
pub use section::{
    DEFAULT_MAX_DECOMPRESSED_BYTES_PER_SECTION, DEFAULT_MAX_RECORDS_PER_SECTION,
    DEFAULT_MAX_SECTIONS, SectionLimits, decompress_section, decompress_section_with_limits,
    extract_text_from_para_text, parse_section_records, parse_section_records_with_options,
};
pub use summary::{HwpSummaryInfo, parse_summary_info};
