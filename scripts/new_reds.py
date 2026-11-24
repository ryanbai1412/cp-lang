#!/usr/bin/env python3
"""Run a nextest command; fail only on the tests that pass at the base.

    new_reds.py [--base REV] -- CMD...

A land is refused for the reds it brings, not for main's own (Perf C's
-Wredundant-move fix was refused for 4 tests red on clean main). CMD runs
as given. When it fails, each test its FAIL lines name is run again at the
base (default HEAD, which the commit tier stages against) in a scratch
worktree with its own target dir, by the same CMD with `-E` naming only
those tests. A test failing there too is printed as "pre-existing red,
owner <lane>" and logged for the culprit finder, and doesn't refuse; one
that passes there, or is new, does. A base that doesn't build proves
nothing, so its tests refuse. Results are cached per base tree, so the
next land on the same base runs nothing again. A failure no FAIL line
names (a build error) refuses as before.
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from culprit import describe, failing_tests, source_of


def git(*args, cwd=None, check=True):
    return subprocess.run(["git", *args], cwd=cwd, check=check, text=True, capture_output=True).stdout.strip()


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
    wt = common / "cpl-base-wt"
    if (wt / ".git").exists():
        git("checkout", "-q", "-f", "--detach", base, cwd=wt)
    else:
        git("worktree", "prune")
        git("worktree", "add", "-q", "-f", "--detach", str(wt), base)
    return wt


def at_base(cmd, tests, base, common):
    """{test: 'fail' | 'pass' | 'unbuilt'} for `tests` at `base`."""
    wt = base_checkout(common, base)
    env = {**os.environ, "CARGO_TARGET_DIR": os.environ.get("CPL_BASE_TARGET", str(common / "cpl-base-target"))}
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
    for t in new:
        print(f"new-reds: NEW red: {' '.join(t)} passes at the base {base[:9]} (or doesn't build there) "
              "and fails with this change", file=sys.stderr)
    return 1 if new else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
