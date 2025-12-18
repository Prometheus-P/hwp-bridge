// crates/hwp-core/src/parser/docinfo/border_fill.rs

//! BorderFill (테두리/배경) 파서
//!
//! HWP DocInfo 스트림의 BorderFill 레코드를 파싱합니다.
//! 레코드 태그: 0x04 (BORDER_FILL)

use hwp_types::{
    BorderFill, BorderLine, BorderLineType, FillGradient, FillImage, FillInfo, GradientType,
};
use nom::{
    IResult,
    number::complete::{le_i8, le_u8, le_u16, le_u32},
};

use crate::parser::primitives::parse_colorref;

/// BorderFill 레코드 파싱
///
/// # Format (가변 길이)
/// - properties: u16 = 2 bytes
/// - left border: 6 bytes (type, thickness, color)
/// - right border: 6 bytes
/// - top border: 6 bytes
/// - bottom border: 6 bytes
/// - diagonal border: 6 bytes
/// - fill_type: u32 = 4 bytes
/// - background_color: u32 = 4 bytes
/// - pattern_color: u32 = 4 bytes
/// - pattern_type: u32 = 4 bytes
/// - [optional] image_info (if fill_type & 0x04)
/// - [optional] gradient_info (if fill_type & 0x02)
pub fn parse_border_fill(input: &[u8]) -> IResult<&[u8], BorderFill> {
    // 속성 플래그
    let (input, properties) = le_u16(input)?;

    // 5개 테두리 선
    let (input, left) = parse_border_line(input)?;
    let (input, right) = parse_border_line(input)?;
    let (input, top) = parse_border_line(input)?;
    let (input, bottom) = parse_border_line(input)?;
    let (input, diagonal) = parse_border_line(input)?;

    // 채우기 정보
    let (input, fill) = parse_fill_info(input)?;

    Ok((
        input,
        BorderFill {
            properties,
            left,
            right,
            top,
            bottom,
            diagonal,
            fill,
        },
    ))
}

/// 테두리 선 파싱 (6 bytes)
fn parse_border_line(input: &[u8]) -> IResult<&[u8], BorderLine> {
    let (input, line_type_raw) = le_u8(input)?;
    let (input, thickness) = le_u8(input)?;
    let (input, color) = parse_colorref(input)?;

    Ok((
        input,
        BorderLine {
            line_type: BorderLineType::from_u8(line_type_raw),
            thickness,
            color,
        },
    ))
}

/// 채우기 정보 파싱
fn parse_fill_info(input: &[u8]) -> IResult<&[u8], FillInfo> {
    // 최소 16바이트 필요
    if input.len() < 16 {
        return Ok((
            input,
            FillInfo {
                fill_type: 0,
                background_color: 0xFFFFFFFF,
                pattern_color: 0,
                pattern_type: 0,
                image: None,
                gradient: None,
            },
        ));
    }

    let (input, fill_type) = le_u32(input)?;
    let (input, background_color) = parse_colorref(input)?;
    let (input, pattern_color) = parse_colorref(input)?;
    let (input, pattern_type) = le_u32(input)?;

    // 이미지 채우기 (fill_type bit 2)
    let (input, image) = if fill_type & 0x04 != 0 && input.len() >= 5 {
        let (input, brightness) = le_i8(input)?;
        let (input, contrast) = le_i8(input)?;
        let (input, effect) = le_u8(input)?;
        let (input, bin_data_id) = le_u16(input)?;

        (
            input,
            Some(FillImage {
                brightness,
                contrast,
                effect,
                bin_data_id,
            }),
        )
    } else {
        (input, None)
    };

    // 그라데이션 채우기 (fill_type bit 1)
    let (input, gradient) = if fill_type & 0x02 != 0 && input.len() >= 17 {
        let (input, gradient_type_raw) = le_u8(input)?;
        let (input, start_color) = parse_colorref(input)?;
        let (input, end_color) = parse_colorref(input)?;
        let (input, angle) = le_u16(input)?;
        let (input, center_x) = le_u16(input)?;
        let (input, center_y) = le_u16(input)?;
        let (input, blur) = le_u16(input)?;

        (
            input,
            Some(FillGradient {
                gradient_type: GradientType::from_u8(gradient_type_raw),
                start_color,
                end_color,
                angle,
                center_x,
                center_y,
                blur,
            }),
        )
    } else {
        (input, None)
    };

    Ok((
        input,
        FillInfo {
            fill_type,
            background_color,
            pattern_color,
            pattern_type,
            image,
            gradient,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_border_line_data(line_type: u8, thickness: u8, color: u32) -> Vec<u8> {
        let mut data = vec![line_type, thickness];
        data.extend_from_slice(&color.to_le_bytes());
        data
    }

    fn create_basic_border_fill_data() -> Vec<u8> {
        let mut data = Vec::new();

        // properties
        data.extend_from_slice(&0x0001u16.to_le_bytes());

        // 5 borders (solid, black, 1px)
        for _ in 0..5 {
            data.extend(create_border_line_data(1, 1, 0x000000));
        }

        // fill info (16 bytes minimum)
        data.extend_from_slice(&0x01u32.to_le_bytes()); // fill_type: solid
        data.extend_from_slice(&0xFFFFFFu32.to_le_bytes()); // background: white
        data.extend_from_slice(&0x000000u32.to_le_bytes()); // pattern_color
        data.extend_from_slice(&0u32.to_le_bytes()); // pattern_type

        data
    }

    #[test]
    fn test_should_parse_border_fill_when_valid_data() {
        // Arrange
        let data = create_basic_border_fill_data();

        // Act
        let result = parse_border_fill(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, bf) = result.unwrap();
        assert!(remaining.is_empty());

        assert_eq!(bf.properties, 0x0001);
        assert!(bf.has_3d_effect());
        assert!(bf.left.is_visible());
        assert_eq!(bf.left.line_type, BorderLineType::Solid);
        assert_eq!(bf.left.thickness, 1);
        assert_eq!(bf.left.color, 0x000000);
    }

    #[test]
    fn test_should_parse_border_line_types() {
        // Arrange
        let types = [
            (0, BorderLineType::None),
            (1, BorderLineType::Solid),
            (2, BorderLineType::Dash),
            (8, BorderLineType::Double),
            (12, BorderLineType::Wave),
        ];

        for (raw, expected) in types {
            let mut data = create_basic_border_fill_data();
            // left border type is at offset 2
            data[2] = raw;

            // Act
            let (_, bf) = parse_border_fill(&data).unwrap();

            // Assert
            assert_eq!(bf.left.line_type, expected);
        }
    }

    #[test]
    fn test_should_parse_solid_fill() {
        // Arrange
        let data = create_basic_border_fill_data();

        // Act
        let (_, bf) = parse_border_fill(&data).unwrap();

        // Assert
        assert!(bf.fill.is_solid());
        assert!(!bf.fill.is_gradient());
        assert!(!bf.fill.is_image());
        assert_eq!(bf.fill.background_color, 0xFFFFFF);
    }

    #[test]
    fn test_should_parse_gradient_fill() {
        // Arrange
        let mut data = Vec::new();

        // properties
        data.extend_from_slice(&0u16.to_le_bytes());

        // 5 borders (none)
        for _ in 0..5 {
            data.extend(create_border_line_data(0, 0, 0));
        }

        // fill info with gradient
        data.extend_from_slice(&0x02u32.to_le_bytes()); // fill_type: gradient
        data.extend_from_slice(&0xFFFFFFu32.to_le_bytes()); // background
        data.extend_from_slice(&0u32.to_le_bytes()); // pattern_color
        data.extend_from_slice(&0u32.to_le_bytes()); // pattern_type

        // gradient info (17 bytes)
        data.push(1); // gradient_type: Radial
        data.extend_from_slice(&0xFF0000u32.to_le_bytes()); // start: red
        data.extend_from_slice(&0x0000FFu32.to_le_bytes()); // end: blue
        data.extend_from_slice(&45u16.to_le_bytes()); // angle
        data.extend_from_slice(&50u16.to_le_bytes()); // center_x
        data.extend_from_slice(&50u16.to_le_bytes()); // center_y
        data.extend_from_slice(&10u16.to_le_bytes()); // blur

        // Act
        let (_, bf) = parse_border_fill(&data).unwrap();

        // Assert
        assert!(bf.fill.is_gradient());
        let gradient = bf.fill.gradient.unwrap();
        assert_eq!(gradient.gradient_type, GradientType::Radial);
        assert_eq!(gradient.start_color, 0xFF0000);
        assert_eq!(gradient.end_color, 0x0000FF);
        assert_eq!(gradient.angle, 45);
    }

    #[test]
    fn test_should_parse_image_fill() {
        // Arrange
        let mut data = Vec::new();

        // properties
        data.extend_from_slice(&0u16.to_le_bytes());

        // 5 borders (none)
        for _ in 0..5 {
            data.extend(create_border_line_data(0, 0, 0));
        }

        // fill info with image
        data.extend_from_slice(&0x04u32.to_le_bytes()); // fill_type: image
        data.extend_from_slice(&0xFFFFFFu32.to_le_bytes()); // background
        data.extend_from_slice(&0u32.to_le_bytes()); // pattern_color
        data.extend_from_slice(&0u32.to_le_bytes()); // pattern_type

        // image info (5 bytes)
        data.push(10i8 as u8); // brightness
        data.push(0i8 as u8); // contrast
        data.push(0); // effect
        data.extend_from_slice(&5u16.to_le_bytes()); // bin_data_id

        // Act
        let (_, bf) = parse_border_fill(&data).unwrap();

        // Assert
        assert!(bf.fill.is_image());
        let image = bf.fill.image.unwrap();
        assert_eq!(image.brightness, 10);
        assert_eq!(image.bin_data_id, 5);
    }

    #[test]
    fn test_should_handle_empty_fill_gracefully() {
        // Arrange: properties + borders만, fill 없음
        let mut data = Vec::new();
        data.extend_from_slice(&0u16.to_le_bytes());
        for _ in 0..5 {
            data.extend(create_border_line_data(0, 0, 0));
        }
        // fill info 없음 (16바이트 미만)

        // Act
        let result = parse_border_fill(&data);

        // Assert
        assert!(result.is_ok());
        let (_, bf) = result.unwrap();
        assert!(!bf.fill.is_solid());
        assert_eq!(bf.fill.background_color, 0xFFFFFFFF); // 기본값
    }
}
