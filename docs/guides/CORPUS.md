# Corpus Design (V1)

> 원칙: **레포에 HWP 파일을 직접 커밋하지 않는다.** (저작권/민감정보/용량 문제)  
> 대신 “매니페스트 + 해시 + 생성 리포트”로 재현성을 만든다.

---

## 1) 폴더 구조

- `corpus/`
  - `manifest.json` : 코퍼스 메타데이터(추적/재현용)
  - `README.md` : 수집/정리 규칙
  - `local/` : (gitignore) 실제 파일 저장 위치
- `reports/`
  - `YYYYMMDD-run.json` : 러너 결과 (gitignore 권장)
- `scripts/`
  - `corpus_scan.py` : 로컬 파일 스캔 → manifest 생성/갱신
  - `corpus_run.sh` : CLI/MCP로 배치 실행

---

## 2) 수집 규칙 (법/리스크 최소화)
- “공개 배포” 문서라도 라이선스/저작권 확인
- 민감정보(주민번호/연락처/주소) 포함된 문서 금지
- 가능한 출처:
  - 공개 양식(정부/기관) 중 **재배포 허용** 문서
  - 본인이 작성한 테스트 문서(추천)
  - 회사 내부 문서는 레포 외부에서만 관리

---

## 3) manifest.json 스키마 (권장)
각 항목:
- `id`: 문자열(파일명에서 자동 생성 가능)
- `sha256`
- `size_bytes`
- `category`: A/B/C
- `flags`: { compressed, encrypted, distribution }
- `source`: { url, license_note }
- `notes`

---

## 4) 최소 코퍼스 권장량
- A: 60개
- B: 25개
- C: 15개
총 100개 이상에서 V1 합격 판정.

---

## 5) 골든 Fixtures (레포에 포함 가능한 최소 샘플)
레포에 포함할 수 있는 샘플은:
- 직접 생성한 문서(저작권 100% 본인)
- 또는 명시적으로 재배포 허용된 소형 문서

추천:
- `fixtures/`에 5~10개만 포함
- 나머지는 `corpus/local/`에서만 실행

---

## 6) V1 Gate 실행

공식 릴리즈 판정은 `scripts/v1_gate.py`로 한다.

```bash
cargo build --release -p hwp-cli
python3 scripts/v1_gate.py --ci --min-corpus-size 100
```

- 결과 파일:
  - `reports/v1_gate/*_summary.json`
  - `reports/v1_gate/*_details.jsonl`
