# DATA_MODEL.md - HwpBridge Data Models

> **Version:** 1.0.0
> **Author:** @Architect
> **Status:** Draft
> **Last Updated:** 2025-12-09

---

## 1. Overview

이 문서는 HwpBridge 프로젝트에서 사용하는 데이터 모델을 정의합니다.

---

## 2. Core Types (hwp-types)

### 2.1 Error Types

```rust
/// HWP 처리 중 발생하는 모든 에러
#[derive(Error, Debug)]
pub enum HwpError {
    /// 파일 I/O 에러
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    /// OLE 컨테이너 처리 에러
    #[error("OLE Storage Error: {0}")]
    OleError(String),

    /// 잘못된 HWP 시그니처
    #[error("Invalid HWP Signature")]
    InvalidSignature,

    /// 지원하지 않는 HWP 버전
    #[error("Unsupported HWP Version: {0}")]
    UnsupportedVersion(String),

    /// 암호화된 문서
    #[error("Encrypted Document (Cannot Process)")]
    Encrypted,

    /// 배포용 문서
    #[error("Distribution Document (Read-Only/Encrypted Body)")]
    DistributionOnly,

    /// 파싱 에러
    #[error("Parse Error: {0}")]
    ParseError(String),

    /// Google Drive API 에러
    #[error("Google Drive API Error: {0}")]
    GoogleDriveError(String),
}
```

---

### 2.2 Version

```rust
/// HWP 파일 버전 (예: 5.1.0.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HwpVersion {
    /// 메이저 버전 (5 = HWP 5.x)
    pub major: u8,
    /// 마이너 버전
    pub minor: u8,
    /// 빌드 번호
    pub build: u8,
    /// 리비전
    pub revision: u8,
}

impl HwpVersion {
    /// 바이트 배열에서 버전 파싱 (Little-Endian)
    /// 저장 순서: [revision, build, minor, major]
    pub fn from_bytes(bytes: [u8; 4]) -> Self;

    /// HWP 5.0 이상 지원 여부
    pub fn is_supported(&self) -> bool;
}
```

---

### 2.3 Document Properties

```rust
/// 문서 속성 비트 플래그 (FileHeader offset 36, 4 bytes)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentProperties(u32);

impl DocumentProperties {
    /// Bit 0: 본문 압축 여부
    pub fn is_compressed(&self) -> bool;

    /// Bit 1: 암호화 여부 (비밀번호 설정)
    pub fn is_encrypted(&self) -> bool;

    /// Bit 2: 배포용 문서 (본문 암호화, 편집 불가)
    pub fn is_distribution(&self) -> bool;

    /// Bit 3: 스크립트 저장 여부
    pub fn has_script(&self) -> bool;

    /// Bit 4: DRM 보안 적용
    pub fn has_drm(&self) -> bool;

    /// Bit 5: XMLTemplate 스토리지 존재
    pub fn has_xml_template(&self) -> bool;

    /// Bit 6: 문서 이력 관리
    pub fn has_history(&self) -> bool;

    /// Bit 7: 전자 서명 정보 존재
    pub fn has_signature(&self) -> bool;

    /// Bit 8: 공인 인증서 암호화
    pub fn has_cert_encryption(&self) -> bool;

    /// Bit 11: CCL (Creative Commons License) 문서
    pub fn is_ccl(&self) -> bool;

    /// Bit 12: 모바일 최적화
    pub fn is_mobile_optimized(&self) -> bool;

    /// Bit 14: 변경 추적 활성화
    pub fn has_track_changes(&self) -> bool;

    /// Bit 15: 공공누리(KOGL) 저작권
    pub fn is_kogl(&self) -> bool;
}
```

---

### 2.4 FileHeader

```rust
/// HWP FileHeader (256 bytes)
/// OLE 스트림 경로: /FileHeader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHeader {
    /// 파일 버전
    pub version: HwpVersion,
    /// 문서 속성 플래그
    pub properties: DocumentProperties,
}

impl FileHeader {
    /// Fail-Fast 검증
    /// - 버전 체크 (5.0 미만 거부)
    /// - 암호화 문서 거부
    /// - 배포용 문서 거부
    pub fn validate(&self) -> Result<(), HwpError>;
}
```

---

### 2.5 Document Metadata

```rust
/// 문서 메타데이터 (사용자 표시용)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    /// 문서 제목
    pub title: String,
    /// 작성자
    pub author: String,
    /// 생성 일시
    pub created_at: String,
    /// 암호화 여부
    pub is_encrypted: bool,
    /// 배포용 문서 여부
    pub is_distribution: bool,
}
```

---

### 2.6 HwpDocument

```rust
/// 파싱 결과물 최상위 구조체
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HwpDocument {
    /// 문서 메타데이터
    pub metadata: DocumentMetadata,
    /// 본문 내용 (임시: String, 추후 Vec<Section>)
    pub content: String,
}
```

---

### 2.7 Convert Options

```rust
/// 변환 옵션
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// 이미지 추출 여부
    pub extract_images: bool,
    /// 주석 포함 여부
    pub include_comments: bool,
}
```

---

## 3. Parser Types (hwp-core)

### 3.1 Record Header

```rust
/// HWP 레코드 헤더 (4 bytes)
/// 모든 레코드는 이 헤더로 시작
#[derive(Debug, Clone, Copy)]
pub struct RecordHeader {
    /// 태그 ID (10 bits, 0-1023)
    pub tag_id: u16,
    /// 레벨 (10 bits, 중첩 깊이)
    pub level: u16,
    /// 데이터 크기 (12 bits, 최대 4095)
    /// 4095인 경우 다음 4바이트에 실제 크기
    pub size: u32,
}

impl RecordHeader {
    /// 바이트에서 파싱
    pub fn parse(data: &[u8]) -> Result<(Self, usize), HwpError>;
}
```

**Binary Layout:**

```
┌─────────────────────────────────────────────┐
│            Record Header (4 bytes)          │
├────────────┬────────────┬───────────────────┤
│  Tag ID    │   Level    │      Size         │
│  (10 bits) │  (10 bits) │    (12 bits)      │
├────────────┴────────────┴───────────────────┤
│  Bit: 0-9     10-19        20-31            │
└─────────────────────────────────────────────┘
```

---

### 3.2 HWP Tags (Constants)

```rust
/// HWP 레코드 태그 상수
pub mod tags {
    // === DocInfo Tags (0x00 - 0x1F) ===
    pub const DOCUMENT_PROPERTIES: u16 = 0x00;
    pub const ID_MAPPINGS: u16 = 0x01;
    pub const BIN_DATA: u16 = 0x02;
    pub const FACE_NAME: u16 = 0x03;
    pub const BORDER_FILL: u16 = 0x04;
    pub const CHAR_SHAPE: u16 = 0x07;
    pub const TAB_DEF: u16 = 0x08;
    pub const PARA_SHAPE: u16 = 0x09;
    pub const STYLE: u16 = 0x0A;
    pub const MEMO_SHAPE: u16 = 0x0B;

    // === BodyText Tags (0x40 - 0x7F) ===
    pub const PARA_HEADER: u16 = 0x42;
    pub const PARA_TEXT: u16 = 0x43;
    pub const PARA_CHAR_SHAPE: u16 = 0x44;
    pub const PARA_LINE_SEG: u16 = 0x45;
    pub const PARA_RANGE_TAG: u16 = 0x46;
    pub const CTRL_HEADER: u16 = 0x47;

    // === Table Tags ===
    pub const TABLE: u16 = 0x4D;
    pub const TABLE_CELL: u16 = 0x4E;

    // === Shape Tags ===
    pub const SHAPE_COMPONENT: u16 = 0x51;
    pub const SHAPE_COMPONENT_LINE: u16 = 0x52;
    pub const SHAPE_COMPONENT_RECTANGLE: u16 = 0x53;
    pub const SHAPE_COMPONENT_ELLIPSE: u16 = 0x54;
    pub const SHAPE_COMPONENT_PICTURE: u16 = 0x57;
}
```

---

### 3.3 CharShape (글자 모양)

```rust
/// 글자 모양 정의 (DocInfo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharShape {
    /// 글꼴 ID 참조 (언어별)
    pub font_ids: [u16; 7],
    /// 글꼴 비율 (%)
    pub font_scales: [u8; 7],
    /// 자간
    pub char_spacing: [i8; 7],
    /// 상대 크기 (%)
    pub relative_sizes: [u8; 7],
    /// 기준 크기 (1/100 pt)
    pub base_size: u32,
    /// 글자 색상 (COLORREF)
    pub text_color: u32,
    /// 밑줄 색상
    pub underline_color: u32,
    /// 음영 색상
    pub shade_color: u32,
    /// 그림자 색상
    pub shadow_color: u32,
    /// 속성 플래그 (Bold, Italic, Underline 등)
    pub attr: CharShapeAttr,
}

/// 글자 속성 플래그
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CharShapeAttr(u32);

impl CharShapeAttr {
    pub fn is_bold(&self) -> bool;
    pub fn is_italic(&self) -> bool;
    pub fn underline_type(&self) -> UnderlineType;
    pub fn is_strikethrough(&self) -> bool;
    pub fn is_superscript(&self) -> bool;
    pub fn is_subscript(&self) -> bool;
}
```

---

### 3.4 ParaShape (문단 모양)

```rust
/// 문단 모양 정의 (DocInfo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParaShape {
    /// 속성 플래그
    pub attr: ParaShapeAttr,
    /// 왼쪽 여백 (HWPUNIT)
    pub margin_left: i32,
    /// 오른쪽 여백
    pub margin_right: i32,
    /// 들여쓰기
    pub indent: i32,
    /// 문단 위 간격
    pub margin_top: i32,
    /// 문단 아래 간격
    pub margin_bottom: i32,
    /// 줄 간격 (%)
    pub line_spacing: i32,
    /// 탭 정의 ID
    pub tab_def_id: u16,
    /// 번호/글머리 ID
    pub para_head_id: u16,
    /// 테두리/배경 ID
    pub border_fill_id: u16,
}

/// 문단 정렬
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Alignment {
    Left = 0,
    Right = 1,
    Center = 2,
    Justify = 3,
    Distribute = 4,
}
```

---

### 3.5 Paragraph

```rust
/// 문단 (BodyText)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    /// 문단 텍스트
    pub text: String,
    /// 문단 모양 ID 참조
    pub para_shape_id: u16,
    /// 글자 위치별 CharShape ID
    pub char_shapes: Vec<(u32, u16)>,  // (position, shape_id)
    /// 인라인 컨트롤 (표, 그림 등)
    pub controls: Vec<Control>,
}
```

---

### 3.6 Table

```rust
/// 표 컨트롤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// 행 수
    pub rows: u16,
    /// 열 수
    pub cols: u16,
    /// 셀 목록
    pub cells: Vec<TableCell>,
    /// 테두리/배경 ID
    pub border_fill_id: u16,
}

/// 표 셀
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// 열 주소
    pub col: u16,
    /// 행 주소
    pub row: u16,
    /// 열 병합 수
    pub col_span: u16,
    /// 행 병합 수
    pub row_span: u16,
    /// 셀 너비
    pub width: u32,
    /// 셀 높이
    pub height: u32,
    /// 셀 내용 (문단 목록)
    pub paragraphs: Vec<Paragraph>,
}
```

---

### 3.7 BinData (이미지)

```rust
/// 바이너리 데이터 (이미지, OLE 등)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinData {
    /// 데이터 ID
    pub id: u16,
    /// 저장 타입
    pub storage_type: BinDataType,
    /// 파일 확장자
    pub extension: String,
    /// 원본 데이터
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BinDataType {
    /// 링크 (외부 파일)
    Link = 0,
    /// 내장 (스토리지에 저장)
    Embedding = 1,
    /// 스토리지 (OLE)
    Storage = 2,
}
```

---

## 4. Converter Output Types

### 4.1 HtmlOutput

```rust
/// HTML 변환 결과
#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlOutput {
    /// HTML 문자열
    pub html: String,
    /// 인라인 스타일 (CSS)
    pub styles: String,
    /// 추출된 이미지 (Base64)
    pub images: Vec<ImageData>,
}
```

---

### 4.2 MarkdownOutput

```rust
/// Markdown 변환 결과
#[derive(Debug, Serialize, Deserialize)]
pub struct MarkdownOutput {
    /// Markdown 문자열
    pub markdown: String,
    /// 이미지 참조 목록
    pub image_refs: Vec<String>,
}
```

---

## 5. API Response Types

### 5.1 ConvertResponse

```rust
/// 변환 API 응답
#[derive(Debug, Serialize, Deserialize)]
pub struct ConvertResponse {
    pub success: bool,
    pub data: Option<ConvertData>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConvertData {
    pub content: String,
    pub metadata: DocumentMetadata,
    pub images: Vec<ImageData>,
}
```

---

### 5.2 ApiError

```rust
/// API 에러 응답
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}
```

---

## 6. Entity Relationship

```
┌─────────────────────────────────────────────────────────────┐
│                    Entity Relationships                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  FileHeader                                                 │
│      │                                                      │
│      ├──▶ HwpVersion                                        │
│      └──▶ DocumentProperties                                │
│                                                             │
│  HwpDocument                                                │
│      │                                                      │
│      ├──▶ DocumentMetadata                                  │
│      │                                                      │
│      └──▶ Section[] ─────▶ Paragraph[]                      │
│                               │                             │
│                               ├──▶ CharShape (ref)          │
│                               ├──▶ ParaShape (ref)          │
│                               └──▶ Control[]                │
│                                       │                     │
│                                       ├──▶ Table            │
│                                       │       └──▶ Cell[]   │
│                                       │                     │
│                                       └──▶ Picture          │
│                                               └──▶ BinData  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 7. Type Conversion Reference

### 7.1 HWP Unit → CSS

| HWP Unit | Conversion | CSS Unit |
|----------|------------|----------|
| HWPUNIT (1/7200 inch) | ÷ 7200 × 96 | px |
| 1/100 pt | ÷ 100 | pt |
| % | / 100 | em/% |

### 7.2 Color → CSS

```rust
// HWP COLORREF (BBGGRR) → CSS
fn colorref_to_css(color: u32) -> String {
    let r = color & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = (color >> 16) & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}
```

---

**Author:** @Architect
**Date:** 2025-12-09
