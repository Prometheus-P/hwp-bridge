
# TDD Guide - HwpBridge

> **Version:** 1.0.0
> **Author:** @QA
> **Last Updated:** 2025-12-09

---

## 1. TDD 원칙

### 1.1 Core Cycle

```
┌─────────────────────────────────────────────────────────────┐
│                      TDD Cycle                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│     ┌─────────┐                                             │
│     │   RED   │  Write a failing test                       │
│     └────┬────┘                                             │
│          │                                                  │
│          ▼                                                  │
│     ┌─────────┐                                             │
│     │  GREEN  │  Write minimal code to pass                 │
│     └────┬────┘                                             │
│          │                                                  │
│          ▼                                                  │
│     ┌─────────┐                                             │
│     │REFACTOR │  Clean up, keep tests passing               │
│     └────┬────┘                                             │
│          │                                                  │
│          ▼                                                  │
│     ┌─────────┐                                             │
│     │ COMMIT  │  Save progress                              │
│     └─────────┘                                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Three Laws of TDD

1. **테스트 없이 프로덕션 코드를 작성하지 않는다**
2. **실패하는 테스트가 있을 때만 프로덕션 코드를 작성한다**
3. **테스트를 통과하는 최소한의 코드만 작성한다**

### 1.3 Tidy First

**구조 변경(Tidy)과 기능 변경(Behavior)을 분리합니다:**

```bash
# ❌ Bad: 하나의 커밋에 섞임
git commit -m "feat: add table parsing and refactor record module"

# ✅ Good: 분리된 커밋
git commit -m "refactor(core): extract RecordParser trait"
git commit -m "feat(core): add table parsing"
```

---

## 2. 테스트 작성 가이드

### 2.1 Test Naming Convention

```rust
#[test]
fn test_should_{expected_behavior}_when_{condition}() {
    // ...
}
```

**Examples:**

```rust
// Good
fn test_should_parse_header_when_signature_valid() { }
fn test_should_return_error_when_file_encrypted() { }
fn test_should_extract_text_when_section_compressed() { }

// Bad
fn test_parse_header() { }  // 무엇을 테스트하는지 불명확
fn test_error() { }         // 어떤 에러인지 불명확
fn test1() { }              // 의미 없음
```

### 2.2 Test Structure (AAA Pattern)

```rust
#[test]
fn test_should_parse_valid_header_when_signature_matches() {
    // ═══════════════════════════════════════════════════════
    // Arrange (Given) - 테스트 데이터 준비
    // ═══════════════════════════════════════════════════════
    let data = create_test_header(
        [0, 0, 1, 5],  // version 5.1.0.0
        0b0001,        // compressed only
    );

    // ═══════════════════════════════════════════════════════
    // Act (When) - 테스트 대상 실행
    // ═══════════════════════════════════════════════════════
    let result = parse_file_header(&data);

    // ═══════════════════════════════════════════════════════
    // Assert (Then) - 결과 검증
    // ═══════════════════════════════════════════════════════
    assert!(result.is_ok());
    let header = result.unwrap();
    assert_eq!(header.version.major, 5);
    assert!(header.properties.is_compressed());
}
```

### 2.3 Test Helpers

```rust
// crates/hwp-core/src/parser/header.rs

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════
    // Test Helpers
    // ═══════════════════════════════════════════════════════

    /// 테스트용 FileHeader 바이트 배열 생성
    fn create_test_header(version: [u8; 4], properties: u32) -> Vec<u8> {
        let mut data = vec![0u8; FILE_HEADER_SIZE];
        data[0..32].copy_from_slice(HWP_SIGNATURE);
        data[32..36].copy_from_slice(&version);
        data[36..40].copy_from_slice(&properties.to_le_bytes());
        data
    }

    /// 암호화된 문서용 테스트 헤더
    fn create_encrypted_header() -> Vec<u8> {
        create_test_header([0, 0, 1, 5], 0b0010)  // bit 1 = encrypted
    }

    /// 배포용 문서용 테스트 헤더
    fn create_distribution_header() -> Vec<u8> {
        create_test_header([0, 0, 1, 5], 0b0100)  // bit 2 = distribution
    }

    // ═══════════════════════════════════════════════════════
    // Tests
    // ═══════════════════════════════════════════════════════

    #[test]
    fn test_should_parse_valid_header() { ... }
}
```

### 2.4 Error Testing

```rust
#[test]
fn test_should_return_encrypted_error_when_document_encrypted() {
    // Arrange
    let data = create_encrypted_header();

    // Act
    let header = parse_file_header(&data).unwrap();
    let result = header.validate();

    // Assert - 에러 타입 매칭
    assert!(matches!(result, Err(HwpError::Encrypted)));
}

#[test]
fn test_should_return_parse_error_with_message_when_data_too_short() {
    // Arrange
    let data = vec![0u8; 100];  // Too short

    // Act
    let result = parse_file_header(&data);

    // Assert - 에러 메시지 확인
    match result {
        Err(HwpError::ParseError(msg)) => {
            assert!(msg.contains("too short"));
        }
        _ => panic!("Expected ParseError"),
    }
}
```

### 2.5 Parameterized Tests

```rust
#[test]
fn test_should_detect_all_property_flags() {
    let test_cases = [
        (0b0000_0001, "compressed", |p: &DocumentProperties| p.is_compressed()),
        (0b0000_0010, "encrypted", |p: &DocumentProperties| p.is_encrypted()),
        (0b0000_0100, "distribution", |p: &DocumentProperties| p.is_distribution()),
        (0b0000_1000, "script", |p: &DocumentProperties| p.has_script()),
        (0b0001_0000, "drm", |p: &DocumentProperties| p.has_drm()),
    ];

    for (bits, name, check_fn) in test_cases {
        let props = DocumentProperties::from_bits(bits);
        assert!(
            check_fn(&props),
            "Property '{}' should be true for bits {:08b}",
            name,
            bits
        );
    }
}
```

---

## 3. TDD 실전 예제

### 3.1 새 기능 추가: RecordHeader 파싱

**Step 1: RED - 실패하는 테스트 작성**

```rust
// crates/hwp-core/src/parser/record.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_record_header_when_size_normal() {
        // Arrange
        // Tag: 0x043 (PARA_TEXT), Level: 0, Size: 100
        // Binary: tttt_tttt_ttll_llll_llll_ssss_ssss_ssss
        let data: [u8; 4] = [
            0x43,        // tag low 8 bits
            0x00,        // tag high 2 bits + level low 6 bits
            0x64,        // size low 8 bits (100)
            0x00,        // size high 4 bits
        ];

        // Act
        let result = RecordHeader::parse(&data);

        // Assert
        assert!(result.is_ok());
        let (header, consumed) = result.unwrap();
        assert_eq!(header.tag_id, 0x43);
        assert_eq!(header.level, 0);
        assert_eq!(header.size, 100);
        assert_eq!(consumed, 4);
    }
}
```

**Step 2: GREEN - 테스트 통과하는 최소 구현**

```rust
// crates/hwp-core/src/parser/record.rs

pub struct RecordHeader {
    pub tag_id: u16,
    pub level: u16,
    pub size: u32,
}

impl RecordHeader {
    pub fn parse(data: &[u8]) -> Result<(Self, usize), HwpError> {
        if data.len() < 4 {
            return Err(HwpError::ParseError("Record header too short".into()));
        }

        let dword = u32::from_le_bytes(data[0..4].try_into().unwrap());

        let tag_id = (dword & 0x3FF) as u16;           // bits 0-9
        let level = ((dword >> 10) & 0x3FF) as u16;    // bits 10-19
        let size = (dword >> 20) & 0xFFF;              // bits 20-31

        Ok((Self { tag_id, level, size }, 4))
    }
}
```

**Step 3: 추가 테스트 - Extended Size**

```rust
#[test]
fn test_should_parse_extended_size_when_size_is_4095() {
    // Arrange
    // Size = 4095 (0xFFF) means extended size in next 4 bytes
    let mut data = vec![0u8; 8];
    data[0] = 0x43;                              // tag
    data[1] = 0x00;                              // level
    data[2] = 0xFF;                              // size = 4095
    data[3] = 0xFF;                              // (indicates extended)
    data[4..8].copy_from_slice(&10000u32.to_le_bytes());  // actual size

    // Act
    let result = RecordHeader::parse(&data);

    // Assert
    let (header, consumed) = result.unwrap();
    assert_eq!(header.size, 10000);
    assert_eq!(consumed, 8);  // 4 + 4 extended
}
```

**Step 4: REFACTOR - 코드 정리**

```rust
impl RecordHeader {
    const HEADER_SIZE: usize = 4;
    const EXTENDED_SIZE_MARKER: u32 = 0xFFF;

    pub fn parse(data: &[u8]) -> Result<(Self, usize), HwpError> {
        Self::validate_length(data, Self::HEADER_SIZE)?;

        let dword = Self::read_u32_le(data);
        let (tag_id, level, size) = Self::decode_header(dword);

        if size == Self::EXTENDED_SIZE_MARKER {
            Self::parse_extended(data, tag_id, level)
        } else {
            Ok((Self { tag_id, level, size }, Self::HEADER_SIZE))
        }
    }

    fn decode_header(dword: u32) -> (u16, u16, u32) {
        let tag_id = (dword & 0x3FF) as u16;
        let level = ((dword >> 10) & 0x3FF) as u16;
        let size = (dword >> 20) & 0xFFF;
        (tag_id, level, size)
    }

    fn parse_extended(data: &[u8], tag_id: u16, level: u16) -> Result<(Self, usize), HwpError> {
        Self::validate_length(data, 8)?;
        let size = Self::read_u32_le(&data[4..]);
        Ok((Self { tag_id, level, size }, 8))
    }

    // ... helper methods
}
```

**Step 5: COMMIT**

```bash
git add .
git commit -m "feat(core): add RecordHeader parsing

- Parse 4-byte record header (tag, level, size)
- Support extended size for records > 4095 bytes
- Add comprehensive unit tests

Closes #15"
```

---

## 4. 테스트 카테고리

### 4.1 Unit Tests

```rust
// 파일 내 #[cfg(test)] 모듈
// 개별 함수/메서드의 동작 검증

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_display() {
        let v = HwpVersion::new(5, 1, 0, 0);
        assert_eq!(v.to_string(), "5.1.0.0");
    }
}
```

### 4.2 Integration Tests

```rust
// crates/hwp-core/tests/integration.rs
// 여러 모듈 간의 상호작용 검증

use hwp_core::{HwpOleFile, HwpError};
use std::fs::File;

#[test]
fn test_should_parse_real_hwp_file() {
    // fixtures/sample.hwp 필요
    let file = File::open("tests/fixtures/sample.hwp").unwrap();
    let hwp = HwpOleFile::open(file).unwrap();

    assert_eq!(hwp.header().version.major, 5);
}

#[test]
fn test_should_reject_encrypted_file() {
    let file = File::open("tests/fixtures/encrypted.hwp").unwrap();
    let result = HwpOleFile::open(file);

    assert!(matches!(result, Err(HwpError::Encrypted)));
}
```

### 4.3 Property-Based Tests (Optional)

```rust
// Cargo.toml: proptest = "1.0"

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_version_roundtrip(
        major in 0u8..=255,
        minor in 0u8..=255,
        build in 0u8..=255,
        revision in 0u8..=255,
    ) {
        let version = HwpVersion::new(major, minor, build, revision);
        let bytes = version.to_bytes();
        let parsed = HwpVersion::from_bytes(bytes);

        prop_assert_eq!(version, parsed);
    }
}
```

---

## 5. Mocking & Test Doubles

### 5.1 Trait-Based Mocking

```rust
// 프로덕션 코드
pub trait FileReader {
    fn read_stream(&mut self, path: &str) -> Result<Vec<u8>, HwpError>;
}

pub struct HwpParser<R: FileReader> {
    reader: R,
}

impl<R: FileReader> HwpParser<R> {
    pub fn parse_doc_info(&mut self) -> Result<DocInfo, HwpError> {
        let data = self.reader.read_stream("/DocInfo")?;
        // parse...
    }
}

// 테스트 코드
#[cfg(test)]
mod tests {
    struct MockReader {
        streams: HashMap<String, Vec<u8>>,
    }

    impl FileReader for MockReader {
        fn read_stream(&mut self, path: &str) -> Result<Vec<u8>, HwpError> {
            self.streams
                .get(path)
                .cloned()
                .ok_or_else(|| HwpError::OleError("Not found".into()))
        }
    }

    #[test]
    fn test_parse_doc_info() {
        let mut reader = MockReader {
            streams: HashMap::from([
                ("/DocInfo".into(), create_doc_info_data()),
            ]),
        };

        let mut parser = HwpParser { reader };
        let result = parser.parse_doc_info();

        assert!(result.is_ok());
    }
}
```

### 5.2 Test Fixtures

```
crates/hwp-core/tests/
├── fixtures/
│   ├── sample.hwp           # 정상 문서
│   ├── encrypted.hwp        # 암호화 문서
│   ├── distribution.hwp     # 배포용 문서
│   ├── v3.hwp               # 구버전 (지원 안함)
│   ├── with_table.hwp       # 표 포함
│   └── with_images.hwp      # 이미지 포함
└── integration.rs
```

---

## 6. 테스트 실행

### 6.1 Commands

```bash
# 전체 테스트
cargo test --workspace

# 특정 crate
cargo test -p hwp-core

# 특정 테스트
cargo test test_should_parse_valid_header

# 특정 모듈
cargo test parser::header

# 출력 포함
cargo test -- --nocapture

# 병렬 실행 제한
cargo test -- --test-threads=1

# 무시된 테스트 포함
cargo test -- --include-ignored
```

### 6.2 nextest (더 빠른 테스트)

```bash
# 설치
cargo install cargo-nextest

# 실행
cargo nextest run --workspace

# 실패한 테스트만 재실행
cargo nextest run --workspace --failed
```

### 6.3 Watch Mode

```bash
# 파일 변경 시 자동 테스트
cargo watch -x "test --workspace"

# 특정 테스트만
cargo watch -x "test -p hwp-core parser::header"
```

---

## 7. 커버리지

### 7.1 cargo-tarpaulin

```bash
# 설치
cargo install cargo-tarpaulin

# 실행
cargo tarpaulin --workspace --out Html

# 임계값 설정
cargo tarpaulin --workspace --fail-under 80
```

### 7.2 Coverage Report

```
┌─────────────────────────────────────────────────────────────┐
│                    Coverage Report                          │
├─────────────────────────────────────────────────────────────┤
│  Crate          │ Lines  │ Covered │ Coverage              │
├─────────────────┼────────┼─────────┼───────────────────────┤
│  hwp-types      │   150  │   142   │   94.7%  ✓           │
│  hwp-core       │   800  │   720   │   90.0%  ✓           │
│  hwp-cli        │   100  │    75   │   75.0%  ⚠           │
│  hwp-mcp        │   150  │   120   │   80.0%  ✓           │
├─────────────────┼────────┼─────────┼───────────────────────┤
│  Total          │  1400  │  1217   │   86.9%  ✓           │
└─────────────────────────────────────────────────────────────┘

Target: ≥ 80% (core logic)
```

---

## 8. CI Integration

```yaml
# .github/workflows/ci.yml

test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Run tests
      run: cargo test --workspace --all-features

coverage:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin
    - name: Run coverage
      run: cargo tarpaulin --workspace --fail-under 80
```

---

## 9. Anti-Patterns

### 9.1 피해야 할 패턴

```rust
// ❌ Bad: 테스트 없이 구현
fn parse_header(data: &[u8]) -> Result<Header, Error> {
    // 바로 구현...
}

// ❌ Bad: 너무 많은 assert
#[test]
fn test_everything() {
    // 100줄의 테스트...
    assert!(a);
    assert!(b);
    // ...
    assert!(z);
}

// ❌ Bad: 테스트 간 의존성
static mut GLOBAL_STATE: i32 = 0;

#[test]
fn test_1() {
    unsafe { GLOBAL_STATE = 1; }
}

#[test]
fn test_2() {
    // test_1이 먼저 실행되어야 함
    unsafe { assert_eq!(GLOBAL_STATE, 1); }
}

// ❌ Bad: 불명확한 테스트 이름
#[test]
fn test() { }
#[test]
fn test2() { }
```

### 9.2 권장 패턴

```rust
// ✅ Good: 테스트 먼저
#[test]
fn test_should_parse_header() {
    // 먼저 작성
}

fn parse_header(data: &[u8]) -> Result<Header, Error> {
    // 나중에 구현
}

// ✅ Good: 하나의 테스트, 하나의 검증
#[test]
fn test_should_parse_tag_id() {
    let header = parse(...);
    assert_eq!(header.tag_id, expected);
}

#[test]
fn test_should_parse_level() {
    let header = parse(...);
    assert_eq!(header.level, expected);
}

// ✅ Good: 독립적인 테스트
#[test]
fn test_independent_1() {
    let data = create_fresh_data();
    // ...
}

#[test]
fn test_independent_2() {
    let data = create_fresh_data();
    // ...
}
```

---

**"Test First. Code Second. Refactor Always."**
