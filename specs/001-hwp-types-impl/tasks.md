# Tasks: hwp-types 완전 구현

**Input**: Design documents from `/specs/001-hwp-types-impl/`
**Prerequisites**: plan.md, spec.md, data-model.md, research.md, quickstart.md

**Tests**: TDD는 constitution에 의해 필수입니다. 각 타입에 대한 테스트를 포함합니다.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Project**: `crates/hwp-types/src/` (Rust workspace crate)
- **Tests**: 각 모듈 파일 내 `#[cfg(test)]` 섹션

---

## Phase 1: Setup

**Purpose**: 프로젝트 구조 확인 및 기본 설정

- [x] T001 Verify existing hwp-types crate structure in `crates/hwp-types/`
- [x] T002 Verify serde dependency in `crates/hwp-types/Cargo.toml`
- [x] T003 [P] Run `cargo build -p hwp-types` to confirm baseline compiles

---

## Phase 2: Foundational / User Story 5 - 레코드 태그 상수 (Blocking Prerequisites)

**Purpose**: 다른 모든 타입에서 사용하는 기반 타입 구현 (tags, RecordHeader)

**⚠️ CRITICAL**: 이 Phase 완료 전까지 User Story 구현 불가

### Tests for Foundational ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T004 [P] Write test for RecordHeader creation in `crates/hwp-types/src/record.rs`
- [x] T005 [P] Write test for tags constants in `crates/hwp-types/src/tags.rs`

### Implementation for Foundational

- [x] T006 [P] Create `crates/hwp-types/src/tags.rs` with HWP record tag constants per data-model.md section 6
- [x] T007 [P] Create `crates/hwp-types/src/record.rs` with RecordHeader struct per data-model.md section 5
- [x] T008 Add `pub mod tags;` and `pub mod record;` to `crates/hwp-types/src/lib.rs`
- [x] T009 Run `cargo test -p hwp-types` to verify foundational tests pass

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - 문서 구조 타입 (Priority: P1) 🎯 MVP

**Goal**: HWP 문서의 계층 구조(Section, Paragraph)를 표현하는 타입 제공

**Independent Test**: HwpDocument에 Section과 Paragraph를 추가하고 계층 구조 확인

### Tests for User Story 1 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T010 [P] [US1] Write test `test_should_create_empty_section_when_default` in `crates/hwp-types/src/document.rs`
- [x] T011 [P] [US1] Write test `test_should_store_paragraph_text_when_created` in `crates/hwp-types/src/document.rs`
- [x] T012 [P] [US1] Write test `test_should_add_paragraph_to_section_when_pushed` in `crates/hwp-types/src/document.rs`

### Implementation for User Story 1

- [x] T013 [US1] Create `crates/hwp-types/src/document.rs` with Section struct per data-model.md section 1
- [x] T014 [US1] Add Paragraph struct to `crates/hwp-types/src/document.rs` per data-model.md section 1
- [x] T015 [US1] Add `pub mod document;` to `crates/hwp-types/src/lib.rs` and re-export types
- [x] T016 [US1] Update HwpDocument in `crates/hwp-types/src/lib.rs` to use `sections: Vec<Section>` instead of `content: String`
- [x] T017 [US1] Run `cargo test -p hwp-types` to verify US1 tests pass

**Checkpoint**: User Story 1 완료 - 문서 구조 타입 독립적으로 테스트 가능

---

## Phase 4: User Story 2 - 스타일 타입 (Priority: P1)

**Goal**: 글자 모양(CharShape)과 문단 모양(ParaShape)을 표현하는 타입 제공

**Independent Test**: CharShapeAttr.is_bold(), ParaShapeAttr.alignment() 메서드 검증

### Tests for User Story 2 ⚠️

- [x] T018 [P] [US2] Write test `test_should_return_true_when_bold_bit_set` in `crates/hwp-types/src/style.rs`
- [x] T019 [P] [US2] Write test `test_should_return_false_when_bold_bit_not_set` in `crates/hwp-types/src/style.rs`
- [x] T020 [P] [US2] Write test `test_should_return_alignment_center_when_bits_match` in `crates/hwp-types/src/style.rs`
- [x] T021 [P] [US2] Write test `test_should_create_default_charshape_when_default` in `crates/hwp-types/src/style.rs`

### Implementation for User Story 2

- [x] T022 [US2] Create `crates/hwp-types/src/style.rs` with Alignment enum per data-model.md section 2
- [x] T023 [US2] Add CharShapeAttr struct with bit flag methods to `crates/hwp-types/src/style.rs`
- [x] T024 [US2] Add CharShape struct to `crates/hwp-types/src/style.rs`
- [x] T025 [US2] Add ParaShapeAttr struct with bit flag methods to `crates/hwp-types/src/style.rs`
- [x] T026 [US2] Add ParaShape struct to `crates/hwp-types/src/style.rs`
- [x] T027 [US2] Add `pub mod style;` to `crates/hwp-types/src/lib.rs` and re-export types
- [x] T028 [US2] Add `char_shapes: Vec<CharShape>` and `para_shapes: Vec<ParaShape>` to HwpDocument
- [x] T029 [US2] Run `cargo test -p hwp-types` to verify US2 tests pass

**Checkpoint**: User Story 2 완료 - 스타일 타입 독립적으로 테스트 가능

---

## Phase 5: User Story 3 - 컨트롤 타입 (Priority: P2)

**Goal**: 표, 이미지 등 인라인 컨트롤을 표현하는 타입 제공

**Independent Test**: Table에 셀 추가하고 병합 정보 확인

### Tests for User Story 3 ⚠️

- [x] T030 [P] [US3] Write test `test_should_create_table_with_dimensions_when_set` in `crates/hwp-types/src/control.rs`
- [x] T031 [P] [US3] Write test `test_should_store_cell_span_when_set` in `crates/hwp-types/src/control.rs`
- [x] T032 [P] [US3] Write test `test_should_create_picture_with_bindata_id_when_set` in `crates/hwp-types/src/control.rs`

### Implementation for User Story 3

- [x] T033 [US3] Create `crates/hwp-types/src/control.rs` with Table struct per data-model.md section 3
- [x] T034 [US3] Add TableCell struct to `crates/hwp-types/src/control.rs`
- [x] T035 [US3] Add Picture struct to `crates/hwp-types/src/control.rs`
- [x] T036 [US3] Add Control enum (Table, Picture, Unknown) to `crates/hwp-types/src/control.rs`
- [x] T037 [US3] Add `pub mod control;` to `crates/hwp-types/src/lib.rs` and re-export types
- [x] T038 [US3] Update Paragraph in document.rs to use `controls: Vec<Control>`
- [x] T039 [US3] Run `cargo test -p hwp-types` to verify US3 tests pass

**Checkpoint**: User Story 3 완료 - 컨트롤 타입 독립적으로 테스트 가능

---

## Phase 6: User Story 4 - 바이너리 데이터 타입 (Priority: P2)

**Goal**: 임베디드 이미지, OLE 객체를 저장하는 타입 제공

**Independent Test**: BinData 생성 및 BinDataType 구분 확인

### Tests for User Story 4 ⚠️

- [x] T040 [P] [US4] Write test `test_should_store_bindata_with_extension_when_created` in `crates/hwp-types/src/bindata.rs`
- [x] T041 [P] [US4] Write test `test_should_distinguish_bindata_types_when_compared` in `crates/hwp-types/src/bindata.rs`

### Implementation for User Story 4

- [x] T042 [US4] Create `crates/hwp-types/src/bindata.rs` with BinDataType enum per data-model.md section 4
- [x] T043 [US4] Add BinData struct to `crates/hwp-types/src/bindata.rs`
- [x] T044 [US4] Add `pub mod bindata;` to `crates/hwp-types/src/lib.rs` and re-export types
- [x] T045 [US4] Add `bin_data: Vec<BinData>` to HwpDocument
- [x] T046 [US4] Run `cargo test -p hwp-types` to verify US4 tests pass

**Checkpoint**: User Story 4 완료 - 바이너리 데이터 타입 독립적으로 테스트 가능

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: 최종 검증 및 문서화

- [x] T047 [P] Add doc comments to all public types in all new modules; verify all public types derive Debug
- [x] T048 [P] Verify JSON serialization roundtrip for all types with serde_json
- [x] T049 Run `cargo fmt --all` and fix formatting
- [x] T050 Run `cargo clippy --workspace -- -D warnings` and fix warnings
- [x] T051 Run `cargo test --workspace` for full test suite
- [x] T052 Run `cargo doc -p hwp-types` and verify documentation
- [x] T053 Validate against quickstart.md examples
- [x] T054 [Safety] Run `grep -r 'unwrap\|expect' crates/hwp-types/src/` and verify zero matches in non-test code

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational - 🎯 MVP
- **US2 (Phase 4)**: Depends on Foundational - can parallel with US1
- **US3 (Phase 5)**: Depends on US1 (uses Paragraph, Control)
- **US4 (Phase 6)**: Depends on Foundational - can parallel with US1/US2
- **Polish (Phase 7)**: Depends on all user stories

### User Story Dependencies

```
Foundational (tags, RecordHeader)
     ↓
  ┌──┴──────────┬──────────────┐
  ↓             ↓              ↓
 US1           US2            US4
(document)   (style)       (bindata)
  ↓             
 US3           
(control)     
  ↓
Polish
```

### Within Each User Story

1. Tests MUST be written and FAIL before implementation (TDD - constitution requirement)
2. Types before re-exports
3. Simple types before complex types
4. Update lib.rs after module completion
5. Verify tests pass before checkpoint

### Parallel Opportunities

**Phase 2 (Foundational)**:
- T004, T005 (tests) can run in parallel
- T006, T007 (implementation) can run in parallel

**Phase 3-6 (User Stories)**:
- US1, US2, US4 can start in parallel after Foundational
- All tests within a phase marked [P] can run in parallel

---

## Parallel Example: Foundational Phase

```bash
# Launch all foundational tests together:
Task: "Write test for RecordHeader creation in crates/hwp-types/src/record.rs"
Task: "Write test for tags constants in crates/hwp-types/src/tags.rs"

# Launch all foundational implementations together:
Task: "Create crates/hwp-types/src/tags.rs with HWP record tag constants"
Task: "Create crates/hwp-types/src/record.rs with RecordHeader struct"
```

## Parallel Example: User Story 1

```bash
# Launch all US1 tests together:
Task: "Write test test_should_create_empty_section_when_default"
Task: "Write test test_should_store_paragraph_text_when_created"
Task: "Write test test_should_add_paragraph_to_section_when_pushed"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (tags, RecordHeader)
3. Complete Phase 3: User Story 1 (document.rs)
4. **STOP and VALIDATE**: `cargo test -p hwp-types`
5. MVP ready - 기본 문서 구조 표현 가능

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 (document) → 문서 계층 구조 ✅
3. Add US2 (style) → 스타일 정보 ✅
4. Add US3 (control) → 표/이미지 ✅
5. Add US4 (bindata) → 바이너리 데이터 ✅
6. Polish → 최종 검증

### Task Count Summary

| Phase | Task Count | Parallel Tasks |
|-------|------------|----------------|
| Setup | 3 | 1 |
| Foundational | 6 | 4 |
| US1 (P1) | 8 | 3 |
| US2 (P1) | 12 | 4 |
| US3 (P2) | 10 | 3 |
| US4 (P2) | 7 | 2 |
| Polish | 7 | 2 |
| **Total** | **53** | **19** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- TDD is NON-NEGOTIABLE per constitution.md - tests MUST fail before implementation
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
