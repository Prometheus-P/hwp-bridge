// crates/hwp-core/src/parser/docinfo/numbering.rs

//! Numbering definition (HWPTAG_NUMBERING) parser.

use hwp_types::{Numbering, NumberingLevel};
use nom::{
    IResult,
    number::complete::{le_u16, le_u32},
};

use super::para_head::parse_para_head_info;
use crate::parser::primitives::parse_utf16le_string;

/// Parse numbering definition.
pub fn parse_numbering(input: &[u8]) -> IResult<&[u8], Numbering> {
    let (mut input, head) = parse_para_head_info(input)?;

    let mut levels = Vec::new();
    for _ in 0..7 {
        if input.len() < 2 {
            break;
        }
        let (rest, format) = match parse_utf16le_string(input) {
            Ok(result) => result,
            Err(_) => break,
        };
        levels.push(NumberingLevel {
            format,
            start_number: None,
            start_number_extended: None,
        });
        input = rest;
    }

    for level in levels.iter_mut() {
        if input.len() < 2 {
            break;
        }
        let (rest, start) = le_u16(input)?;
        level.start_number = Some(start);
        input = rest;
    }

    if input.len() >= 4 * levels.len() {
        for level in levels.iter_mut() {
            let (rest, start) = le_u32(input)?;
            level.start_number_extended = Some(start);
            input = rest;
        }
    }

    let mut extended_levels = Vec::new();
    for _ in 0..3 {
        if input.len() < 2 {
            break;
        }
        let (rest, format) = match parse_utf16le_string(input) {
            Ok(result) => result,
            Err(_) => break,
        };
        extended_levels.push(NumberingLevel {
            format,
            start_number: None,
            start_number_extended: None,
        });
        input = rest;
    }

    if input.len() >= 4 * extended_levels.len() {
        for level in extended_levels.iter_mut() {
            let (rest, start) = le_u32(input)?;
            level.start_number_extended = Some(start);
            input = rest;
        }
    }

    Ok((
        input,
        Numbering {
            head,
            levels,
            extended_levels,
        },
    ))
}
