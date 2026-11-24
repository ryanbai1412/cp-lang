#!/usr/bin/env python3
"""Run a nextest command; fail only on the tests that pass at the base.

    new_reds.py [--base REV] -- CMD...

A land is refused for the reds it brings, not for main's own (ci/R40; Perf C's
-Wredundant-move fix was refused for 4 tests red on clean main). CMD runs
as given. When it fails, each test its FAIL lines name is run again at the
base (default HEAD, which the commit tier stages against) in a scratch
worktree with its own target dir, by the same CMD with `-E` naming only
those tests. A test failing there too is printed as "pre-existing red,
owner <lane>" and logged for the culprit finder, and doesn't refuse; one
that passes there, or is new, is run again alone at the tip: one that passes
alone fails only under the full run's load or order (a race), so it is printed
as "flaky at the tip, owner <lane>", logged, and doesn't refuse; CI's full run
still sees it. One that fails alone too refuses. A base that doesn't build proves
nothing, so its tests refuse. Results are cached per base tree, so the
next land on the same base runs nothing again. A failure no FAIL line
names (a build error) refuses as before.
"""

import argparse
import contextlib
import fcntl
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from culprit import describe, failing_tests, source_of


# What git exports to a hook to name the repo it runs for: a pre-commit hook gets the
# relative GIT_INDEX_FILE=.git/index, which in the base worktree (where .git is a file)
# fails every checkout with "index.lock: Not a directory".
REPO_ENV = ("GIT_DIR", "GIT_INDEX_FILE", "GIT_WORK_TREE", "GIT_PREFIX", "GIT_COMMON_DIR")


def base_env(env=None):
    """`env` (default os.environ) for a run in the base worktree: no hook repo variables."""
    return {k: v for k, v in (os.environ if env is None else env).items() if k not in REPO_ENV}


def git(*args, cwd=None, check=True, code=False):
    env = base_env() if cwd is not None else None
    done = subprocess.run(["git", *args], cwd=cwd, env=env, check=check, text=True, capture_output=True)
    return done.returncode if code else done.stdout.strip()


def tee(cmd, cwd=None, env=None):
    """Runs `cmd`, streaming its output to stderr; returns (code, output)."""
    proc = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    out = []
    for line in proc.stdout:
        sys.stderr.write(line)
        out.append(line)
    return proc.wait(), "".join(out)


# An aggregate test's rows (`cpl-finding: <test> <row> +<excess>`): one test id covers
# every file and cause, so with it red at the base a land is judged by its rows.
FINDING = re.compile(r"cpl-finding: (\S+) (.+) \+(\d+)\s*$", re.M)


def findings(out):
    """{test name: {row: excess}} from an output's cpl-finding lines."""
    rows = {}
    for test, row, n in FINDING.findall(out):
        rows.setdefault(test, {})[row] = int(n)
    return rows


def only(cmd, tests):
    """`cmd` with its -E filter replaced by one naming just `tests`."""
    expr = " | ".join(f"(binary_id({b}) & test(={t}))" for b, t in tests)
    cmd = list(cmd)
    if "-E" in cmd:
        cmd[cmd.index("-E") + 1] = expr
    else:
        cmd += ["-E", expr]
    return cmd


def base_checkout(common, base, fresh=False, wt=None):
    """The scratch base worktree `wt` (default slot 0's) at `base`, rebuilt when it can't
    check out (slice 3's `git checkout -f --detach` failed in it, and no base comparison ran)."""
    wt = wt or common / "cpl-base-wt"
    if not fresh and (wt / ".git").exists() and not git("checkout", "-q", "-f", "--detach", base, cwd=wt, check=False, code=True):
        return wt
    if wt.exists():
        print(f"new-reds: rebuilding the base worktree {wt}", file=sys.stderr)
        target = wt / "target"
        if target.is_symlink():
            target.unlink()
        shutil.rmtree(wt, ignore_errors=True)
    git("worktree", "prune", cwd=common)
    git("worktree", "add", "-q", "-f", "--detach", str(wt), base, cwd=common)
    return wt


def save_cache(cache_file, cache):
    """Writes the base-verdict cache whole: two lands of one clone share it, and a reader
    never sees half a file."""
    cache_file.parent.mkdir(exist_ok=True)
    tmp = cache_file.with_name(f"{cache_file.name}.{os.getpid()}")
    tmp.write_text(json.dumps(cache, indent=1, sort_keys=True) + "\n")
    tmp.replace(cache_file)


BASE_SLOTS = 3


@contextlib.contextmanager
def base_slot(common, base):
    """(worktree, CARGO_TARGET_DIR) at `base`, held under a flock through checkout, build and
    rerun. Concurrent lands of one clone shared one worktree and target, so one's checkout
    swapped the source under another's build (d2fc481a: "the base didn't build", E0599 on a
    variant the base had, and main's reds counted as the change's). Each slot keeps its own
    worktree and target, so a build there stays incremental and never mixes two bases."""
    root = Path(os.environ.get("CPL_BASE_TARGET", str(common / "cpl-base-target")))
    slots = [(common / ("cpl-base-wt" + (f"-{i}" if i else "")), Path(f"{root}-{i}" if i else root))
             for i in range(BASE_SLOTS)]
    held = None
    for wt, target in slots:
        f = open(f"{wt}.lock", "w")
        try:
            fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
            held = f, wt, target
            break
        except BlockingIOError:
            f.close()
    if held is None:
        wt, target = slots[os.getpid() % BASE_SLOTS]
        print(f"new-reds: every base worktree is busy; waiting for {wt}", file=sys.stderr)
        f = open(f"{wt}.lock", "w")
        fcntl.flock(f, fcntl.LOCK_EX)
        held = f, wt, target
    f, wt, target = held
    try:
        yield base_checkout(common, base, wt=wt), target
    finally:
        f.close()


def at_base(cmd, tests, base, common):
    """({test: 'fail' | 'pass' | 'unbuilt'}, findings) for `tests` at `base`."""
    with base_slot(common, base) as (wt, target):
        return rerun_at_base(cmd, tests, base, common, wt, target)


STALE_CPL = "this dev `cpl` reads std from"


def still_at(wt, base):
    """Whether the worktree `wt` is still exactly `base`: its HEAD, and no tracked file changed.
    A land.sh from before per-land slots (or any base_checkout outside base_slot) can still
    check out another tree under a run, and that run measured neither base (Tools' and L2's
    silent_unknowns rows: cause:call +8 "there", +18 at a clean base)."""
    return (git("rev-parse", "HEAD", cwd=wt, check=False) == git("rev-parse", f"{base}^{{commit}}", cwd=wt, check=False)
            and git("diff", "--quiet", "HEAD", "--", cwd=wt, check=False, code=True) == 0)


def unjudgeable(code, out, wt, base):
    """Why the base run at `wt` can't stand for `base`, if it can't."""
    if code not in (0, 4, 100):
        return "didn't build"
    if STALE_CPL in out:
        # Tools' trace_goldens at 82ccb3f98: a cpl built for another checkout's std
        # failed the base's tests, and the red passed as main's.
        return "ran a cpl built for another checkout"
    if not still_at(wt, base):
        return "moved to another tree during the run"
    return None


def rerun_at_base(cmd, tests, base, common, wt, target):
    env = {**base_env(), "CARGO_TARGET_DIR": str(target)}
    print(f"new-reds: rerunning {len(tests)} failing test(s) at the base {base[:9]}", file=sys.stderr)
    code, out = tee(only(cmd, tests), cwd=wt, env=env)
    why = unjudgeable(code, out, wt, base)
    if why:
        # perf's ord and bw: the base "didn't build" (a test file missing from the
        # worktree, a generated msgs constant not found), and its reds counted as new.
        print(f"new-reds: the base {base[:9]} {why}; rebuilding its worktree and target and trying once more",
              file=sys.stderr)
        wt = base_checkout(common, base, fresh=True, wt=wt)
        # A stale build there (d2fc481a's cpl-interp against an old cpl-rt) fails the
        # same way in a fresh worktree; the slot is ours, so its target starts clean too.
        shutil.rmtree(target, ignore_errors=True)
        code, out = tee(only(cmd, tests), cwd=wt, env=env)
        why = unjudgeable(code, out, wt, base)
    failed = set(failing_tests(out))
    if why:
        print(f"new-reds: the base {base[:9]} {why} in {wt}, so its tests can't be compared; "
              "the reds below count as this change's", file=sys.stderr)
        return {t: "unbuilt" for t in tests}, {}
    return {t: "fail" if t in failed else "pass" for t in tests}, findings(out)


def owner(test, base):
    path = source_of(test, ".")
    sha = git("log", "-1", "--format=%H", base, "--", path or f"crates/{test[0].split('::')[0]}", check=False)
    return (describe(sha, ".")[1] if sha else "") or "unknown"


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", default="HEAD")
    ap.add_argument("cmd", nargs=argparse.REMAINDER)
    a = ap.parse_args(argv)
    cmd = a.cmd[1:] if a.cmd[:1] == ["--"] else a.cmd
    code, out = tee(cmd)
    if code == 0:
        return 0
    tests = failing_tests(out)
    if not tests:
        return code
    common = Path(git("rev-parse", "--path-format=absolute", "--git-common-dir"))
    base = git("rev-parse", a.base)
    cache_file = common / "cpl-base-tests" / f"{git('rev-parse', base + '^{tree}')}.json"
    cache = json.loads(cache_file.read_text()) if cache_file.is_file() else {}
    rows = findings(out)
    todo = [t for t in tests if " ".join(t) not in cache or (t[1] in rows and f"rows {' '.join(t)}" not in cache)]
    if todo:
        results, base_rows = at_base(cmd, todo, base, common)
        for t, r in results.items():
            if r != "unbuilt":
                cache[" ".join(t)] = r
                cache[f"rows {' '.join(t)}"] = base_rows.get(t[1], {})
        save_cache(cache_file, cache)
    new = [t for t in tests if cache.get(" ".join(t)) != "fail"]
    grown = {}
    for t in tests:
        theirs = cache.get(f"rows {' '.join(t)}", {})
        # A base that reports no rows predates them (or failed on a build): judge by the id.
        worse = {r: n for r, n in rows.get(t[1], {}).items() if theirs and t not in new and n > theirs.get(r, 0)}
        if worse:
            grown[t] = {r: (theirs.get(r, 0), n) for r, n in worse.items()}
    old = [t for t in tests if t not in new and t not in grown]
    with open(common / "cpl-preexisting-reds.log", "a") as log:
        for t in old:
            who = owner(t, base)
            print(f"new-reds: pre-existing red, owner {who}: {' '.join(t)} fails at the base {base[:9]} too; "
                  "main's culprit job routes it, not refusing", file=sys.stderr)
            log.write(f"{base} {who} {' '.join(t)}\n")
    if new:
        print(f"new-reds: trying {len(new)} new red(s) alone at the tip", file=sys.stderr)
        code, again = tee(only(cmd, new))
        if code in (0, 4, 100):
            still = set(failing_tests(again))
            flaky = [t for t in new if t not in still]
            new = [t for t in new if t in still]
            with open(common / "cpl-flaky-tests.log", "a") as log:
                for t in flaky:
                    who = owner(t, base)
                    print(f"new-reds: flaky at the tip, owner {who}: {' '.join(t)} failed in the full run, "
                          "passes alone here and at the base; not refusing", file=sys.stderr)
                    log.write(f"{base} {who} {' '.join(t)}\n")
    for t, worse in grown.items():
        print(f"new-reds: NEW rows in {' '.join(t)}, red at the base {base[:9]} too; this change grows "
              + ", ".join(f"{r} (+{b} there, +{n} here)" for r, (b, n) in sorted(worse.items())[:20])
              + (f" and {len(worse) - 20} more" if len(worse) > 20 else ""), file=sys.stderr)
    for t in new:
        print(f"new-reds: NEW red: {' '.join(t)} passes at the base {base[:9]} (or doesn't build there) "
              "and fails with this change", file=sys.stderr)
    return 1 if new or grown else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
