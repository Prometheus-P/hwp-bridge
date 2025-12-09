# Test Strategy - HwpBridge

> **Version:** 1.0.0
> **Author:** @QA
> **Last Updated:** 2025-12-09

---

## 1. Overview

### 1.1 Testing Goals

| Goal | Description |
|------|-------------|
| **품질 보증** | 파싱 정확도 ≥ 95% (비암호화 문서) |
| **회귀 방지** | 변경 시 기존 기능 정상 동작 확인 |
| **문서화** | 테스트가 스펙 역할 수행 |
| **빠른 피드백** | CI에서 5분 이내 결과 확인 |

### 1.2 Testing Pyramid

```
                    ┌─────────┐
                    │   E2E   │  ← 적음 (느림, 비용 높음)
                   ─┴─────────┴─
                  ┌─────────────┐
                  │ Integration │  ← 중간
                 ─┴─────────────┴─
                ┌─────────────────┐
                │   Unit Tests    │  ← 많음 (빠름, 비용 낮음)
                └─────────────────┘
```

---

## 2. Test Categories

### 2.1 Unit Tests

**목적:** 개별 함수/모듈의 정확성 검증

**위치:** 각 소스 파일 내 `#[cfg(test)]` 모듈

**커버리지 목표:** ≥ 80%

| Crate | Focus Areas |
|-------|-------------|
| `hwp-types` | 타입 변환, Display, 비트 플래그 |
| `hwp-core` | 파싱 로직, 에러 처리, 변환 |
| `hwp-cli` | 인자 파싱, 출력 포맷 |
| `hwp-web` | 핸들러 로직, 유효성 검사 |
| `hwp-mcp` | Tool 핸들러, 프로토콜 인코딩 |

**Example:**

```rust
// crates/hwp-core/src/parser/header.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_should_parse_valid_header() { ... }

    #[test]
    fn test_should_reject_invalid_signature() { ... }
}
```

---

### 2.2 Integration Tests

**목적:** 모듈 간 상호작용 검증

**위치:** `crates/*/tests/`

| Test Suite | Description |
|------------|-------------|
| `ole_parsing` | OLE 컨테이너 열기 → 스트림 읽기 |
| `full_parse` | 파일 → HwpDocument 전체 파이프라인 |
| `conversion` | HwpDocument → HTML/Markdown |
| `fail_fast` | 암호화/배포용 문서 즉시 거부 |

**Example:**

```rust
// crates/hwp-core/tests/full_parse.rs
use hwp_core::HwpOleFile;
use std::fs::File;

#[test]
fn test_should_parse_sample_document() {
    let file = File::open("tests/fixtures/sample.hwp").unwrap();
    let hwp = HwpOleFile::open(file).unwrap();

    assert_eq!(hwp.header().version.major, 5);
    assert!(hwp.list_sections().len() > 0);
}
```

---

### 2.3 End-to-End Tests

**목적:** 전체 시스템 동작 검증

**위치:** `tests/e2e/`

| Test Suite | Description |
|------------|-------------|
| `cli_convert` | CLI로 파일 변환 후 출력 검증 |
| `web_upload` | HTTP API 파일 업로드 테스트 |
| `mcp_tools` | MCP Tool 호출 시나리오 |

**Example:**

```rust
// tests/e2e/cli_convert.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_convert_to_html() {
    let mut cmd = Command::cargo_bin("hwp-cli").unwrap();

    cmd.arg("convert")
        .arg("tests/fixtures/sample.hwp")
        .arg("-o")
        .arg("/tmp/output.html")
        .arg("--format")
        .arg("html")
        .assert()
        .success()
        .stdout(predicate::str::contains("Converted successfully"));

    assert!(std::path::Path::new("/tmp/output.html").exists());
}
```

---

### 2.4 Performance Tests

**목적:** 성능 요구사항 충족 확인

**위치:** `benches/`

| Benchmark | Target |
|-----------|--------|
| `parse_header` | < 1ms |
| `parse_1mb_doc` | < 100ms |
| `parse_10mb_doc` | < 1s |
| `convert_to_html` | < 500ms/MB |

**Example:**

```rust
// benches/parsing.rs
use criterion::{criterion_group, criterion_main, Criterion};
use hwp_core::parse_file_header;

fn bench_parse_header(c: &mut Criterion) {
    let data = include_bytes!("../tests/fixtures/header_only.bin");

    c.bench_function("parse_file_header", |b| {
        b.iter(|| parse_file_header(data))
    });
}

criterion_group!(benches, bench_parse_header);
criterion_main!(benches);
```

---

### 2.5 Security Tests

**목적:** 보안 취약점 탐지

| Test Type | Tool | Focus |
|-----------|------|-------|
| Dependency Audit | `cargo audit` | 알려진 취약점 |
| Fuzzing | `cargo fuzz` | 파싱 크래시 |
| Input Validation | Unit tests | 악성 입력 처리 |

**Fuzzing Example:**

```rust
// fuzz/fuzz_targets/parse_header.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use hwp_core::parse_file_header;

fuzz_target!(|data: &[u8]| {
    // Should not panic for any input
    let _ = parse_file_header(data);
});
```

---

## 3. Test Data Management

### 3.1 Fixtures

```
tests/fixtures/
├── valid/
│   ├── sample.hwp           # 기본 문서
│   ├── with_table.hwp       # 표 포함
│   ├── with_images.hwp      # 이미지 포함
│   ├── with_styles.hwp      # 다양한 스타일
│   ├── multipage.hwp        # 여러 페이지
│   └── large_10mb.hwp       # 대용량 파일
│
├── invalid/
│   ├── encrypted.hwp        # 암호화 문서
│   ├── distribution.hwp     # 배포용 문서
│   ├── v3_old.hwp           # 구버전
│   ├── corrupted.hwp        # 손상된 파일
│   └── not_hwp.pdf          # HWP 아닌 파일
│
└── generated/
    └── header_only.bin      # 테스트용 생성 데이터
```

### 3.2 Fixture Guidelines

- **민감 정보 제거:** 실제 문서의 개인정보 삭제
- **최소 크기:** 테스트에 필요한 최소 데이터만 포함
- **버전 관리:** 픽스처도 Git으로 관리
- **문서화:** 각 픽스처의 특성 README에 기록

### 3.3 Generated Test Data

```rust
// Helper functions for generating test data
pub mod test_helpers {
    pub fn create_valid_header() -> Vec<u8> { ... }
    pub fn create_encrypted_header() -> Vec<u8> { ... }
    pub fn create_section_with_text(text: &str) -> Vec<u8> { ... }
    pub fn create_table(rows: usize, cols: usize) -> Vec<u8> { ... }
}
```

---

## 4. Test Environment

### 4.1 Local Development

```bash
# 단위 테스트 (빠름)
cargo test --workspace

# 통합 테스트 포함
cargo test --workspace --all-features

# 느린 테스트 제외
cargo test --workspace -- --skip slow

# 특정 테스트만
cargo test -p hwp-core parser::header
```

### 4.2 CI Environment

| Stage | Tests | Timeout |
|-------|-------|---------|
| PR Check | Unit + Integration | 5 min |
| Main | All + Coverage | 10 min |
| Nightly | All + Fuzz + Bench | 30 min |

### 4.3 Environment Variables

```bash
# 테스트 실행 설정
RUST_TEST_THREADS=4          # 병렬 실행 수
RUST_BACKTRACE=1             # 스택 트레이스
HWP_TEST_FIXTURES=./fixtures # 픽스처 경로

# 로깅
RUST_LOG=hwp_core=debug      # 디버그 로그
```

---

## 5. Test Coverage

### 5.1 Coverage Requirements

| Category | Minimum | Target |
|----------|---------|--------|
| Overall | 70% | 85% |
| Core (hwp-core) | 80% | 90% |
| Parser modules | 85% | 95% |
| Error paths | 75% | 85% |

### 5.2 Coverage Tools

```bash
# cargo-tarpaulin (Linux)
cargo tarpaulin --workspace --out Html

# cargo-llvm-cov (Cross-platform)
cargo llvm-cov --workspace --html

# Coverage report location
open target/tarpaulin/tarpaulin-report.html
```

### 5.3 Coverage Exclusions

```rust
// 커버리지에서 제외할 코드
#[cfg(not(tarpaulin_include))]
fn debug_print() {
    // 디버그 전용 코드
}

// 또는 attribute 사용
#[coverage(off)]
fn unreachable_in_practice() {
    // 이론적으로만 도달 가능
}
```

---

## 6. Test Quality Metrics

### 6.1 Key Metrics

| Metric | Definition | Target |
|--------|------------|--------|
| **Coverage** | 실행된 코드 비율 | ≥ 80% |
| **Mutation Score** | 변형 감지율 | ≥ 70% |
| **Test Duration** | 전체 테스트 시간 | < 5 min |
| **Flaky Rate** | 불안정 테스트 비율 | < 1% |

### 6.2 Mutation Testing

```bash
# cargo-mutants 설치
cargo install cargo-mutants

# 실행
cargo mutants --workspace

# 결과 예시
Mutation testing complete:
  294 mutants tested
  271 killed (92.2%)
  23 survived (7.8%)  ← 테스트 보강 필요
```

### 6.3 Test Reliability

**Flaky Test 방지:**

```rust
// ❌ Bad: 시간 의존적
#[test]
fn test_timeout() {
    let start = Instant::now();
    do_something();
    assert!(start.elapsed() < Duration::from_secs(1));  // Flaky!
}

// ✅ Good: 결정적
#[test]
fn test_result() {
    let result = do_something();
    assert_eq!(result, expected);
}

// ❌ Bad: 순서 의존적
static mut COUNTER: i32 = 0;

// ✅ Good: 독립적
#[test]
fn test_independent() {
    let counter = AtomicI32::new(0);  // 테스트별 상태
}
```

---

## 7. Test Documentation

### 7.1 Test Case Specification

각 주요 기능에 대한 테스트 케이스 문서:

```markdown
## TC-001: FileHeader Parsing

### Preconditions
- Valid HWP 5.x file

### Test Steps
1. Open file
2. Read FileHeader stream
3. Parse 256 bytes

### Expected Results
- Version correctly parsed
- Properties flags correctly decoded

### Edge Cases
- [ ] Minimum valid header (256 bytes)
- [ ] Maximum version (255.255.255.255)
- [ ] All property flags set
```

### 7.2 Test Reports

CI에서 생성되는 리포트:

- **JUnit XML:** CI 통합용
- **HTML Coverage:** 상세 커버리지
- **Benchmark Report:** 성능 추이

---

## 8. Continuous Testing

### 8.1 CI Pipeline

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - name: Unit Tests
        run: cargo test --workspace

      - name: Integration Tests
        run: cargo test --workspace -- --ignored

      - name: Coverage
        run: cargo tarpaulin --workspace --fail-under 80
```

### 8.2 Pre-commit Hooks

```bash
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cargo-test
        name: cargo test
        entry: cargo test --workspace
        language: system
        pass_filenames: false
```

### 8.3 Test on PR

```yaml
# PR에서 실행되는 테스트
on:
  pull_request:
    paths:
      - 'crates/**/*.rs'
      - 'Cargo.toml'
      - 'Cargo.lock'
```

---

## 9. Test Maintenance

### 9.1 Regular Tasks

| Task | Frequency | Owner |
|------|-----------|-------|
| Flaky test 수정 | Weekly | @QA |
| 커버리지 리뷰 | Bi-weekly | @QA |
| 픽스처 업데이트 | As needed | @Dev |
| 벤치마크 분석 | Monthly | @Dev |

### 9.2 Test Deprecation

```rust
// 더 이상 유효하지 않은 테스트
#[test]
#[ignore = "Deprecated: Feature removed in v2.0"]
fn test_old_feature() { }

// TODO: 향후 추가할 테스트
#[test]
#[ignore = "TODO: Implement when table parsing is complete"]
fn test_complex_table() { }
```

### 9.3 Test Refactoring

- **중복 제거:** Helper 함수로 추출
- **가독성 개선:** AAA 패턴 적용
- **속도 개선:** 불필요한 I/O 제거

---

## 10. Test Checklist

### 10.1 Before PR

- [ ] 모든 새 코드에 테스트 작성
- [ ] `cargo test --workspace` 통과
- [ ] 커버리지 80% 이상 유지
- [ ] 새 에러 케이스 테스트 추가
- [ ] 성능 민감한 코드 벤치마크 추가

### 10.2 Code Review

- [ ] 테스트가 의미 있는 동작 검증
- [ ] 엣지 케이스 포함
- [ ] 명확한 테스트 이름
- [ ] AAA 패턴 준수
- [ ] 불필요한 의존성 없음

---

## Appendix: Test Commands Reference

```bash
# 기본 실행
cargo test                              # 전체
cargo test -p hwp-core                  # 특정 crate
cargo test test_name                    # 특정 테스트
cargo test -- --nocapture              # 출력 표시

# 필터링
cargo test parser::                     # 모듈 필터
cargo test -- --skip slow              # 제외
cargo test -- --ignored                # ignored만

# 병렬 제어
cargo test -- --test-threads=1         # 순차 실행

# 커버리지
cargo tarpaulin --workspace --out Html

# 벤치마크
cargo bench

# Fuzzing
cargo +nightly fuzz run parse_header

# Mutation
cargo mutants --workspace
```

---

**"Quality is not an act, it is a habit." - Aristotle**
