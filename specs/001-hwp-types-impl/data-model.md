# Data Model: hwp-types 완전 구현

**Feature**: 001-hwp-types-impl
**Date**: 2025-12-11
**Source**: [docs/specs/DATA_MODEL.md](../../../docs/specs/DATA_MODEL.md)

## Overview

hwp-types 크레이트에 추가할 타입들의 상세 정의입니다. 기존 타입(HwpError, HwpVersion, DocumentProperties, FileHeader, HwpDocument, DocumentMetadata, ConvertOptions)은 유지하고 새 타입을 추가합니다.

---

## 1. Document Structure Types (document.rs)

### Section

```rust
/// 문서 섹션
/// 하나의 HWP 문서는 여러 섹션으로 구성됨
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section {
    /// 섹션 내 문단 목록
    pub paragraphs: Vec<Paragraph>,
}
```

### Paragraph

```rust
/// 문단
/// 텍스트와 스타일 참조, 인라인 컨트롤 포함
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Paragraph {
    /// 문단 텍스트
    pub text: String,
    /// 문단 모양 ID 참조 (DocInfo의 ParaShape 인덱스)
    pub para_shape_id: u16,
    /// 글자 위치별 CharShape ID: (position, shape_id)
    pub char_shapes: Vec<(u32, u16)>,
    /// 인라인 컨트롤 (표, 그림 등)
    pub controls: Vec<Control>,
}
```

---

## 2. Style Types (style.rs)

### CharShape

```rust
/// 글자 모양 정의 (DocInfo 스트림에서 파싱)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharShape {
    /// 글꼴 ID 참조 (언어별 7개: 한글, 영문, 한자, 일문, 기타, 기호, 사용자)
    pub font_ids: [u16; 7],
    /// 글꼴 비율 (%, 언어별)
    pub font_scales: [u8; 7],
    /// 자간 (언어별)
    pub char_spacing: [i8; 7],
    /// 상대 크기 (%, 언어별)
    pub relative_sizes: [u8; 7],
    /// 기준 크기 (1/100 pt 단위)
    pub base_size: u32,
    /// 글자 색상 (COLORREF: 0x00BBGGRR)
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
```

### CharShapeAttr

```rust
/// 글자 속성 비트 플래그
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharShapeAttr(pub u32);

impl CharShapeAttr {
    /// 새 CharShapeAttr 생성
    pub fn from_bits(bits: u32) -> Self { Self(bits) }
    
    /// 원시 비트 값 반환
    pub fn bits(&self) -> u32 { self.0 }
    
    /// Bit 0: 굵게
    pub fn is_bold(&self) -> bool { self.0 & (1 << 0) != 0 }
    
    /// Bit 1: 기울임
    pub fn is_italic(&self) -> bool { self.0 & (1 << 1) != 0 }
    
    /// Bit 2-3: 밑줄 종류 (0=없음, 1=실선, 2=점선, ...)
    pub fn underline_type(&self) -> u8 { ((self.0 >> 2) & 0x03) as u8 }
    
    /// Bit 4-5: 외곽선 종류
    pub fn outline_type(&self) -> u8 { ((self.0 >> 4) & 0x03) as u8 }
    
    /// Bit 6-7: 그림자 종류
    pub fn shadow_type(&self) -> u8 { ((self.0 >> 6) & 0x03) as u8 }
    
    /// Bit 8: 양각
    pub fn is_emboss(&self) -> bool { self.0 & (1 << 8) != 0 }
    
    /// Bit 9: 음각
    pub fn is_engrave(&self) -> bool { self.0 & (1 << 9) != 0 }
    
    /// Bit 10: 위 첨자
    pub fn is_superscript(&self) -> bool { self.0 & (1 << 10) != 0 }
    
    /// Bit 11: 아래 첨자
    pub fn is_subscript(&self) -> bool { self.0 & (1 << 11) != 0 }
    
    /// Bit 12-14: 취소선 종류
    pub fn strikethrough_type(&self) -> u8 { ((self.0 >> 12) & 0x07) as u8 }
}
```

### ParaShape

```rust
/// 문단 모양 정의 (DocInfo 스트림에서 파싱)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParaShape {
    /// 속성 플래그
    pub attr: ParaShapeAttr,
    /// 왼쪽 여백 (HWPUNIT: 1/7200 inch)
    pub margin_left: i32,
    /// 오른쪽 여백
    pub margin_right: i32,
    /// 들여쓰기 (양수: 들여쓰기, 음수: 내어쓰기)
    pub indent: i32,
    /// 문단 위 간격
    pub margin_top: i32,
    /// 문단 아래 간격
    pub margin_bottom: i32,
    /// 줄 간격 (% 또는 고정값)
    pub line_spacing: i32,
    /// 탭 정의 ID
    pub tab_def_id: u16,
    /// 번호/글머리 ID
    pub para_head_id: u16,
    /// 테두리/배경 ID
    pub border_fill_id: u16,
}
```

### ParaShapeAttr

```rust
/// 문단 속성 비트 플래그
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParaShapeAttr(pub u32);

impl ParaShapeAttr {
    pub fn from_bits(bits: u32) -> Self { Self(bits) }
    pub fn bits(&self) -> u32 { self.0 }
    
    /// Bit 0-1: 줄 간격 종류 (0=%, 1=고정, 2=여백만, 3=최소)
    pub fn line_spacing_type(&self) -> u8 { (self.0 & 0x03) as u8 }
    
    /// Bit 2-4: 정렬 (0=양쪽, 1=왼쪽, 2=오른쪽, 3=가운데, 4=배분, 5=나눔)
    pub fn alignment(&self) -> Alignment {
        match (self.0 >> 2) & 0x07 {
            0 => Alignment::Justify,
            1 => Alignment::Left,
            2 => Alignment::Right,
            3 => Alignment::Center,
            4 => Alignment::Distribute,
            _ => Alignment::Left,
        }
    }
}
```

### Alignment

```rust
/// 문단 정렬
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    #[default]
    Left = 0,
    Center = 1,
    Right = 2,
    Justify = 3,
    Distribute = 4,
}
```

---

## 3. Control Types (control.rs)

### Control

```rust
/// 인라인 컨트롤 (표, 그림 등)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Control {
    /// 표
    Table(Table),
    /// 그림
    Picture(Picture),
    /// 알 수 없는 컨트롤
    Unknown {
        /// 컨트롤 타입 코드
        ctrl_id: u32,
    },
}
```

### Table

```rust
/// 표 컨트롤
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
```

### TableCell

```rust
/// 표 셀
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableCell {
    /// 열 주소 (0-based)
    pub col: u16,
    /// 행 주소 (0-based)
    pub row: u16,
    /// 열 병합 수 (1 = 병합 없음)
    pub col_span: u16,
    /// 행 병합 수 (1 = 병합 없음)
    pub row_span: u16,
    /// 셀 너비 (HWPUNIT)
    pub width: u32,
    /// 셀 높이 (HWPUNIT)
    pub height: u32,
    /// 셀 내용 (문단 목록)
    pub paragraphs: Vec<Paragraph>,
}
```

### Picture

```rust
/// 그림 컨트롤
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Picture {
    /// BinData ID 참조
    pub bin_data_id: u16,
    /// 너비 (HWPUNIT)
    pub width: u32,
    /// 높이 (HWPUNIT)
    pub height: u32,
}
```

---

## 4. BinData Types (bindata.rs)

### BinData

```rust
/// 바이너리 데이터 (이미지, OLE 객체 등)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BinData {
    /// 데이터 ID
    pub id: u16,
    /// 저장 타입
    pub storage_type: BinDataType,
    /// 파일 확장자 (예: "png", "jpg")
    pub extension: String,
    /// 원본 데이터
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}
```

### BinDataType

```rust
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
```

---

## 5. Record Types (record.rs)

### RecordHeader

```rust
/// HWP 레코드 헤더 (4바이트 또는 8바이트)
/// 모든 HWP 레코드는 이 헤더로 시작
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHeader {
    /// 태그 ID (10 bits, 0-1023)
    pub tag_id: u16,
    /// 레벨 (10 bits, 중첩 깊이)
    pub level: u16,
    /// 데이터 크기 (12 bits 기본, 확장 시 4바이트 추가)
    /// 0xFFF(4095)인 경우 뒤따르는 4바이트가 실제 크기
    pub size: u32,
}
```

---

## 6. Tag Constants (tags.rs)

```rust
/// HWP 레코드 태그 상수
pub mod tags {
    // === DocInfo Tags (0x00 - 0x1F) ===
    /// 문서 속성
    pub const DOCUMENT_PROPERTIES: u16 = 0x00;
    /// ID 매핑 테이블
    pub const ID_MAPPINGS: u16 = 0x01;
    /// 바이너리 데이터
    pub const BIN_DATA: u16 = 0x02;
    /// 글꼴 이름
    pub const FACE_NAME: u16 = 0x03;
    /// 테두리/배경
    pub const BORDER_FILL: u16 = 0x04;
    /// 글자 모양
    pub const CHAR_SHAPE: u16 = 0x07;
    /// 탭 정의
    pub const TAB_DEF: u16 = 0x08;
    /// 문단 모양
    pub const PARA_SHAPE: u16 = 0x09;
    /// 스타일
    pub const STYLE: u16 = 0x0A;
    /// 메모 모양
    pub const MEMO_SHAPE: u16 = 0x0B;

    // === BodyText Tags (0x40 - 0x7F) ===
    /// 문단 헤더
    pub const PARA_HEADER: u16 = 0x42;
    /// 문단 텍스트
    pub const PARA_TEXT: u16 = 0x43;
    /// 문단 글자 모양
    pub const PARA_CHAR_SHAPE: u16 = 0x44;
    /// 문단 라인 세그먼트
    pub const PARA_LINE_SEG: u16 = 0x45;
    /// 문단 범위 태그
    pub const PARA_RANGE_TAG: u16 = 0x46;
    /// 컨트롤 헤더
    pub const CTRL_HEADER: u16 = 0x47;

    // === Table Tags ===
    /// 표
    pub const TABLE: u16 = 0x4D;
    /// 표 셀
    pub const TABLE_CELL: u16 = 0x4E;

    // === Shape Tags ===
    /// 도형 컴포넌트
    pub const SHAPE_COMPONENT: u16 = 0x51;
    /// 선
    pub const SHAPE_COMPONENT_LINE: u16 = 0x52;
    /// 사각형
    pub const SHAPE_COMPONENT_RECTANGLE: u16 = 0x53;
    /// 타원
    pub const SHAPE_COMPONENT_ELLIPSE: u16 = 0x54;
    /// 그림
    pub const SHAPE_COMPONENT_PICTURE: u16 = 0x57;
}
```

---

## 7. Updated HwpDocument

기존 HwpDocument를 Section 벡터를 사용하도록 업데이트:

```rust
/// 파싱 결과물 최상위 구조체
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HwpDocument {
    /// 문서 메타데이터
    pub metadata: DocumentMetadata,
    /// 문서 섹션 목록 (NEW)
    pub sections: Vec<Section>,
    /// 글자 모양 목록 (DocInfo에서 파싱)
    pub char_shapes: Vec<CharShape>,
    /// 문단 모양 목록 (DocInfo에서 파싱)
    pub para_shapes: Vec<ParaShape>,
    /// 바이너리 데이터 목록
    pub bin_data: Vec<BinData>,
}
```

---

## Entity Relationships

```
HwpDocument
├── metadata: DocumentMetadata
├── sections: Vec<Section>
│   └── paragraphs: Vec<Paragraph>
│       ├── text: String
│       ├── para_shape_id → para_shapes[id]
│       ├── char_shapes → char_shapes[id]
│       └── controls: Vec<Control>
│           ├── Table
│           │   └── cells: Vec<TableCell>
│           │       └── paragraphs: Vec<Paragraph>
│           └── Picture
│               └── bin_data_id → bin_data[id]
├── char_shapes: Vec<CharShape>
├── para_shapes: Vec<ParaShape>
└── bin_data: Vec<BinData>
```
