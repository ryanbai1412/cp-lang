#!/usr/bin/env python3
"""Whether each std fn costs what its `cost:` line says: time per call at n
and at 4n, against the growth the documented complexity predicts.

    python3 bench/scaling.py [--cpl PATH] [--write] [--check] [CASE...]

A case is bench/scaling/<name>.cp. It reads `n`, builds what the call needs,
and marks each line whose time is the fn's with `// timed`; a header names
the fn as `cpl doc --format json std` does (`// scaling: bag.SortedBag.add`),
and may hold symbols at a constant (`// fixed: K`) or set the base size
(`// n: 4096`, default N). Each case runs under `cpl profile --native`, the
judge's C++ with a counter per statement, RUNS times at n and at 4n; the
time per hit of the timed lines (the fastest run) gives the measured growth.
The documented growth is the `cost:` formula evaluated at 4n over n, every
symbol but the fixed ones standing for n. A case fails when the measured
growth is over SLACK times the documented one (the fn is slower than its
docs) or under it by SLACK squared (the docs overstate it).

--write records every case and the std fns with a `cost:` and no case in
research/perf/scaling.json, which scripts/debt-ratchet.py reads
(`scaling_mismatches` is a hard zero; `std_fns_unscaled` only shrinks).
--check fails on a mismatch and when that file is stale. Enforces the
stdlib register's "The obvious code is the fast code" and "The documented
cost is the real cost" (rules std-api/R16, std-docs/R3).
"""
import argparse, glob, json, math, os, re, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sizes import ROOT, stale  # noqa: E402

CASES = os.path.join(ROOT, "bench", "scaling")
BOARD = os.path.join(ROOT, "research", "perf", "scaling.json")
N, RUNS, SLACK = 1 << 14, 3, 2.0
MIN_NS = 2_000_000


def std_costs(cpl):
    """`module.Owner.fn` -> its `cost:` text, for every std item that has one."""
    out = subprocess.run([cpl, "doc", "--format", "json", "std"], cwd=ROOT,
                         capture_output=True, text=True, check=True).stdout
    costs = {}

    def walk(items, prefix):
        for it in items:
            name = f"{prefix}.{it['name']}"
            cost = (it.get("facts") or {}).get("cost")
            if cost:
                costs[name] = cost
            walk(it.get("members") or [], name)

    for m in json.loads(out)["modules"]:
        walk(m["items"], m["name"])
    return costs


def big_o(cost):
    """The formula inside the first `\\(O(...)\\)` of a `cost:` line, or None."""
    m = re.search(r"\\\(O\((.*?)\)\\\)", cost)
    return m.group(1) if m else None


def growth(formula, fixed=(), k=4.0, n=float(N)):
    """f(k n) / f(n) for a cost formula in the docs' TeX, every symbol not in
    `fixed` standing for n."""
    f = formula
    f = re.sub(r"\\lvert\s*(.*?)\s*\\rvert", r"(\1)", f)
    f = f.replace("\\_", "_").replace("³", "^3").replace("²", "^2")
    f = re.sub(r"\\mathit\{(\w+)\}", r"\1", f)
    f = re.sub(r"(\w+)\.(\w+)", r"\1_\2", f)
    f = re.sub(r"\\deg\s*", "", f)
    f = re.sub(r"\\sqrt\s*(\w+)", r"sqrt(\1)", f)
    f = re.sub(r"\\(?:operatorname|mathrm)\{(\w+)\}", r"\1", f)
    f = re.sub(r"\\sqrt\{([^}]*)\}", r"sqrt(\1)", f)
    f = re.sub(r"\\log\^(\d+)\s*(\w+|\([^)]*\))", r"(lg(\2)**\1)", f)
    f = re.sub(r"\\log\s*(\w+|\([^)]*\))", r"lg(\1)", f)
    f = re.sub(r"\\alpha\s*\(([^)]*)\)", "1", f)
    f = f.replace("\\cdot", "*").replace("\\times", "*").replace("\\min", "min").replace("\\max", "max")
    f = f.replace("^", "**").replace("{", "(").replace("}", ")").replace("\\,", "")
    f = re.sub(r"([\w)])\s+(?=[\w(])", r"\1*", f)
    if "\\" in f:
        raise ValueError(f"can't read the cost formula {formula!r}")

    def at(x):
        names = {s: (1.0 if s in fixed else x) for s in re.findall(r"[A-Za-z_]\w*", f)
                 if s not in ("lg", "sqrt", "min", "max")}
        return eval(f, {"__builtins__": {}, "lg": lambda v: math.log2(max(v, 2.0)),
                        "sqrt": math.sqrt, "min": min, "max": max}, names)

    return at(k * n) / at(n)


def header(text):
    """The case's `// key: value` header lines."""
    out = {}
    for line in text.splitlines():
        m = re.match(r"//\s*(scaling|fixed|n):\s*(.*)", line)
        if not m:
            break
        out[m.group(1)] = m.group(2).strip()
    return out


def per_hit(cpl, case, n, timed, tmp):
    """Fastest nanoseconds per hit of the `timed` lines over RUNS runs at size n."""
    inp = os.path.join(tmp, f"{os.path.basename(case)}.{n}.in")
    with open(inp, "w") as f:
        f.write(f"{n}\n")
    best = None
    for _ in range(RUNS):
        run = subprocess.run([cpl, "profile", "--native", case, inp, "--format", "json"],
                             cwd=ROOT, capture_output=True, text=True, timeout=300)
        if run.returncode != 0:
            raise RuntimeError(run.stderr.strip().splitlines()[0] if run.stderr.strip() else "profile failed")
        lines = [l for l in json.loads(run.stdout)["lines"] if l["line"] in timed]
        ns, hits = sum(l["time_ns"] for l in lines), sum(l["hit_count"] for l in lines)
        if not hits:
            raise RuntimeError("no timed line ran")
        if ns < MIN_NS:
            raise RuntimeError(f"the timed lines take {ns} ns at n={n}, under {MIN_NS}; raise `// n:`")
        best = min(best or math.inf, ns / hits)
    return best


def judge(measured, documented):
    """Whether a measured growth agrees with the documented one."""
    return documented / SLACK ** 2 <= measured <= documented * SLACK


def run_case(cpl, case, costs, tmp):
    text = open(case).read()
    head = header(text)
    rel = os.path.relpath(case, ROOT)
    fn = head.get("scaling")
    rec = {"fn": fn}
    if fn not in costs:
        return rel, {**rec, "ok": False, "why": f"`// scaling: {fn}` names no std fn with a cost"}
    timed = {i for i, l in enumerate(text.splitlines(), 1) if l.rstrip().endswith("// timed")}
    if not timed:
        return rel, {**rec, "ok": False, "why": "no line is marked `// timed`"}
    n = int(head.get("n", N))
    fixed = tuple(s.strip() for s in head.get("fixed", "").split(",") if s.strip())
    formula = big_o(costs[fn])
    try:
        documented = growth(formula, fixed, n=float(n)) if formula else None
        if documented is None:
            raise ValueError(f"no O(...) in its cost: {costs[fn]}")
        measured = per_hit(cpl, case, 4 * n, timed, tmp) / per_hit(cpl, case, n, timed, tmp)
    except (RuntimeError, ValueError, subprocess.TimeoutExpired) as e:
        return rel, {**rec, "ok": False, "why": str(e)}
    ok = judge(measured, documented)
    return rel, {**rec, "cost": formula, "n": n, "documented": round(documented, 2),
                 "measured": round(measured, 2), "ok": ok}


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--cpl", default=os.path.join(ROOT, "target", "debug", "cpl"))
    ap.add_argument("--write", action="store_true")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("-j", type=int, default=max(1, (os.cpu_count() or 2) // 2))
    ap.add_argument("cases", nargs="*")
    args = ap.parse_args()
    newer = stale(args.cpl)
    if newer:
        print(f"scaling: {args.cpl} is older than {newer}; rebuild it or pass --cpl", file=sys.stderr)
        return 1
    costs = std_costs(args.cpl)
    cases = args.cases or sorted(glob.glob(os.path.join(CASES, "*.cp")))
    import tempfile
    with tempfile.TemporaryDirectory() as tmp, ThreadPoolExecutor(args.j) as pool:
        results = dict(pool.map(lambda c: run_case(args.cpl, os.path.abspath(c), costs, tmp), cases))
    scaled = {r["fn"] for r in results.values()}
    board = {"cases": results, "unscaled": sorted(set(costs) - scaled)}
    bad = 0
    for rel, r in sorted(results.items()):
        if r["ok"]:
            print(f"scaling: ok {rel}: {r['fn']} grows {r['measured']}x at 4n, documented "
                  f"O({r['cost']}) {r['documented']}x")
        else:
            bad += 1
            why = r.get("why") or (f"grows {r['measured']}x at 4n but its cost O({r['cost']}) "
                                   f"says {r['documented']}x")
            print(f"scaling: {rel}: {r['fn']} {why}", file=sys.stderr)
    print(f"scaling: {len(results) - bad} of {len(results)} cases agree with the docs; "
          f"{len(board['unscaled'])} std fns with a cost have no case")
    if args.write and not args.cases:
        with open(BOARD, "w") as f:
            json.dump(board, f, indent=1, sort_keys=True)
            f.write("\n")
    if args.check:
        try:
            old = json.load(open(BOARD))
        except (OSError, ValueError):
            old = {}
        if not args.cases and (old.get("unscaled") != board["unscaled"]
                               or sorted(old.get("cases", {})) != sorted(results)):
            print(f"scaling: {os.path.relpath(BOARD, ROOT)} is stale; run bench/scaling.py --write",
                  file=sys.stderr)
            return 1
        return 1 if bad else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
