// crates/hwp-core/src/parser/docinfo/style.rs

//! Style definition (HWPTAG_STYLE) parser.

use hwp_types::StyleRecord;
use nom::{
    IResult,
    number::complete::{le_i16, le_u8, le_u16},
};

use crate::parser::primitives::parse_utf16le_string;

/// Parse style definition.
pub fn parse_style(input: &[u8]) -> IResult<&[u8], StyleRecord> {
    let (input, name) = parse_utf16le_string(input)?;
    let (input, english_name) = parse_utf16le_string(input)?;
    let (input, properties) = le_u8(input)?;
    let (input, next_style_id) = le_u8(input)?;
    let (input, language_id) = le_i16(input)?;
    let (input, para_shape_id) = le_u16(input)?;
    let (input, char_shape_id) = le_u16(input)?;

    Ok((
        input,
        StyleRecord {
            name,
            english_name,
            properties,
            next_style_id,
            language_id,
            para_shape_id,
            char_shape_id,
        },
    ))
}
