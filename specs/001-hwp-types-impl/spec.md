# Feature Specification: hwp-types 완전 구현

**Feature Branch**: `001-hwp-types-impl`
**Created**: 2025-12-11
**Status**: Draft
**Input**: User description: "hwp-types 구현"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 문서 구조 타입 사용 (Priority: P1)

hwp-core 개발자가 HWP 문서의 계층 구조(Document → Section → Paragraph)를 표현하기 위해 hwp-types의 타입들을 사용한다.

**Why this priority**: 문서 구조 타입은 파싱 결과를 저장하는 핵심 데이터 구조로, 모든 후속 기능(변환, 출력)의 기반이 됨

**Independent Test**: hwp-types만 의존하여 HwpDocument 구조체를 생성하고 Section, Paragraph를 추가할 수 있음

**Acceptance Scenarios**:

1. **Given** hwp-types 크레이트가 있을 때, **When** HwpDocument를 생성하고 Section과 Paragraph를 추가하면, **Then** 계층 구조가 올바르게 표현됨
2. **Given** 문단에 텍스트와 스타일 참조가 있을 때, **When** Paragraph 구조체를 생성하면, **Then** 텍스트, para_shape_id, char_shapes 벡터가 저장됨

---

### User Story 2 - 스타일 타입 정의 (Priority: P1)

hwp-core 개발자가 글자 모양(CharShape)과 문단 모양(ParaShape)을 파싱 결과로 저장하기 위해 타입을 사용한다.

**Why this priority**: 스타일 정보는 HTML/Markdown 변환 시 필수적이며, DocInfo 스트림 파싱의 핵심 결과물

**Independent Test**: CharShape와 ParaShape 구조체를 생성하고 속성 플래그를 조회할 수 있음

**Acceptance Scenarios**:

1. **Given** CharShapeAttr 비트 플래그가 있을 때, **When** is_bold(), is_italic() 메서드를 호출하면, **Then** 올바른 불리언 값 반환
2. **Given** ParaShape가 있을 때, **When** 정렬 속성을 조회하면, **Then** Alignment enum 값으로 반환됨

---

### User Story 3 - 컨트롤 타입 정의 (Priority: P2)

hwp-core 개발자가 표, 이미지 등 인라인 컨트롤을 표현하기 위해 Control enum과 관련 타입을 사용한다.

**Why this priority**: 표와 이미지는 문서 변환에서 중요하지만, 기본 텍스트 추출 후 구현 가능

**Independent Test**: Table 구조체를 생성하고 셀을 추가하여 표 구조를 표현할 수 있음

**Acceptance Scenarios**:

1. **Given** Table 구조체가 있을 때, **When** rows, cols, cells를 설정하면, **Then** 표 구조가 올바르게 표현됨
2. **Given** TableCell이 있을 때, **When** 병합 정보(col_span, row_span)를 설정하면, **Then** 셀 병합이 표현됨

---

### User Story 4 - 바이너리 데이터 타입 (Priority: P2)

hwp-core 개발자가 임베디드 이미지, OLE 객체 등을 저장하기 위해 BinData 타입을 사용한다.

**Why this priority**: 이미지 추출은 문서 변환의 부가 기능으로, 텍스트 추출 후 구현 가능

**Independent Test**: BinData 구조체를 생성하고 이미지 데이터를 저장할 수 있음

**Acceptance Scenarios**:

1. **Given** 이미지 바이트 데이터가 있을 때, **When** BinData를 생성하면, **Then** id, extension, data가 저장됨
2. **Given** BinDataType enum이 있을 때, **When** 저장 타입을 확인하면, **Then** Link, Embedding, Storage 구분 가능

---

### User Story 5 - 레코드 태그 상수 (Priority: P1)

hwp-core 개발자가 HWP 레코드를 파싱할 때 태그 ID를 비교하기 위해 상수를 사용한다.

**Why this priority**: 레코드 파싱의 핵심으로, 모든 스트림 파싱에 필요

**Independent Test**: tags 모듈의 상수들을 사용하여 태그 ID를 비교할 수 있음

**Acceptance Scenarios**:

1. **Given** tags 모듈이 있을 때, **When** PARA_HEADER 상수를 참조하면, **Then** 0x42 값을 얻음
2. **Given** 파싱된 tag_id가 있을 때, **When** tags::CHAR_SHAPE와 비교하면, **Then** 일치 여부 확인 가능

---

### Edge Cases

- 빈 문서(섹션/문단 없음)를 HwpDocument로 표현할 수 있어야 함
- CharShape의 font_ids 배열이 7개 언어를 모두 지원해야 함 (한글, 영문, 한자, 일문, 기타, 기호, 사용자)
- 확장 레코드 크기(4095 초과) 표현을 위해 RecordHeader.size가 u32여야 함
- 빈 테이블(셀 없음)도 유효한 Table로 표현 가능해야 함

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: hwp-types MUST 문서 계층 구조 타입 제공 (Section, Paragraph)
- **FR-002**: hwp-types MUST 글자 모양 타입 제공 (CharShape, CharShapeAttr)
- **FR-003**: hwp-types MUST 문단 모양 타입 제공 (ParaShape, Alignment)
- **FR-004**: hwp-types MUST 컨트롤 타입 제공 (Control enum, Table, TableCell)
- **FR-005**: hwp-types MUST 바이너리 데이터 타입 제공 (BinData, BinDataType)
- **FR-006**: hwp-types MUST HWP 레코드 태그 상수 모듈 제공 (tags)
- **FR-007**: hwp-types MUST 레코드 헤더 타입 제공 (RecordHeader)
- **FR-008**: 모든 공개 타입 MUST serde Serialize/Deserialize derive 적용
- **FR-009**: 모든 공개 타입 MUST Debug derive 적용
- **FR-010**: hwp-types MUST 파싱 로직을 포함하지 않음 (타입 정의만)

### Key Entities

- **Section**: 문서의 섹션, 문단(Paragraph) 목록 포함
- **Paragraph**: 텍스트, 스타일 참조, 인라인 컨트롤 포함
- **CharShape**: 글자 모양 정의 (폰트, 크기, 색상, 속성)
- **ParaShape**: 문단 모양 정의 (정렬, 여백, 줄간격)
- **Control**: 인라인 컨트롤 (표, 이미지 등) 열거형
- **Table/TableCell**: 표 구조와 셀 정의
- **BinData**: 임베디드 바이너리 데이터 (이미지, OLE)
- **RecordHeader**: HWP 레코드 공통 헤더 (tag_id, level, size)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: hwp-core에서 hwp-types의 모든 타입을 import하여 HWP 문서 구조를 완전히 표현할 수 있음
- **SC-002**: DATA_MODEL.md에 정의된 모든 Core Types와 Parser Types가 hwp-types에 구현됨
- **SC-003**: 모든 공개 타입이 serde로 JSON 직렬화/역직렬화 가능
- **SC-004**: cargo doc으로 생성된 문서에서 모든 공개 타입과 메서드에 문서 주석 존재
- **SC-005**: cargo test --workspace 통과 (기존 테스트 유지)
- **SC-006**: cargo clippy --workspace -- -D warnings 경고 없음

## Assumptions

- HWP 5.0 포맷 명세(docs/한글문서파일형식_5.0_revision1.3.pdf)를 기준으로 함
- DATA_MODEL.md의 타입 정의를 Source of Truth로 사용
- 파싱 로직(from_bytes 등)은 hwp-core에서 구현하며, hwp-types는 타입 정의만 담당
- UnderlineType 등 세부 enum은 필요 시 추가 구현

## Dependencies

- 기존 hwp-types 크레이트 (HwpError, HwpVersion, DocumentProperties, FileHeader 등)
- serde, thiserror 크레이트
