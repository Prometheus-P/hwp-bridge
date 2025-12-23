// crates/hwp-core/src/parser/docinfo/compatibility.rs

//! Compatibility record parsers.

use hwp_types::{CompatibleDocument, LayoutCompatibility};
use nom::{IResult, number::complete::le_u32};

/// Parse compatible document record.
pub fn parse_compatible_document(input: &[u8]) -> IResult<&[u8], CompatibleDocument> {
    let (input, program) = le_u32(input)?;
    Ok((input, CompatibleDocument { program }))
}

/// Parse layout compatibility record.
pub fn parse_layout_compatibility(input: &[u8]) -> IResult<&[u8], LayoutCompatibility> {
    let (input, char_level) = le_u32(input)?;
    let (input, paragraph_level) = le_u32(input)?;
    let (input, section_level) = le_u32(input)?;
    let (input, object_level) = le_u32(input)?;
    let (input, field_level) = le_u32(input)?;

    Ok((
        input,
        LayoutCompatibility {
            char_level,
            paragraph_level,
            section_level,
            object_level,
            field_level,
        },
    ))
}
