# Plan: Phase 1 - Core 엔진 기초

> **Status:** In Progress
> **Target:** hwp-core FileHeader 파싱 및 Fail-Fast 구현

---

## 목표

HWP 5.0 파일의 FileHeader를 파싱하여:
1. 파일 시그니처 검증
2. 버전 확인 (5.0 이상)
3. 암호화 문서 감지 → `HwpError::Encrypted`
4. 배포용 문서 감지 → `HwpError::DistributionOnly`

---

## HWP FileHeader 구조 (256 bytes)

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 32 | Signature | "HWP Document File" (null-padded) |
| 32 | 4 | Version | 파일 버전 (예: 5.1.0.1) |
| 36 | 4 | Properties | 속성 비트 플래그 |
| 40 | 216 | Reserved | 예약 영역 |

### Properties Bit Flags

| Bit | Description |
|-----|-------------|
| 0 | 압축 여부 |
| 1 | 암호화 여부 |
| 2 | 배포용 문서 |
| 3 | 스크립트 저장 |
| 4 | DRM 보안 |
| 5 | XMLTemplate 스토리지 존재 |
| 6 | 문서 이력 관리 |
| 7 | 전자 서명 정보 존재 |
| 8 | 공인 인증서 암호화 |
| 9 | 전자 서명 예비 저장 |
| 10 | 공인 인증서 DRM |
| 11 | CCL 문서 |
| 12 | 모바일 최적화 |
| 13 | 개인 정보 보안 |
| 14 | 변경 추적 |
| 15 | 공공누리(KOGL) 저작권 |

---

## 구현 계획

### Step 1: hwp-types 확장
- [ ] `FileHeader` 구조체 추가
- [ ] `HwpVersion` 구조체 추가
- [ ] `DocumentProperties` bitflags 추가

### Step 2: hwp-core 파싱 로직
- [ ] OLE 컨테이너 열기 (`cfb` crate)
- [ ] "FileHeader" 스트림 읽기
- [ ] 바이트 파싱 → `FileHeader` 구조체
- [ ] Fail-Fast 검증 로직

### Step 3: 테스트
- [ ] 정상 HWP 파일 파싱 테스트
- [ ] 암호화 문서 감지 테스트
- [ ] 배포용 문서 감지 테스트
- [ ] 잘못된 시그니처 테스트

---

## 파일 변경 목록

```
crates/hwp-types/src/lib.rs    # FileHeader, HwpVersion 추가
crates/hwp-core/src/lib.rs     # 모듈 구조 재구성
crates/hwp-core/src/parser/    # 새 디렉토리
  ├── mod.rs
  ├── header.rs                # FileHeader 파싱
  └── ole.rs                   # OLE 컨테이너 처리
```

---

## TDD Cycle

```
RED    → 실패하는 테스트 작성
GREEN  → 최소 구현으로 테스트 통과
REFACTOR → 코드 정리
COMMIT → 커밋
```
