# HWPBridge (Rust Edition) - HWP 5.0 Format Specification

이 문서는 HWP 5.0 문서 포맷에 기반하여 `hwp-core`와 `hwp-type` Rust 모듈을 완벽하게 구현할 수 있도록 구성된 기술 명세서다.

---

## 1. 파일 구조 개요

HWP 5.0은 Microsoft Compound File Binary Format (OLE2)을 기반으로 하며, 내부는 스트림(Stream)과 스토리지(Storage) 계층 구조로 구성된다.

### 주요 스트림

| 스트림 이름         | 설명 |
|------------------|------|
| FileHeader       | 문서 시그니처, 버전, 속성 플래그 등 메타 정보 (256B) |
| DocInfo          | 문서 메타데이터 및 스타일 정의, zlib 압축 가능 |
| DocOptions       | 배포용, DRM, 전자서명 등 부가 설정 |
| BodyText/Section*| 본문 텍스트, 스타일, 컨트롤 포함 섹션별 스트림, zlib 압축 가능 |
| BinData          | 이미지, 사운드, OLE 객체 등 이진 데이터 스토리지 |
| Scripts          | 매크로 스크립트 저장 공간 |
| ViewText         | 배포용 문서에서 본문 대체 텍스트 스트림 |

---

## 2. 레코드 구조

### 공통 레코드 헤더 (4바이트)

```
| Bits    | 필드명     |
|---------|------------|
| 0-9     | Tag ID     |
| 10-19   | Level      |
| 20-31   | Size       |
```

- Size == 0xFFF이면 확장 길이(DWORD)가 뒤따름
- Tag ID는 레코드 종류 구분자

### 주요 Tag ID 예시

| Tag ID       | 설명 |
|--------------|------|
| 0x10         | 문서 속성 (DocInfo) |
| 0x30         | CharShape (글자모양) |
| 0x31         | ParaShape (문단모양) |
| 0x50         | Style 정의 |
| 0x100        | Paragraph Header |
| 0x101        | Paragraph Text |
| 0x102~0x103  | 문단 내 글자모양 / 정렬 정보 |
| 0x104        | 표 컨트롤 시작 |

---

## 3. 주요 스트림 설명

### FileHeader (256B 고정)

```rust
struct FileHeader {
  signature: [u8; 32], // "HWP Document File"
  version: u32,        // 예: 0x05000300 (5.0.3.0)
  flags: u32,          // bit0=압축, bit1=암호화, bit2=배포용 등
  encrypt_version: u32,
  license_country: u8,
  // ... Reserved
}
```

### DocInfo

zlib 압축될 수 있으며, 내부는 연속된 레코드들:

- 문서 속성 (문단ID, 페이지 시작번호 등)
- 글자모양(CharShape), 문단모양(ParaShape), 스타일(Style)
- 테두리/배경(BorderFill), 글꼴 목록(FaceName), 번호정의(Numbering), 탭 정의(Tabs)

### BodyText/Section*

각 섹션은 다음과 같은 구조를 가진 레코드 시퀀스로 구성된다:

- 문단 헤더 (HWPTAG_PARA_HEADER)
- 문단 텍스트 (HWPTAG_PARA_TEXT)
- 글자모양/정렬 정보 레코드
- 컨트롤 (표, 이미지, 수식, 도형 등)

---

## 4. zlib 압축 처리

- FileHeader의 `flags & 0x01 != 0` → 압축됨
- DocInfo, BodyText/Section* 스트림은 zlib 해제 필요
- zlib 헤더 확인 (보통 0x78 0x9C)로 중복 체크 가능

---

## 5. Rust 타입 정의 (`hwp-type`)

```rust
pub struct HwpDocument {
  pub header: FileHeader,
  pub doc_info: DocInfo,
  pub sections: Vec<Section>,
  pub bin_data: Vec<BinData>,
}

pub struct Section {
  pub paragraphs: Vec<Paragraph>,
}

pub struct Paragraph {
  pub header: ParaHeader,
  pub runs: Vec<TextRun>,
  pub char_shapes: Vec<CharShape>,
}

pub struct CharShape {
  pub font_size: u32,
  pub bold: bool,
  pub italic: bool,
  pub color: u32,
  pub underline_color: u32,
  pub shadow_color: u32,
}

pub enum ParagraphAlign {
  Left = 0, Center, Right, Justify, Distribute,
}
```

---

## 6. `hwp-core` 파싱 흐름

1. OLE 파일 열기 → 스트림 목록 확인
2. FileHeader 읽기 → 압축, 암호화 플래그 확인
3. DocInfo 읽기 (압축 해제 후 레코드 파싱)
4. Section 스트림들 순회
    - zlib 해제
    - 레코드 파싱 → 문단 AST 구성
5. BinData 로딩 → base64 or raw
6. AST (`HwpDocument`) 생성 및 HTML/Markdown 변환 준비

---

## 7. 확장성 고려 사항

- AST는 변환기를 위해 계층적(`Doc → Section → Paragraph → Run`)으로 설계
- `serde::Serialize/Deserialize` 호환 유지 (향후 JSON/HTML export용)
- 컨트롤 레코드(표, 이미지, 수식 등)는 trait 또는 enum 구조로 모델링
- BinData 스트림은 MIME 타입 자동 추론 또는 명시

---

## 참고 문서

- [한글 문서 포맷 5.0 revision 1.3 (HWP)]
- [배포용 문서 명세 revision 1.2]
- [수식 명세 revision 1.3]
- [차트 명세 revision 1.2]
- [HWPML 3.0 명세 revision 1.2]

