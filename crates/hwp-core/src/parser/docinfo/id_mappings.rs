// crates/hwp-core/src/parser/docinfo/id_mappings.rs

//! ID mappings (HWPTAG_ID_MAPPINGS) parser.

use hwp_types::IdMappings;
use nom::IResult;

/// Parse ID mappings record.
pub fn parse_id_mappings(input: &[u8]) -> IResult<&[u8], IdMappings> {
    let mut counts = [0i32; 18];
    let mut offset = 0usize;
    let mut idx = 0usize;

    while offset + 4 <= input.len() && idx < counts.len() {
        let value = i32::from_le_bytes(
            input[offset..offset + 4]
                .try_into()
                .expect("slice with exact length"),
        );
        counts[idx] = value;
        idx += 1;
        offset += 4;
    }

    Ok((&input[offset..], IdMappings { counts }))
}
