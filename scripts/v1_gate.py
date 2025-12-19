#!/usr/bin/env python3
"""V1 quality gate for hwp-bridge.

Make "perfect" measurable.

This gate is designed for **private corpora**:
- The repository does NOT contain HWP files.
- You store documents under `corpus/local` (gitignored), and commit only the
  `corpus/manifest.json` (hashes + metadata).

Inputs:
  - corpus directory (default: corpus/local)
  - manifest file (default: corpus/manifest.json)
  - built `hwp` CLI binary (default: target/release/hwp)

Outputs:
  - reports/v1_gate/<timestamp>_summary.json
  - reports/v1_gate/<timestamp>_details.jsonl

Exit code:
  - 0: thresholds met
  - 2: thresholds NOT met
  - 3: setup missing (no corpus / no binary)

This script never uploads files.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class RunResult:
    relpath: str
    category: Optional[str]
    size_bytes: int
    ok: bool
    error: Optional[str]
    timing_ms: int

    # Determinism is checked on StructuredDocument JSON output.
    out_sha256_a: Optional[str]
    out_sha256_b: Optional[str]
    deterministic: Optional[bool]

    # Optional: markdown determinism (when enabled)
    md_sha256_a: Optional[str] = None
    md_sha256_b: Optional[str] = None
    md_deterministic: Optional[bool] = None

    # Basic content stats (from JSON)
    sections: Optional[int] = None
    paragraphs: Optional[int] = None
    tables: Optional[int] = None


def sha256_bytes(b: bytes) -> str:
    h = hashlib.sha256()
    h.update(b)
    return h.hexdigest()


def run_cmd(cmd: list[str], timeout_s: int) -> tuple[int, bytes, bytes, int]:
    """Run command, return (code, stdout, stderr, elapsed_ms)."""
    t0 = time.time()
    try:
        p = subprocess.run(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout_s,
            check=False,
        )
        ms = int((time.time() - t0) * 1000)
        return p.returncode, p.stdout, p.stderr, ms
    except subprocess.TimeoutExpired as e:
        ms = int((time.time() - t0) * 1000)
        stdout = e.stdout or b""
        stderr = e.stderr or b""
        return 124, stdout, stderr, ms


def classify_error(stderr: bytes, stdout: bytes, code: int) -> str:
    text = (stderr + b"\n" + stdout).decode("utf-8", errors="replace").lower()
    if "encrypted" in text or "encryption" in text:
        return "encrypted"
    if "distribution" in text:
        return "distribution"
    if "size limit" in text or "sizelimit" in text or "limit exceeded" in text:
        return "size_limit"
    if code == 124:
        return "timeout"
    return "parse_error"


def percentiles(values: list[int]) -> dict[str, int]:
    if not values:
        return {"p50": 0, "p95": 0, "p99": 0}
    v = sorted(values)

    def pick(p: float) -> int:
        idx = max(0, min(len(v) - 1, int(round(p * (len(v) - 1)))))
        return int(v[idx])

    return {"p50": pick(0.50), "p95": pick(0.95), "p99": pick(0.99)}


def load_category_map(manifest_path: Path) -> dict[str, Optional[str]]:
    if not manifest_path.exists():
        return {}
    try:
        data = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception:
        return {}

    items = data.get("items") or []
    m: dict[str, Optional[str]] = {}
    for it in items:
        rel = it.get("relpath")
        if not rel:
            continue
        cat = it.get("category")
        if isinstance(cat, str):
            cat_u = cat.strip().upper()
            if cat_u in {"A", "B", "C"}:
                m[rel] = cat_u
            else:
                m[rel] = None
        else:
            m[rel] = None
    return m


def find_corpus_files(corpus_dir: Path, category_map: dict[str, Optional[str]]) -> list[tuple[Path, str]]:
    """Return list of (path, relpath). Prefer manifest relpaths when available."""
    files: list[tuple[Path, str]] = []

    # Prefer manifest-defined ordering when present.
    if category_map:
        for rel in category_map.keys():
            p = corpus_dir / rel
            if p.exists() and p.is_file():
                files.append((p, rel))
        if files:
            return files

    for p in sorted(corpus_dir.rglob("*.hwp")):
        if p.is_file():
            rel = p.relative_to(corpus_dir).as_posix()
            files.append((p, rel))
    return files


def rate(ok: int, total: int) -> float:
    return (ok / total) if total else 0.0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus-dir", default="corpus/local")
    ap.add_argument("--manifest", default="corpus/manifest.json")
    ap.add_argument("--hwp-bin", default="target/release/hwp")
    ap.add_argument("--timeout-s", type=int, default=30)
    ap.add_argument("--check-markdown", action="store_true", help="Also run `hwp markdown` twice for determinism (slower)")

    # V1 acceptance defaults
    ap.add_argument("--min-corpus-size", type=int, default=100)
    ap.add_argument("--min-success-a", type=float, default=0.95)
    ap.add_argument("--min-success-b", type=float, default=0.85)
    ap.add_argument("--min-success-c", type=float, default=0.80)
    ap.add_argument("--min-deterministic-rate", type=float, default=0.99)

    ap.add_argument("--max-files", type=int, default=0, help="0 = no limit")
    ap.add_argument("--ci", action="store_true", help="CI-friendly output")

    args = ap.parse_args()

    root = Path(__file__).resolve().parents[1]
    corpus_dir = (root / args.corpus_dir).resolve()
    manifest_path = (root / args.manifest).resolve()
    hwp_bin = (root / args.hwp_bin).resolve()

    if not corpus_dir.exists():
        print(f"[v1_gate] corpus dir not found: {corpus_dir}")
        return 3

    category_map = load_category_map(manifest_path)
    files = find_corpus_files(corpus_dir, category_map)

    if args.max_files and args.max_files > 0:
        files = files[: args.max_files]

    if not files:
        print(f"[v1_gate] no .hwp files under: {corpus_dir}")
        return 3

    if not hwp_bin.exists():
        print(f"[v1_gate] hwp binary not found: {hwp_bin}")
        print("Build it: cargo build --release -p hwp-cli")
        return 3

    out_dir = root / "reports" / "v1_gate"
    out_dir.mkdir(parents=True, exist_ok=True)
    ts = time.strftime("%Y%m%d-%H%M%S")
    details_path = out_dir / f"{ts}_details.jsonl"
    summary_path = out_dir / f"{ts}_summary.json"

    results: list[RunResult] = []

    for p, rel in files:
        size = p.stat().st_size
        cat = category_map.get(rel)

        # 1) info (catches encrypted/distribution early)
        code_i, out_i, err_i, ms_i = run_cmd([str(hwp_bin), "info", str(p)], timeout_s=args.timeout_s)

        # 2) structured JSON twice for determinism check
        code_a, out_a, err_a, ms_a = run_cmd([str(hwp_bin), "json", str(p)], timeout_s=args.timeout_s)
        code_b, out_b, err_b, ms_b = run_cmd([str(hwp_bin), "json", str(p)], timeout_s=args.timeout_s)

        md_code_a = md_code_b = 0
        md_out_a = md_out_b = b""
        md_err_a = md_err_b = b""
        ms_m1 = ms_m2 = 0
        md_sha_a = md_sha_b = None
        md_det: Optional[bool] = None

        if args.check_markdown:
            md_code_a, md_out_a, md_err_a, ms_m1 = run_cmd([str(hwp_bin), "markdown", str(p)], timeout_s=args.timeout_s)
            md_code_b, md_out_b, md_err_b, ms_m2 = run_cmd([str(hwp_bin), "markdown", str(p)], timeout_s=args.timeout_s)
            if md_code_a == 0 and md_code_b == 0:
                md_sha_a = sha256_bytes(md_out_a)
                md_sha_b = sha256_bytes(md_out_b)
                md_det = (md_sha_a == md_sha_b)

        ok = (code_i == 0) and (code_a == 0) and (code_b == 0) and (not args.check_markdown or ((md_code_a == 0) and (md_code_b == 0)))
        err: Optional[str] = None
        sha_a = sha_b = None
        det: Optional[bool] = None
        stats = None

        if ok:
            sha_a = sha256_bytes(out_a)
            sha_b = sha256_bytes(out_b)
            det = sha_a == sha_b

            # Basic stats from JSON output (doc structure)
            try:
                doc_obj = json.loads(out_a.decode("utf-8", errors="replace"))
                sections = int(len(doc_obj.get("sections", [])))
                paragraphs = 0
                tables = 0
                for sec in doc_obj.get("sections", []):
                    for block in sec.get("content", []):
                        if block.get("type") == "paragraph":
                            paragraphs += 1
                        elif block.get("type") == "table":
                            tables += 1
                stats = (sections, paragraphs, tables)
            except Exception:
                stats = None
        else:
            err = classify_error(
                err_i + err_a + err_b + md_err_a + md_err_b,
                out_i + out_a + out_b + md_out_a + md_out_b,
                (code_a or code_i or md_code_a or md_code_b),
            )

        timing_ms = ms_i + ms_a + ms_b + ms_m1 + ms_m2
        r = RunResult(
            relpath=rel,
            category=cat,
            size_bytes=size,
            ok=ok,
            error=err,
            timing_ms=timing_ms,
            out_sha256_a=sha_a,
            out_sha256_b=sha_b,
            deterministic=det,
            md_sha256_a=md_sha_a,
            md_sha256_b=md_sha_b,
            md_deterministic=md_det,
            sections=(stats[0] if stats else None),
            paragraphs=(stats[1] if stats else None),
            tables=(stats[2] if stats else None),
        )
        results.append(r)

        with details_path.open("a", encoding="utf-8") as w:
            w.write(json.dumps(r.__dict__, ensure_ascii=False) + "\n")

    # Aggregate
    total = len(results)
    ok_count = sum(1 for r in results if r.ok)
    det_count = sum(1 for r in results if r.ok and r.deterministic)

    det_rate = rate(det_count, ok_count)
    timings = [r.timing_ms for r in results if r.ok]
    timing_stats = percentiles(timings)

    failures_by_type: dict[str, int] = {}
    for r in results:
        if r.ok:
            continue
        failures_by_type[r.error or "unknown"] = failures_by_type.get(r.error or "unknown", 0) + 1

    # Per-category
    cat_totals = {"A": 0, "B": 0, "C": 0, "_": 0}
    cat_ok = {"A": 0, "B": 0, "C": 0, "_": 0}
    for r in results:
        k = r.category if r.category in {"A", "B", "C"} else "_"
        cat_totals[k] += 1
        if r.ok:
            cat_ok[k] += 1

    cat_rates = {k: round(rate(cat_ok[k], cat_totals[k]), 4) for k in cat_totals.keys()}

    thresholds = {
        "min_corpus_size": args.min_corpus_size,
        "min_success": {
            "A": args.min_success_a,
            "B": args.min_success_b,
            "C": args.min_success_c,
        },
        "min_deterministic_rate": args.min_deterministic_rate,
    }

    summary = {
        "timestamp": ts,
        "total_files": total,
        "ok": ok_count,
        "failed": total - ok_count,
        "per_category": {
            "totals": cat_totals,
            "ok": cat_ok,
            "success_rate": cat_rates,
        },
        "deterministic_rate": round(det_rate, 4),
        "timing_ms": timing_stats,
        "failures_by_type": failures_by_type,
        "thresholds": thresholds,
        "artifacts": {
            "details_jsonl": str(details_path.relative_to(root)),
            "summary_json": str(summary_path.relative_to(root)),
        },
        "notes": {
            "category_underscore": "Files without category in manifest.json are counted under '_' and are not gated unless you label them A/B/C.",
        },
    }

    summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")

    # Gate evaluation
    gate_ok = True

    if args.min_corpus_size and total < args.min_corpus_size:
        gate_ok = False

    # Only enforce per-category thresholds if there are any files in that category
    if cat_totals["A"] > 0 and cat_rates["A"] < args.min_success_a:
        gate_ok = False
    if cat_totals["B"] > 0 and cat_rates["B"] < args.min_success_b:
        gate_ok = False
    if cat_totals["C"] > 0 and cat_rates["C"] < args.min_success_c:
        gate_ok = False

    if det_rate < args.min_deterministic_rate:
        gate_ok = False

    if args.ci:
        print(
            f"[v1_gate] total={total} ok={ok_count} det_rate={summary['deterministic_rate']} "
            f"A={cat_rates['A']} B={cat_rates['B']} C={cat_rates['C']} p95={timing_stats['p95']}ms"
        )
        print(f"[v1_gate] summary: {summary_path}")

    if gate_ok:
        return 0

    print("[v1_gate] FAILED thresholds")
    print(json.dumps(summary, ensure_ascii=False, indent=2))
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
