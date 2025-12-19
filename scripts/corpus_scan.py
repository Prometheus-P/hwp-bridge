#!/usr/bin/env python3
"""Corpus scanner

Scans `corpus/local` for .hwp files and generates/updates `corpus/manifest.json`.
Does NOT upload any files. Only hashes + metadata.

Usage:
  python scripts/corpus_scan.py
"""

import hashlib, json, os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LOCAL = ROOT / "corpus" / "local"
MANIFEST = ROOT / "corpus" / "manifest.json"

def sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

def main():
    LOCAL.mkdir(parents=True, exist_ok=True)

    items = []
    for p in sorted(LOCAL.rglob("*.hwp")):
        rel = p.relative_to(LOCAL).as_posix()
        items.append({
            "id": rel.replace("/", "__"),
            "relpath": rel,
            "sha256": sha256_file(p),
            "size_bytes": p.stat().st_size,
            "category": None,
            "flags": {},
            "source": {"url": None, "license_note": None},
            "notes": None,
        })

    out = {"version": "1", "generated_from": "corpus/local", "items": items}
    MANIFEST.write_text(json.dumps(out, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Wrote {MANIFEST} with {len(items)} items")

if __name__ == "__main__":
    main()
