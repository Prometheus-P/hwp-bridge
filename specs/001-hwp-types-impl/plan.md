# Implementation Plan: hwp-types 완전 구현

**Branch**: `001-hwp-types-impl` | **Date**: 2025-12-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-hwp-types-impl/spec.md`

## Summary

hwp-types 크레이트에 HWP 문서 파싱 결과를 저장하기 위한 공용 타입들을 추가합니다. 문서 계층 구조(Section, Paragraph), 스타일 타입(CharShape, ParaShape), 컨트롤 타입(Table, BinData), 레코드 태그 상수(tags 모듈)를 구현합니다. 모든 타입은 serde Serialize/Deserialize를 지원하며, 파싱 로직은 포함하지 않습니다.

## Technical Context

**Language/Version**: Rust 1.85+ (2024 Edition)
**Primary Dependencies**: serde (직렬화), thiserror (에러 정의)
**Storage**: N/A (타입 정의만, 저장 로직 없음)
**Testing**: cargo test (단위 테스트)
**Target Platform**: 크로스 플랫폼 (hwp-types는 기본 타입 크레이트, no_std 호환 가능)
**Project Type**: Rust workspace crate
**Performance Goals**: N/A (타입 정의만, 런타임 성능 영향 없음)
**Constraints**: 파싱 로직 포함 금지 (constitution I. Crate-Based Architecture)
**Scale/Scope**: DATA_MODEL.md에 정의된 ~15개 타입 구현

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-Based Architecture | ✅ PASS | hwp-types에 타입만 추가, 파싱 로직 없음 |
| II. Test-Driven Development | ✅ PASS | 각 타입에 대한 단위 테스트 작성 예정 |
| III. Fail-Fast Validation | N/A | 타입 정의만, 검증 로직은 hwp-core |
| IV. Multi-Interface Protocol | N/A | 타입 정의만, 인터페이스 구현 없음 |
| V. Performance & Safety | ✅ PASS | unwrap/expect 미사용, HwpError 활용 |

**Gate Result**: ✅ PASS - 모든 관련 원칙 준수

## Project Structure

### Documentation (this feature)

```text
specs/001-hwp-types-impl/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (타입 상세 정의)
├── quickstart.md        # Phase 1 output (사용 가이드)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/hwp-types/
├── Cargo.toml           # 기존 (수정 불필요)
└── src/
    ├── lib.rs           # 기존 타입 + 새 모듈 re-export
    ├── document.rs      # Section, Paragraph (NEW)
    ├── style.rs         # CharShape, ParaShape, Alignment (NEW)
    ├── control.rs       # Control, Table, TableCell (NEW)
    ├── bindata.rs       # BinData, BinDataType (NEW)
    ├── record.rs        # RecordHeader (NEW)
    └── tags.rs          # HWP 레코드 태그 상수 (NEW)
```

**Structure Decision**: 기존 hwp-types 크레이트 구조를 유지하면서 새 모듈을 추가합니다. 각 모듈은 관련 타입을 그룹화하고, lib.rs에서 pub use로 re-export합니다.

## Complexity Tracking

> 모든 Constitution 원칙을 준수하므로 위반 사항 없음

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (없음) | - | - |
