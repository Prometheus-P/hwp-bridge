// crates/hwp-core/src/parser/docinfo/para_shape.rs

//! ParaShape (문단 모양) 파서
//!
//! HWP DocInfo 스트림의 ParaShape 레코드를 파싱합니다.
//! 레코드 태그: 0x19 (PARA_SHAPE)

use hwp_types::{LineSpaceType, ParaShape, ParaShapeAttr};
use nom::{
    IResult,
    number::complete::{le_i16, le_i32, le_u16, le_u32},
};

/// ParaShape 레코드 최소 크기 (바이트)
pub const PARA_SHAPE_MIN_SIZE: usize = 54;

/// ParaShape 레코드 파싱
///
/// # Format (최소 54 바이트)
/// - attr: u32 = 4 bytes (정렬, 줄바꿈 등)
/// - margin_left: i32 = 4 bytes
/// - margin_right: i32 = 4 bytes
/// - indent: i32 = 4 bytes
/// - margin_top: i32 = 4 bytes
/// - margin_bottom: i32 = 4 bytes
/// - line_spacing: i32 = 4 bytes
/// - tab_def_id: u16 = 2 bytes
/// - para_head_id: u16 = 2 bytes
/// - border_fill_id: u16 = 2 bytes
/// - border_space_left: i16 = 2 bytes
/// - border_space_right: i16 = 2 bytes
/// - border_space_top: i16 = 2 bytes
/// - border_space_bottom: i16 = 2 bytes
/// - attr2: u32 = 4 bytes
/// - attr3: u32 = 4 bytes
/// - line_space_type: u32 = 4 bytes
pub fn parse_para_shape(input: &[u8]) -> IResult<&[u8], ParaShape> {
    // 속성1
    let (input, attr_bits) = le_u32(input)?;

    // 여백 정보
    let (input, margin_left) = le_i32(input)?;
    let (input, margin_right) = le_i32(input)?;
    let (input, indent) = le_i32(input)?;
    let (input, margin_top) = le_i32(input)?;
    let (input, margin_bottom) = le_i32(input)?;
    let (input, line_spacing) = le_i32(input)?;

    // ID 참조
    let (input, tab_def_id) = le_u16(input)?;
    let (input, para_head_id) = le_u16(input)?;
    let (input, border_fill_id) = le_u16(input)?;

    // 테두리 여백
    let (input, border_space_left) = le_i16(input)?;
    let (input, border_space_right) = le_i16(input)?;
    let (input, border_space_top) = le_i16(input)?;
    let (input, border_space_bottom) = le_i16(input)?;

    // 추가 속성 (선택적)
    let (input, attr2) = if input.len() >= 4 {
        le_u32(input)?
    } else {
        (input, 0)
    };

    let (input, attr3) = if input.len() >= 4 {
        le_u32(input)?
    } else {
        (input, 0)
    };

    let (input, line_space_type_raw) = if input.len() >= 4 {
        le_u32(input)?
    } else {
        (input, 0)
    };

    Ok((
        input,
        ParaShape {
            attr: ParaShapeAttr::from_bits(attr_bits),
            margin_left,
            margin_right,
            indent,
            margin_top,
            margin_bottom,
            line_spacing,
            tab_def_id,
            para_head_id,
            border_fill_id,
            border_space_left,
            border_space_right,
            border_space_top,
            border_space_bottom,
            attr2,
            attr3,
            line_space_type: LineSpaceType::from_u8(line_space_type_raw as u8),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hwp_types::Alignment;

    fn create_para_shape_data() -> Vec<u8> {
        let mut data = Vec::new();

        // attr: 0b0000_1100 = center alignment (bits 2-4 = 3)
        data.extend_from_slice(&0b0000_1100u32.to_le_bytes());

        // margins
        data.extend_from_slice(&1000i32.to_le_bytes()); // left
        data.extend_from_slice(&1000i32.to_le_bytes()); // right
        data.extend_from_slice(&500i32.to_le_bytes()); // indent
        data.extend_from_slice(&200i32.to_le_bytes()); // top
        data.extend_from_slice(&200i32.to_le_bytes()); // bottom
        data.extend_from_slice(&160i32.to_le_bytes()); // line_spacing (160%)

        // IDs
        data.extend_from_slice(&0u16.to_le_bytes()); // tab_def_id
        data.extend_from_slice(&0u16.to_le_bytes()); // para_head_id
        data.extend_from_slice(&1u16.to_le_bytes()); // border_fill_id

        // border spaces
        data.extend_from_slice(&100i16.to_le_bytes()); // left
        data.extend_from_slice(&100i16.to_le_bytes()); // right
        data.extend_from_slice(&50i16.to_le_bytes()); // top
        data.extend_from_slice(&50i16.to_le_bytes()); // bottom

        // additional attributes
        data.extend_from_slice(&0u32.to_le_bytes()); // attr2
        data.extend_from_slice(&0u32.to_le_bytes()); // attr3
        data.extend_from_slice(&0u32.to_le_bytes()); // line_space_type (Percent)

        data
    }

    #[test]
    fn test_should_parse_para_shape_when_valid_data() {
        // Arrange
        let data = create_para_shape_data();

        // Act
        let result = parse_para_shape(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, shape) = result.unwrap();
        assert!(remaining.is_empty());

        assert_eq!(shape.alignment(), Alignment::Center);
        assert_eq!(shape.margin_left, 1000);
        assert_eq!(shape.margin_right, 1000);
        assert_eq!(shape.indent, 500);
        assert_eq!(shape.line_spacing, 160);
        assert_eq!(shape.border_fill_id, 1);
        assert_eq!(shape.border_space_left, 100);
        assert_eq!(shape.line_space_type, LineSpaceType::Percent);
    }

    #[test]
    fn test_should_parse_para_shape_minimal() {
        // Arrange: 최소 크기만 포함
        let mut data = Vec::new();

        // 필수 필드만
        data.extend_from_slice(&0b0000_0100u32.to_le_bytes()); // left alignment
        data.extend_from_slice(&0i32.to_le_bytes()); // margins
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&180i32.to_le_bytes()); // line_spacing

        data.extend_from_slice(&0u16.to_le_bytes()); // IDs
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());

        data.extend_from_slice(&0i16.to_le_bytes()); // border spaces
        data.extend_from_slice(&0i16.to_le_bytes());
        data.extend_from_slice(&0i16.to_le_bytes());
        data.extend_from_slice(&0i16.to_le_bytes());

        // Act
        let result = parse_para_shape(&data);

        // Assert
        assert!(result.is_ok());
        let (_, shape) = result.unwrap();
        assert_eq!(shape.alignment(), Alignment::Left);
        assert_eq!(shape.line_spacing, 180);
        assert_eq!(shape.attr2, 0); // 기본값
    }

    #[test]
    fn test_should_detect_indent_and_outdent() {
        // Arrange
        let mut data = create_para_shape_data();
        // indent 위치: attr(4) + left(4) + right(4) = 12
        let indent_offset = 12;

        // Positive indent
        data[indent_offset..indent_offset + 4].copy_from_slice(&500i32.to_le_bytes());
        let (_, shape1) = parse_para_shape(&data).unwrap();
        assert!(shape1.has_indent());
        assert!(!shape1.has_outdent());

        // Negative indent (outdent)
        data[indent_offset..indent_offset + 4].copy_from_slice(&(-500i32).to_le_bytes());
        let (_, shape2) = parse_para_shape(&data).unwrap();
        assert!(shape2.has_indent()); // indent != 0
        assert!(shape2.has_outdent()); // indent < 0
    }

    #[test]
    fn test_should_parse_different_alignments() {
        // Arrange
        let alignments = [
            (0b0000_0000u32, Alignment::Justify),    // 0
            (0b0000_0100u32, Alignment::Left),       // 1
            (0b0000_1000u32, Alignment::Right),      // 2
            (0b0000_1100u32, Alignment::Center),     // 3
            (0b0001_0000u32, Alignment::Distribute), // 4
        ];

        for (attr_bits, expected_alignment) in alignments {
            let mut data = create_para_shape_data();
            data[0..4].copy_from_slice(&attr_bits.to_le_bytes());

            let (_, shape) = parse_para_shape(&data).unwrap();
            assert_eq!(
                shape.alignment(),
                expected_alignment,
                "Failed for bits: {:#010b}",
                attr_bits
            );
        }
    }

    #[test]
    fn test_should_parse_line_space_types() {
        // Arrange
        let mut data = create_para_shape_data();
        let lsp_offset = data.len() - 4; // 마지막 4바이트

        let types = [
            (0u32, LineSpaceType::Percent),
            (1u32, LineSpaceType::Fixed),
            (2u32, LineSpaceType::SpaceOnly),
            (3u32, LineSpaceType::AtLeast),
        ];

        for (raw, expected) in types {
            data[lsp_offset..lsp_offset + 4].copy_from_slice(&raw.to_le_bytes());
            let (_, shape) = parse_para_shape(&data).unwrap();
            assert_eq!(shape.line_space_type, expected);
        }
    }
}
