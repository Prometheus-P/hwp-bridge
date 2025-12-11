# CLAUDE.md - HWP Bridge 프로젝트 AI 작업 규칙

> 이 파일은 AI 어시스턴트가 프로젝트 작업 시 반드시 따라야 할 규칙을 정의합니다.
>
> **프로젝트 유형:** 1인 개발
> **상위 규칙:** [.specify/memory/constitution.md](.specify/memory/constitution.md)

---

## 1. Git Workflow

### Branch 정책
- **main/dev 직접 push 금지** - 항상 feature 브랜치 사용
- Branch naming: `feature/`, `fix/`, `refactor/`, `docs/`, `test/`, `chore/`

```bash
# 올바른 워크플로우
git checkout -b feature/add-table-parsing
# ... 작업 ...
git push origin feature/add-table-parsing
# main에 머지 시 PR 또는 로컬 머지 후 push
```

### 커밋/머지 전 체크리스트
- [ ] `cargo test --workspace` 통과
- [ ] `cargo fmt --all` 실행
- [ ] `cargo clippy --workspace` 경고 없음
- [ ] 관련 스펙 문서 업데이트

---

## 2. Spec-First 개발

- **스펙 문서가 Source of Truth**
- 코드 작성 전 반드시 관련 스펙 문서 확인/업데이트
- 스펙 문서 위치: `docs/` 또는 `.specify/`

### 스펙 문서 우선순위
1. `.specify/memory/constitution.md` (프로젝트 헌법 - 최상위)
2. `.specify/` 내 feature spec
3. `docs/` 내 기술 문서
4. `CONTEXT.md` (프로젝트 개요)

---

## 3. TDD 필수 (NON-NEGOTIABLE)

> 상세 규칙: constitution.md - II. Test-Driven Development

### Red-Green-Refactor 사이클
```
RED → GREEN → REFACTOR → COMMIT
```

1. **RED:** 실패하는 테스트 먼저 작성
2. **GREEN:** 테스트 통과하는 최소 구현
3. **REFACTOR:** 코드 개선 (테스트 유지)
4. **COMMIT:** 커밋 전 반드시 테스트 통과 확인

### 테스트 네이밍
```rust
#[test]
fn test_should_{expected}_when_{condition}() {
    // Arrange
    // Act
    // Assert
}
```

### 커밋 전 필수 명령
```bash
cargo test --workspace
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

### 핵심 규칙
- **hwp-core 테스트 커버리지 ≥80%**
- Tidy 커밋(리팩토링)과 Behavior 커밋(기능) 분리
- `.unwrap()` / `.expect()` 라이브러리 코드에서 금지

---

## 4. 아키텍처 패턴

> 상세 규칙: constitution.md - I. Crate-Based Architecture

### Crate 의존성 (필수 준수)
```
hwp-types (base) → hwp-core (parser) → hwp-cli / hwp-web / hwp-mcp
```

**핵심 규칙:**
- `hwp-types`: 타입, 에러 정의만
- `hwp-core`: 모든 파싱/변환 로직 (애플리케이션 crate에서 파싱 구현 금지)
- 애플리케이션 crate: `hwp-core` 기능을 노출하는 thin wrapper
- **순환 의존성 금지**

### Parser/Strategy 패턴 (hwp-core)
```
hwp-core/
├── parser/         # 파일 타입별 파서
│   ├── header.rs
│   ├── section.rs
│   └── ...
└── extractor/      # 추출 전략
    ├── text.rs
    └── ...
```

### Fail-Fast 검증 (필수)
- FileHeader 먼저 파싱/검증
- 암호화 문서 → `HwpError::Encrypted` 즉시 반환
- 배포용 문서 → `HwpError::DistributionOnly` 즉시 반환

---

## 5. 환경 설정

### 단일 .env 정책
- 루트에 `.env.example` 제공
- 서비스별 환경변수는 prefix로 구분: `HWP_WEB_`, `HWP_MCP_`
- 실제 `.env`는 gitignore 처리

```bash
# .env.example
HWP_WEB_PORT=8080
HWP_MCP_PORT=3000
```

---

## 6. AI 결과 처리

### Preview-Only 원칙
- AI가 생성/수정한 코드는 항상 **Preview 상태**
- **사람이 최종 검토 및 승인 책임**
- AI는 제안만 하고, 최종 판단은 사람이 수행

### AI 작업 시 주의사항
- 대규모 리팩토링 전 반드시 사용자 확인
- 보안 관련 코드 변경 시 명시적 경고
- 삭제/덮어쓰기 전 확인
- constitution.md 원칙 위반 시 즉시 알림

---

## 7. 보안 및 감사

### Audit Log
- 중요 작업은 로그 기록
- 파일 변경 이력 추적

### 데이터 격리
- 케이스(문서) 단위 데이터 격리
- 파일 해시로 무결성 검증

### 금지 사항
- 하드코딩된 credential 커밋 금지
- `.env`, `credentials.json` 등 민감 파일 커밋 금지

---

## 8. 프로젝트 구조

```
hwp-bridge/
├── Cargo.toml              # Workspace manifest
├── CLAUDE.md               # AI 작업 규칙 (이 파일)
├── CONTEXT.md              # 프로젝트 컨텍스트
├── CONTRIBUTING.md         # 기여 가이드
├── .env.example            # 환경변수 템플릿
└── crates/
    ├── hwp-types/          # 공용 타입, 에러 정의
    ├── hwp-core/           # 핵심 파싱 로직
    ├── hwp-cli/            # CLI 인터페이스
    ├── hwp-web/            # Web API 서버 (Axum)
    └── hwp-mcp/            # MCP 서버
```

---

## 9. 커밋 메시지 규칙

```
<type>(<scope>): <subject>
```

### Types
| Type | Description |
|------|-------------|
| `feat` | 새 기능 |
| `fix` | 버그 수정 |
| `docs` | 문서 변경 |
| `refactor` | 리팩토링 |
| `test` | 테스트 추가/수정 |
| `chore` | 빌드, CI 등 |

### Scopes
| Scope | Crate |
|-------|-------|
| `types` | hwp-types |
| `core` | hwp-core |
| `cli` | hwp-cli |
| `web` | hwp-web |
| `mcp` | hwp-mcp |

---

## 10. 빠른 참조

```bash
# 빌드
cargo build --workspace

# 테스트
cargo test --workspace

# 포맷
cargo fmt --all

# 린트
cargo clippy --workspace

# CLI 실행
cargo run -p hwp-cli -- --help

# Web 서버 실행
cargo run -p hwp-web
```

---

**Last Updated:** 2025-12-11

## Active Technologies
- Rust 1.85+ (2024 Edition) + serde (직렬화), thiserror (에러 정의) (001-hwp-types-impl)
- N/A (타입 정의만, 저장 로직 없음) (001-hwp-types-impl)

## Recent Changes
- 001-hwp-types-impl: Added Rust 1.85+ (2024 Edition) + serde (직렬화), thiserror (에러 정의)
