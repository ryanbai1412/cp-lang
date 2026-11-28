#!/usr/bin/env python3
"""Write cpl-next's case lists from one CI run's 16 lists@ shards.

    uv run python scripts/next-lists-from-run.py RUN_ID [--dir DIR]

Each lists@N/M gate part keeps what it measured as the artifact next-lists-N-M
(target/cpl-next/shard-N-of-M). A fresh, whole run's lists are those shards put together, so
a regeneration names the run it came from and never a hand edit. Fails unless every shard is
there and no case is in two of them.
"""
import argparse
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LISTS = ROOT / "crates/cpl-next/tests"
# The lists every_case_is_where_the_lists_say compares and writes, each a case id per row first.
NAMES = ("passing.txt", "wrong.tsv", "masked.txt", "lowered.txt", "error-typed.txt")


def shards(run: str, into: Path) -> list[Path]:
    got = subprocess.run(["gh", "run", "download", run, "--pattern", "next-lists-*", "--dir", str(into)],
                         capture_output=True, text=True)
    if got.returncode:
        sys.exit(f"next-lists-from-run: gh run download {run} failed: {got.stderr.strip()}")
    return sorted(p for p in into.glob("next-lists-*/shard-*") if p.is_dir())


def merge(dirs: list[Path]) -> dict[str, str]:
    """Each list's rows from every shard, in all_cases' order (byte order of the id)."""
    sizes = {d.name.rsplit("-", 1)[-1] for d in dirs}
    if len(sizes) != 1 or len(dirs) != int(next(iter(sizes))):
        sys.exit(f"next-lists-from-run: {len(dirs)} shards, not every one of a run: "
                 + ", ".join(d.name for d in dirs))
    out = {}
    for name in NAMES:
        rows: dict[str, str] = {}
        for d in dirs:
            path = d / name
            if not path.is_file():
                sys.exit(f"next-lists-from-run: {d.name} has no {name}")
            for line in path.read_text(encoding="utf-8").splitlines():
                if not line:
                    continue
                case = line.split("\t", 1)[0]
                if case in rows:
                    sys.exit(f"next-lists-from-run: {case} is in {name} of two shards")
                rows[case] = line
        out[name] = "".join(rows[c] + "\n" for c in sorted(rows, key=lambda c: c.encode()))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("run")
    ap.add_argument("--dir", type=Path, help="shards already downloaded here (no gh)")
    args = ap.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        dirs = sorted(p for p in args.dir.glob("**/shard-*") if p.is_dir()) if args.dir \
            else shards(args.run, Path(tmp))
        lists = merge(dirs)
    for name, text in lists.items():
        old = (LISTS / name).read_text(encoding="utf-8") if (LISTS / name).is_file() else ""
        (LISTS / name).write_text(text, encoding="utf-8")
        print(f"{name}: {len(old.splitlines())} -> {len(text.splitlines())} rows")
    print(f"next-lists-from-run: lists written from run {args.run}; name it in the commit (List-run: {args.run})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
