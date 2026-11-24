#!/usr/bin/env python3
"""Measure how many small code changes a crate's tests catch.

Runs cargo-mutants on one crate (optionally only on code a diff touched,
or one shard of the whole crate) and prints the mutation score: caught
mutants over caught plus missed. A missed mutant is a change to the code
that no test notices. Mutants that don't build or time out aren't scored.

Usage:
    python3 scripts/mutation-score.py -p cpl-lint [--since REV] [--shard K/N] [-o DIR]
        [--test-package cpl-lsp ...]   # also run tests of crates that call it
        [--nextest] [--profile ci]     # stop at the first failing test; optimized tests
    python3 scripts/mutation-score.py --report DIR   # score a finished run
    add --min 0.98 to fail (exit 1) when the score is below 0.98
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


def report(out_dir: Path, least: float | None = None) -> int:
    outcomes = json.loads((out_dir / "mutants.out" / "outcomes.json").read_text(encoding="utf-8"))
    s = score(outcomes)
    for name in missed_names(outcomes):
        print(f"missed: {name}")
    pct = "n/a" if s["score"] is None else f"{s['score']:.1%}"
    print(f"mutation-score: {pct} ({s['caught']} caught, {s['missed']} missed, "
          f"{s['timeout']} timeout, {s['unviable']} unviable)")
    return below(s, least)


def below(s: dict, least: float | None) -> int:
    if least is None or (s["score"] is not None and s["score"] >= least):
        return 0
    print(f"mutation-score: below the required {least:.1%}; add tests for the missed mutants")
    return 1


def mutants_command(package: str, out: Path, jobs: str, test_packages: list[str],
                    nextest: bool = False, profile: str | None = None,
                    files: list[str] = (), tests: list[str] = (),
                    extra: list[str] = ()) -> list[str]:
    """The cargo-mutants run. It keeps the baseline: skipping it turns a test
    already red on the tree into a caught mutant each, and a 100% score.
    Under nextest a caught mutant ends at its first failing test, so a crate
    whose corpus tests take minutes is scored in hours, not days."""
    cmd = ["cargo", "mutants", "-p", package, "-o", str(out), "-j", jobs, "--no-shuffle"]
    if test_packages:
        cmd += ["--test-package", ",".join([package, *test_packages])]
    if nextest:
        cmd += ["--test-tool", "nextest"]
    if profile:
        cmd += ["--profile", profile]
    for file in files:
        cmd += ["-f", file]
    cmd += extra
    if tests:
        cmd += ["--", *(arg for test in tests for arg in ("--test", test))]
    return cmd


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("-p", "--package", help="crate to mutate, e.g. cpl-lint")
    parser.add_argument("--since", help="only mutate code changed since this revision")
    parser.add_argument("--shard", help="K/N: run one of N slices of the crate's mutants")
    parser.add_argument("-o", "--output", type=Path, help="where cargo-mutants writes mutants.out")
    parser.add_argument("--test-package", action="append", default=[],
                        help="also run this crate's tests, e.g. a caller whose tests cover the mutated code")
    parser.add_argument("-j", "--jobs", default="4")
    parser.add_argument("--nextest", action="store_true",
                        help="run tests with nextest, which stops at the first failure")
    parser.add_argument("--profile", help="cargo profile to build tests in, e.g. ci")
    parser.add_argument("-f", "--file", action="append", default=[],
                        help="only mutate this source file, e.g. crates/cpl-cli/src/dap.rs")
    parser.add_argument("--test", action="append", default=[],
                        help="only run this test target, e.g. dap")
    parser.add_argument("--report", type=Path, help="only score a finished run in DIR")
    parser.add_argument("--min", type=float, help="exit 1 when the score is below this, e.g. 0.98")
    args = parser.parse_args(argv)
    if args.report:
        return report(args.report, args.min)
    if not args.package:
        parser.error("-p/--package is required unless --report is given")
    out = args.output or Path(tempfile.mkdtemp(prefix=f"mutants-{args.package}-"))
    extra = ["--shard", args.shard] if args.shard else []
    if args.since:
        diff = out / "in.diff"
        out.mkdir(parents=True, exist_ok=True)
        diff.write_text(subprocess.run(["git", "diff", args.since, "--", f"crates/{args.package}/src"],
                                       cwd=ROOT, capture_output=True, text=True, check=True).stdout)
        extra += ["--in-diff", str(diff)]
    cmd = mutants_command(args.package, out, args.jobs, args.test_package,
                          args.nextest, args.profile, args.file, args.test, extra)
    subprocess.run(cmd, cwd=ROOT)
    return report(out, args.min)


if __name__ == "__main__":
    sys.exit(main())
