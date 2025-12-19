# P0 Implementation Checklist (V1 Gate)

> 목적: Smithery/오픈 유통에서 “안 죽는 파서” + “재현 가능한 출력”을 만든다.

---

## 0) 정의
- P0 = 배포 후 바로 욕 먹는 문제 / 서버 죽는 문제 / 신뢰 박살 문제
- PR은 “작게, 기계적으로” 쪼갠다.

---

## P0-1. Compression flag 반영 (uncompressed 섹션 대응)
**증상**: `header.properties.is_compressed == false` 문서에서 zlib 디컴프 시도 → 실패  
**작업**
- `hwp-core`에서 `parse_section_records_with_options(data, is_compressed, limits)` 도입
- `HwpTextExtractor`가 `ole.header().properties.is_compressed()`로 분기
- `hwp-mcp`도 동일 로직 적용

**완료 조건**
- TEST_MATRIX T005/T006 통과

---

## P0-2. Decompression bomb 방어 (상한)
**증상**: 디컴프가 무한정 메모리/CPU 사용 → DoS  
**작업**
- `HwpError::SizeLimitExceeded` 추가
- `decompress_section_with_limits(data, max_decompressed_bytes)` 구현
- 기본값 + 바이너리(env/flag)로 조정 가능

**완료 조건**
- T007 통과 (panic 없이 SizeLimitExceeded)
- Smithery smoke에서 큰 입력에도 서버 유지

---

## P0-3. Record count 상한
**증상**: record iterator가 과도하게 돌며 CPU 폭탄  
**작업**
- 섹션 단위 `max_records_per_section` 도입
- 초과 시 SizeLimitExceeded

**완료 조건**
- T008 통과

---

## P0-4. Output determinism 체크
**작업**
- JSON/MD 출력이 입력에 대해 결정적이어야 함
- 러너에서 해시 10회 비교

**완료 조건**
- T009 통과

---

## P0-5. Corpus/Report 파이프라인
**작업**
- `corpus/manifest.json` + 스캔 스크립트
- `reports/*.json` 출력
- 문서화

**완료 조건**
- 코퍼스 100개에서 run 리포트 생성 가능

---

## PR 쪼개기 (권장)
1) docs: V1_ACCEPTANCE + TEST_MATRIX + CORPUS
2) types: HwpError::SizeLimitExceeded + metadata schema_version
3) core: safe decompress + compression flag + record cap
4) cli: limit flags + 적용
5) mcp: env limits + compression flag + record cap
6) tests: unit tests + smoke scripts
