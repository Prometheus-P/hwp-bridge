// crates/hwp-core/src/parser/docinfo/char_shape.rs

//! CharShape (글자 모양) 파서
//!
//! HWP DocInfo 스트림의 CharShape 레코드를 파싱합니다.
//! 레코드 태그: 0x07 (CHAR_SHAPE)

use hwp_types::{CharShape, CharShapeAttr};
use nom::{
    IResult,
    number::complete::{le_i8, le_i32, le_u16, le_u32},
};

use crate::parser::primitives::{
    parse_colorref, parse_i8_array_7, parse_u8_array_7, parse_u16_array_7,
};

/// CharShape 레코드 최소 크기 (바이트)
pub const CHAR_SHAPE_MIN_SIZE: usize = 72;

/// CharShape 레코드 파싱
///
/// # Format (최소 72 바이트)
/// - font_ids: [u16; 7] = 14 bytes
/// - font_scales: [u8; 7] = 7 bytes
/// - char_spacing: [i8; 7] = 7 bytes
/// - relative_sizes: [u8; 7] = 7 bytes
/// - char_offsets: [i8; 7] = 7 bytes
/// - base_size: i32 = 4 bytes
/// - attr: u32 = 4 bytes
/// - shadow_gap_x: i8 = 1 byte
/// - shadow_gap_y: i8 = 1 byte
/// - text_color: u32 = 4 bytes
/// - underline_color: u32 = 4 bytes
/// - shade_color: u32 = 4 bytes
/// - shadow_color: u32 = 4 bytes
/// - border_fill_id: u16 = 2 bytes (선택적, 버전에 따라)
/// - 추가 필드 (버전에 따라 가변)
pub fn parse_char_shape(input: &[u8]) -> IResult<&[u8], CharShape> {
    // 언어별 배열 (7개씩)
    let (input, font_ids) = parse_u16_array_7(input)?;
    let (input, font_scales) = parse_u8_array_7(input)?;
    let (input, char_spacing) = parse_i8_array_7(input)?;
    let (input, relative_sizes) = parse_u8_array_7(input)?;
    let (input, char_offsets) = parse_i8_array_7(input)?;

    // 기본 속성
    let (input, base_size) = le_i32(input)?;
    let (input, attr_bits) = le_u32(input)?;
    let (input, shadow_gap_x) = le_i8(input)?;
    let (input, shadow_gap_y) = le_i8(input)?;

    // 색상 정보
    let (input, text_color) = parse_colorref(input)?;
    let (input, underline_color) = parse_colorref(input)?;
    let (input, shade_color) = parse_colorref(input)?;
    let (input, shadow_color) = parse_colorref(input)?;

    // 선택적: border_fill_id (남은 데이터가 있으면 파싱)
    let (input, border_fill_id) = if input.len() >= 2 {
        let (input, id) = le_u16(input)?;
        (input, id)
    } else {
        (input, 0)
    };

    Ok((
        input,
        CharShape {
            font_ids,
            font_scales,
            char_spacing,
            relative_sizes,
            char_offsets,
            base_size,
            attr: CharShapeAttr::from_bits(attr_bits),
            shadow_gap_x,
            shadow_gap_y,
            text_color,
            underline_color,
            shade_color,
            shadow_color,
            border_fill_id,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_char_shape_data() -> Vec<u8> {
        let mut data = Vec::new();

        // font_ids: [0, 1, 2, 3, 4, 5, 6]
        for i in 0u16..7 {
            data.extend_from_slice(&i.to_le_bytes());
        }

        // font_scales: [100; 7]
        data.extend_from_slice(&[100u8; 7]);

        // char_spacing: [0; 7]
        data.extend_from_slice(&[0i8 as u8; 7]);

        // relative_sizes: [100; 7]
        data.extend_from_slice(&[100u8; 7]);

        // char_offsets: [0; 7]
        data.extend_from_slice(&[0i8 as u8; 7]);

        // base_size: 1000 (10pt)
        data.extend_from_slice(&1000i32.to_le_bytes());

        // attr: 0x03 (bold + italic)
        data.extend_from_slice(&0x03u32.to_le_bytes());

        // shadow_gap: (10, 10)
        data.push(10i8 as u8);
        data.push(10i8 as u8);

        // text_color: 0x000000 (black)
        data.extend_from_slice(&0x000000u32.to_le_bytes());

        // underline_color: 0x0000FF (red in BGR)
        data.extend_from_slice(&0x0000FFu32.to_le_bytes());

        // shade_color: 0xFFFFFF (white)
        data.extend_from_slice(&0xFFFFFFu32.to_le_bytes());

        // shadow_color: 0x808080 (gray)
        data.extend_from_slice(&0x808080u32.to_le_bytes());

        // border_fill_id: 1
        data.extend_from_slice(&1u16.to_le_bytes());

        data
    }

    #[test]
    fn test_should_parse_char_shape_when_valid_data() {
        // Arrange
        let data = create_char_shape_data();

        // Act
        let result = parse_char_shape(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, shape) = result.unwrap();
        assert!(remaining.is_empty());

        assert_eq!(shape.font_ids, [0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(shape.font_scales, [100; 7]);
        assert_eq!(shape.base_size, 1000);
        assert!(shape.attr.is_bold());
        assert!(shape.attr.is_italic());
        assert_eq!(shape.shadow_gap_x, 10);
        assert_eq!(shape.shadow_gap_y, 10);
        assert_eq!(shape.text_color, 0x000000);
        assert_eq!(shape.border_fill_id, 1);
    }

    #[test]
    fn test_should_parse_char_shape_without_border_fill_id() {
        // Arrange: 데이터에서 border_fill_id 제외
        let mut data = create_char_shape_data();
        data.truncate(data.len() - 2); // border_fill_id 제거

        // Act
        let result = parse_char_shape(&data);

        // Assert
        assert!(result.is_ok());
        let (_, shape) = result.unwrap();
        assert_eq!(shape.border_fill_id, 0); // 기본값
    }

    #[test]
    fn test_should_extract_char_shape_attributes() {
        // Arrange
        let mut data = create_char_shape_data();
        // attr 위치: font_ids(14) + scales(7) + spacing(7) + sizes(7) + offsets(7) + base_size(4) = 46
        let attr_offset = 14 + 7 + 7 + 7 + 7 + 4;
        let attr: u32 = 0b0000_0111; // bold + italic + underline type 1
        data[attr_offset..attr_offset + 4].copy_from_slice(&attr.to_le_bytes());

        // Act
        let (_, shape) = parse_char_shape(&data).unwrap();

        // Assert
        assert!(shape.attr.is_bold());
        assert!(shape.attr.is_italic());
        assert_eq!(shape.attr.underline_type(), 1);
    }

    #[test]
    fn test_should_return_correct_size_pt() {
        // Arrange
        let data = create_char_shape_data();
        let (_, shape) = parse_char_shape(&data).unwrap();

        // Act
        let size_pt = shape.size_pt();

        // Assert
        assert!((size_pt - 10.0).abs() < 0.01); // 1000 / 100 = 10pt
    }
}
