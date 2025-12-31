// crates/hwp-core/src/parser/docinfo/tab_def.rs

//! Tab definition (HWPTAG_TAB_DEF) parser.

use hwp_types::{TabDef, TabStop};
use nom::{
    IResult,
    number::complete::{le_i32, le_u8, le_u16, le_u32},
};

/// Parse a tab definition record.
pub fn parse_tab_def(input: &[u8]) -> IResult<&[u8], TabDef> {
    let (input, properties) = le_u32(input)?;
    let (mut input, count) = le_u16(input)?;

    let mut tabs = Vec::new();
    for _ in 0..count {
        if input.len() < 8 {
            break;
        }
        let (rest, position) = le_i32(input)?;
        let (rest, kind) = le_u8(rest)?;
        let (rest, leader) = le_u8(rest)?;
        let (rest, _reserved) = le_u16(rest)?;
        tabs.push(TabStop {
            position,
            kind,
            leader,
        });
        input = rest;
    }

    Ok((input, TabDef { properties, tabs }))
}
