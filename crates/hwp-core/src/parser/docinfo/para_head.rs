// crates/hwp-core/src/parser/docinfo/para_head.rs

//! Paragraph head info parser shared by numbering/bullets.

use hwp_types::ParaHeadInfo;
use nom::{
    IResult,
    number::complete::{le_i16, le_u32},
};

/// Parse paragraph head info (8 bytes).
pub fn parse_para_head_info(input: &[u8]) -> IResult<&[u8], ParaHeadInfo> {
    let (input, properties) = le_u32(input)?;
    let (input, width_adjust) = le_i16(input)?;
    let (input, distance) = le_i16(input)?;

    Ok((
        input,
        ParaHeadInfo {
            properties,
            width_adjust,
            distance,
        },
    ))
}
