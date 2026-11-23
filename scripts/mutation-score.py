#!/usr/bin/env python3
"""Measure how many small code changes a crate's tests catch.

Runs cargo-mutants on one crate (optionally only on code a diff touched,
or one shard of the whole crate) and prints the mutation score: caught
mutants over caught plus missed. A missed mutant is a change to the code
that no test notices. Mutants that don't build or time out aren't scored.

Usage:
    python3 scripts/mutation-score.py -p cpl-lint [--since REV] [--shard K/N] [-o DIR]
    python3 scripts/mutation-score.py --report DIR   # score a finished run
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def score(outcomes: dict) -> dict:
    caught, missed = outcomes.get("caught", 0), outcomes.get("missed", 0)
    scored = caught + missed
    return {
        "caught": caught,
        "missed": missed,
        "timeout": outcomes.get("timeout", 0),
        "unviable": outcomes.get("unviable", 0),
        "score": round(caught / scored, 3) if scored else None,
    }


def missed_names(outcomes: dict) -> list[str]:
    return [o["scenario"]["Mutant"]["name"] for o in outcomes.get("outcomes", [])
            if o.get("summary") == "MissedMutant" and isinstance(o.get("scenario"), dict)]


def report(out_dir: Path) -> int:
    outcomes = json.loads((out_dir / "mutants.out" / "outcomes.json").read_text(encoding="utf-8"))
    s = score(outcomes)
    for name in missed_names(outcomes):
        print(f"missed: {name}")
    pct = "n/a" if s["score"] is None else f"{s['score']:.1%}"
    print(f"mutation-score: {pct} ({s['caught']} caught, {s['missed']} missed, "
          f"{s['timeout']} timeout, {s['unviable']} unviable)")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("-p", "--package", help="crate to mutate, e.g. cpl-lint")
    parser.add_argument("--since", help="only mutate code changed since this revision")
    parser.add_argument("--shard", help="K/N: run one of N slices of the crate's mutants")
    parser.add_argument("-o", "--output", type=Path, help="where cargo-mutants writes mutants.out")
    parser.add_argument("-j", "--jobs", default="4")
    parser.add_argument("--report", type=Path, help="only score a finished run in DIR")
    args = parser.parse_args(argv)
    if args.report:
        return report(args.report)
    if not args.package:
        parser.error("-p/--package is required unless --report is given")
    out = args.output or Path(tempfile.mkdtemp(prefix=f"mutants-{args.package}-"))
    cmd = ["cargo", "mutants", "-p", args.package, "-o", str(out), "-j", args.jobs,
           "--no-shuffle", "--baseline", "skip"]
    if args.shard:
        cmd += ["--shard", args.shard]
    if args.since:
        diff = out / "in.diff"
        out.mkdir(parents=True, exist_ok=True)
        diff.write_text(subprocess.run(["git", "diff", args.since, "--", f"crates/{args.package}/src"],
                                       cwd=ROOT, capture_output=True, text=True, check=True).stdout)
        cmd += ["--in-diff", str(diff)]
    subprocess.run(cmd, cwd=ROOT)
    return report(out)


if __name__ == "__main__":
    sys.exit(main())
