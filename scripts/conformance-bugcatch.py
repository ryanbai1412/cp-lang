#!/usr/bin/env python3
"""Bug-catch rate: revert past compiler fixes one at a time, run the suite.

For each fix commit (subject `syntax|check|ir|interp|codegen: ...` naming a
fix and touching compiler `src/`), the compiler-source part of the commit is
reverse-applied (three-way when it no longer applies cleanly) in a scratch
worktree at HEAD, cpl-conform is rebuilt, and the conformance suite is rerun.
The fix is caught when a case that passes at HEAD fails with the fix reverted.
A fix that no longer reverts on HEAD is measured at its own commit with HEAD's
cases. Tests are never reverted. Commits that do not build are reported, not
counted.

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
FIXY = re.compile(r"\b(fix|fixes|reject|accept|wrong|crash|panic|hang|trap|miscompil\w*|regression|stop|bug)\b", re.IGNORECASE)


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


HUNG = {"(suite timed out: the revert hangs a case)": "hang"}
SUITE_TIMEOUT = 900


def broken(base, got):
    if got is HUNG:
        return list(HUNG)
    return sorted(k for k, v in base.items() if v == "pass" and got.get(k) != "pass")


def run_suite(wt, env, board, runner=None):
    """`runner` is HEAD's cpl-conform: it drives that tree's `cpl` binary
    through `CPL_CONFORM_CLI`."""
    crates = ["-p", "cpl-cli"] if runner else ["-p", "cpl-conform", "-p", "cpl-cli"]
    build = subprocess.run(["cargo", "build", "--release", "-q", *crates], cwd=wt, env=env,
                           capture_output=True, text=True, check=False)
    if build.returncode:
        return None
    exe = Path(env["CARGO_TARGET_DIR"]) / "release" / "cpl-conform"
    if runner:
        exe, env = runner, dict(env, CPL_CONFORM_CLI=str(Path(env["CARGO_TARGET_DIR"]) / "release" / "cpl"))
    try:
        subprocess.run([str(exe), "conformance", "--write", "--scoreboard", str(board)], cwd=wt, env=env,
                       capture_output=True, text=True, check=False, timeout=SUITE_TIMEOUT)
    except subprocess.TimeoutExpired:
        return HUNG
    return json.loads(board.read_text())["suites"]["conformance"]["cases"]


def at_fix(wt, env, work, sha, files, head):
    """Fallback when the fix no longer reverse-applies on HEAD: check out the
    fix commit with HEAD's cases, then the same tree with the fix's src/ files
    at their parent. Caught = a case that passes with the fix fails without."""
    git("checkout", "-f", "-q", sha, cwd=wt)
    git("clean", "-fdq", cwd=wt)
    git("checkout", head, "--", "spec/conformance", cwd=wt)
    # HEAD's runner even when the tree has its own, so every tree gets the
    # same harness (case timeouts, case ids); only the compiler differs.
    runner = work / "runner" / "cpl-conform"
    fixed = run_suite(wt, env, work / "fix.json", runner)
    if fixed is None or fixed is HUNG:
        return "fix commit does not build or hangs", []
    gone = [f for f in files if git("cat-file", "-e", f"{sha}^:{f}", cwd=wt, check=False).returncode]
    keep = [f for f in files if f not in gone]
    if keep:
        git("checkout", f"{sha}^", "--", *keep, cwd=wt)
    for f in gone:
        (wt / f).unlink(missing_ok=True)
    got = run_suite(wt, env, work / "rev.json", runner)
    git("checkout", "-f", "-q", head, cwd=wt)
    git("clean", "-fdq", cwd=wt)
    if got is None:
        return "revert does not build", []
    broke = broken(fixed, got)
    return ("caught at fix" if broke else "missed at fix"), broke


def main():
    global SUITE_TIMEOUT
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=40)
    ap.add_argument("--timeout", type=int, default=SUITE_TIMEOUT, help="seconds per suite run; a hang counts as caught")
    ap.add_argument("--commits", nargs="*")
    ap.add_argument("--work", default=str(ROOT.parent / "cpl-bugcatch"))
    ap.add_argument("--out", default=str(ROOT / "plan/domains/spec/bugcatch.md"))
    a = ap.parse_args()
    SUITE_TIMEOUT = a.timeout
    work = Path(a.work)
    wt = work / "tree"
    work.mkdir(parents=True, exist_ok=True)
    if wt.exists():
        git("worktree", "remove", "--force", str(wt), check=False)
        shutil.rmtree(wt, ignore_errors=True)
    head = git("rev-parse", "HEAD").stdout.strip()
    git("worktree", "prune")
    git("worktree", "add", "--detach", str(wt), "HEAD")
    env = dict(os.environ, CARGO_TARGET_DIR=str(work / "target"))
    todo = [(c, git("log", "-1", "--format=%s", c).stdout.strip()) for c in a.commits] if a.commits else candidates(a.limit)
    rows = []
    try:
        base = run_suite(wt, env, work / "base.json")
        if base is None or base is HUNG:
            sys.exit("bugcatch: HEAD does not build or the suite hangs")
        (work / "runner").mkdir(exist_ok=True)
        shutil.copy2(Path(env["CARGO_TARGET_DIR"]) / "release" / "cpl-conform", work / "runner" / "cpl-conform")
        for sha, subj in todo:
            git("checkout", "-f", "-q", head, cwd=wt)
            git("clean", "-fdq", cwd=wt)
            files = src_only(git("show", "--format=", "--name-only", sha).stdout.split())
            patch = git("diff", f"{sha}^", sha, "--", *files).stdout
            ap_ = subprocess.run(["git", "apply", "-R"], cwd=wt, input=patch, text=True, capture_output=True, check=False)
            if ap_.returncode:
                git("checkout", "-f", "-q", "HEAD", cwd=wt)
                ap_ = subprocess.run(["git", "apply", "-R", "--3way"], cwd=wt, input=patch, text=True,
                                     capture_output=True, check=False)
                if ap_.returncode == 0 and git("diff", "--name-only", "--diff-filter=U", cwd=wt).stdout.strip():
                    ap_ = subprocess.CompletedProcess(ap_.args, 1)
            if ap_.returncode:
                res, broke = at_fix(wt, env, work, sha, files, head)
                rows.append((sha, subj, res, broke))
                print(f"{sha[:8]} {res} ({len(broke)} cases)", flush=True)
                continue
            got = run_suite(wt, env, work / "rev.json")
            if got is None:
                rows.append((sha, subj, "revert does not build", []))
                print(f"{sha[:8]} skip (does not build)", flush=True)
                continue
            broke = broken(base, got)
            rows.append((sha, subj, "caught" if broke else "missed", broke))
            print(f"{sha[:8]} {'caught' if broke else 'MISSED'} ({len(broke)} cases)", flush=True)
    finally:
        git("worktree", "remove", "--force", str(wt), check=False)
    caught = sum(r[2].startswith("caught") for r in rows)
    missed = sum(r[2].startswith("missed") for r in rows)
    lines = [
        "# Bug-catch rate",
        "",
        f"Generated by `uv run scripts/conformance-bugcatch.py` at `{head[:9]}`: each compiler fix's `src/` change",
        "reverse-applied alone, then the conformance suite rerun. Caught = a case that passes at HEAD fails.",
        "A fix that no longer reverse-applies on HEAD is measured at the fix commit with HEAD's cases",
        "(\"at fix\"): its `src/` files are put back to the parent and the cases that passed are rerun.",
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
