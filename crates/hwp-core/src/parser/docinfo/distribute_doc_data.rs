// crates/hwp-core/src/parser/docinfo/distribute_doc_data.rs

//! Distribute document data (HWPTAG_DISTRIBUTE_DOC_DATA) parser.

use hwp_types::DistributeDocData;
use nom::{IResult, bytes::complete::take};

/// Parse distribute document data.
pub fn parse_distribute_doc_data(input: &[u8]) -> IResult<&[u8], DistributeDocData> {
    let (input, data) = take(input.len())(input)?;
    Ok((
        input,
        DistributeDocData {
            data: data.to_vec(),
        },
    ))
}
