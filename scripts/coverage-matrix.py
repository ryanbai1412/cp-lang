#!/usr/bin/env python3
"""Line, region and function coverage of each crate's source, by its own tests and by all.

Rule: handbook/process/quality.md#how-suites-are-measured. A crate's own
column runs only `cargo nextest run -p <crate>`; the all column runs the whole
workspace once. A low own number means the crate relies on other crates'
tests, which break for unrelated reasons; a low all number is code no test
reaches. Needs cargo-llvm-cov and the llvm-tools rustup component.

Usage:
    python3 scripts/coverage-matrix.py [-p CRATE ...] [--no-own] [-o target/coverage]
    python3 scripts/coverage-matrix.py --report target/coverage   # table of a finished run
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KINDS = ("lines", "regions", "functions")


def crates() -> list[str]:
    return sorted(p.parent.name for p in (ROOT / "crates").glob("*/Cargo.toml"))


def per_crate(report: dict, names: list[str]) -> dict[str, dict[str, list[int]]]:
    """Covered and total counts of each kind, summed over each crate's `src/` files."""
    out = {c: {k: [0, 0] for k in KINDS} for c in names}
    for f in report["data"][0]["files"]:
        rel = Path(f["filename"]).resolve()
        try:
            parts = rel.relative_to(ROOT).parts
        except ValueError:
            continue
        if len(parts) > 2 and parts[0] == "crates" and parts[2] == "src" and parts[1] in out:
            for k in KINDS:
                out[parts[1]][k][0] += f["summary"][k]["covered"]
                out[parts[1]][k][1] += f["summary"][k]["count"]
    return out


def pct(pair: list[int]) -> str:
    return f"{100 * pair[0] / pair[1]:.0f}%" if pair[1] else "-"


def table(own: dict, all_: dict | None) -> str:
    head = "| Crate | Own lines | Own regions | Own functions |"
    if all_ is not None:
        head += " All lines | All regions | All functions |"
    rows = [head, "|" + "---|" * (head.count("|") - 1)]
    for c in sorted(set(own) | set(all_ or {})):
        cells = [pct(own[c][k]) if c in own else "" for k in KINDS]
        if all_ is not None:
            cells += [pct(all_[c][k]) if c in all_ else "" for k in KINDS]
        rows.append(f"| {c} | " + " | ".join(cells) + " |")
    return "\n".join(rows) + "\n"


def run(args: list[str], out: Path) -> dict:
    cmd = ["cargo", "llvm-cov", "nextest", "--json", "--summary-only",
           "--output-path", str(out), "--no-fail-fast", *args]
    subprocess.run(cmd, cwd=ROOT, check=False)
    return json.loads(out.read_text())


def report(out: Path) -> str:
    own = {}
    for f in sorted(out.glob("own-*.json")):
        c = f.stem.removeprefix("own-")
        own[c] = per_crate(json.loads(f.read_text()), [c])[c]
    whole = out / "all.json"
    all_ = per_crate(json.loads(whole.read_text()), crates()) if whole.is_file() else None
    return table(own, all_)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("-p", dest="crates", action="append", help="only these crates' own runs")
    ap.add_argument("--no-own", action="store_true", help="only the whole-workspace run")
    ap.add_argument("-o", type=Path, default=ROOT / "target/coverage")
    ap.add_argument("--report", type=Path, help="print the table of a finished run")
    a = ap.parse_args()
    if a.report:
        sys.stdout.write(report(a.report))
        return 0
    a.o.mkdir(parents=True, exist_ok=True)
    run(["--workspace"], a.o / "all.json")
    if not a.no_own:
        for c in a.crates or crates():
            run(["-p", c], a.o / f"own-{c}.json")
    text = report(a.o)
    (a.o / "matrix.md").write_text(text)
    sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
