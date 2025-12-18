// crates/hwp-core/src/parser/bodytext/mod.rs

//! BodyText 스트림 파서
//!
//! HWP 문서의 BodyText 스트림 (Section0, Section1, ...)을 파싱합니다.
//! 표, 그림 등의 컨트롤이 BodyText에 포함됩니다.

pub mod table;

pub use table::{parse_table, parse_table_cell};
