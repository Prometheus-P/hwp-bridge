# V1 Release Gate (Corpus-based)

“완벽” 같은 단어로 변명하지 말고, **코퍼스 기반 수치**로 릴리즈 합격/불합격을 판단한다.

이 레포는 저작권/민감정보 리스크 때문에 **HWP 문서를 커밋하지 않는다.**
대신 `corpus/manifest.json`(해시/메타) + `reports/`(결과)로 재현성을 만든다.

---

## 1) V1 합격 기준 요약

상세 기준은 `docs/specs/V1_ACCEPTANCE.md`.

- 코퍼스 총량: **100개 이상** (릴리즈 게이트 기준)
- 성공률(카테고리별):
  - A(기본): **≥ 95%**
  - B(표 위주): **≥ 85%**
  - C(혼합): **≥ 80%**
- 결정성(determinism): 동일 파일을 같은 바이너리로 변환했을 때 결과가 **항상 동일**
  - 기본 기준: `deterministic_rate ≥ 0.99`
- 안정성: panic 금지, 리소스 상한 초과는 정상 실패로 처리

---

## 2) 로컬에서 코퍼스 준비

1) 실제 문서는 `corpus/local/`에만 둔다 (gitignore).

```bash
mkdir -p corpus/local
# 여기에 .hwp 파일 복사
```

2) 매니페스트 생성/갱신

```bash
python3 scripts/corpus_scan.py
# corpus/manifest.json 생성/갱신
```

3) (권장) `corpus/manifest.json`에서 각 아이템의 `category`를 A/B/C로 분류

- A: 일반 문장/목록/간단 서식
- B: 표가 많은 문서
- C: 표 + 컨트롤 + 다국어 등 혼합/복잡

> 분류가 귀찮으면 당장은 생략해도 된다. 다만 릴리즈 판정이 “전체 성공률” 위주로 단순해진다.

---

## 3) 로컬에서 V1 게이트 실행

1) CLI 빌드

```bash
cargo build --release -p hwp-cli
```

2) V1 게이트 실행

```bash
python3 scripts/v1_gate.py --ci \
  --corpus-dir corpus/local \
  --manifest corpus/manifest.json \
  --hwp-bin target/release/hwp \
  --min-corpus-size 100
```

3) 결과 확인

- `reports/v1_gate/<timestamp>_summary.json`
- `reports/v1_gate/<timestamp>_details.jsonl`

요약(JSON)에는:
- 총 파일 수, 성공/실패, 카테고리별 성공률, 실패 원인 분포
- determinism 비율
- p50/p95/p99 처리 시간

---

## 4) 리소스 상한(운영 기본값)

코퍼스에 따라 “폭탄”이 섞일 수 있다. 상한은 안전장치다.

- 입력 크기 상한: `HWP_MAX_FILE_BYTES` (기본 25MB)
- 섹션 디컴프 상한: `HWP_MAX_DECOMPRESSED_BYTES_PER_SECTION` (기본 64MB)
- 섹션 레코드 상한: `HWP_MAX_RECORDS_PER_SECTION` (기본 200,000)

Smithery 배포 환경에서도 동일하게 적용할 수 있다.

---

## 5) GitHub Actions에서 게이트 켜기

레포에는 코퍼스를 올리지 않기 때문에, CI에서는 **비공개 ZIP**을 내려받아 실행한다.

### 5.1 Secrets 등록

GitHub → Settings → Secrets and variables → Actions

- `CORPUS_ZIP_URL`: 코퍼스 zip 다운로드 URL (예: 프리사인드 URL)
- (선택) `CORPUS_ZIP_SHA256`: zip 파일의 SHA256

> PR이 fork에서 열리면 secrets가 주입되지 않기 때문에, 그 경우 워크플로는 자동으로 스킵된다.

### 5.2 워크플로

이미 `.github/workflows/v1-gate.yml`이 포함되어 있다.

- secrets가 있으면: 다운로드 → `cargo build` → `scripts/v1_gate.py` 실행
- secrets가 없으면: 스킵하고 성공 처리

---

## 6) 트러블슈팅

- 성공률이 낮다
  - `details.jsonl`에서 `error` 타입별로 빈도를 보고, 상한 초과인지(정상) 파싱 실패인지(개선 대상)부터 구분한다.

- determinism이 깨진다
  - 출력에 타임스탬프/랜덤 값이 섞였는지 확인한다.
  - 정렬 순서(테이블/문단)와 공백 규칙을 고정한다.

- CI에서 시간이 너무 오래 걸린다
  - `--timeout-s`를 조절하거나, PR 단계에서는 `--max-files`로 샘플링하고 릴리즈에서만 전체를 돌린다.

