// crates/hwp-core/src/parser/docinfo/bullet.rs

//! Bullet definition (HWPTAG_BULLET) parser.

use hwp_types::{Bullet, ImageBullet};
use nom::{
    IResult,
    number::complete::{le_i32, le_i8, le_u16, le_u8},
};

use super::para_head::parse_para_head_info;

/// Parse bullet definition.
pub fn parse_bullet(input: &[u8]) -> IResult<&[u8], Bullet> {
    let (input, head) = parse_para_head_info(input)?;
    let (input, bullet_char) = le_u16(input)?;
    let (input, image_bullet_id) = le_i32(input)?;

    let (input, image_bullet) = if input.len() >= 4 {
        let (input, contrast) = le_i8(input)?;
        let (input, brightness) = le_i8(input)?;
        let (input, effect) = le_u8(input)?;
        let (input, image_id) = le_u8(input)?;
        (
            input,
            Some(ImageBullet {
                contrast,
                brightness,
                effect,
                image_id,
            }),
        )
    } else {
        (input, None)
    };

    let (input, check_char) = if input.len() >= 2 {
        le_u16(input)?
    } else {
        (input, 0)
    };

    Ok((
        input,
        Bullet {
            head,
            bullet_char,
            image_bullet_id,
            image_bullet,
            check_char,
        },
    ))
}
