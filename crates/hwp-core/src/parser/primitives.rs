// crates/hwp-core/src/parser/primitives.rs

//! nom 기반 기본 파서 프리미티브
//!
//! HWP 바이너리 포맷 파싱에 필요한 기본 파서 함수들을 정의합니다.

use nom::{
    IResult, Parser,
    bytes::complete::take,
    multi::count,
    number::complete::{le_i8, le_i16, le_i32, le_u8, le_u16, le_u32},
};

/// 7개 언어별 u16 배열 파싱
pub fn parse_u16_array_7(input: &[u8]) -> IResult<&[u8], [u16; 7]> {
    let (input, vec) = count(le_u16, 7).parse(input)?;
    let arr: [u16; 7] = vec.try_into().unwrap();
    Ok((input, arr))
}

/// 7개 언어별 u8 배열 파싱
pub fn parse_u8_array_7(input: &[u8]) -> IResult<&[u8], [u8; 7]> {
    let (input, vec) = count(le_u8, 7).parse(input)?;
    let arr: [u8; 7] = vec.try_into().unwrap();
    Ok((input, arr))
}

/// 7개 언어별 i8 배열 파싱
pub fn parse_i8_array_7(input: &[u8]) -> IResult<&[u8], [i8; 7]> {
    let (input, vec) = count(le_i8, 7).parse(input)?;
    let arr: [i8; 7] = vec.try_into().unwrap();
    Ok((input, arr))
}

/// UTF-16LE 문자열 파싱 (길이 prefix 포함)
/// 형식: u16(문자 개수) + UTF-16LE 바이트
pub fn parse_utf16le_string(input: &[u8]) -> IResult<&[u8], String> {
    let (input, char_count) = le_u16(input)?;
    let byte_count = char_count as usize * 2;
    let (input, bytes) = take(byte_count)(input)?;

    // UTF-16LE → String 변환
    let utf16_chars: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    let string = String::from_utf16_lossy(&utf16_chars);
    Ok((input, string))
}

/// 고정 길이 UTF-16LE 문자열 파싱 (길이 prefix 없음)
pub fn parse_utf16le_fixed(input: &[u8], char_count: usize) -> IResult<&[u8], String> {
    let byte_count = char_count * 2;
    let (input, bytes) = take(byte_count)(input)?;

    let utf16_chars: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    let string = String::from_utf16_lossy(&utf16_chars);
    Ok((input, string))
}

/// COLORREF 파싱 (4바이트, 실제로는 3바이트 사용: 0x00BBGGRR)
pub fn parse_colorref(input: &[u8]) -> IResult<&[u8], u32> {
    le_u32(input)
}

/// HWPUNIT 파싱 (i32, 1/7200 inch 단위)
pub fn parse_hwpunit(input: &[u8]) -> IResult<&[u8], i32> {
    le_i32(input)
}

/// HWPUNIT16 파싱 (i16, 1/7200 inch 단위)
pub fn parse_hwpunit16(input: &[u8]) -> IResult<&[u8], i16> {
    le_i16(input)
}

/// N바이트 스킵
pub fn skip_bytes(input: &[u8], n: usize) -> IResult<&[u8], ()> {
    let (input, _) = take(n)(input)?;
    Ok((input, ()))
}

/// 조건부 파서 - 남은 바이트가 충분할 때만 파싱
pub fn parse_optional<'a, O, F>(
    input: &'a [u8],
    min_bytes: usize,
    parser: F,
) -> IResult<&'a [u8], Option<O>>
where
    F: FnOnce(&'a [u8]) -> IResult<&'a [u8], O>,
{
    if input.len() >= min_bytes {
        let (input, value) = parser(input)?;
        Ok((input, Some(value)))
    } else {
        Ok((input, None))
    }
}

/// bool 파싱 (1바이트)
pub fn parse_bool(input: &[u8]) -> IResult<&[u8], bool> {
    le_u8.map(|v| v != 0).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_u16_array_7() {
        // Arrange
        let data: Vec<u8> = (0u16..7).flat_map(|i| i.to_le_bytes()).collect();

        // Act
        let (remaining, arr) = parse_u16_array_7(&data).unwrap();

        // Assert
        assert!(remaining.is_empty());
        assert_eq!(arr, [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_should_parse_u8_array_7() {
        // Arrange
        let data = [10u8, 20, 30, 40, 50, 60, 70];

        // Act
        let (remaining, arr) = parse_u8_array_7(&data).unwrap();

        // Assert
        assert!(remaining.is_empty());
        assert_eq!(arr, [10, 20, 30, 40, 50, 60, 70]);
    }

    #[test]
    fn test_should_parse_i8_array_7() {
        // Arrange
        let data = [0i8, -1, 2, -3, 4, -5, 6].map(|i| i as u8);

        // Act
        let (remaining, arr) = parse_i8_array_7(&data).unwrap();

        // Assert
        assert!(remaining.is_empty());
        assert_eq!(arr, [0, -1, 2, -3, 4, -5, 6]);
    }

    #[test]
    fn test_should_parse_utf16le_string() {
        // Arrange: "ABC" in UTF-16LE with length prefix
        let mut data = vec![0x03, 0x00]; // length = 3
        data.extend_from_slice(&[0x41, 0x00]); // 'A'
        data.extend_from_slice(&[0x42, 0x00]); // 'B'
        data.extend_from_slice(&[0x43, 0x00]); // 'C'

        // Act
        let (remaining, string) = parse_utf16le_string(&data).unwrap();

        // Assert
        assert!(remaining.is_empty());
        assert_eq!(string, "ABC");
    }

    #[test]
    fn test_should_parse_korean_utf16le_string() {
        // Arrange: "한글" in UTF-16LE with length prefix
        let mut data = vec![0x02, 0x00]; // length = 2
        data.extend_from_slice(&[0x5C, 0xD5]); // '한' (U+D55C)
        data.extend_from_slice(&[0x00, 0xAE]); // '글' (U+AE00)

        // Act
        let (remaining, string) = parse_utf16le_string(&data).unwrap();

        // Assert
        assert!(remaining.is_empty());
        assert_eq!(string, "한글");
    }

    #[test]
    fn test_should_parse_empty_utf16le_string() {
        // Arrange
        let data = vec![0x00, 0x00]; // length = 0

        // Act
        let (remaining, string) = parse_utf16le_string(&data).unwrap();

        // Assert
        assert!(remaining.is_empty());
        assert_eq!(string, "");
    }

    #[test]
    fn test_should_parse_colorref() {
        // Arrange: Blue color (0x00FF0000 = BGR format)
        let data = [0x00, 0x00, 0xFF, 0x00]; // Blue in COLORREF

        // Act
        let (remaining, color) = parse_colorref(&data).unwrap();

        // Assert
        assert!(remaining.is_empty());
        assert_eq!(color, 0x00FF0000);
    }

    #[test]
    fn test_should_skip_bytes() {
        // Arrange
        let data = [1, 2, 3, 4, 5];

        // Act
        let (remaining, _) = skip_bytes(&data, 3).unwrap();

        // Assert
        assert_eq!(remaining, &[4, 5]);
    }

    #[test]
    fn test_should_parse_optional_when_enough_bytes() {
        // Arrange
        let data = [0x42, 0x00, 0x00, 0x00];

        // Act
        let (remaining, value) = parse_optional(&data, 4, le_u32).unwrap();

        // Assert
        assert!(remaining.is_empty());
        assert_eq!(value, Some(0x42));
    }

    #[test]
    fn test_should_return_none_when_not_enough_bytes() {
        // Arrange
        let data = [0x42, 0x00];

        // Act
        let (remaining, value) = parse_optional(&data, 4, le_u32).unwrap();

        // Assert
        assert_eq!(remaining, &[0x42, 0x00]);
        assert_eq!(value, None);
    }

    #[test]
    fn test_should_parse_bool() {
        // Arrange & Act & Assert
        let (_, val) = parse_bool(&[0x00]).unwrap();
        assert!(!val);

        let (_, val) = parse_bool(&[0x01]).unwrap();
        assert!(val);

        let (_, val) = parse_bool(&[0xFF]).unwrap();
        assert!(val);
    }
}
