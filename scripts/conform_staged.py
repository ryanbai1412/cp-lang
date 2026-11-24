#!/usr/bin/env python3
"""Run the staged spec/conformance cases through cpl-conform (commit tier).

    scripts/conform_staged.py CPL_CONFORM CASE...
    scripts/conform_staged.py --build CASE...   build cpl-cli and cpl-conform first (land.sh)
    scripts/conform_staged.py --base REV [--build] ...   a case failing at REV too is main's
    scripts/conform_staged.py --base REV --check PROGRAM...   `cpl check` them, same rule (land.sh)

A staged case that fails blocks the commit unless HEAD's
research/scoreboard.json already records it as failing: no new failing case,
and no marker line in the case (`conformance_failing` counts the board's
failures, target 0). cpl-conform's message names the fix, e.g. the line to
mark `// <- here` (conformance/R1, conformance/R3). Each case runs twice, and one whose verdict differs
between the runs is a flake and blocks too. With `--base`, a new failure is
run again at REV (new_reds.py's scratch worktree and target, cached per base
tree): failing there too, it is printed as "pre-existing red, owner <lane>"
and logged for the culprit finder, and doesn't refuse (f33d10f7 was refused
for a case red on main). A base that doesn't build, or lacks the case, refuses.
`--check` holds corpus programs calling a changed std signature to the same
rule with `cpl check` (VM-expr was refused for noi09p4/spec.cp, red on main).
`--all` runs the whole suite once when the change since REV touches a compiler
crate or a std module the prelude reaches, under the same rule; a new red that
passes alone at the tip is a race, logged and not refused (cea2d50e4 changed
only cpl-check, cpl-ir and prelude std, and std-graph/bfs-over-named-product-domain
went red on main: its `bfs` comes from the prelude, with no `use` to find it by).
"""
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PREFIX = "spec/conformance/"
BOARD = "research/scoreboard.json"
FAIL = re.compile(r"^  (\S+\.cp): (?:fail|bad|panic|infra)")
COMPILER = tuple(f"crates/{c}/" for c in (
    "cpl-syntax", "cpl-diag", "cpl-check", "cpl-ir", "cpl-interp", "cpl-codegen", "cpl-rt")) + (
    "crates/cpl-cli/src/", "crates/cpl-conform/src/")
USE_STD = re.compile(r"^\s*(?:pub\s+)?use\s+std\.(\w+)", re.M)
sys.path.insert(0, str(ROOT / "scripts"))


def board_failing(board_text):
    """Case ids the scoreboard JSON `board_text` records as not passing."""
    if not board_text:
        return set()
    cases = json.loads(board_text).get("suites", {}).get("conformance", {}).get("cases", {})
    return {c for c, v in cases.items() if v != "pass"}


def head_board():
    """HEAD's scoreboard, so staging a board edit can't excuse a failure."""
    r = subprocess.run(["git", "show", f"HEAD:{BOARD}"], cwd=ROOT, capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def new_failures(case, output, known):
    """The failure report for `case` in cpl-conform's --verbose output, unless
    the scoreboard already records it as failing."""
    if case in known:
        return []
    lines, out = output.splitlines(), []
    for i, line in enumerate(lines):
        m = FAIL.match(line)
        if m and m[1] == case:
            out.append(line)
            out += [l for l in lines[i + 1:i + 12] if l.startswith("  ") and not FAIL.match(l)]
    return out


def verdict(case, output):
    """`fail` when cpl-conform's --verbose output fails `case`, else `pass`."""
    return "fail" if any((m := FAIL.match(l)) and m[1] == case for l in output.splitlines()) else "pass"


def flake(case, first, second):
    """The report for a case whose two runs disagree, else nothing."""
    a, b = verdict(case, first), verdict(case, second)
    return [] if a == b else [f"  {case}: fail: flaky, {a} then {b} on the same build"]


def base_cache(base):
    """The cpl-base-tests cache file for `base`'s tree, and what it holds."""
    import new_reds
    common = Path(new_reds.git("rev-parse", "--path-format=absolute", "--git-common-dir", cwd=ROOT))
    cache_file = common / "cpl-base-tests" / f"{new_reds.git('rev-parse', base + '^{tree}', cwd=ROOT)}.json"
    return cache_file, json.loads(cache_file.read_text()) if cache_file.is_file() else {}


def base_suite_crashes(base, run=subprocess.run):
    """Whether the whole suite stops short at `base` (a stack overflow, the memory
    limit), cached with the base's verdicts so it runs once per base."""
    import new_reds
    cache_file, cache = base_cache(base)
    if "suite:crash" not in cache:
        common = cache_file.parent.parent
        wt = new_reds.base_checkout(common, base)
        target = Path(os.environ.get("CPL_BASE_TARGET", str(common / "cpl-base-target")))
        env = {**os.environ, "CARGO_TARGET_DIR": str(target)}
        print(f"conform_staged: the suite stopped short at the tip; running it at the base {base[:9]}",
              file=sys.stderr)
        if run(["cargo", "build", "-q", "-p", "cpl-cli", "-p", "cpl-conform"], cwd=wt, env=env).returncode:
            return False
        r = run([str(target / "debug" / "cpl-conform"), "conformance"], cwd=wt, env=env,
                capture_output=True, text=True)
        cache["suite:crash"] = r.returncode not in (0, 1)
        cache_file.parent.mkdir(exist_ok=True)
        cache_file.write_text(json.dumps(cache, indent=1, sort_keys=True) + "\n")
    return cache["suite:crash"]


def base_verdicts(cases, base, run=subprocess.run):
    """{case: 'fail' | 'pass' | 'absent' | 'unbuilt'} for `cases` at `base`."""
    import new_reds
    cache_file, cache = base_cache(base)
    common = cache_file.parent.parent
    todo = [c for c in cases if f"conform:{c}" not in cache]
    if todo:
        with new_reds.base_slot(common, base) as (wt, target):
            env = {**os.environ, "CARGO_TARGET_DIR": str(target)}
            print(f"conform_staged: rerunning {len(todo)} failing case(s) at the base {base[:9]}", file=sys.stderr)
            if run(["cargo", "build", "-q", "-p", "cpl-cli", "-p", "cpl-conform"], cwd=wt, env=env).returncode:
                return {c: cache.get(f"conform:{c}", "unbuilt") for c in cases}
            for c in todo:
                if not (wt / PREFIX / c).is_file():
                    cache[f"conform:{c}"] = "absent"
                    continue
                r = run([str(target / "debug" / "cpl-conform"), "conformance", "--verbose", c], cwd=wt, env=env,
                        capture_output=True, text=True)
                cache[f"conform:{c}"] = verdict(c, r.stdout + r.stderr)
            new_reds.save_cache(cache_file, cache)
    return {c: cache[f"conform:{c}"] for c in cases}


def base_checks(programs, base, run=subprocess.run):
    """{program: 'fail' | 'pass' | 'absent' | 'unbuilt'} for `cpl check` of `programs` at `base`."""
    import new_reds
    cache_file, cache = base_cache(base)
    common = cache_file.parent.parent
    todo = [p for p in programs if f"check:{p}" not in cache]
    if todo:
        with new_reds.base_slot(common, base) as (wt, target):
            env = {**os.environ, "CARGO_TARGET_DIR": str(target)}
            print(f"conform_staged: checking {len(todo)} failing program(s) at the base {base[:9]}", file=sys.stderr)
            if run(["cargo", "build", "-q", "-p", "cpl-cli"], cwd=wt, env=env).returncode:
                return {p: cache.get(f"check:{p}", "unbuilt") for p in programs}
            for p in todo:
                if not (wt / p).is_file():
                    cache[f"check:{p}"] = "absent"
                    continue
                r = run([str(target / "debug" / "cpl"), "check", p], cwd=wt, env=env, capture_output=True, text=True)
                cache[f"check:{p}"] = "fail" if r.returncode else "pass"
            new_reds.save_cache(cache_file, cache)
    return {p: cache[f"check:{p}"] for p in programs}


def check_programs(programs, base, run=subprocess.run):
    """`cpl check` each program at the tip; 1 if one fails that checks at `base`."""
    if run(["cargo", "build", "-q", "-p", "cpl-cli"], cwd=ROOT).returncode:
        return 1
    target = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
    cpl = str((target if target.is_absolute() else ROOT / target) / "debug" / "cpl")
    failing = []
    for p in programs:
        if not (ROOT / p).is_file():
            continue
        r = run([cpl, "check", p], cwd=ROOT, capture_output=True, text=True)
        if r.returncode:
            sys.stderr.write(r.stdout + r.stderr)
            failing.append(p)
    if not failing:
        return 0
    rev = subprocess.run(["git", "rev-parse", base], cwd=ROOT, capture_output=True, text=True).stdout.strip() or base
    at = base_checks(failing, rev, run)
    common = Path(subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], cwd=ROOT,
                                 capture_output=True, text=True).stdout.strip() or ROOT / ".git")
    new = []
    with open(common / "cpl-preexisting-reds.log", "a") as log:
        for p in failing:
            if at[p] != "fail":
                print(f"conform_staged: NEW red: {p} checks at the base {rev[:9]} ({at[p]}) and fails with "
                      "this change; update it in this change", file=sys.stderr)
                new.append(p)
                continue
            who = owner(p, rev, path=p)
            print(f"conform_staged: pre-existing red, owner {who}: {p} fails cpl check at the base {rev[:9]} "
                  "too; main's culprit job routes it, not refusing", file=sys.stderr)
            log.write(f"{rev} {who} check:{p}\n")
    return 1 if new else 0


def prelude_reach(read=lambda p: (ROOT / p).read_text(encoding="utf-8")):
    """The std files every program sees: std/prelude.cp and each module it `use`s, transitively."""
    seen, todo = set(), ["std/prelude.cp"]
    while todo:
        p = todo.pop()
        if p in seen:
            continue
        try:
            text = read(p)
        except OSError:
            continue
        seen.add(p)
        todo += [f"std/{m}.cp" for m in USE_STD.findall(text)]
    return seen


def needs_all(paths, reach):
    """Whether a change to `paths` can move any case: a compiler crate, or std the prelude reaches."""
    return any(p.startswith(COMPILER) or p in reach for p in paths)


def all_failing(output):
    """{case: its failure report} for every case cpl-conform's --verbose output fails."""
    return {m[1]: new_failures(m[1], output, set()) for l in output.splitlines() if (m := FAIL.match(l))}


def timed_out(case, output):
    """Whether cpl-conform's --verbose output fails `case` on its time limit."""
    return any("timed out after" in l for l in new_failures(case, output, set()))


def base_alone(case, base, run=subprocess.run):
    """cpl-conform's --verbose output for `case` alone at `base`, run now, under the tip's load."""
    import new_reds
    common = Path(new_reds.git("rev-parse", "--path-format=absolute", "--git-common-dir", cwd=ROOT))
    wt = new_reds.base_checkout(common, base)
    target = Path(os.environ.get("CPL_BASE_TARGET", str(common / "cpl-base-target")))
    env = {**os.environ, "CARGO_TARGET_DIR": str(target)}
    if run(["cargo", "build", "-q", "-p", "cpl-cli", "-p", "cpl-conform"], cwd=wt, env=env).returncode:
        return ""
    r = run([str(target / "debug" / "cpl-conform"), "conformance", "--verbose", case], cwd=wt, env=env,
            capture_output=True, text=True)
    return r.stdout + r.stderr


def run_all(conform, base, run=subprocess.run):
    """The whole suite at the tip; 1 if a case fails that passes at `base` and fails alone here too."""
    rev = subprocess.run(["git", "rev-parse", base], cwd=ROOT, capture_output=True, text=True).stdout.strip() or base
    t0 = time.monotonic()
    r = run([conform, "conformance", "--verbose"], cwd=ROOT, capture_output=True, text=True)
    failing = all_failing(r.stdout + r.stderr)
    if r.returncode not in (0, 1) and not base_suite_crashes(rev, run):
        print("\n".join((r.stdout + r.stderr).splitlines()[-5:]))
        print(f"conform_staged: the full suite stopped short (exit {r.returncode}), and it runs to the end at the base "
              f"{rev[:9]}; find the case with `cpl-conform conformance --verbose <dir>` per directory and fix it")
        return 1
    at = base_verdicts(sorted(failing), rev, run) if failing else {}
    common = Path(subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], cwd=ROOT,
                                 capture_output=True, text=True).stdout.strip() or ROOT / ".git")
    new = []
    with open(common / "cpl-preexisting-reds.log", "a") as log, open(common / "cpl-flaky-tests.log", "a") as flaky:
        for c in failing:
            if at[c] == "fail":
                who = owner(c, rev)
                print(f"conform_staged: pre-existing red, owner {who}: {PREFIX}{c} fails at the base {rev[:9]} too; "
                      "main's culprit job routes it, not refusing", file=sys.stderr)
                log.write(f"{rev} {who} conform:{c}\n")
                continue
            alone = run([conform, "conformance", "--verbose", c], cwd=ROOT, capture_output=True, text=True)
            if verdict(c, alone.stdout + alone.stderr) == "pass":
                who = owner(c, rev)
                print(f"conform_staged: flaky at the tip, owner {who}: {PREFIX}{c} failed in the full run, "
                      "passes alone; not refusing", file=sys.stderr)
                flaky.write(f"{rev} {who} conform:{c}\n")
                continue
            # Load, not the change (6c6573a25, codegen-only, at load avg 30): the base, timed right after, times out too.
            if timed_out(c, alone.stdout + alone.stderr) and timed_out(c, base_alone(c, rev, run)):
                who = owner(c, rev)
                print(f"conform_staged: load, owner {who}: {PREFIX}{c} times out alone here and at the base "
                      f"{rev[:9]} run right after it; not refusing", file=sys.stderr)
                flaky.write(f"{rev} {who} conform:{c}\n")
                continue
            print(f"conform_staged: NEW red: {PREFIX}{c} is {at[c]} at the base {rev[:9]} and fails with this change",
                  file=sys.stderr)
            new.append(c)
    if new:
        print("\n".join(l for c in new for l in failing[c]))
        print(f"conform_staged: {len(new)} case(s) of the full suite fail that pass at the base; fix them")
        return 1
    print(f"conform_staged: the full suite has no new red against {rev[:9]} ({time.monotonic() - t0:.0f} s)",
          file=sys.stderr)
    return 0


def owner(case, base, path=None):
    from culprit import describe
    import new_reds
    sha = new_reds.git("log", "-1", "--format=%H", base, "--", path or PREFIX + case, cwd=ROOT, check=False)
    return (describe(sha, str(ROOT))[1] if sha else "") or "unknown"


def main(argv, run=subprocess.run):
    base = None
    if argv[:1] == ["--base"]:
        base, argv = argv[1], argv[2:]
    if argv[:1] == ["--check"] and base:
        return check_programs(argv[1:], base, run)
    if argv[:1] == ["--all"] and base:
        paths = subprocess.run(["git", "diff", "--name-only", base, "HEAD"], cwd=ROOT, capture_output=True,
                               text=True).stdout.split()
        if not needs_all(paths, prelude_reach()):
            return 0
        if subprocess.run(["git", "diff", "--quiet", "HEAD", "--", "crates", "std", "Cargo.toml", "Cargo.lock"],
                          cwd=ROOT).returncode:
            print("conform_staged: uncommitted changes under crates/ or std/ would be tested in place of what "
                  "lands; commit or stash them", file=sys.stderr)
            return 1
        if run(["cargo", "build", "-q", "-p", "cpl-cli", "-p", "cpl-conform"], cwd=ROOT).returncode:
            return 1
        target = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
        return run_all(str((target if target.is_absolute() else ROOT / target) / "debug" / "cpl-conform"), base, run)
    if argv[:1] == ["--build"]:
        if run(["cargo", "build", "-q", "-p", "cpl-cli", "-p", "cpl-conform"], cwd=ROOT).returncode:
            return 1
        target = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
        argv = [str((target if target.is_absolute() else ROOT / target) / "debug" / "cpl-conform"), *argv[1:]]
    conform, paths = argv[0], argv[1:]
    known = board_failing(head_board())
    failing, bad = {}, []
    for path in paths:
        if not (ROOT / path).is_file():
            continue
        case = path.removeprefix(PREFIX)
        runs = [run([conform, "conformance", "--verbose", case], cwd=ROOT, capture_output=True, text=True)
                for _ in range(2)]
        first, second = (r.stdout + r.stderr for r in runs)
        if found := new_failures(case, first, known):
            failing[case] = found
        bad += flake(case, first, second)
    if base and failing:
        rev = subprocess.run(["git", "rev-parse", base], cwd=ROOT, capture_output=True, text=True).stdout.strip() or base
        at = base_verdicts(sorted(failing), rev, run)
        old = [c for c in failing if at[c] == "fail"]
        common = Path(subprocess.run(["git", "rev-parse", "--path-format=absolute", "--git-common-dir"], cwd=ROOT,
                                     capture_output=True, text=True).stdout.strip() or ROOT / ".git")
        with open(common / "cpl-preexisting-reds.log", "a") as log:
            for c in old:
                who = owner(c, rev)
                print(f"conform_staged: pre-existing red, owner {who}: {PREFIX}{c} fails at the base {rev[:9]} too; "
                      "main's culprit job routes it, not refusing", file=sys.stderr)
                log.write(f"{rev} {who} conform:{c}\n")
                del failing[c]
        for c in failing:
            print(f"conform_staged: NEW red: {PREFIX}{c} is {at[c]} at the base {rev[:9]} and fails with this change",
                  file=sys.stderr)
    bad = [line for lines in failing.values() for line in lines] + bad
    if bad:
        print("\n".join(bad))
        print(f"conform_staged: {sum(bool(FAIL.match(l)) for l in bad)} staged case(s) fail that {BOARD} records as passing or new; fix them")
        return 1
    print(f"conform_staged: {len(paths)} staged case(s) pass or were already failing")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
