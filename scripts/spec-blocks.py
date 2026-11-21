#!/usr/bin/env python3
"""Measure how many ```cp blocks in design/spec/*.md pass `cpl check`.

Usage:
    scripts/spec-blocks.py [--cpl PATH] [--verbose]
"""
from __future__ import annotations

import argparse
import glob
import os
import re
import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FENCE_RE = re.compile(r"```cp\n(.*?)```", re.S)


def default_cpl() -> str:
    for rel in ("target/release/cpl", "target/debug/cpl"):
        p = ROOT / rel
        if p.is_file() and os.access(p, os.X_OK):
            return str(p)
    return "cpl"


def extract_blocks(spec_dir: Path) -> list[tuple[str, int, int, str]]:
    out: list[tuple[str, int, int, str]] = []
    for path in sorted(glob.glob(str(spec_dir / "*.md"))):
        name = os.path.basename(path)
        text = Path(path).read_text(encoding="utf-8")
        for idx, m in enumerate(FENCE_RE.finditer(text)):
            line = text.count("\n", 0, m.start()) + 1
            out.append((name, idx, line, textwrap.dedent(m.group(1))))
    return out


def first_error_line(output: str) -> str:
    for ln in output.splitlines():
        s = ln.strip()
        if s.startswith("error"):
            return s
    for ln in output.splitlines():
        s = ln.strip()
        if s:
            return s
    return "(no output)"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--cpl", default=default_cpl(), help="path to cpl binary")
    ap.add_argument("--spec-dir", default=str(ROOT / "design" / "spec"))
    ap.add_argument("--verbose", "-v", action="store_true", help="print every block's status")
    args = ap.parse_args()

    blocks = extract_blocks(Path(args.spec_dir))
    passed = 0
    for name, idx, line, code in blocks:
        with tempfile.NamedTemporaryFile("w", suffix=".cp", delete=False, encoding="utf-8") as tf:
            tf.write(code)
            tmp = tf.name
        try:
            res = subprocess.run([args.cpl, "check", tmp], capture_output=True, text=True)
        finally:
            os.unlink(tmp)
        if res.returncode == 0:
            passed += 1
            if args.verbose:
                print(f"PASS {name}#{idx} (line {line})")
        elif args.verbose:
            err = first_error_line(res.stderr + "\n" + res.stdout)
            print(f"FAIL {name}#{idx} (line {line}): {err}")

    print(f"spec blocks: {passed}/{len(blocks)} pass `cpl check`")
    return 0


if __name__ == "__main__":
    sys.exit(main())
