// crates/hwp-core/src/parser/docinfo/mod.rs

//! DocInfo 스트림 파서
//!
//! HWP 문서의 DocInfo 스트림을 파싱합니다.
//! DocInfo에는 글꼴, 글자 모양, 문단 모양, 테두리/배경 등의 정의가 포함됩니다.

pub mod bin_data;
pub mod border_fill;
pub mod char_shape;
pub mod face_name;
pub mod para_shape;

pub use bin_data::parse_bin_data;
pub use border_fill::parse_border_fill;
pub use char_shape::parse_char_shape;
pub use face_name::parse_face_name;
pub use para_shape::parse_para_shape;

use hwp_types::{BinData, BorderFill, CharShape, FaceName, HwpError, ParaShape};

use crate::parser::record::{RecordIterator, tags};

/// DocInfo 파싱 결과
#[derive(Debug, Default)]
pub struct DocInfo {
    /// 바이너리 데이터 목록 (이미지, OLE 등)
    pub bin_data: Vec<BinData>,
    /// 글꼴 목록 (언어별로 구분됨)
    pub face_names: Vec<FaceName>,
    /// 글자 모양 목록
    pub char_shapes: Vec<CharShape>,
    /// 문단 모양 목록
    pub para_shapes: Vec<ParaShape>,
    /// 테두리/배경 목록
    pub border_fills: Vec<BorderFill>,
}

impl DocInfo {
    /// 새 빈 DocInfo 생성
    pub fn new() -> Self {
        Self::default()
    }

    /// 글꼴 ID로 FaceName 조회
    pub fn get_face_name(&self, id: usize) -> Option<&FaceName> {
        self.face_names.get(id)
    }

    /// 글자 모양 ID로 CharShape 조회
    pub fn get_char_shape(&self, id: usize) -> Option<&CharShape> {
        self.char_shapes.get(id)
    }

    /// 문단 모양 ID로 ParaShape 조회
    pub fn get_para_shape(&self, id: usize) -> Option<&ParaShape> {
        self.para_shapes.get(id)
    }

    /// 테두리/배경 ID로 BorderFill 조회
    pub fn get_border_fill(&self, id: usize) -> Option<&BorderFill> {
        self.border_fills.get(id)
    }

    /// BinData ID로 조회
    pub fn get_bin_data(&self, id: usize) -> Option<&BinData> {
        self.bin_data.get(id)
    }
}

/// DocInfo 스트림 파싱
///
/// 레코드 시퀀스를 순회하며 각 타입별 레코드를 파싱합니다.
pub fn parse_docinfo(data: &[u8]) -> Result<DocInfo, HwpError> {
    let mut result = DocInfo::new();
    let mut bin_data_id: u16 = 0;

    for record_result in RecordIterator::new(data) {
        let record = record_result?;

        match record.header.tag_id {
            tags::BIN_DATA => {
                if let Ok((_, bin_data)) = parse_bin_data(&record.data, bin_data_id) {
                    result.bin_data.push(bin_data);
                    bin_data_id += 1;
                }
            }
            tags::FACE_NAME => {
                if let Ok((_, face_name)) = parse_face_name(&record.data) {
                    result.face_names.push(face_name);
                }
            }
            tags::CHAR_SHAPE => {
                if let Ok((_, char_shape)) = parse_char_shape(&record.data) {
                    result.char_shapes.push(char_shape);
                }
            }
            tags::PARA_SHAPE => {
                if let Ok((_, para_shape)) = parse_para_shape(&record.data) {
                    result.para_shapes.push(para_shape);
                }
            }
            tags::BORDER_FILL => {
                if let Ok((_, border_fill)) = parse_border_fill(&record.data) {
                    result.border_fills.push(border_fill);
                }
            }
            _ => {
                // 다른 태그는 무시 (ID_MAPPINGS, STYLE 등)
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 레코드 헤더 생성 헬퍼
    fn create_record(tag_id: u16, level: u16, data: &[u8]) -> Vec<u8> {
        let size = data.len() as u32;
        assert!(size < 4095, "Size must be < 4095 for this test helper");

        let dword: u32 = (tag_id as u32) | ((level as u32) << 10) | ((size as u32) << 20);

        let mut result = Vec::new();
        result.extend_from_slice(&dword.to_le_bytes());
        result.extend_from_slice(data);
        result
    }

    fn create_face_name_record(name: &str) -> Vec<u8> {
        let mut data = vec![0x00]; // properties
        let utf16: Vec<u16> = name.encode_utf16().collect();
        data.extend_from_slice(&(utf16.len() as u16).to_le_bytes());
        for ch in utf16 {
            data.extend_from_slice(&ch.to_le_bytes());
        }
        create_record(tags::FACE_NAME, 0, &data)
    }

    fn create_char_shape_record() -> Vec<u8> {
        let mut data = Vec::new();

        // font_ids: 7 * u16
        for i in 0u16..7 {
            data.extend_from_slice(&i.to_le_bytes());
        }
        // font_scales, char_spacing, relative_sizes, char_offsets: 7 * u8 each
        data.extend_from_slice(&[100u8; 7]); // scales
        data.extend_from_slice(&[0u8; 7]); // spacing
        data.extend_from_slice(&[100u8; 7]); // sizes
        data.extend_from_slice(&[0u8; 7]); // offsets

        // base_size, attr, shadow gaps
        data.extend_from_slice(&1000i32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.push(0); // shadow_gap_x
        data.push(0); // shadow_gap_y

        // colors
        data.extend_from_slice(&0x000000u32.to_le_bytes()); // text
        data.extend_from_slice(&0x000000u32.to_le_bytes()); // underline
        data.extend_from_slice(&0xFFFFFFu32.to_le_bytes()); // shade
        data.extend_from_slice(&0x808080u32.to_le_bytes()); // shadow

        create_record(tags::CHAR_SHAPE, 0, &data)
    }

    #[test]
    fn test_should_parse_empty_docinfo() {
        // Arrange
        let data: &[u8] = &[];

        // Act
        let result = parse_docinfo(data);

        // Assert
        assert!(result.is_ok());
        let docinfo = result.unwrap();
        assert!(docinfo.face_names.is_empty());
        assert!(docinfo.char_shapes.is_empty());
    }

    #[test]
    fn test_should_parse_docinfo_with_face_names() {
        // Arrange
        let mut data = Vec::new();
        data.extend(create_face_name_record("Arial"));
        data.extend(create_face_name_record("맑은 고딕"));

        // Act
        let result = parse_docinfo(&data);

        // Assert
        assert!(result.is_ok());
        let docinfo = result.unwrap();
        assert_eq!(docinfo.face_names.len(), 2);
        assert_eq!(docinfo.face_names[0].name, "Arial");
        assert_eq!(docinfo.face_names[1].name, "맑은 고딕");
    }

    #[test]
    fn test_should_parse_docinfo_with_char_shapes() {
        // Arrange
        let mut data = Vec::new();
        data.extend(create_char_shape_record());
        data.extend(create_char_shape_record());

        // Act
        let result = parse_docinfo(&data);

        // Assert
        assert!(result.is_ok());
        let docinfo = result.unwrap();
        assert_eq!(docinfo.char_shapes.len(), 2);
        assert_eq!(docinfo.char_shapes[0].base_size, 1000);
    }

    #[test]
    fn test_should_skip_unknown_tags() {
        // Arrange
        let mut data = Vec::new();
        data.extend(create_face_name_record("TestFont"));
        // 알 수 없는 태그 (0xFF)
        data.extend(create_record(0xFF, 0, &[1, 2, 3, 4]));
        data.extend(create_char_shape_record());

        // Act
        let result = parse_docinfo(&data);

        // Assert
        assert!(result.is_ok());
        let docinfo = result.unwrap();
        assert_eq!(docinfo.face_names.len(), 1);
        assert_eq!(docinfo.char_shapes.len(), 1);
    }

    #[test]
    fn test_should_get_items_by_id() {
        // Arrange
        let mut data = Vec::new();
        data.extend(create_face_name_record("Font0"));
        data.extend(create_face_name_record("Font1"));
        data.extend(create_char_shape_record());

        let docinfo = parse_docinfo(&data).unwrap();

        // Act & Assert
        assert!(docinfo.get_face_name(0).is_some());
        assert_eq!(docinfo.get_face_name(0).unwrap().name, "Font0");
        assert!(docinfo.get_face_name(1).is_some());
        assert!(docinfo.get_face_name(2).is_none());

        assert!(docinfo.get_char_shape(0).is_some());
        assert!(docinfo.get_char_shape(1).is_none());

        assert!(docinfo.get_para_shape(0).is_none()); // 파싱하지 않음
        assert!(docinfo.get_border_fill(0).is_none());
    }
}
