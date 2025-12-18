// crates/hwp-core/src/parser/docinfo/face_name.rs

//! FaceName (글꼴 이름) 파서
//!
//! HWP DocInfo 스트림의 FaceName 레코드를 파싱합니다.
//! 레코드 태그: 0x03 (FACE_NAME)

use hwp_types::{FaceName, Panose, SubstituteFontType};
use nom::{IResult, bytes::complete::take, number::complete::le_u8};

use crate::parser::primitives::parse_utf16le_string;

/// FaceName 레코드 파싱
///
/// # Format (가변 길이)
/// - properties: u8 = 1 byte
/// - name: UTF-16LE string (length-prefixed)
/// - \[optional\] substitute_type: u8 (if properties & 0x01)
/// - \[optional\] substitute_name: UTF-16LE string (if properties & 0x01)
/// - \[optional\] panose: \[u8; 10\] (if properties & 0x80)
/// - \[optional\] default_name: UTF-16LE string (if properties & 0x04)
pub fn parse_face_name(input: &[u8]) -> IResult<&[u8], FaceName> {
    // 속성 플래그
    let (input, properties) = le_u8(input)?;

    // 글꼴 이름 (필수)
    let (input, name) = parse_utf16le_string(input)?;

    // 대체 글꼴 (선택적: bit 0)
    let (input, substitute_type, substitute_name) = if properties & 0x01 != 0 && !input.is_empty() {
        let (input, sub_type_raw) = le_u8(input)?;
        let sub_type = SubstituteFontType::from_u8(sub_type_raw);

        // 대체 글꼴 이름
        if input.len() >= 2 {
            let (input, sub_name) = parse_utf16le_string(input)?;
            (input, sub_type, sub_name)
        } else {
            (input, sub_type, String::new())
        }
    } else {
        (input, SubstituteFontType::Unknown, String::new())
    };

    // PANOSE 정보 (선택적: bit 7)
    let (input, panose) = if properties & 0x80 != 0 && input.len() >= 10 {
        let (input, panose_bytes) = take(10usize)(input)?;
        let panose_arr: [u8; 10] = panose_bytes.try_into().unwrap();
        (input, Some(Panose::new(panose_arr)))
    } else {
        (input, None)
    };

    // 기본 글꼴 이름 (선택적: bit 2)
    let (input, default_name) = if properties & 0x04 != 0 && input.len() >= 2 {
        let (input, def_name) = parse_utf16le_string(input)?;
        (input, def_name)
    } else {
        (input, String::new())
    };

    Ok((
        input,
        FaceName {
            properties,
            name,
            substitute_type,
            substitute_name,
            panose,
            default_name,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_utf16le_string(s: &str) -> Vec<u8> {
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let mut data = Vec::new();
        data.extend_from_slice(&(utf16.len() as u16).to_le_bytes());
        for ch in utf16 {
            data.extend_from_slice(&ch.to_le_bytes());
        }
        data
    }

    #[test]
    fn test_should_parse_simple_face_name() {
        // Arrange: 속성 없음, 이름만
        let mut data = vec![0x00]; // properties: 0
        data.extend(create_utf16le_string("Arial"));

        // Act
        let result = parse_face_name(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, face) = result.unwrap();
        assert!(remaining.is_empty());
        assert_eq!(face.name, "Arial");
        assert_eq!(face.properties, 0);
        assert!(!face.has_substitute());
        assert!(!face.has_panose());
    }

    #[test]
    fn test_should_parse_korean_font_name() {
        // Arrange
        let mut data = vec![0x00];
        data.extend(create_utf16le_string("맑은 고딕"));

        // Act
        let (_, face) = parse_face_name(&data).unwrap();

        // Assert
        assert_eq!(face.name, "맑은 고딕");
    }

    #[test]
    fn test_should_parse_face_name_with_substitute() {
        // Arrange
        let mut data = vec![0x01]; // properties: has substitute
        data.extend(create_utf16le_string("Custom Font"));
        data.push(0x01); // substitute_type: TrueType
        data.extend(create_utf16le_string("Arial"));

        // Act
        let result = parse_face_name(&data);

        // Assert
        assert!(result.is_ok());
        let (_, face) = result.unwrap();
        assert!(face.has_substitute());
        assert_eq!(face.substitute_type, SubstituteFontType::TrueType);
        assert_eq!(face.substitute_name, "Arial");
    }

    #[test]
    fn test_should_parse_face_name_with_panose() {
        // Arrange
        let mut data = vec![0x80]; // properties: has PANOSE
        data.extend(create_utf16le_string("Times New Roman"));
        // PANOSE: 10 bytes
        data.extend_from_slice(&[2, 2, 6, 3, 5, 4, 5, 2, 3, 4]);

        // Act
        let result = parse_face_name(&data);

        // Assert
        assert!(result.is_ok());
        let (_, face) = result.unwrap();
        assert!(face.has_panose());
        let panose = face.panose.unwrap();
        assert_eq!(panose.family_type, 2); // Latin Text
        assert_eq!(panose.weight, 6); // Semi-bold
    }

    #[test]
    fn test_should_parse_face_name_with_default() {
        // Arrange
        let mut data = vec![0x04]; // properties: has default
        data.extend(create_utf16le_string("Fancy Font"));
        data.extend(create_utf16le_string("굴림")); // default font

        // Act
        let result = parse_face_name(&data);

        // Assert
        assert!(result.is_ok());
        let (_, face) = result.unwrap();
        assert!(face.has_default());
        assert_eq!(face.default_name, "굴림");
    }

    #[test]
    fn test_should_parse_face_name_with_all_options() {
        // Arrange
        let mut data = vec![0x85]; // properties: substitute + default + panose
        data.extend(create_utf16le_string("한컴돋움"));
        data.push(0x01); // substitute_type: TrueType
        data.extend(create_utf16le_string("맑은 고딕"));
        data.extend_from_slice(&[2, 1, 5, 3, 0, 0, 0, 0, 0, 0]); // PANOSE
        data.extend(create_utf16le_string("함초롬돋움")); // default

        // Act
        let result = parse_face_name(&data);

        // Assert
        assert!(result.is_ok());
        let (_, face) = result.unwrap();
        assert_eq!(face.name, "한컴돋움");
        assert!(face.has_substitute());
        assert_eq!(face.substitute_name, "맑은 고딕");
        assert!(face.has_panose());
        assert!(face.has_default());
        assert_eq!(face.default_name, "함초롬돋움");
    }

    #[test]
    fn test_should_handle_empty_optional_fields_gracefully() {
        // Arrange: 속성이 있지만 데이터가 부족한 경우
        let mut data = vec![0x01]; // has substitute
        data.extend(create_utf16le_string("TestFont"));
        // substitute 데이터 없음

        // Act
        let result = parse_face_name(&data);

        // Assert
        assert!(result.is_ok());
        let (_, face) = result.unwrap();
        assert_eq!(face.name, "TestFont");
        // substitute 파싱 시도하지만 데이터 없음
    }
}
