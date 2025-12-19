# Test Matrix (V1)

> 목적: “어떤 문서에서 무엇이 깨지는지”를 체계적으로 재현/추적한다.  
> 결과는 `corpus/manifest.(yml|json)` + `reports/*.json`로 남긴다.

---

## 1) Dimensions

### 1.1 Format / Container
- HWP 5.x (OLE/CFB)
- (Future) HWPX (zip+xml)

### 1.2 Properties
- Compression: compressed / uncompressed (`header.properties.is_compressed`)
- Security: encrypted / distribution / normal
- Integrity: 정상 / 손상(truncated/invalid record header)

### 1.3 Content Features
- Plain paragraphs
- Lists / bullets
- Tables (simple / merged cell / nested)
- Controls (text box, shapes)
- Footnotes/endnotes
- Headers/footers
- Multi-language (KO/EN/JA/CJK mixed)
- Large documents (many sections/records)

### 1.4 Outputs
- inspect JSON
- plain text
- structured JSON
- semantic markdown

---

## 2) Core Test Cases (필수)

| ID | Category | Input | Expected | Assertion |
|---|---|---|---|---|
| T001 | Header | valid HWP | OK | version/properties parsed |
| T002 | Header | invalid signature | Error | InvalidSignature |
| T003 | Security | encrypted doc | Error | Encrypted |
| T004 | Security | distribution doc | Error | DistributionOnly |
| T005 | Compression | uncompressed section | OK | no zlib attempt |
| T006 | Compression | compressed section | OK | decompress ok |
| T007 | Limits | decompress > max | Error | SizeLimitExceeded (no panic) |
| T008 | Limits | records > max | Error | SizeLimitExceeded |
| T009 | Determinism | same file x10 | OK | output hash stable |
| T010 | Robustness | corrupted record header | Error | ParseError, no panic |

---

## 3) Runner Output Schema (권장)

각 파일마다 다음을 기록:
- `id, sha256, size_bytes`
- `header`: version, flags
- `result`: success/error + error_code
- `metrics`: sections, records, paragraphs, tables, char_count
- `timing_ms`, `peak_rss_mb`(가능하면)

---

## 4) CI Integration (추천)
- PR마다: 고정 골든 fixtures(소수) + 단위 테스트
- nightly/주기적으로: 전체 코퍼스(로컬 또는 사내 러너) 실행 후 리포트 업로드
