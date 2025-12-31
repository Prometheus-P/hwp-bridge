// crates/hwp-core/src/parser/docinfo/doc_data.rs

//! DocData (HWPTAG_DOC_DATA) parser.

use hwp_types::{ParameterItem, ParameterSet, ParameterValue};
use nom::{
    IResult,
    number::complete::{le_i8, le_i16, le_i32, le_u8, le_u16, le_u32},
};

use crate::parser::primitives::parse_utf16le_string;

const PIT_NULL: u16 = 0;
const PIT_BSTR: u16 = 1;
const PIT_I1: u16 = 2;
const PIT_I2: u16 = 3;
const PIT_I4: u16 = 4;
const PIT_I: u16 = 5;
const PIT_UI1: u16 = 6;
const PIT_UI2: u16 = 7;
const PIT_UI4: u16 = 8;
const PIT_UI: u16 = 9;
const PIT_SET: u16 = 0x8000;
const PIT_ARRAY: u16 = 0x8001;
const PIT_BINDATA: u16 = 0x8002;

/// Parse DocData record into parameter sets.
pub fn parse_doc_data(mut input: &[u8]) -> IResult<&[u8], Vec<ParameterSet>> {
    let mut sets = Vec::new();
    while input.len() >= 4 {
        let before = input.len();
        let (rest, set) = parse_parameter_set(input)?;
        if rest.len() == before {
            break;
        }
        sets.push(set);
        input = rest;
    }

    Ok((input, sets))
}

fn parse_parameter_set(input: &[u8]) -> IResult<&[u8], ParameterSet> {
    let (input, set_id) = le_u16(input)?;
    let (mut input, item_count) = le_i16(input)?;
    let count = if item_count < 0 {
        0
    } else {
        item_count as usize
    };

    let mut items = Vec::with_capacity(count);
    for _ in 0..count {
        let (rest, item) = parse_parameter_item(input)?;
        items.push(item);
        input = rest;
    }

    Ok((input, ParameterSet { id: set_id, items }))
}

fn parse_parameter_item(input: &[u8]) -> IResult<&[u8], ParameterItem> {
    let (input, item_id) = le_u16(input)?;
    let (input, item_type) = le_u16(input)?;
    let (input, value) = parse_parameter_value(input, item_type)?;

    Ok((input, ParameterItem { id: item_id, value }))
}

fn parse_parameter_value(input: &[u8], item_type: u16) -> IResult<&[u8], ParameterValue> {
    match item_type {
        PIT_NULL => {
            let (input, value) = le_u32(input)?;
            Ok((input, ParameterValue::Null(value)))
        }
        PIT_BSTR => {
            let (input, value) = parse_utf16le_string(input)?;
            Ok((input, ParameterValue::BStr(value)))
        }
        PIT_I1 => {
            let (input, value) = le_i8(input)?;
            Ok((input, ParameterValue::I1(value)))
        }
        PIT_I2 => {
            let (input, value) = le_i16(input)?;
            Ok((input, ParameterValue::I2(value)))
        }
        PIT_I4 => {
            let (input, value) = le_i32(input)?;
            Ok((input, ParameterValue::I4(value)))
        }
        PIT_I => {
            let (input, value) = le_i32(input)?;
            Ok((input, ParameterValue::I(value)))
        }
        PIT_UI1 => {
            let (input, value) = le_u8(input)?;
            Ok((input, ParameterValue::Ui1(value)))
        }
        PIT_UI2 => {
            let (input, value) = le_u16(input)?;
            Ok((input, ParameterValue::Ui2(value)))
        }
        PIT_UI4 => {
            let (input, value) = le_u32(input)?;
            Ok((input, ParameterValue::Ui4(value)))
        }
        PIT_UI => {
            let (input, value) = le_u32(input)?;
            Ok((input, ParameterValue::Ui(value)))
        }
        PIT_SET => {
            let (input, set) = parse_parameter_set(input)?;
            Ok((input, ParameterValue::Set(Box::new(set))))
        }
        PIT_ARRAY => {
            let (mut input, count) = le_i16(input)?;
            let set_count = if count < 0 { 0 } else { count as usize };
            let mut sets = Vec::with_capacity(set_count);
            for _ in 0..set_count {
                let (rest, set) = parse_parameter_set(input)?;
                sets.push(set);
                input = rest;
            }
            Ok((input, ParameterValue::Array(sets)))
        }
        PIT_BINDATA => {
            let (input, id) = le_u16(input)?;
            Ok((input, ParameterValue::Bindata(id)))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Alt,
        ))),
    }
}
