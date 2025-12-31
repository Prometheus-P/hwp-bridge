// crates/hwp-core/src/parser/docinfo/mod.rs

//! DocInfo 스트림 파서
//!
//! HWP 문서의 DocInfo 스트림을 파싱합니다.
//! DocInfo에는 글꼴, 글자 모양, 문단 모양, 테두리/배경 등의 정의가 포함됩니다.

pub mod bin_data;
pub mod border_fill;
pub mod bullet;
pub mod char_shape;
pub mod compatibility;
pub mod distribute_doc_data;
pub mod doc_data;
pub mod document_properties;
pub mod face_name;
pub mod id_mappings;
pub mod numbering;
mod para_head;
pub mod para_shape;
pub mod style;
pub mod tab_def;

pub use bin_data::parse_bin_data;
pub use border_fill::parse_border_fill;
pub use bullet::parse_bullet;
pub use char_shape::parse_char_shape;
pub use compatibility::{parse_compatible_document, parse_layout_compatibility};
pub use distribute_doc_data::parse_distribute_doc_data;
pub use doc_data::parse_doc_data;
pub use document_properties::parse_document_properties;
pub use face_name::parse_face_name;
pub use id_mappings::parse_id_mappings;
pub use numbering::parse_numbering;
pub use para_shape::parse_para_shape;
pub use style::parse_style;
pub use tab_def::parse_tab_def;

use hwp_types::{
    BinData, BorderFill, Bullet, CharShape, CompatibleDocument, DistributeDocData,
    DocInfoProperties, FaceName, HwpError, IdMappings, LayoutCompatibility, Numbering, ParaShape,
    ParameterSet, StyleRecord, TabDef,
};

use crate::parser::record::{RecordIterator, tags};

/// DocInfo 파싱 결과
#[derive(Debug, Default)]
pub struct DocInfo {
    /// 문서 속성
    pub properties: Option<DocInfoProperties>,
    /// ID 매핑 테이블
    pub id_mappings: Option<IdMappings>,
    /// 바이너리 데이터 목록 (이미지, OLE 등)
    pub bin_data: Vec<BinData>,
    /// 글꼴 목록 (언어별로 구분됨)
    pub face_names: Vec<FaceName>,
    /// 글자 모양 목록
    pub char_shapes: Vec<CharShape>,
    /// 탭 정의 목록
    pub tab_defs: Vec<TabDef>,
    /// 번호 정의 목록
    pub numberings: Vec<Numbering>,
    /// 글머리표 목록
    pub bullets: Vec<Bullet>,
    /// 문단 모양 목록
    pub para_shapes: Vec<ParaShape>,
    /// 테두리/배경 목록
    pub border_fills: Vec<BorderFill>,
    /// 스타일 정의 목록
    pub styles: Vec<StyleRecord>,
    /// 문서 데이터(파라미터 셋)
    pub doc_data: Vec<ParameterSet>,
    /// 배포용 문서 데이터
    pub distribute_doc_data: Option<DistributeDocData>,
    /// 호환 문서 정보
    pub compatible_document: Option<CompatibleDocument>,
    /// 레이아웃 호환성 정보
    pub layout_compatibility: Option<LayoutCompatibility>,
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
            tags::DOCUMENT_PROPERTIES => {
                if let Ok((_, props)) = parse_document_properties(&record.data) {
                    result.properties = Some(props);
                }
            }
            tags::ID_MAPPINGS => {
                if let Ok((_, mappings)) = parse_id_mappings(&record.data) {
                    result.id_mappings = Some(mappings);
                }
            }
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
            tags::TAB_DEF => {
                if let Ok((_, tab_def)) = parse_tab_def(&record.data) {
                    result.tab_defs.push(tab_def);
                }
            }
            tags::NUMBERING => {
                if let Ok((_, numbering)) = parse_numbering(&record.data) {
                    result.numberings.push(numbering);
                }
            }
            tags::BULLET => {
                if let Ok((_, bullet)) = parse_bullet(&record.data) {
                    result.bullets.push(bullet);
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
            tags::STYLE => {
                if let Ok((_, style)) = parse_style(&record.data) {
                    result.styles.push(style);
                }
            }
            tags::DOC_DATA => {
                if let Ok((_, sets)) = parse_doc_data(&record.data) {
                    result.doc_data.extend(sets);
                }
            }
            tags::DISTRIBUTE_DOC_DATA => {
                if let Ok((_, data)) = parse_distribute_doc_data(&record.data) {
                    result.distribute_doc_data = Some(data);
                }
            }
            tags::COMPATIBLE_DOCUMENT => {
                if let Ok((_, doc)) = parse_compatible_document(&record.data) {
                    result.compatible_document = Some(doc);
                }
            }
            tags::LAYOUT_COMPATIBILITY => {
                if let Ok((_, layout)) = parse_layout_compatibility(&record.data) {
                    result.layout_compatibility = Some(layout);
                }
            }
            _ => {
                // 다른 태그는 무시
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

        let dword: u32 = (tag_id as u32) | ((level as u32) << 10) | (size << 20);

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
