#!/usr/bin/env python3
"""Bug-catch rate: revert past compiler fixes one at a time, run the suite.

For each fix commit (subject `syntax|check|ir|interp|codegen: ...` naming a
fix and touching compiler `src/`), the compiler-source part
of the commit is reverse-applied (three-way when it no longer applies cleanly) in a scratch worktree at HEAD, cpl-conform
is rebuilt, and the conformance suite is rerun. The fix is caught when a case
that passes at HEAD fails with the fix reverted. Tests are never reverted.
Commits that no longer reverse-apply or build are reported, not counted.

    uv run scripts/conformance-bugcatch.py [--limit N] [--commits SHA...]
        [--work DIR] [--out plan/domains/spec/bugcatch.md]
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATES = ["cpl-syntax", "cpl-check", "cpl-ir", "cpl-interp", "cpl-codegen"]
SRC = [f"crates/{c}/src" for c in CRATES]
SUBJECT = re.compile(r"^(syntax|check|ir|interp|codegen)(\([^)]*\))?: (.*)$")
FIXY = re.compile(r"\b(fix|fixes|reject|accept|wrong|crash|panic|hang|trap|miscompil\w*|regression|stop|bug)\b", re.I)


def git(*args, cwd=ROOT, check=True, **kw):
    return subprocess.run(["git", *args], cwd=cwd, check=check, text=True, capture_output=True, **kw)


def src_only(paths):
    return [p for p in paths if any(p.startswith(s + "/") for s in SRC) and "/tests/" not in p]


def candidates(limit):
    log = git("log", "--no-merges", "--format=%H%x09%s", "HEAD", "--", *SRC).stdout.splitlines()
    out = []
    for line in log:
        sha, subj = line.split("\t", 1)
        m = SUBJECT.match(subj)
        if not m or not FIXY.search(m.group(3).replace("fix-it", "")):
            continue
        files = git("show", "--format=", "--name-only", sha).stdout.split()
        if src_only(files):
            out.append((sha, subj))
        if len(out) >= limit:
            break
    return out


def run_suite(wt, env, board):
    build = subprocess.run(["cargo", "build", "--release", "-q", "-p", "cpl-conform"], cwd=wt, env=env,
                           capture_output=True, text=True)
    if build.returncode:
        return None
    exe = Path(env["CARGO_TARGET_DIR"]) / "release" / "cpl-conform"
    subprocess.run([str(exe), "conformance", "--write", "--scoreboard", str(board)], cwd=wt, env=env,
                   capture_output=True, text=True)
    return json.loads(board.read_text())["suites"]["conformance"]["cases"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=40)
    ap.add_argument("--commits", nargs="*")
    ap.add_argument("--work", default=str(ROOT.parent / "cpl-bugcatch"))
    ap.add_argument("--out", default=str(ROOT / "plan/domains/spec/bugcatch.md"))
    a = ap.parse_args()
    work = Path(a.work)
    wt = work / "tree"
    work.mkdir(parents=True, exist_ok=True)
    if wt.exists():
        git("worktree", "remove", "--force", str(wt), check=False)
        shutil.rmtree(wt, ignore_errors=True)
    head = git("rev-parse", "--short", "HEAD").stdout.strip()
    git("worktree", "add", "--detach", str(wt), "HEAD")
    env = dict(os.environ, CARGO_TARGET_DIR=str(work / "target"))
    todo = [(c, git("log", "-1", "--format=%s", c).stdout.strip()) for c in a.commits] if a.commits else candidates(a.limit)
    rows = []
    try:
        base = run_suite(wt, env, work / "base.json")
        if base is None:
            sys.exit("bugcatch: HEAD does not build")
        for sha, subj in todo:
            git("checkout", "-f", "-q", "HEAD", cwd=wt)
            git("clean", "-fdq", cwd=wt)
            files = src_only(git("show", "--format=", "--name-only", sha).stdout.split())
            patch = git("diff", f"{sha}^", sha, "--", *files).stdout
            ap_ = subprocess.run(["git", "apply", "-R"], cwd=wt, input=patch, text=True, capture_output=True)
            if ap_.returncode:
                git("checkout", "-f", "-q", "HEAD", cwd=wt)
                ap_ = subprocess.run(["git", "apply", "-R", "--3way"], cwd=wt, input=patch, text=True,
                                     capture_output=True)
                if ap_.returncode == 0 and git("diff", "--name-only", "--diff-filter=U", cwd=wt).stdout.strip():
                    ap_ = subprocess.CompletedProcess(ap_.args, 1)
            if ap_.returncode:
                rows.append((sha, subj, "does not reverse-apply", []))
                print(f"{sha[:8]} skip (does not reverse-apply)", flush=True)
                continue
            got = run_suite(wt, env, work / "rev.json")
            if got is None:
                rows.append((sha, subj, "revert does not build", []))
                print(f"{sha[:8]} skip (does not build)", flush=True)
                continue
            broke = sorted(k for k, v in base.items() if v == "pass" and got.get(k) != "pass")
            rows.append((sha, subj, "caught" if broke else "missed", broke))
            print(f"{sha[:8]} {'caught' if broke else 'MISSED'} ({len(broke)} cases)", flush=True)
    finally:
        git("worktree", "remove", "--force", str(wt), check=False)
    caught = sum(r[2] == "caught" for r in rows)
    missed = sum(r[2] == "missed" for r in rows)
    lines = [
        "# Bug-catch rate",
        "",
        f"Generated by `uv run scripts/conformance-bugcatch.py` at `{head}`: each compiler fix's `src/` change",
        "reverse-applied alone, then the conformance suite rerun. Caught = a case that passes at HEAD fails.",
        "",
        f"**{caught} of {caught + missed} revertible fixes caught"
        + (f" ({100 * caught // max(1, caught + missed)}%)" if caught + missed else "")
        + f"**; {len(rows) - caught - missed} of {len(rows)} no longer revert or build on HEAD.",
        "",
        "| commit | subject | result | cases that fail |",
        "|---|---|---|---|",
    ]
    for sha, subj, res, broke in rows:
        shown = ", ".join(f"`{b}`" for b in broke[:3]) + (f" +{len(broke) - 3}" if len(broke) > 3 else "")
        lines.append(f"| `{sha[:8]}` | {subj.replace('|', '/')} | {res} | {shown} |")
    Path(a.out).write_text("\n".join(lines) + "\n")
    print(f"bugcatch: {caught}/{caught + missed} caught, {len(rows) - caught - missed} not revertible; wrote {a.out}")


if __name__ == "__main__":
    main()
