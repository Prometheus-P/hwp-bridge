# V1 Acceptance Criteria (Release Gate)

> 목적: “완벽” 같은 감정어 금지. **수치/재현/안정성**으로 합격 여부를 판정한다.  
> 대상: `hwp-core` / `hwp-cli` / `hwp-mcp` / `hwp-wasm`  
> 범위: **HWP 5.x (OLE/CFB)**, 비암호화, 비배포용 문서. (HWPX는 V2)

---

## 1) Release Scope (명시적)

### In-scope (V1에서 “지원”이라고 말해도 되는 것)
- HWP 5.x OLE 컨테이너 열기, `/FileHeader` 검증
- `BodyText/SectionN` 텍스트 추출
- 출력:
  - `plain-text` (텍스트)
  - `structured-json` (StructuredDocument)
  - `semantic-markdown` (LLM-friendly)

### Out-of-scope (V1에서 “지원”이라고 말하면 안 되는 것)
- 암호화 문서(Encrypted)
- 배포용 문서(DistributionOnly)
- 이미지 추출 / 정확한 레이아웃 렌더링
- 수식/각주/머리말/꼬리말/필드코드 “완전” 보존
- HWPX

---

## 2) Functional Gates (기능 합격 기준)

### 2.1 Parsing Success Rate (코퍼스 기반)
- 코퍼스 분류:
  - **A(기본)**: 일반 문장/목록/간단 서식
  - **B(표 위주)**: 표가 많은 문서
  - **C(혼합)**: 표 + 컨트롤 + 다국어 등

합격 기준:
- A: **성공률 ≥ 95%**
- B: **성공률 ≥ 85%**
- C: **성공률 ≥ 80%**

성공 정의:
- 프로그램이 **panic 없이** 종료
- `inspect` 결과에서 `is_encrypted=false`, `is_distribution=false`
- `extract/to_json/to_markdown` 중 최소 1개가 “유효 결과”를 반환
- (옵션) `warnings[]`는 허용

실패 정의(정상 실패 포함):
- `Encrypted`, `DistributionOnly`, `InvalidSignature`, `UnsupportedVersion`
- `SizeLimitExceeded` (보안상 정상)

### 2.2 Output Determinism (재현성)
- 동일 파일에 대해 **동일 버전의 바이너리**는 다음을 만족해야 한다:
  - `structured-json`의 SHA256이 **항상 동일**
  - `semantic-markdown`은 공백/개행을 포함해 **항상 동일** (권장)

### 2.3 Error Taxonomy (에러가 “예측 가능”해야 함)
- 암호화 문서 → `HwpError::Encrypted`
- 배포용 문서 → `HwpError::DistributionOnly`
- 디컴프 폭탄/상한 초과 → `HwpError::SizeLimitExceeded`
- 손상/파싱 실패 → `HwpError::ParseError` (원인 메시지 포함)

---

## 3) Non-Functional Gates (안정성/보안)

### 3.1 No Panic
- 모든 코퍼스 입력에서 **panic 금지**
- CI에서 `RUST_BACKTRACE=1`로 실행 (panic 발생 시 실패)

### 3.2 Resource Limits (DoS 방지)
기본 상한(권장):
- 입력 파일 크기: 25MB (`HWP_MAX_FILE_BYTES`)
- 섹션 디컴프 최대: 64MB (`HWP_MAX_DECOMPRESSED_BYTES_PER_SECTION`)
- 섹션 레코드 최대: 200_000 (`HWP_MAX_RECORDS_PER_SECTION`)
- (MCP) 처리 타임아웃: 3s~10s (환경에 따라 조정)

### 3.3 Smithery Runtime Smoke Test
- Smithery에서 설치/실행 후:
  - `hwp.inspect` → 정상 JSON
  - `hwp.to_markdown` → 결과 문자열 반환
  - 큰 파일/폭탄 입력 → **정상적으로 SizeLimitExceeded** 반환 (서버 유지)

---

## 4) Exit Criteria Checklist (릴리즈 체크)
- [ ] 코퍼스 100개 이상 / 카테고리 A,B,C 분포 확보
- [ ] Success rate 기준 달성
- [ ] Determinism 테스트 통과
- [ ] SizeLimitExceeded 테스트 통과
- [ ] README에 “지원/미지원” 표 업데이트
- [ ] CHANGELOG에 breaking change 여부 명시
