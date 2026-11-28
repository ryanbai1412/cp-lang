#!/usr/bin/env python3
"""The freeze's behaviour half: a change to a crate the old pipeline is built from leaves what
it prints byte-identical. `debt-ratchet.py`'s `frozen_edits` sees lines added to the frozen
crates; this sees a change outside them that changes them anyway, as 9ad9cbbd1 did by changing
cpl-syntax under the old checker. `cpl build <case> -o -` (its C++, its diagnostics, its exit
status) runs on every conformance case at HEAD and at the merge base with origin/main, and any
case whose output differs fails the commit tier. The base's outputs are cached by the tree of
every input, so a lane pays for them once per main.

    uv run python scripts/old-pipeline-same.py [--base REF]

Rule: handbook/design/next/migration.md#rules (9: the old pipeline is frozen).
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# The crates `cpl build` runs through on the old pipeline, shared ones included, and the std it reads.
INPUTS = (
    "crates/cpl-cli", "crates/cpl-check", "crates/cpl-ir", "crates/cpl-interp", "crates/cpl-codegen",
    "crates/cpl-syntax", "crates/cpl-next-syntax", "crates/cpl-diag", "crates/cpl-prim",
    "crates/cpl-config", "crates/cpl-contract", "crates/cpl-json", "crates/cpl-rt",
    "crates/cpl-check/frozen/std", "Cargo.lock",
)
CACHE = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "cpl" / "old-pipeline-same"


def git(*args: str, cwd: Path = ROOT) -> str:
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()


def key(ref: str) -> str:
    """The trees of every input at `ref`: equal keys build the same old pipeline."""
    trees = git("ls-tree", ref, "--", *INPUTS) + "\n".join(git("ls-tree", ref, "--", f"{p}/") for p in INPUTS)
    return hashlib.sha256(trees.encode()).hexdigest()[:20]


def cases(tree: Path) -> list[str]:
    return sorted(str(p.relative_to(tree)) for p in (tree / "spec/conformance").glob("*/*.cp"))


def outputs(tree: Path, target: Path, names: list[str]) -> dict[str, str]:
    """Each case's `cpl build -o -`, hashed, from a cpl built in `tree` into `target`."""
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    subprocess.run(["cargo", "build", "-q", "-p", "cpl-cli"], cwd=tree, env=env, check=True)
    cpl = target / "debug" / "cpl"

    def one(name: str) -> tuple[str, str]:
        r = subprocess.run([cpl, "build", name, "-o", "-"], cwd=tree, capture_output=True, timeout=120)
        return name, hashlib.sha256(r.stdout + b"\0" + r.stderr + b"\0" + str(r.returncode).encode()).hexdigest()

    with cf.ThreadPoolExecutor(os.cpu_count() or 4) as ex:
        return dict(ex.map(one, names))


def base_outputs(base: str, names: list[str]) -> dict[str, str]:
    path = CACHE / f"{key(base)}.json"
    if path.exists():
        return json.loads(path.read_text())
    tree = CACHE / "base"
    if not (tree / ".git").exists():
        CACHE.mkdir(parents=True, exist_ok=True)
        git("worktree", "add", "-q", "--detach", str(tree), base)
    else:
        git("checkout", "-q", "--detach", "--force", base, cwd=tree)
    got = outputs(tree, CACHE / "target", names)
    path.write_text(json.dumps(got))
    return got


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default=None, help="the ref to compare with (default: merge base with origin/main)")
    args = ap.parse_args()
    base = git("rev-parse", args.base) if args.base else git("merge-base", "origin/main", "HEAD")
    if key(base) == key("HEAD") and not git("status", "--porcelain", "--", *INPUTS):
        print("old-pipeline-same: no old-pipeline input changed")
        return 0
    names = sorted(set(cases(ROOT)) & set(cases_at(base)))
    ours = outputs(ROOT, ROOT / "target", names)
    theirs = base_outputs(base, names)
    differ = [n for n in names if n in theirs and ours[n] != theirs[n]]
    if differ:
        print(f"old-pipeline-same: the old pipeline's `cpl build -o -` changed on {len(differ)} of "
              f"{len(names)} cases against {base[:9]}; the freeze keeps its output byte-identical "
              f"(handbook/design/next/migration.md#rules, rule 9):", file=sys.stderr)
        for n in differ[:40]:
            print(f"  {n}", file=sys.stderr)
        return 1
    print(f"old-pipeline-same: {len(names)} cases byte-identical to {base[:9]}")
    return 0


def cases_at(ref: str) -> list[str]:
    return [l for l in git("ls-tree", "-r", "--name-only", ref, "--", "spec/conformance").splitlines()
            if l.endswith(".cp") and l.count("/") == 3]


if __name__ == "__main__":
    sys.exit(main())
