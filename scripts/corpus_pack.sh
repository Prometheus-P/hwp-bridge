#!/usr/bin/env bash
set -euo pipefail

# Pack `corpus/local` into a zip suitable for CI download.
#
# Usage:
#   bash scripts/corpus_pack.sh  # writes ./corpus_local.zip
#
# Notes:
# - This DOES NOT add the zip to git.
# - Upload the resulting zip to a private location (S3, GCS, etc.) and set the
#   download URL as GitHub Actions secret: CORPUS_ZIP_URL.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORPUS_DIR="$ROOT/corpus/local"
OUT="${1:-$ROOT/corpus_local.zip}"

if [[ ! -d "$CORPUS_DIR" ]]; then
  echo "corpus dir not found: $CORPUS_DIR" >&2
  exit 1
fi

rm -f "$OUT"
(
  cd "$CORPUS_DIR"
  # zip contents, not the parent folder
  zip -qr "$OUT" .
)

if command -v sha256sum >/dev/null 2>&1; then
  echo "sha256: $(sha256sum "$OUT" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  echo "sha256: $(shasum -a 256 "$OUT" | awk '{print $1}')"
else
  echo "sha256 tool not found"
fi

echo "wrote: $OUT"
