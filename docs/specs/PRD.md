> ⚠️ Note (Option A): `hwp-web` is intentionally excluded/disabled in this repo snapshot.
> This document may still mention `hwp-web` as a *planned* component.

# PRD: HwpBridge - HWP to Google Docs Converter

> **Version:** 1.0.0
> **Author:** @PM
> **Status:** Approved
> **Last Updated:** 2025-12-09

---

## 1. Executive Summary

HwpBridge는 한글(HWP) 문서를 Google Docs로 변환하는 고성능 Rust 기반 솔루션입니다. 웹 인터페이스와 MCP(Model Context Protocol) 서버를 통해 일반 사용자와 AI 에이전트 모두에게 서비스를 제공합니다.

---

## 2. Problem Statement

### 2.1 Current Pain Points

| Pain Point | Impact | Affected Users |
|------------|--------|----------------|
| HWP 파일 열람 불가 | 비한컴오피스 환경에서 문서 접근 불가 | 해외 사용자, Mac/Linux 사용자 |
| 수동 변환 필요 | 한컴오피스 설치 후 별도 저장 필요 | 모든 사용자 |
| AI 문서 분석 불가 | LLM이 HWP 내용 직접 접근 불가 | AI 개발자, Claude/Cursor 사용자 |
| 대용량 파일 처리 지연 | 기존 Python 파서의 성능 한계 | 대량 문서 처리 사용자 |

### 2.2 Target Users

1. **일반 사용자:** HWP 파일을 Google Docs로 변환하려는 사용자
2. **개발자:** HWP 파싱 기능을 애플리케이션에 통합하려는 개발자
3. **AI 사용자:** Claude, Cursor 등에서 HWP 문서를 분석하려는 사용자

---

## 3. Goals & Success Metrics

### 3.1 Goals

| Priority | Goal | Description |
|----------|------|-------------|
| P0 | 텍스트 추출 | HWP 5.0+ 문서에서 본문 텍스트 100% 추출 |
| P0 | Fail-Fast | 암호화/배포용 문서 즉시 감지 및 거부 |
| P1 | 스타일 보존 | Bold, Italic, 글꼴 크기 등 기본 서식 유지 |
| P1 | Google Docs 변환 | HTML → Google Docs 자동 업로드 |
| P2 | 이미지 추출 | 임베디드 이미지 Base64 인코딩 |
| P2 | 표 변환 | 표 구조 및 셀 병합 유지 |
| P3 | MCP 서버 | AI 에이전트용 Tool 인터페이스 |

### 3.2 Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| 파싱 성공률 | ≥ 95% (비암호화 문서) | 테스트 문서 셋 |
| 변환 속도 | < 3초 (10MB 이하) | 벤치마크 |
| 메모리 사용 | < 100MB (일반 문서) | 프로파일링 |
| API 응답 시간 | < 5초 (p95) | 모니터링 |

---

## 4. User Stories

### 4.1 Epic: 문서 변환

```
US-001: HWP 파일 업로드 및 변환
As a 일반 사용자
I want to HWP 파일을 웹에서 업로드
So that Google Docs로 변환된 문서를 받을 수 있다

Acceptance Criteria:
- [ ] Drag & Drop 파일 업로드 지원
- [ ] 10MB 이하 파일 업로드 가능
- [ ] 변환 진행률 표시
- [ ] Google Docs 링크 반환
- [ ] 에러 시 사용자 친화적 메시지 표시
```

```
US-002: 암호화 문서 감지
As a 사용자
I want to 암호화된 문서 업로드 시 즉시 알림
So that 불필요한 대기 없이 다른 방법을 찾을 수 있다

Acceptance Criteria:
- [ ] 파일 헤더 분석 후 2초 내 암호화 여부 판단
- [ ] "이 문서는 암호화되어 있습니다" 메시지 표시
- [ ] 배포용 문서도 동일하게 처리
```

### 4.2 Epic: AI 연동

```
US-003: MCP Tool로 HWP 읽기
As a AI 개발자
I want to Claude에서 MCP Tool로 HWP 내용 조회
So that HWP 문서 기반 질의응답이 가능하다

Acceptance Criteria:
- [ ] read_hwp_summary Tool: 메타데이터 반환
- [ ] read_hwp_content Tool: Markdown 본문 반환
- [ ] convert_to_gdocs Tool: Google Docs 링크 반환
```

### 4.3 Epic: CLI 도구

```
US-004: 로컬 파일 변환
As a 개발자
I want to CLI로 HWP 파일을 변환
So that 자동화 스크립트에서 활용할 수 있다

Acceptance Criteria:
- [ ] hwp-cli convert input.hwp -o output.html
- [ ] hwp-cli info input.hwp (메타데이터 출력)
- [ ] hwp-cli extract-images input.hwp -d ./images
- [ ] Exit code로 성공/실패 구분
```

---

## 5. Functional Requirements

### 5.1 HWP 파싱 (hwp-core)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-001 | OLE2 컨테이너 열기 및 스트림 추출 | P0 |
| FR-002 | FileHeader 파싱 및 버전 검증 | P0 |
| FR-003 | 암호화/배포용 문서 Fail-Fast | P0 |
| FR-004 | DocInfo 스트림 파싱 (스타일 정보) | P1 |
| FR-005 | BodyText/Section 압축 해제 | P0 |
| FR-006 | HWPTAG_PARA_TEXT 텍스트 추출 | P0 |
| FR-007 | HWPTAG_CHAR_SHAPE 스타일 매핑 | P1 |
| FR-008 | HWPTAG_TABLE 표 구조 파싱 | P2 |
| FR-009 | BinData 이미지 추출 | P2 |

### 5.2 웹 서비스 (hwp-web) — planned (disabled)

> Option A에서는 `hwp-web`을 포함하지 않습니다.
> `hwp-web` 요구사항은 아래 문서로 이동했습니다:
> - ../../future/hwp-web/PRD_WEB.md

### 5.3 MCP 서버 (hwp-mcp)

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-015 | Stdio 기반 MCP 프로토콜 구현 | P1 |
| FR-016 | read_hwp_summary Tool | P1 |
| FR-017 | read_hwp_content Tool | P1 |
| FR-018 | convert_to_gdocs Tool | P2 |

---

## 6. Non-Functional Requirements

### 6.1 Performance

| Metric | Requirement |
|--------|-------------|
| 파싱 속도 | 1MB당 < 100ms |
| 메모리 피크 | 파일 크기의 3배 이하 |
| 동시 처리 | 100 concurrent requests |

### 6.2 Security

| Requirement | Description |
|-------------|-------------|
| 파일 검증 | 업로드 파일 시그니처 검증 |
| 임시 파일 | 처리 후 즉시 삭제 |
| OAuth 토큰 | 사용자별 토큰, 서버 저장 금지 |
| 로깅 | 민감 정보 마스킹 |

### 6.3 Reliability

| Metric | Target |
|--------|--------|
| Uptime | 99.9% |
| Error Rate | < 0.1% |
| Recovery Time | < 30초 |

---

## 7. Out of Scope (v1.0)

- HWPX (OOXML 기반) 지원
- HWP 수식 → LaTeX 변환
- 다국어 OCR
- 실시간 협업 기능
- 온프레미스 설치형 버전

---

## 8. RICE Prioritization

| Feature | Reach | Impact | Confidence | Effort | Score |
|---------|-------|--------|------------|--------|-------|
| 텍스트 추출 | 10 | 10 | 9 | 3 | 300 |
| Fail-Fast | 10 | 8 | 10 | 1 | 800 |
| 스타일 보존 | 8 | 7 | 7 | 4 | 98 |
| Google Docs 변환 | 9 | 9 | 8 | 5 | 130 |
| 표 변환 | 6 | 6 | 6 | 6 | 36 |
| MCP 서버 | 4 | 9 | 8 | 4 | 72 |

---

## 9. Dependencies

### 9.1 External Services

| Service | Purpose | Risk |
|---------|---------|------|
| Google Drive API | 문서 업로드 | API Quota 제한 |
| Google OAuth | 사용자 인증 | 인증 복잡성 |

### 9.2 Technical Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| cfb | 0.12 | OLE2 파싱 |
| nom | 8.0 | 바이너리 파싱 |
| flate2 | 1.1 | zlib 압축 해제 |
| axum | 0.8 | Web 프레임워크 |
| tokio | 1.48 | Async 런타임 |

---

## 10. Milestones

| Milestone | Target | Deliverables |
|-----------|--------|--------------|
| M1: Core Parser | Week 1 | FileHeader, 텍스트 추출 |
| M2: HTML Converter | Week 3 | 스타일 매핑, 표/이미지 |
| M3: Web Service | Week 5 | REST API, Google 연동 |
| M4: MCP Server | Week 5 | MCP Tools |
| M5: Release | Week 6 | Docker, 문서화 |

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| HWP | Hangul Word Processor 파일 포맷 |
| OLE | Object Linking and Embedding |
| MCP | Model Context Protocol |
| HWPTAG | HWP 내부 바이너리 레코드 태그 |

---

**Approved by:** @PM
**Date:** 2025-12-09
