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
import json
import os
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


def only(cmd, tests):
    """`cmd` with its -E filter replaced by one naming just `tests`."""
    expr = " | ".join(f"(binary_id({b}) & test(={t}))" for b, t in tests)
    cmd = list(cmd)
    if "-E" in cmd:
        cmd[cmd.index("-E") + 1] = expr
    else:
        cmd += ["-E", expr]
    return cmd


def base_checkout(common, base):
    """The scratch base worktree at `base`, rebuilt when it can't check out (slice 3's
    `git checkout -f --detach` failed in it, and no base comparison ran)."""
    wt = common / "cpl-base-wt"
    if (wt / ".git").exists() and not git("checkout", "-q", "-f", "--detach", base, cwd=wt, check=False, code=True):
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


def at_base(cmd, tests, base, common):
    """{test: 'fail' | 'pass' | 'unbuilt'} for `tests` at `base`."""
    wt = base_checkout(common, base)
    env = {**base_env(), "CARGO_TARGET_DIR": os.environ.get("CPL_BASE_TARGET", str(common / "cpl-base-target"))}
    print(f"new-reds: rerunning {len(tests)} failing test(s) at the base {base[:9]}", file=sys.stderr)
    code, out = tee(only(cmd, tests), cwd=wt, env=env)
    failed = set(failing_tests(out))
    if code not in (0, 4, 100):
        return {t: "fail" if t in failed else "unbuilt" for t in tests}
    return {t: "fail" if t in failed else "pass" for t in tests}


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
    todo = [t for t in tests if " ".join(t) not in cache]
    if todo:
        for t, r in at_base(cmd, todo, base, common).items():
            if r != "unbuilt":
                cache[" ".join(t)] = r
        cache_file.parent.mkdir(exist_ok=True)
        cache_file.write_text(json.dumps(cache, indent=1, sort_keys=True) + "\n")
    new = [t for t in tests if cache.get(" ".join(t)) != "fail"]
    old = [t for t in tests if t not in new]
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
    for t in new:
        print(f"new-reds: NEW red: {' '.join(t)} passes at the base {base[:9]} (or doesn't build there) "
              "and fails with this change", file=sys.stderr)
    return 1 if new else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
