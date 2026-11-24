#!/usr/bin/env python3
"""Whether each std fn costs what its `cost:` line says: time per call at n
and at 4n, against the growth the documented complexity predicts.

    python3 bench/scaling.py [--cpl PATH] [--write] [--check] [CASE...]

A case is bench/scaling/<name>.cp. It reads `n`, builds what the calls
need, and marks each line whose time is a fn's with `// timed: <fn>`, the
fn named as `cpl doc --format json std` does (`bag.SortedBag.add`); one
case may time several fns. A header may name the fn a bare `// timed`
means (`// scaling: bag.SortedBag.add`), hold symbols at a constant
(`// fixed: K`), give one as a formula in n (`// let: k = \\log n; d = \\sqrt n`,
for a fn whose cost names a size the case derives from n) or set the base size (`// n: 4096`, default N). Each case runs under `cpl profile --native`, the
judge's C++ with a counter per statement, RUNS times at n and at 4n; the
time per hit of the timed lines (the fastest run) gives the measured growth.
The documented growth is the `cost:` formula evaluated at 4n over n, every
symbol but the fixed ones standing for n; a cost written ``as `fn` ``
reads that fn's. A callback the fn calls is written on its own line, out
of the timed one, and marked `// within: <fn>`: a profile charges a
callback's time to its own line, which counts to the fn's time but not
to its hits. A case fails when the measured
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
    for name, cost in costs.items():
        costs[name] = alias(name, cost, costs)
    return costs


def alias(name, cost, costs):
    """A cost written ``as `other` `` (a sibling) or ``as [`Owner.fn`](...)`` (in the same
    module) reads the other fn's cost."""
    m = re.match(r"as \[?`([\w.]+)`", cost)
    if not m or big_o(cost):
        return cost
    module, owner = name.split(".")[0], name.rsplit(".", 1)[0]
    for other in (f"{owner}.{m.group(1)}", f"{module}.{m.group(1)}"):
        if other in costs and other != name:
            return alias(other, costs[other], costs)
    return cost


def big_o(cost):
    """The formula inside the first `\\(O(...)\\)` of a `cost:` line, or None."""
    m = re.search(r"\\\(O\((.*?)\)\\\)", cost)
    return m.group(1) if m else None


BUILTINS = ("lg", "sqrt", "min", "max")


def _arg(f, i):
    """The end of the argument of a function written without parentheses
    (`\\log n`, `\\log(n + m)`, `\\log \\max(a, b)`) starting at `f[i]`."""
    m = re.match(r"[A-Za-z_]\w*|\d+", f[i:])
    j = i + (m.end() if m else 0)
    if j < len(f) and f[j] == "(" and (not m or m.group(0)[0].isalpha()):
        depth = 0
        for j in range(j, len(f)):
            depth += {"(": 1, ")": -1}.get(f[j], 0)
            if depth == 0:
                return j + 1
    if j == i:
        raise ValueError(f"no argument at {f[i:]!r}")
    return j


def python(formula):
    """A cost formula in the docs' TeX as a Python expression over its
    symbols, each spelt `v_<symbol>`."""
    f = formula
    f = re.sub(r"\\lvert\s*(.*?)\s*\\rvert", r"(\1)", f)
    f = f.replace("\\_", "_").replace("³", "^3").replace("²", "^2")
    f = re.sub(r"\\mathit\{(\w+)\}", r"\1", f)
    f = re.sub(r"(\w+)\.(\w+)", r"\1_\2", f)
    f = re.sub(r"\\deg\s*", "", f)
    f = re.sub(r"\\(?:operatorname|mathrm)\{(\w+)\}", r"\1", f)
    f = re.sub(r"\\sqrt\{([^}]*)\}", r"sqrt(\1)", f)
    f = re.sub(r"\\alpha\s*\(([^)]*)\)", "1", f)
    f = f.replace("\\min", "min").replace("\\max", "max")
    # `\log^k x`, `\log x`, `log_b x` and `\sqrt x`, innermost (rightmost) first
    op = re.compile(r"\\log\^(\d+)\s*|\\log\s*|\blog_(\w+)\s*|\\sqrt\s*")
    while ms := list(op.finditer(f)):
        m = ms[-1]
        end = _arg(f, m.end())
        a = f[m.end():end]
        if m.group(0).startswith("\\sqrt"):
            r = f"sqrt({a})"
        elif m.group(1):
            r = f"(lg({a})**{m.group(1)})"
        elif m.group(2):
            r = f"(lg({a})/lg({m.group(2)}))"
        else:
            r = f"lg({a})"
        f = f[:m.start()] + r + f[end:]
    f = f.replace("\\cdot", "*").replace("\\times", "*")
    f = f.replace("^", "**").replace("{", "(").replace("}", ")").replace("\\,", "")
    # a symbol written as a function of others (`\mathit{nodes}(\mathit{self})`, `d(x)`) is one symbol
    while m := re.search(r"\b(?!(?:%s)\()([A-Za-z_]\w*)\(" % "|".join(BUILTINS), f):
        end = _arg(f, m.start())
        f = f[:m.start()] + m.group(1) + f[end:]
    f = re.sub(r"([\w)])\s+(?=[\w(])", r"\1*", f)
    if "\\" in f:
        raise ValueError(f"can't read the cost formula {formula!r}")
    return re.sub(r"\b(?!(?:%s)\()([A-Za-z_]\w*)" % "|".join(BUILTINS), r"v_\1", f)


def growth(formula, fixed=(), k=4.0, n=float(N), let=None):
    """f(k n) / f(n) for a cost formula in the docs' TeX. A symbol in `fixed`
    stands for 1, one in `let` for its formula in `n` (`k = \\log n` when the
    case's n is 2^k), and every other symbol for n."""
    let = {s: python(e) for s, e in (let or {}).items()}
    env = {"__builtins__": {}, "lg": lambda v: math.log2(max(v, 2.0)),
           "sqrt": math.sqrt, "min": min, "max": max}

    def at(x, f):
        names = {}
        for v in set(re.findall(r"\bv_\w+", f)):
            s = v[2:]
            names[v] = 1.0 if s in fixed else at(x, let[s]) if s in let and s != "n" else x
        try:
            return float(eval(f, env, names))
        except (ArithmeticError, TypeError, SyntaxError, NameError) as e:
            raise ValueError(f"can't evaluate the cost formula {formula!r}: {e}") from e

    f = python(formula)
    return at(k * n, f) / at(n, f)


def header(text):
    """The case's `// key: value` header lines."""
    out = {}
    for line in text.splitlines():
        m = re.match(r"//\s*(scaling|fixed|let|n):\s*(.*)", line)
        if not m:
            break
        out[m.group(1)] = m.group(2).strip()
    return out


def per_hit(cpl, case, n, groups, tmp, within=None):
    """fn -> fastest nanoseconds per hit of its timed lines over RUNS runs at size n, with the
    time (not the hits) of its `within` lines."""
    within = within or {}
    inp = os.path.join(tmp, f"{os.path.basename(case)}.{n}.in")
    with open(inp, "w") as f:
        f.write(f"{n}\n")
    best = {}
    for _ in range(RUNS):
        run = subprocess.run([cpl, "profile", "--native", case, inp, "--format", "json"],
                             cwd=ROOT, capture_output=True, text=True, timeout=300)
        if run.returncode != 0:
            raise RuntimeError(run.stderr.strip().splitlines()[0] if run.stderr.strip() else "profile failed")
        by_line = {l["line"]: l for l in json.loads(run.stdout)["lines"]}
        for fn, timed in groups.items():
            lines = [by_line[i] for i in timed if i in by_line]
            ns, hits = sum(l["time_ns"] for l in lines), sum(l["start_count"] for l in lines)
            ns += sum(by_line[i]["time_ns"] for i in within.get(fn, ()) if i in by_line)
            if not hits:
                raise RuntimeError(f"no line timing {fn} ran")
            if ns < MIN_NS:
                raise RuntimeError(f"the lines timing {fn} take {ns} ns at n={n}, under {MIN_NS}; raise `// n:`")
            best[fn] = min(best.get(fn, math.inf), ns / hits)
    return best


def timed_lines(text, default):
    """fn -> the line numbers marked `// timed` (the header's fn) or `// timed: fn`."""
    groups = {}
    for i, line in enumerate(text.splitlines(), 1):
        m = re.search(r"//\s*timed(?::\s*(\S+))?\s*$", line)
        if m:
            groups.setdefault(m.group(1) or default, set()).add(i)
    return groups


def within_lines(text):
    """fn -> the line numbers marked `// within: fn`: the body of a callback the fn calls, whose
    time is the fn's and whose hits are not."""
    groups = {}
    for i, line in enumerate(text.splitlines(), 1):
        m = re.search(r"//\s*within:\s*(\S+)\s*$", line)
        if m:
            groups.setdefault(m.group(1), set()).add(i)
    return groups


def judge(measured, documented):
    """Whether a measured growth agrees with the documented one."""
    return documented / SLACK ** 2 <= measured <= documented * SLACK


def run_case(cpl, case, costs, tmp):
    """(`case::fn`, record) for each fn the case times."""
    text = open(case).read()
    head = header(text)
    rel = os.path.relpath(case, ROOT)
    groups = timed_lines(text, head.get("scaling"))
    if not groups:
        return [(f"{rel}::", {"fn": None, "ok": False, "why": "no line is marked `// timed`"})]
    n = int(head.get("n", N))
    fixed = tuple(s.strip() for s in head.get("fixed", "").split(",") if s.strip())
    let = dict((p.strip() for p in d.split("=", 1)) for d in head.get("let", "").split(";") if d.strip())
    out, documented = [], {}
    for fn in groups:
        formula = big_o(costs[fn]) if fn in costs else None
        try:
            if fn not in costs:
                raise ValueError(f"`{fn}` is no std fn with a cost")
            if not formula:
                raise ValueError(f"no O(...) in its cost: {costs[fn]}")
            documented[fn] = (formula, growth(formula, fixed, n=float(n), let=let))
        except ValueError as e:
            out.append((f"{rel}::{fn}", {"fn": fn, "ok": False, "why": str(e)}))
    try:
        within = within_lines(text)
        small = per_hit(cpl, case, n, groups, tmp, within)
        big = per_hit(cpl, case, 4 * n, groups, tmp, within)
    except (RuntimeError, subprocess.TimeoutExpired) as e:
        return out + [(f"{rel}::{fn}", {"fn": fn, "ok": False, "why": str(e)}) for fn in documented]
    for fn, (formula, want) in documented.items():
        measured = big[fn] / small[fn]
        out.append((f"{rel}::{fn}", {"fn": fn, "cost": formula, "n": n, "documented": round(want, 2),
                                      "measured": round(measured, 2), "ok": judge(measured, want)}))
    return out


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
        results = dict(r for rs in pool.map(lambda c: run_case(args.cpl, os.path.abspath(c), costs, tmp), cases)
                       for r in rs)
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
                               or sorted(old.get("cases", {})) != sorted(results)
                               or any(not r["ok"] for r in old.get("cases", {}).values()) != bool(bad)):
            print(f"scaling: {os.path.relpath(BOARD, ROOT)} is stale; run bench/scaling.py --write",
                  file=sys.stderr)
            return 1
        return 1 if bad else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
