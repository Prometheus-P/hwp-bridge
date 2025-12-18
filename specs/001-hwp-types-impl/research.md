# Research: hwp-types 완전 구현

**Feature**: 001-hwp-types-impl
**Date**: 2025-12-11
**Status**: Complete

## Research Tasks

### 1. HWP 문서 구조 타입 정의

**Question**: Section과 Paragraph 구조를 어떻게 정의할 것인가?

**Decision**: DATA_MODEL.md의 정의를 따름
- `Section`: paragraphs 벡터 포함
- `Paragraph`: text, para_shape_id, char_shapes 벡터, controls 벡터 포함

**Rationale**: DATA_MODEL.md가 프로젝트의 Source of Truth로 지정됨 (spec.md Assumptions)

**Alternatives considered**:
- AST 형태의 복잡한 트리 구조 → 불필요한 복잡성, 단순한 계층 구조로 충분

### 2. 스타일 타입 필드 정의

**Question**: CharShape와 ParaShape의 필드 구성은?

**Decision**: DATA_MODEL.md 3.3, 3.4 섹션을 따름
- `CharShape`: font_ids[7], base_size, text_color, attr 등
- `ParaShape`: margin_left/right, indent, line_spacing, alignment 등
- `CharShapeAttr`, `ParaShapeAttr`: 비트 플래그 구조체

**Rationale**: HWP 5.0 공식 명세에 기반한 DATA_MODEL.md 정의

**Alternatives considered**:
- 단순화된 스타일 (bold, italic만) → 향후 HTML 변환 시 정보 손실

### 3. 컨트롤 타입 설계

**Question**: Control enum과 Table 타입을 어떻게 구성할 것인가?

**Decision**: 
- `Control` enum: Table, Picture, Unknown 변형 포함
- `Table`: rows, cols, cells 벡터, border_fill_id
- `TableCell`: col, row, col_span, row_span, width, height, paragraphs

**Rationale**: DATA_MODEL.md 3.6 섹션 정의

**Alternatives considered**:
- trait 기반 다형성 → enum이 더 단순하고 serde 호환성 우수

### 4. BinData 타입 설계

**Question**: 바이너리 데이터를 어떻게 표현할 것인가?

**Decision**:
- `BinData`: id, storage_type, extension, data 벡터
- `BinDataType` enum: Link, Embedding, Storage

**Rationale**: DATA_MODEL.md 3.7 섹션 정의

**Alternatives considered**:
- 이미지만 지원 → OLE 객체도 BinData이므로 일반화 필요

### 5. 레코드 태그 상수

**Question**: 태그 상수를 어떻게 구성할 것인가?

**Decision**:
- `tags` 모듈에 pub const로 정의
- DocInfo 태그 (0x00-0x1F), BodyText 태그 (0x40-0x7F), Table/Shape 태그 그룹화

**Rationale**: DATA_MODEL.md 3.2 섹션 정의, hwp_rust_spec.md 참조

**Alternatives considered**:
- enum 기반 → 상수가 더 유연하고 비교 연산 간단

### 6. RecordHeader 위치

**Question**: RecordHeader는 hwp-types에 포함해야 하는가?

**Decision**: hwp-types에 포함
- 파싱 로직은 hwp-core에서 구현
- RecordHeader 타입 정의는 hwp-types에서 제공

**Rationale**: constitution I에 따라 타입 정의는 hwp-types, 파싱 로직은 hwp-core

**Alternatives considered**:
- hwp-core에만 포함 → hwp-core 외부에서 RecordHeader 사용 시 의존성 문제

## Unresolved Items

없음 - 모든 연구 항목 해결됨

## References

- [docs/specs/DATA_MODEL.md](../../../docs/specs/DATA_MODEL.md) - 타입 정의 Source of Truth
- [docs/hwp_rust_spec.md](../../../docs/hwp_rust_spec.md) - HWP 포맷 명세
- [.specify/memory/constitution.md](../../../.specify/memory/constitution.md) - 아키텍처 원칙
