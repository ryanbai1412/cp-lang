#!/usr/bin/env python3
"""The freeze's behaviour half: a change to a crate the old pipeline is built from leaves what
it does the same. `debt-ratchet.py`'s `frozen_edits` sees lines added to the frozen crates;
this sees a change outside them that changes them anyway, as 9ad9cbbd1 did by changing
cpl-syntax under the old checker. Every conformance case runs at HEAD and at the merge base
with origin/main, and a case fails the commit tier when what it does differs:

- `cpl build <case> -o -`: its exit status, the codes of its diagnostics, and its C++ with
  generated symbols' numeric suffixes masked (`f_S23_282` reads as `f_S#_#`);
- when that C++ differs anyway (std-sourced text the pinned std changed), `cpl run <case>` on
  its `// input:`: exit status, stdout and the codes of what it reports (its trap class).

A case whose masked C++ differs but whose run is the same passes, so renumbering and std
text the pin changes are not a behaviour change, and a case the base refused may change; an
`// engines: next` case is not the old pipeline's and isn't run (coordinator ruling at the 6a31a3747^ re-pin).
The base's outputs are cached by the tree of every input, so a lane pays for them once per main.

    uv run python scripts/old-pipeline-same.py [--base REF]

Rule: handbook/design/next/migration.md#rules (9: the old pipeline is frozen).
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import hashlib
import json
import os
import re
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
# Bumped when what a case's record holds changes, so an older cache isn't compared.
RECORD = "2"
# A generated symbol's numeric suffix, which numbering std's items differently renumbers.
SUFFIX = re.compile(rb"(?<=_)\d+\b|(?<=_S)\d+(?=_)")
CODE = re.compile(rb"\b[ERWL]\d{4}\b")


def digest(*parts: bytes) -> str:
    return hashlib.sha256(b"\0".join(parts)).hexdigest()


def codes(text: bytes) -> bytes:
    return b" ".join(sorted(set(CODE.findall(text))))


def case_input(text: str) -> bytes:
    """A case's `// input: "..."` header, its escapes read, or nothing."""
    m = re.search(r'^// input: (".*")$', text, re.M)
    return json.loads(m.group(1)).encode() if m else b""


def git(*args: str, cwd: Path = ROOT) -> str:
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()


def key(ref: str) -> str:
    """The trees of every input at `ref`: equal keys build the same old pipeline."""
    trees = git("ls-tree", ref, "--", *INPUTS) + "\n".join(git("ls-tree", ref, "--", f"{p}/") for p in INPUTS)
    return hashlib.sha256(trees.encode()).hexdigest()[:20]


def cases(tree: Path) -> list[str]:
    """The conformance cases the old pipeline runs: not `// engines: next` ones."""
    return sorted(str(p.relative_to(tree)) for p in (tree / "spec/conformance").glob("*/*.cp")
                  if not re.search(r"^// engines: .*\bnext\b", p.read_text(encoding="utf-8"), re.M))


def outputs(tree: Path, target: Path, names: list[str]) -> dict[str, dict[str, str]]:
    """Each case's record, from a cpl built in `tree` into `target`: `build` (exit status and
    diagnostic codes), `cpp` (the C++ with symbol suffixes masked) and `run` (exit status,
    stdout and reported codes of `cpl run`, for a case that builds)."""
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    subprocess.run(["cargo", "build", "-q", "-p", "cpl-cli"], cwd=tree, env=env, check=True)
    cpl = target / "debug" / "cpl"

    def one(name: str) -> tuple[str, dict[str, str]]:
        b = subprocess.run([cpl, "build", name, "-o", "-"], cwd=tree, capture_output=True, timeout=120)
        rec = {"exit": str(b.returncode), "build": digest(str(b.returncode).encode(), codes(b.stderr)),
               "cpp": digest(SUFFIX.sub(b"#", b.stdout))}
        if b.returncode == 0:
            given = case_input((tree / name).read_text(encoding="utf-8"))
            try:
                r = subprocess.run([cpl, "run", name], cwd=tree, input=given, capture_output=True, timeout=120)
                rec["run"] = digest(str(r.returncode).encode(), r.stdout, codes(r.stderr))
            except subprocess.TimeoutExpired:
                rec["run"] = "timeout"
        return name, rec

    with cf.ThreadPoolExecutor(os.cpu_count() or 4) as ex:
        return dict(ex.map(one, names))


def base_outputs(base: str, names: list[str]) -> dict[str, str]:
    path = CACHE / f"{key(base)}.v{RECORD}.json"
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
    differ, masked = changed(names, ours, theirs)
    if differ:
        print(f"old-pipeline-same: the old pipeline does something else on {len(differ)} of "
              f"{len(names)} cases against {base[:9]}; the freeze keeps its behaviour "
              f"(handbook/design/next/migration.md#rules, rule 9):", file=sys.stderr)
        for n, why in differ:
            print(f"  {n}: {why}", file=sys.stderr)
        return 1
    print(f"old-pipeline-same: {len(names)} cases behave as at {base[:9]} "
          f"({masked} with other C++ but the same run)")
    return 0


def changed(names: list[str], ours: dict, theirs: dict) -> tuple[list[tuple[str, str]], int]:
    """The cases whose build or run differs, each with what differs, and how many differ
    only in masked C++ text while running the same. A case the base already refused may start
    building or be refused otherwise: the old pipeline only has to keep what it accepted
    (coordinator ruling at the 6a31a3747^ re-pin)."""
    differ, masked = [], 0
    for n in names:
        a, b = ours.get(n), theirs.get(n)
        if a is None or b is None or a == b:
            continue
        if b["exit"] != "0":
            continue
        if a["build"] != b["build"]:
            differ.append((n, "exit status or diagnostic codes"))
        elif a.get("run") != b.get("run"):
            differ.append((n, "run: exit status, stdout or trap class"))
        else:
            masked += 1
    return differ, masked


def cases_at(ref: str) -> list[str]:
    return [l for l in git("ls-tree", "-r", "--name-only", ref, "--", "spec/conformance").splitlines()
            if l.endswith(".cp") and l.count("/") == 3]


if __name__ == "__main__":
    sys.exit(main())
