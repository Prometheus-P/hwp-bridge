# Quickstart: hwp-types 사용 가이드

**Feature**: 001-hwp-types-impl
**Date**: 2025-12-11

## 설치

hwp-types는 HWP Bridge 워크스페이스의 일부입니다.

```toml
# Cargo.toml (다른 crate에서 의존성 추가)
[dependencies]
hwp-types = { path = "../hwp-types" }
```

## 기본 사용법

### 1. 문서 구조 생성

```rust
use hwp_types::{HwpDocument, Section, Paragraph, DocumentMetadata};

// 빈 문서 생성
let mut doc = HwpDocument::default();

// 메타데이터 설정
doc.metadata = DocumentMetadata {
    title: "테스트 문서".to_string(),
    author: "작성자".to_string(),
    ..Default::default()
};

// 섹션과 문단 추가
let paragraph = Paragraph {
    text: "Hello, HWP!".to_string(),
    para_shape_id: 0,
    char_shapes: vec![(0, 0)],  // 위치 0부터 CharShape ID 0
};

let section = Section {
    paragraphs: vec![paragraph],
};

doc.sections.push(section);
```

### 2. 스타일 정의

```rust
use hwp_types::{CharShape, CharShapeAttr, ParaShape, Alignment};

// 글자 모양 생성
let char_shape = CharShape {
    base_size: 1000,  // 10pt (1/100 pt 단위)
    text_color: 0x00000000,  // 검정
    attr: CharShapeAttr::from_bits(0x01),  // Bold
    ..Default::default()
};

// Bold 여부 확인
if char_shape.attr.is_bold() {
    println!("Bold text");
}

// 문단 모양 생성
let para_shape = ParaShape {
    margin_left: 0,
    margin_right: 0,
    line_spacing: 160,  // 160%
    ..Default::default()
};

// 정렬 확인
match para_shape.attr.alignment() {
    Alignment::Center => println!("가운데 정렬"),
    Alignment::Left => println!("왼쪽 정렬"),
    _ => {}
}
```

### 3. 표 생성

```rust
use hwp_types::{Control, Table, TableCell};

// 2x2 표 생성
let table = Table {
    rows: 2,
    cols: 2,
    cells: vec![
        TableCell {
            col: 0, row: 0,
            col_span: 1, row_span: 1,
            width: 7200, height: 3600,  // HWPUNIT
            text: "셀 1".to_string(),
        },
        TableCell {
            col: 1, row: 0,
            col_span: 1, row_span: 1,
            width: 7200, height: 3600,
            text: "셀 2".to_string(),
        },
        // ... 나머지 셀
    ],
    border_fill_id: 0,
};

// 컨트롤로 감싸서 문단에 추가
let control = Control::Table(table);
```

### 4. 레코드 태그 사용

```rust
use hwp_types::tags;

fn process_record(tag_id: u16) {
    match tag_id {
        tags::PARA_HEADER => println!("문단 헤더"),
        tags::PARA_TEXT => println!("문단 텍스트"),
        tags::CHAR_SHAPE => println!("글자 모양"),
        tags::TABLE => println!("표"),
        _ => println!("Unknown tag: 0x{:02X}", tag_id),
    }
}
```

### 5. JSON 직렬화

```rust
use hwp_types::HwpDocument;
use serde_json;

let doc = HwpDocument::default();

// JSON으로 직렬화
let json = serde_json::to_string_pretty(&doc)?;
println!("{}", json);

// JSON에서 역직렬화
let parsed: HwpDocument = serde_json::from_str(&json)?;
```

## 모듈 구조

```
hwp_types
├── HwpError           # 에러 타입
├── HwpVersion         # 버전 정보
├── DocumentProperties # 문서 속성 플래그
├── FileHeader         # 파일 헤더
├── HwpDocument        # 문서 최상위 구조체
├── DocumentMetadata   # 메타데이터
├── Section            # 섹션
├── Paragraph          # 문단
├── CharShape          # 글자 모양
├── CharShapeAttr      # 글자 속성 플래그
├── ParaShape          # 문단 모양
├── ParaShapeAttr      # 문단 속성 플래그
├── Alignment          # 정렬 enum
├── Control            # 컨트롤 enum
├── Table              # 표
├── TableCell          # 표 셀
├── Picture            # 그림
├── BinData            # 바이너리 데이터
├── BinDataType        # 바이너리 데이터 타입
├── RecordHeader       # 레코드 헤더
├── ConvertOptions     # 변환 옵션
└── tags               # 레코드 태그 상수 모듈
```

## 주의사항

1. **파싱 로직 없음**: hwp-types는 타입 정의만 제공합니다. 실제 HWP 파일 파싱은 hwp-core를 사용하세요.

2. **HWPUNIT**: 크기/위치 값은 HWPUNIT 단위 (1/7200 inch)입니다.
   - 1 inch = 7200 HWPUNIT
   - pt → HWPUNIT: pt × 100

3. **COLORREF**: 색상은 BGR 순서 (0x00BBGGRR)입니다.
   - 빨강: 0x000000FF
   - 파랑: 0x00FF0000

4. **ID 참조**: para_shape_id, char_shape_id 등은 HwpDocument의 해당 벡터 인덱스입니다.
