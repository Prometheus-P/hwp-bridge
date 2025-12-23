> ⚠️ Note (Option A): `hwp-web` is excluded/disabled. Planned web materials moved to `future/`.

# Contributing to HwpBridge

Copyright (c) 2025 HwpBridge. All Rights Reserved.

---

## License Notice

This project uses a **proprietary license**. By contributing, you agree that:

1. Your contributions become part of the proprietary codebase
2. You grant HwpBridge full rights to use, modify, and distribute your contributions
3. You have the right to submit the contribution

For details, see `LICENSE` and `COMMERCIAL_LICENSE.md`.

---

HwpBridge 프로젝트에 기여해 주셔서 감사합니다! 이 문서는 기여 가이드라인을 설명합니다.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [Development Workflow](#development-workflow)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Commit Guidelines](#commit-guidelines)
8. [Pull Request Process](#pull-request-process)
9. [Issue Guidelines](#issue-guidelines)

---

## Code of Conduct

이 프로젝트는 모든 기여자에게 존중과 포용의 환경을 제공합니다.

- 건설적인 피드백을 제공합니다
- 다양한 관점을 존중합니다
- 커뮤니티에 도움이 되는 행동을 합니다

---

## Getting Started

### Prerequisites

- **Rust:** 1.85+ (2024 Edition)
- **Git:** 2.30+
- **cargo-watch** (선택): `cargo install cargo-watch`
- **cargo-nextest** (선택): `cargo install cargo-nextest`

### Fork & Clone

```bash
# 1. GitHub에서 Fork
# 2. Clone
git clone https://github.com/YOUR_USERNAME/hwp-bridge.git
cd hwp-bridge

# 3. Upstream 추가
git remote add upstream https://github.com/ORIGINAL_OWNER/hwp-bridge.git
```

---

## Development Setup

### Build

```bash
# 전체 빌드
cargo build --workspace

# Release 빌드
cargo build --workspace --release

# 특정 crate만 빌드
cargo build -p hwp-core
```

### Run

```bash
# CLI
cargo run -p hwp-cli -- --help

# Web Server
(planned) cargo run -p hwp-web  # crate not included in Option A

# MCP Server
cargo run -p hwp-mcp
```

### Watch Mode

```bash
# 파일 변경 시 자동 재빌드
cargo watch -x "build --workspace"

# 테스트 자동 실행
cargo watch -x "test --workspace"
```

---

## Development Workflow

### 1. Issue 확인

작업 전 관련 Issue가 있는지 확인하세요. 없다면 새 Issue를 생성합니다.

### 2. Branch 생성

```bash
# main에서 최신 코드 가져오기
git checkout main
git pull upstream main

# 새 브랜치 생성
git checkout -b feature/add-table-parsing
git checkout -b fix/header-validation
git checkout -b docs/update-readme
```

**Branch Naming:**

| Prefix | Use Case |
|--------|----------|
| `feature/` | 새 기능 |
| `fix/` | 버그 수정 |
| `refactor/` | 리팩토링 |
| `docs/` | 문서 수정 |
| `test/` | 테스트 추가 |
| `chore/` | 빌드, CI 등 |

### 3. TDD Cycle

```
RED → GREEN → REFACTOR → COMMIT
```

1. **RED:** 실패하는 테스트 작성
2. **GREEN:** 테스트 통과하는 최소 구현
3. **REFACTOR:** 코드 개선 (테스트 유지)
4. **COMMIT:** 변경 사항 커밋

### 4. Push & PR

```bash
git push origin feature/add-table-parsing
```

GitHub에서 Pull Request를 생성합니다.

---

## Coding Standards

### Rust Style

```bash
# 포맷 검사
cargo fmt --all -- --check

# 포맷 적용
cargo fmt --all

# Clippy 린트
cargo clippy --workspace --all-targets -- -D warnings
```

### Code Metrics

| Metric | Target |
|--------|--------|
| 함수 길이 | ≤ 20줄 |
| 파일 길이 | ≤ 400줄 |
| 중첩 깊이 | ≤ 3단계 |
| 매개변수 | ≤ 4개 |

### Documentation

```rust
/// 함수의 목적을 설명합니다.
///
/// # Arguments
///
/// * `path` - HWP 파일 경로
///
/// # Returns
///
/// 파싱된 FileHeader 또는 에러
///
/// # Errors
///
/// - `HwpError::InvalidSignature` - 잘못된 시그니처
/// - `HwpError::Encrypted` - 암호화된 문서
///
/// # Examples
///
/// ```rust
/// let header = parse_file_header(&data)?;
/// ```
pub fn parse_file_header(data: &[u8]) -> Result<FileHeader, HwpError> {
    // ...
}
```

### Error Handling

```rust
// ✅ Good: 명시적 에러 타입
pub fn parse(data: &[u8]) -> Result<Document, HwpError>

// ❌ Bad: anyhow in library code
pub fn parse(data: &[u8]) -> anyhow::Result<Document>

// ✅ Good: Context 추가
.map_err(|e| HwpError::ParseError(format!("Section {}: {}", idx, e)))?

// ❌ Bad: unwrap in production code
let value = data.get(0).unwrap();
```

---

## Testing Guidelines

### Test Structure

```
crates/hwp-core/src/
├── parser/
│   ├── header.rs        # 구현
│   └── header.rs        # 테스트 (같은 파일 내 #[cfg(test)])
└── tests/               # 통합 테스트
    └── integration.rs
```

### Test Naming

```rust
#[test]
fn test_should_{expected}_when_{condition}() {
    // Given
    let data = create_test_data();

    // When
    let result = parse_file_header(&data);

    // Then
    assert!(result.is_ok());
}
```

**Examples:**

```rust
fn test_should_parse_valid_header_when_signature_matches() { }
fn test_should_return_error_when_file_is_encrypted() { }
fn test_should_extract_text_when_document_is_compressed() { }
```

### Test Coverage

```bash
# Coverage 측정 (cargo-tarpaulin 설치 필요)
cargo tarpaulin --workspace --out Html

# 최소 커버리지: 80% (핵심 로직)
```

### Running Tests

```bash
# 전체 테스트
cargo test --workspace

# 특정 crate
cargo test -p hwp-core

# 특정 테스트
cargo test test_should_parse_valid_header

# 출력 포함
cargo test -- --nocapture

# nextest 사용 (빠름)
cargo nextest run --workspace
```

---

## Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

| Type | Description |
|------|-------------|
| `feat` | 새 기능 |
| `fix` | 버그 수정 |
| `docs` | 문서 변경 |
| `style` | 포맷, 세미콜론 등 |
| `refactor` | 리팩토링 |
| `test` | 테스트 추가/수정 |
| `chore` | 빌드, CI 등 |
| `perf` | 성능 개선 |

### Scopes

| Scope | Crate |
|-------|-------|
| `types` | hwp-types |
| `core` | hwp-core |
| `cli` | hwp-cli |
| `web (planned)` | hwp-web (not included) |
| `mcp` | hwp-mcp |

### Examples

```
feat(core): add FileHeader parsing

- Parse signature (32 bytes)
- Parse version (4 bytes)
- Parse properties flags (4 bytes)
- Add Fail-Fast validation

Closes #12
```

```
fix(core): handle extended record size

Records with size > 4095 use 4-byte extended size.
Previously this caused incorrect parsing.

Fixes #45
```

### Tidy vs Behavior

구조 변경(Tidy)과 기능 변경(Behavior)은 **별도 커밋**으로 분리합니다.

```bash
# Tidy commit (구조만 변경, 기능 동일)
refactor(core): extract parse_signature function

# Behavior commit (기능 변경)
feat(core): validate signature before parsing
```

---

## Pull Request Process

### Before Creating PR

- [ ] `cargo fmt --all` 실행
- [ ] `cargo clippy --workspace` 경고 없음
- [ ] `cargo test --workspace` 통과
- [ ] 관련 문서 업데이트
- [ ] CHANGELOG.md 업데이트 (필요시)

### PR Title

커밋 메시지와 동일한 형식:

```
feat(core): add table parsing support
fix(web): handle large file uploads
docs: update API specification
```

### PR Description

PR 템플릿을 따라 작성합니다:

- Summary
- Changes
- Test Plan
- Related Issues

### Review Process

1. CI 통과 확인
2. 최소 1명 리뷰어 승인
3. 모든 코멘트 해결
4. Squash and Merge

---

## Issue Guidelines

### Bug Report

- **제목:** `[BUG] 암호화 문서 파싱 시 패닉 발생`
- **내용:**
  - 재현 단계
  - 예상 동작
  - 실제 동작
  - 환경 정보 (OS, Rust 버전)
  - 가능하면 테스트 파일

### Feature Request

- **제목:** `[FEATURE] HWPX 포맷 지원`
- **내용:**
  - 사용 사례
  - 제안하는 해결책
  - 대안
  - 추가 컨텍스트

### Labels

| Label | Description |
|-------|-------------|
| `bug` | 버그 |
| `enhancement` | 새 기능 |
| `documentation` | 문서 |
| `good first issue` | 입문자용 |
| `help wanted` | 도움 요청 |
| `P0` / `P1` / `P2` | 우선순위 |

---

## Questions?

- **Issue:** GitHub Issues 사용
- **Discussion:** GitHub Discussions 사용

감사합니다! 🎉
