// crates/hwp-types/src/bindata.rs

//! 바이너리 데이터 타입
//!
//! 임베디드 이미지, OLE 객체 등의 바이너리 데이터를 표현합니다.

use serde::{Deserialize, Serialize};

/// 바이너리 데이터 저장 타입
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinDataType {
    /// 링크 (외부 파일 참조)
    Link = 0,
    /// 내장 (스트림에 저장)
    #[default]
    Embedding = 1,
    /// 스토리지 (OLE 객체)
    Storage = 2,
}

impl BinDataType {
    /// 숫자 값에서 BinDataType 생성
    pub fn from_value(value: u16) -> Self {
        match value {
            0 => BinDataType::Link,
            1 => BinDataType::Embedding,
            2 => BinDataType::Storage,
            _ => BinDataType::Embedding, // default fallback
        }
    }

    /// BinDataType을 숫자 값으로 변환
    pub fn to_value(&self) -> u16 {
        match self {
            BinDataType::Link => 0,
            BinDataType::Embedding => 1,
            BinDataType::Storage => 2,
        }
    }

    /// 링크 타입인지 확인
    pub fn is_link(&self) -> bool {
        matches!(self, BinDataType::Link)
    }

    /// 내장 타입인지 확인
    pub fn is_embedding(&self) -> bool {
        matches!(self, BinDataType::Embedding)
    }

    /// 스토리지(OLE) 타입인지 확인
    pub fn is_storage(&self) -> bool {
        matches!(self, BinDataType::Storage)
    }
}

/// 바이너리 데이터 (이미지, OLE 객체 등)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BinData {
    /// 데이터 ID
    pub id: u16,
    /// 속성 플래그
    /// Bit 0-1: 저장 타입 (0=Link, 1=Embedding, 2=Storage)
    /// Bit 2: 압축 여부
    /// Bit 3: 경로로 접근
    pub properties: u16,
    /// 저장 타입
    pub storage_type: BinDataType,
    /// 절대 경로 (Link 타입일 때)
    pub abs_path: String,
    /// 상대 경로 (Link 타입일 때)
    pub rel_path: String,
    /// 파일 확장자 (예: "png", "jpg")
    pub extension: String,
    /// 원본 데이터
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

impl BinData {
    /// 새 BinData 생성
    pub fn new(id: u16, storage_type: BinDataType) -> Self {
        Self {
            id,
            storage_type,
            ..Default::default()
        }
    }

    /// 빈 BinData 생성
    pub fn empty() -> Self {
        Self::default()
    }

    /// 확장자 설정
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.extension = ext.into();
        self
    }

    /// 데이터 설정
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// 데이터 크기 반환
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    /// 데이터가 비어있는지 확인
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 이미지 데이터인지 확인 (확장자 기반)
    pub fn is_image(&self) -> bool {
        let ext = self.extension.to_lowercase();
        matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tif" | "tiff" | "wmf" | "emf"
        )
    }

    /// OLE 객체인지 확인
    pub fn is_ole(&self) -> bool {
        self.extension.to_lowercase() == "ole"
    }

    /// 압축된 데이터인지 확인 (bit 2)
    pub fn is_compressed(&self) -> bool {
        self.properties & 0x04 != 0
    }

    /// 경로로 접근하는지 확인 (bit 3)
    pub fn is_access_by_path(&self) -> bool {
        self.properties & 0x08 != 0
    }

    /// 속성 플래그에서 저장 타입 추출
    pub fn type_from_properties(&self) -> BinDataType {
        BinDataType::from_value(self.properties & 0x03)
    }

    /// 절대 경로 설정
    pub fn with_abs_path(mut self, path: impl Into<String>) -> Self {
        self.abs_path = path.into();
        self
    }

    /// 상대 경로 설정
    pub fn with_rel_path(mut self, path: impl Into<String>) -> Self {
        self.rel_path = path.into();
        self
    }

    /// 속성 플래그 설정
    pub fn with_properties(mut self, props: u16) -> Self {
        self.properties = props;
        self.storage_type = BinDataType::from_value(props & 0x03);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // BinDataType Tests (US4)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_distinguish_bindata_types_when_compared() {
        // Arrange
        let link = BinDataType::Link;
        let embedding = BinDataType::Embedding;
        let storage = BinDataType::Storage;

        // Act & Assert
        assert_ne!(link, embedding);
        assert_ne!(embedding, storage);
        assert_ne!(storage, link);
    }

    #[test]
    fn test_should_return_default_embedding_when_default() {
        // Arrange & Act
        let default_type = BinDataType::default();

        // Assert
        assert_eq!(default_type, BinDataType::Embedding);
    }

    #[test]
    fn test_should_create_from_value_when_valid() {
        // Arrange & Act & Assert
        assert_eq!(BinDataType::from_value(0), BinDataType::Link);
        assert_eq!(BinDataType::from_value(1), BinDataType::Embedding);
        assert_eq!(BinDataType::from_value(2), BinDataType::Storage);
    }

    #[test]
    fn test_should_fallback_to_embedding_when_invalid_value() {
        // Arrange & Act
        let invalid = BinDataType::from_value(99);

        // Assert
        assert_eq!(invalid, BinDataType::Embedding);
    }

    #[test]
    fn test_should_convert_to_value_correctly() {
        // Arrange & Act & Assert
        assert_eq!(BinDataType::Link.to_value(), 0);
        assert_eq!(BinDataType::Embedding.to_value(), 1);
        assert_eq!(BinDataType::Storage.to_value(), 2);
    }

    #[test]
    fn test_should_check_type_with_helper_methods() {
        // Arrange
        let link = BinDataType::Link;
        let embedding = BinDataType::Embedding;
        let storage = BinDataType::Storage;

        // Act & Assert
        assert!(link.is_link());
        assert!(!link.is_embedding());
        assert!(!link.is_storage());

        assert!(!embedding.is_link());
        assert!(embedding.is_embedding());
        assert!(!embedding.is_storage());

        assert!(!storage.is_link());
        assert!(!storage.is_embedding());
        assert!(storage.is_storage());
    }

    // ═══════════════════════════════════════════════════════════════
    // BinData Tests (US4)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_store_bindata_with_extension_when_created() {
        // Arrange & Act
        let bin_data = BinData::new(1, BinDataType::Embedding).with_extension("png");

        // Assert
        assert_eq!(bin_data.id, 1);
        assert_eq!(bin_data.storage_type, BinDataType::Embedding);
        assert_eq!(bin_data.extension, "png");
    }

    #[test]
    fn test_should_store_data_when_with_data() {
        // Arrange
        let image_bytes = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes

        // Act
        let bin_data = BinData::new(1, BinDataType::Embedding)
            .with_extension("png")
            .with_data(image_bytes.clone());

        // Assert
        assert_eq!(bin_data.data, image_bytes);
        assert_eq!(bin_data.data_size(), 4);
        assert!(!bin_data.is_empty());
    }

    #[test]
    fn test_should_return_empty_when_no_data() {
        // Arrange & Act
        let bin_data = BinData::empty();

        // Assert
        assert!(bin_data.is_empty());
        assert_eq!(bin_data.data_size(), 0);
    }

    #[test]
    fn test_should_detect_image_by_extension() {
        // Arrange
        let png = BinData::empty().with_extension("png");
        let jpg = BinData::empty().with_extension("JPG");
        let gif = BinData::empty().with_extension("gif");
        let wmf = BinData::empty().with_extension("wmf");
        let ole = BinData::empty().with_extension("ole");
        let doc = BinData::empty().with_extension("doc");

        // Act & Assert
        assert!(png.is_image());
        assert!(jpg.is_image());
        assert!(gif.is_image());
        assert!(wmf.is_image());
        assert!(!ole.is_image());
        assert!(!doc.is_image());
    }

    #[test]
    fn test_should_chain_builder_methods() {
        // Arrange
        let data = vec![0x01, 0x02, 0x03];

        // Act
        let bin_data = BinData::new(42, BinDataType::Storage)
            .with_extension("doc")
            .with_data(data.clone());

        // Assert
        assert_eq!(bin_data.id, 42);
        assert_eq!(bin_data.storage_type, BinDataType::Storage);
        assert_eq!(bin_data.extension, "doc");
        assert_eq!(bin_data.data, data);
    }

    #[test]
    fn test_should_detect_compressed_when_bit_set() {
        // Arrange
        let compressed = BinData::empty().with_properties(0x05); // bit 0 + bit 2

        // Assert
        assert!(compressed.is_compressed());
        assert_eq!(compressed.storage_type, BinDataType::Embedding);
    }

    #[test]
    fn test_should_detect_access_by_path_when_bit_set() {
        // Arrange
        let by_path = BinData::empty().with_properties(0x08); // bit 3

        // Assert
        assert!(by_path.is_access_by_path());
    }

    #[test]
    fn test_should_extract_type_from_properties() {
        // Arrange & Act & Assert
        assert_eq!(
            BinData::empty()
                .with_properties(0x00)
                .type_from_properties(),
            BinDataType::Link
        );
        assert_eq!(
            BinData::empty()
                .with_properties(0x01)
                .type_from_properties(),
            BinDataType::Embedding
        );
        assert_eq!(
            BinData::empty()
                .with_properties(0x02)
                .type_from_properties(),
            BinDataType::Storage
        );
        // With other bits set
        assert_eq!(
            BinData::empty()
                .with_properties(0x0D)
                .type_from_properties(),
            BinDataType::Embedding
        );
    }

    #[test]
    fn test_should_detect_ole_when_extension_ole() {
        // Arrange
        let ole = BinData::empty().with_extension("OLE");
        let not_ole = BinData::empty().with_extension("png");

        // Assert
        assert!(ole.is_ole());
        assert!(!not_ole.is_ole());
    }

    #[test]
    fn test_should_set_paths_with_builder() {
        // Arrange & Act
        let bin_data = BinData::empty()
            .with_abs_path("C:\\Documents\\image.png")
            .with_rel_path("image.png");

        // Assert
        assert_eq!(bin_data.abs_path, "C:\\Documents\\image.png");
        assert_eq!(bin_data.rel_path, "image.png");
    }
}
