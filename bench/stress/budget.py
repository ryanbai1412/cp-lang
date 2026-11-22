#!/usr/bin/env python3
"""ARCHITECTURE.md A5 stress budget benchmark (not part of the fast gate).

Checks that `cpl stress` finds a shrunk counterexample for a planted-bug
false conjecture in under 10 seconds on 3 distinct problem shapes:
  1. sac22cc2p5 (array + queries, typed `read()`, `@alt(program)`)
  2. dmopc20c4p4 (array greedy, `@gen`, part-level `@alt(solve)`)
  3. aac5p4 (constructive output, typed `read()`, `@check`)

Usage:
    python3 bench/stress/budget.py [path/to/cpl]
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BUDGET_S = 10.0

PROBLEMS = [
    (
        "sac22cc2p5",
        "array + queries (@alt(program), typed read)",
        """\
let n: 1..=8 = read()
let q: 1..=8 = read()
let a: [1..=n]1..=20 = read()
var qs: List<(Int, Int, Int)> = []
for _ in 0..q:
    let t: 1..=2 = read()
    if t == 1:
        let idx: 1..=n = read()
        let val: 1..=20 = read()
        qs.push((1, idx, val))
    else:
        let l: 1..=n = read()
        let r: l..=n = read()
        qs.push((2, l, r))
var cur = [0 for _ in 0..=n]
for i in 1..=n: cur[i] = a[i]
for (t, x, y) in qs:
    if t == 1:
        cur[x] = y
    else:
        var s = 0
        for i in x..y: s += cur[i]
        print(s)

@alt(program)
fn slow():
    var cur = [0 for _ in 0..=n]
    for i in 1..=n: cur[i] = a[i]
    for (t, x, y) in qs:
        if t == 1:
            cur[x] = y
        else:
            var s = 0
            for i in x..=y: s += cur[i]
            print(s)
""",
    ),
    (
        "dmopc20c4p4",
        "array greedy (@gen, part @alt(solve))",
        """\
fn solve(n: Int, a: List<Int>) -> Int:
    var ans = 0
    for i in 0..n:
        if a[i] > i: ans += a[i] - i
    ans

@alt(solve)
fn slow_solve(n: Int, a: List<Int>) -> Int:
    var used = [false for _ in 0..n]
    var ops = 0
    for i in 0..n:
        var cnt = 0
        for j in 0..i:
            if used[j]: cnt += 1
        while cnt < a[i]:
            var best = -1
            for j in 0..i:
                if not used[j]: best = j
            used[best] = true
            ops += i - best
            cnt += 1
    ops

@gen
fn make_case(size: Int):
    let n = rand(1, size)
    print(n)
    var a: List<Int> = []
    for i in 0..n:
        a.push(rand(0, i))
    print(a)

let n: 1..=12 = read()
let a: [0..n]0..=12 = read()
print(solve(n, a))
""",
    ),
    (
        "aac5p4",
        "constructive (@check, typed read)",
        """\
let n: 1..=10 = read()
let x: 1..=n = read()
let total = n * (n + 1) / 2
if total % 2 == 1:
    print(-1)
else:
    // Planted bug: always prints 0 flips even when target != x.
    print(0)

@check
fn judge():
    let k: Int = read_output()
    let total = n * (n + 1) / 2
    if total % 2 == 1:
        assert k == -1
    else:
        assert k >= 0
        var pos = x
        for _ in 0..k:
            let d: 1..=n = read_output()
            pos += d
        assert pos == n
""",
    ),
]


def resolve_cpl():
    if len(sys.argv) > 1:
        return sys.argv[1]
    if os.environ.get("CPL"):
        return os.environ["CPL"]
    for profile in ("release", "debug"):
        cand = os.path.join(ROOT, "target", profile, "cpl")
        if os.path.isfile(cand):
            return cand
    subprocess.run(["cargo", "build", "-q", "-p", "cpl-cli"], cwd=ROOT, check=True)
    return os.path.join(ROOT, "target", "debug", "cpl")


def main():
    cpl = resolve_cpl()
    scratch = tempfile.mkdtemp(prefix="cpl-stress-budget-")
    failed = 0
    try:
        print(f"Stress A5 budget benchmark (limit: < {BUDGET_S:.1f} s per problem, cpl: {cpl})")
        for name, shape, src in PROBLEMS:
            path = os.path.join(scratch, f"{name}.cp")
            with open(path, "w") as f:
                f.write(src)
            t0 = time.monotonic()
            proc = subprocess.run(
                [cpl, "stress", path, "--cases", "100", "--seed", "1", "--size", "8", "--format", "json"],
                capture_output=True,
                text=True,
            )
            elapsed = time.monotonic() - t0
            try:
                data = json.loads(proc.stdout)
            except ValueError:
                print(f"FAIL  {name:<12} {elapsed:5.2f}s  invalid JSON (rc={proc.returncode}): {proc.stderr.strip()}")
                failed += 1
                continue
            status = data.get("status")
            ce = data.get("counterexample") or {}
            raw = (ce.get("raw_input") or "").replace("\n", "\\n")
            ok = status == "fail" and elapsed < BUDGET_S
            verdict = "PASS" if ok else "FAIL"
            if not ok:
                failed += 1
            print(
                f"{verdict:<5} {name:<12} {elapsed:5.2f}s (<{BUDGET_S:.0f}s)  "
                f"case={ce.get('case')} seed={ce.get('seed')} shrunk={raw!r}  [{shape}]"
            )
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
