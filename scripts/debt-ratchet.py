#!/usr/bin/env python3
"""Tech-debt ratchet and .rs file-size cap gate.

Checks:
  1. `.rs` file-size cap (`max_rs_lines`, default 2000):
     - Warns on any non-allowlisted `.rs` file under `crates/` over `max_rs_lines`.
     - Fails if any allowlisted offender grows beyond its recorded line cap
       (the allowlist may only shrink).
  2. Debt ratchets (each may only go down; fails if actual > recorded baseline):
     - `stdlib_todo`: `= todo` / standalone `todo` stubs in `spec/std/*.cp` and `stdlib/*.cp`.
     - `rust_ignore`: `#[ignore...]` attributes in `crates/**/*.rs` (excluding comments).
     - `pending_tests`: files under `crates/cpl-cli/tests/pending/`.

Usage:
  python3 scripts/debt-ratchet.py --check
  python3 scripts/debt-ratchet.py --update
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_RATCHET = ROOT / "scripts" / "debt-ratchet.json"
DEFAULT_MAX_RS_LINES = 2000

TODO_STUB_RE = re.compile(r"(?:=\s*todo|^\s*todo)(?:\s*(?://.*)?$)")
RUST_IGNORE_RE = re.compile(r"^\s*#\[ignore\b")


def measure(root: Path = ROOT, max_rs_lines: int = DEFAULT_MAX_RS_LINES) -> dict:
    """Measure current file sizes (> max_rs_lines) and debt ratchet counts."""
    rs_over_cap: dict[str, int] = {}
    rust_ignore = 0
    crates_dir = root / "crates"
    if crates_dir.is_dir():
        for dirpath, _, filenames in os.walk(crates_dir):
            for fname in filenames:
                if not fname.endswith(".rs"):
                    continue
                path = Path(dirpath) / fname
                rel = path.relative_to(root).as_posix()
                lines = path.read_text(encoding="utf-8").splitlines()
                if len(lines) > max_rs_lines:
                    rs_over_cap[rel] = len(lines)
                for line in lines:
                    if RUST_IGNORE_RE.match(line):
                        rust_ignore += 1

    stdlib_todo = 0
    for sub in ("spec/std", "stdlib"):
        for path_str in sorted(glob.glob(str(root / sub / "*.cp"))):
            with open(path_str, encoding="utf-8") as fh:
                for line in fh:
                    if TODO_STUB_RE.search(line):
                        stdlib_todo += 1

    pending_dir = root / "crates" / "cpl-cli" / "tests" / "pending"
    pending_tests = 0
    if pending_dir.is_dir():
        for _, _, filenames in os.walk(pending_dir):
            pending_tests += len(filenames)

    sorted_allowlist = dict(sorted(rs_over_cap.items(), key=lambda kv: (-kv[1], kv[0])))
    return {
        "max_rs_lines": max_rs_lines,
        "rs_file_size_allowlist": sorted_allowlist,
        "stdlib_todo": stdlib_todo,
        "rust_ignore": rust_ignore,
        "pending_tests": pending_tests,
    }


def evaluate(actual: dict, baseline: dict) -> tuple[list[str], list[str], list[str]]:
    """Return (errors, warnings, notes) comparing `actual` against `baseline`."""
    errors: list[str] = []
    warnings: list[str] = []
    notes: list[str] = []

    max_lines = int(baseline.get("max_rs_lines", DEFAULT_MAX_RS_LINES))
    allowlist: dict[str, int] = baseline.get("rs_file_size_allowlist", {})
    actual_over: dict[str, int] = actual.get("rs_file_size_allowlist", {})

    for rel, lines in sorted(actual_over.items()):
        if rel in allowlist:
            cap = allowlist[rel]
            if lines > cap:
                errors.append(
                    f"{rel}: grew from {cap} to {lines} lines (allowlist may only shrink)"
                )
            elif lines < cap:
                notes.append(f"{rel}: shrank from {cap} to {lines} lines")
        else:
            warnings.append(
                f"{rel}: {lines} lines exceeds {max_lines}-line cap (not in allowlist)"
            )

    for rel, cap in sorted(allowlist.items()):
        if rel not in actual_over:
            notes.append(f"{rel}: now <= {max_lines} lines (was {cap}; can be removed from allowlist)")

    for key, label in (
        ("stdlib_todo", "stdlib todo stubs"),
        ("rust_ignore", "Rust #[ignore] attributes"),
        ("pending_tests", "crates/cpl-cli/tests/pending/ files"),
    ):
        cur = int(actual[key])
        base = int(baseline[key])
        if cur > base:
            errors.append(f"{label}: increased from {base} to {cur} (ratchet may only go down)")
        elif cur < base:
            notes.append(f"{label}: improved from {base} to {cur}")

    return errors, warnings, notes


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Check or update the tech-debt ratchet.")
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="Verify against the ratchet file.")
    group.add_argument("--update", action="store_true", help="Write current measurements to the ratchet file.")
    parser.add_argument("--ratchet", type=Path, default=DEFAULT_RATCHET, help="Path to debt-ratchet.json.")
    parser.add_argument("--root", type=Path, default=ROOT, help="Repository root.")
    args = parser.parse_args(argv)

    if args.update:
        max_lines = DEFAULT_MAX_RS_LINES
        if args.ratchet.is_file():
            existing = json.loads(args.ratchet.read_text(encoding="utf-8"))
            max_lines = int(existing.get("max_rs_lines", DEFAULT_MAX_RS_LINES))
        data = measure(args.root, max_lines)
        args.ratchet.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        print(
            f"debt-ratchet: wrote {args.ratchet} "
            f"(rs>{max_lines}={len(data['rs_file_size_allowlist'])}, "
            f"stdlib_todo={data['stdlib_todo']}, "
            f"rust_ignore={data['rust_ignore']}, "
            f"pending_tests={data['pending_tests']})"
        )
        return 0

    if not args.ratchet.is_file():
        print(f"debt-ratchet: missing {args.ratchet}; run --update", file=sys.stderr)
        return 1

    baseline = json.loads(args.ratchet.read_text(encoding="utf-8"))
    max_lines = int(baseline.get("max_rs_lines", DEFAULT_MAX_RS_LINES))
    actual = measure(args.root, max_lines)
    errors, warnings, notes = evaluate(actual, baseline)

    for w in warnings:
        print(f"debt-ratchet: WARNING: {w}", file=sys.stderr)
    for e in errors:
        print(f"debt-ratchet: ERROR: {e}", file=sys.stderr)

    print(
        f"debt-ratchet: rs>{max_lines}={len(actual['rs_file_size_allowlist'])} "
        f"(allowlist {len(baseline.get('rs_file_size_allowlist', {}))}), "
        f"stdlib_todo={actual['stdlib_todo']}/{baseline['stdlib_todo']}, "
        f"rust_ignore={actual['rust_ignore']}/{baseline['rust_ignore']}, "
        f"pending_tests={actual['pending_tests']}/{baseline['pending_tests']}"
    )
    if notes:
        print(
            f"debt-ratchet: {len(notes)} item(s) improved; run `python3 scripts/debt-ratchet.py --update` to tighten"
        )
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
