// crates/hwp-core/src/parser/docinfo/document_properties.rs

//! Document properties (HWPTAG_DOCUMENT_PROPERTIES) parser.

use hwp_types::DocInfoProperties;
use nom::{IResult, number::complete::{le_u16, le_u32}};

/// Parse document properties record.
pub fn parse_document_properties(input: &[u8]) -> IResult<&[u8], DocInfoProperties> {
    let (input, section_count) = le_u16(input)?;
    let (input, page_start_number) = le_u16(input)?;
    let (input, footnote_start_number) = le_u16(input)?;
    let (input, endnote_start_number) = le_u16(input)?;
    let (input, figure_start_number) = le_u16(input)?;
    let (input, table_start_number) = le_u16(input)?;
    let (input, equation_start_number) = le_u16(input)?;
    let (input, list_id) = le_u32(input)?;
    let (input, paragraph_id) = le_u32(input)?;
    let (input, char_pos_in_para) = le_u32(input)?;

    Ok((
        input,
        DocInfoProperties {
            section_count,
            page_start_number,
            footnote_start_number,
            endnote_start_number,
            figure_start_number,
            table_start_number,
            equation_start_number,
            list_id,
            paragraph_id,
            char_pos_in_para,
        },
    ))
}
