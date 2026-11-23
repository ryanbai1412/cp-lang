#!/usr/bin/env python3
"""Enforces conformance/R6 (handbook/guidelines/conformance-checklist.md#rules). Generate random CP Lang programs with known output; run both engines on them.

Each program is a list of statements over a few `var` lists and `Int`s:
view chains (`rev`, `drop`, `take`, `drop_last`, `take_last`), folds
(`sum`, `count`, `max`/`min` with `!`), Euclidean `/` and `%`, bit
operators, in-place `sort`/`reverse`/`push`, loops over views and closures
that write a captured `var`. A Python model computes the expected output, so
a wrong answer both engines share is caught as well as a disagreement.

A failing program is shrunk by dropping statements and written to `--out`.

    python3 scripts/gen-programs.py [-n 200] [--seed 1] [--jobs 8] [--out DIR]
        [--cxxflags '-O1 -g -fsanitize=address,undefined -D_GLIBCXX_DEBUG']
"""
import argparse
import os
import random
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CPL = ROOT / "target/debug/cpl"
LISTS = ["xs", "ys"]
INTS = ["a", "b"]


class Invalid(Exception):
    pass


def ediv(x: int, y: int) -> tuple[int, int]:
    r = x % abs(y)
    return (x - r) // y, r


AVOID: set[str] = set()


def view(rng, depth=0):
    """A view chain as (code suffix, python function on a list)."""
    ops = []
    for _ in range(rng.randint(0, 3)):
        op = rng.choice(["rev", "drop", "take", "drop_last", "take_last"])
        if op == "rev":
            if ops and ops[-1][0] == "rev":
                continue
            ops.append(("rev", None))
        else:
            ops.append((op, rng.randint(0, 5)))
    # `.rev` is only on a Bidir: a list, or a take/drop piece of one, never a Rev.
    seen_rev = False
    clean = []
    for op, k in ops:
        if op == "rev":
            if seen_rev:
                continue
            seen_rev = True
        elif seen_rev and "rev-pieces" in AVOID:
            continue
        clean.append((op, k))
    code = "".join(f".{op}" if k is None else f".{op}({k})" for op, k in clean)

    def apply(xs):
        for op, k in clean:
            if op == "rev":
                xs = xs[::-1]
            elif op == "drop":
                xs = xs[k:]
            elif op == "take":
                xs = xs[:k]
            elif op == "drop_last":
                xs = xs[: max(len(xs) - k, 0)]
            else:
                xs = xs[max(len(xs) - k, 0):]
        return xs

    return code, apply


def int_expr(rng, names, depth=0):
    """(code, python function of env) for a small Int expression."""
    if depth > 2 or rng.random() < 0.3:
        if names and rng.random() < 0.6:
            n = rng.choice(names)
            return n, lambda env, n=n: env[n]
        c = rng.randint(-20, 20)
        return (f"({c})" if c < 0 else str(c)), lambda env, c=c: c
    lc, lf = int_expr(rng, names, depth + 1)
    rc, rf = int_expr(rng, names, depth + 1)
    op = rng.choice(["+", "-", "*", "/", "%", "^", "&", "|", "min", "max"])
    if op in ("min", "max"):
        f = min if op == "min" else max
        return f"{op}({lc}, {rc})", lambda env: f(lf(env), rf(env))
    if op in ("/", "%"):
        def g(env, op=op):
            y = rf(env)
            if y == 0:
                raise Invalid
            q, r = ediv(lf(env), y)
            return q if op == "/" else r
        return f"({lc} {op} {rc})", g
    py = {"+": lambda x, y: x + y, "-": lambda x, y: x - y, "*": lambda x, y: x * y,
          "^": lambda x, y: x ^ y, "&": lambda x, y: x & y, "|": lambda x, y: x | y}[op]
    return f"({lc} {op} {rc})", lambda env: py(lf(env), rf(env))


def fmt(v):
    return " ".join(map(str, v)) if isinstance(v, list) else str(v)


def statement(rng, i):
    """(code lines, apply(env) -> printed lines or raise Invalid)."""
    kind = rng.choice(["print_view", "fold", "fold", "assign", "loop", "closure",
                       "mutate", "mutate", "minmax"])
    L = rng.choice(LISTS)
    vc, vf = view(rng)
    if kind == "print_view":
        return [f"print({L}{vc})"], lambda env: [fmt(vf(env[L]))]
    if kind == "fold":
        f = rng.choice(["sum", "len", "count"])
        if f == "sum":
            return [f"print(sum({L}{vc}))"], lambda env: [str(sum(vf(env[L])))]
        if f == "len":
            return [f"print({L}{vc}.len)"], lambda env: [str(len(vf(env[L])))]
        t = rng.randint(-5, 5)
        return ([f"print(count(x > {t} for x in {L}{vc}))"],
                lambda env: [str(sum(1 for x in vf(env[L]) if x > t))])
    if kind == "minmax":
        f = rng.choice(["max", "min"])
        ec, ef = int_expr(rng, ["x"])
        def ap(env):
            xs = vf(env[L])
            if not xs:
                raise Invalid
            vals = [ef({"x": x}) for x in xs]
            return [str(max(vals) if f == "max" else min(vals))]
        return [f"print({f}({ec} for x in {L}{vc})!)"], ap
    if kind == "assign":
        n = rng.choice(INTS)
        ec, ef = int_expr(rng, INTS)
        def ap(env):
            env[n] = ef(env)
            return [str(env[n])]
        return [f"{n} = {ec}", f"print({n})"], ap
    if kind == "loop":
        n = rng.choice(INTS)
        ec, ef = int_expr(rng, ["x", n])
        def ap(env):
            for x in vf(env[L]):
                env[n] = env[n] + ef({"x": x, n: env[n]})
                if abs(env[n]) > 10**12:
                    raise Invalid
            return [str(env[n])]
        return [f"for x in {L}{vc}:", f"    {n} += {ec}", f"print({n})"], ap
    if kind == "closure":
        n = rng.choice(INTS)
        c = rng.randint(-3, 3)
        def ap(env):
            for x in vf(env[L]):
                env[n] += x * c
            return [str(env[n])]
        return ([f"let f{i} = (x: Int) => {n} += x * ({c})", f"for x in {L}{vc}:",
                 f"    f{i}(x)", f"print({n})"], ap)
    op = rng.choice(["push", "sort", "reverse", "pop"])
    if op == "push":
        c = rng.randint(-9, 9)
        def ap(env):
            if len(env[L]) > 40:
                raise Invalid
            env[L].append(c)
            return []
        return [f"{L}.push({'(%d)' % c if c < 0 else c})"], ap
    if op == "pop":
        def ap(env):
            if not env[L]:
                raise Invalid
            return [str(env[L].pop())]
        return [f"print({L}.pop()!)"], ap
    def ap(env):
        if op == "sort":
            env[L].sort()
        else:
            env[L].reverse()
        return []
    return [f"{L}.{op}()"], ap


def program(rng, n):
    init = {L: [rng.randint(-9, 9) for _ in range(rng.randint(0, 7))] for L in LISTS}
    return init, [statement(rng, i) for i in range(n)]


def render(init, stmts):
    lines = [f"var {L}: List<Int> = [{', '.join(map(str, v))}]" for L, v in init.items()]
    lines += [f"var {n} = 0" for n in INTS]
    for code, _ in stmts:
        lines += code
    out = []
    env = {L: list(v) for L, v in init.items()} | {n: 0 for n in INTS}
    for _, ap in stmts:
        out += ap(env)
    return "\n".join(lines) + "\n", "".join(o + "\n" for o in out)


def run(src: str, leg: str, cxxflags: str | None) -> tuple[int, str, str]:
    """Run `src` on one leg: `interp`, `cxx` (`cpl run --release`) or `san`
    (the built C++ compiled with `cxxflags`)."""
    with tempfile.TemporaryDirectory() as d:
        p = Path(d) / "g.cp"
        p.write_text(src)
        if leg != "san":
            cmd = [str(CPL), "run"] + (["--release"] if leg == "cxx" else []) + [str(p)]
            r = subprocess.run(cmd, stdin=subprocess.DEVNULL, capture_output=True, text=True,
                               timeout=120)
            return r.returncode, r.stdout, r.stderr
        cpp, exe = Path(d) / "g.cpp", Path(d) / "g"
        r = subprocess.run([str(CPL), "build", str(p), "-o", str(cpp)], capture_output=True, text=True)
        if r.returncode:
            return r.returncode, "", r.stderr
        r = subprocess.run(["g++", "-std=c++17", *cxxflags.split(), "-o", str(exe), str(cpp)],
                           capture_output=True, text=True)
        if r.returncode:
            return r.returncode, "", "g++: " + r.stderr[:500]
        r = subprocess.run([str(exe)], stdin=subprocess.DEVNULL, capture_output=True, text=True,
                           timeout=120)
        return r.returncode, r.stdout, r.stderr


def verdict(src, want, cxxflags):
    bad = []
    for leg in ["interp", "cxx"] + (["san"] if cxxflags else []):
        code, out, err = run(src, leg, cxxflags)
        if code != 0 or out != want:
            bad.append((leg, code, out, err))
    return bad


def shrink(init, stmts, cxxflags):
    i = 0
    while i < len(stmts):
        trial = stmts[:i] + stmts[i + 1:]
        try:
            src, want = render(init, trial)
        except Invalid:
            i += 1
            continue
        if verdict(src, want, cxxflags) and "error[E" not in run(src, "interp", None)[2]:
            stmts = trial
        else:
            i += 1
    return stmts


def one(seed, size, cxxflags, out_dir):
    rng = random.Random(seed)
    for _ in range(20):
        init, stmts = program(rng, size)
        try:
            src, want = render(init, stmts)
            break
        except Invalid:
            continue
    else:
        return seed, "skipped", None
    bad = verdict(src, want, cxxflags)
    if not bad:
        return seed, "ok", None
    if any("error[E" in b[3] for b in bad):
        (out_dir / f"reject-{seed}.cp").write_text(src)
        return seed, "rejected", bad[0][3].splitlines()[0] if bad[0][3] else ""
    stmts = shrink(init, stmts, cxxflags)
    src, want = render(init, stmts)
    bad = verdict(src, want, cxxflags)
    (out_dir / f"wrong-{seed}.cp").write_text(src)
    (out_dir / f"wrong-{seed}.want").write_text(want)
    which = ",".join(b[0] for b in bad)
    return seed, "wrong", f"{which}: wrote wrong-{seed}.cp"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("-n", type=int, default=200)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--size", type=int, default=12)
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--out", default="gen-out")
    ap.add_argument("--cxxflags")
    ap.add_argument("--avoid", action="append", default=[],
                    help="leave out a construct with an open bug: rev-pieces (a piece of a .rev)")
    a = ap.parse_args()
    AVOID.update(a.avoid)
    out_dir = Path(a.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    tally = {}
    with ThreadPoolExecutor(a.jobs) as ex:
        for seed, kind, note in ex.map(lambda s: one(s, a.size, a.cxxflags, out_dir),
                                       range(a.seed, a.seed + a.n)):
            tally[kind] = tally.get(kind, 0) + 1
            if note:
                print(f"seed {seed}: {kind}: {note}")
    print(" ".join(f"{k} {v}" for k, v in sorted(tally.items())))
    return 1 if tally.get("wrong") or tally.get("rejected") else 0


if __name__ == "__main__":
    sys.exit(main())
