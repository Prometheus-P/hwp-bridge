#!/usr/bin/env bash
set -euo pipefail

# Runs `hwp` CLI on all corpus files and writes a JSONL report.
# Prefer `scripts/v1_gate.py` for the official V1 gate.
#
# Output: reports/YYYYMMDD-run.jsonl

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORPUS_DIR="$ROOT/corpus/local"
OUT_DIR="$ROOT/reports"
DATE="$(date +%Y%m%d)"
OUT="$OUT_DIR/${DATE}-run.jsonl"

mkdir -p "$OUT_DIR"

HWP_BIN="${HWP_BIN:-$ROOT/target/release/hwp}"

if [[ ! -x "$HWP_BIN" ]]; then
  echo "hwp binary not found at $HWP_BIN"
  echo "Build it: cargo build --release -p hwp-cli"
  exit 1
fi

python3 "$ROOT/scripts/corpus_scan.py" >/dev/null

# capture stderr (logs/errors) when failing; ignore stdout by sending it to /dev/null
while IFS= read -r -d '' f; do
  rel="${f#$CORPUS_DIR/}"

  start="$(python3 - <<'PY'
import time; print(time.time())
PY
)"

  err=""
  if ! err="$($HWP_BIN extract "$f" 2>&1 >/dev/null)"; then
    :
  else
    err=""
  fi

  end="$(python3 - <<'PY'
import time; print(time.time())
PY
)"

  ms="$(python3 - <<PY
print(int((${end} - ${start}) * 1000))
PY
)"

  python3 - <<PY >> "$OUT"
import json
print(json.dumps({
  "file": ${rel@Q},
  "ok": ${"true" if err=="" else "false"},
  "timing_ms": int(${ms}),
  "error": ${err@Q} if ${"true" if err!="" else "false"} else None
}, ensure_ascii=False))
PY

done < <(find "$CORPUS_DIR" -type f -name "*.hwp" -print0)

echo "Wrote $OUT"
