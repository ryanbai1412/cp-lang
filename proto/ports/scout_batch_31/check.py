#!/usr/bin/env python3
"""Differential test harness for scout_batch_31 (grids, BFS, flood fill, simulation).

Compiles each original AC C++ submission from corpus/submissions/ and each
runnable port in today/ (through `proto/cp.py build`, on a temporary copy so
the tracked source is never stamped) and compares their output on small and
medium random inputs. coci25c3p1 iterates a Str, which only the interpreter
supports today, so it runs through `proto/cp.py lib-run` instead.

    python3 proto/ports/scout_batch_31/check.py              # all 10 ports, 150 cases each
    python3 proto/ports/scout_batch_31/check.py cco19p2 500  # one port, 500 cases
"""
import os, random, shutil, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def size(rng, idx, small, medium):
    """Small cases first, medium cases for the last third of a run."""
    return rng.randint(*(small if idx % 3 else medium))


def grid(rng, n, m, alphabet, weights=None):
    return ["".join(rng.choices(alphabet, weights, k=m)) for _ in range(n)]


def gen_ucrpc21f(rng, idx):
    n = size(rng, idx, (1, 6), (20, 60))
    t = rng.choice([0, 1, rng.randint(0, 10), rng.randint(0, 10**9)])
    p = rng.choice([0.1, 0.5, 0.9])
    return f"{n} {t}\n" + "\n".join(grid(rng, n, n, "ab", [1 - p, p])) + "\n"


def gen_ucrpc21d(rng, idx):
    while True:
        n, m = size(rng, idx, (2, 5), (10, 30)), size(rng, idx, (2, 5), (10, 30))
        p = rng.choice([0.4, 0.6, 0.8])
        rows = []
        for i in range(2 * n + 1):
            row = []
            for j in range(2 * m + 1):
                border = i in (0, 2 * n) or j in (0, 2 * m)
                if i % 2 == 0 and j % 2 == 0:
                    row.append("+")
                elif i % 2 == 0:
                    row.append("-" if border or rng.random() < p else ".")
                elif j % 2 == 0:
                    row.append("|" if border or rng.random() < p else ".")
                else:
                    row.append(".")
            rows.append("".join(row))
        # The original prints four sizes unconditionally: keep at least four regions.
        seen, regions = set(), 0
        for i in range(1, 2 * n, 2):
            for j in range(1, 2 * m, 2):
                if (i, j) in seen:
                    continue
                regions += 1
                seen.add((i, j))
                stack = [(i, j)]
                while stack:
                    x, y = stack.pop()
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        if rows[x + dx][y + dy] == "." and (x + 2 * dx, y + 2 * dy) not in seen:
                            seen.add((x + 2 * dx, y + 2 * dy))
                            stack.append((x + 2 * dx, y + 2 * dy))
        if regions >= 4:
            return f"{n} {m}\n" + "\n".join(rows) + "\n"


def gen_dmopc19c4p2(rng, idx):
    n = size(rng, idx, (2, 5), (6, 16))
    cells = [(r, c) for r in range(1, n + 1) for c in range(1, n + 1)]
    e1, e2 = rng.sample(cells, 2)
    s = rng.choice([e1, e2])
    t = rng.choice(cells)
    return "\n".join(f"{a} {b}" for a, b in [(n, ""), s, t, e1, e2]).replace(" \n", "\n") + "\n"


def gen_sac22cc1p3(rng, idx):
    n, m = size(rng, idx, (1, 6), (10, 50)), size(rng, idx, (1, 6), (10, 50))
    w = rng.choice([[6, 2, 2], [2, 4, 4], [1, 1, 6], [3, 5, 1]])
    q = size(rng, idx, (1, 8), (10, 50))
    events = [rng.choice("112") for _ in range(q - 1)] + ["2"]
    return f"{n} {m}\n" + "\n".join(grid(rng, n, m, ".XW", w)) + f"\n{q}\n" + "\n".join(events) + "\n"


def gen_coci25c3p1(rng, idx):
    n = size(rng, idx, (1, 7), (10, 25))
    k = rng.randint(1, max(1, rng.choice([2, n, 2 * n])))
    length = size(rng, idx, (1, 12), (20, 80))
    s = "".join(rng.choice("LRUD" * 3 + "abcxyz") for _ in range(length))
    return f"{n} {k}\n{s}\n"


def gen_coci20c6p1(rng, idx):
    n, m = size(rng, idx, (2, 6), (10, 60)), size(rng, idx, (2, 6), (10, 60))
    p = rng.choice([0.05, 0.3, 0.7])
    g = [list(r) for r in grid(rng, n, m, ".#", [1 - p, p])]
    # The original writes the whole 2 x 2 stamp unchecked: keep stamps on the picture.
    for r in g:
        r[m - 1] = "."
    g[n - 1] = ["."] * m
    return f"{n} {m}\n" + "\n".join("".join(r) for r in g) + "\n"


def gen_imo22p6checker(rng, idx):
    n = size(rng, idx, (1, 5), (8, 40))
    vals = list(range(1, n * n + 1))
    kind = rng.randrange(3)
    if kind == 0:
        rng.shuffle(vals)
    elif kind == 1:                      # nearly sorted: long uphill paths
        for _ in range(n):
            i, j = rng.randrange(n * n), rng.randrange(n * n)
            vals[i], vals[j] = vals[j], vals[i]
    rows = [vals[i * n:(i + 1) * n] for i in range(n)]
    if kind == 2:                        # snake: one path through every cell
        rows = [r if i % 2 == 0 else r[::-1] for i, r in enumerate(rows)]
    return f"{n}\n" + "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_dmopc21c6p4(rng, idx):
    n = size(rng, idx, (4, 8), (10, 60))
    pairs = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    m = rng.randint(1, min(len(pairs), rng.choice([n, 2 * n, n * n])))
    edges = rng.sample(pairs, m)
    if rng.random() < 0.5:               # a path, so the long detours matter
        order = rng.sample(range(1, n + 1), n)
        edges = list({tuple(sorted(e)) for e in edges[:n // 2] + list(zip(order, order[1:]))})
    edges = [e if rng.random() < 0.5 else e[::-1] for e in edges]
    a, b, c, d = rng.sample(range(1, n + 1), 4)
    return f"{n} {len(edges)}\n" + "\n".join(f"{u} {v}" for u, v in edges) + f"\n{a} {b} {c} {d}\n"


def gen_cco19p2(rng, idx):
    n, m = size(rng, idx, (1, 7), (10, 60)), size(rng, idx, (1, 7), (10, 60))
    p = rng.choice([0.1, 0.3, 0.5, 0.7])
    return f"{n} {m}\n" + "\n".join(grid(rng, n, m, ".#", [1 - p, p])) + "\n"


def gen_noi10p3(rng, idx):
    n = size(rng, idx, (1, 4), (5, 25))
    hi = rng.choice([1, 5, 10**6])
    count = 4 * n * (n + 1)
    return f"{n}\n" + "\n".join(str(rng.randint(0, hi)) for _ in range(count)) + "\n"


PORTS = {
    "ucrpc21f": (3944579, gen_ucrpc21f),
    "ucrpc21d": (3944584, gen_ucrpc21d),
    "dmopc19c4p2": (5323330, gen_dmopc19c4p2),
    "sac22cc1p3": (3992599, gen_sac22cc1p3),
    "coci25c3p1": (7730233, gen_coci25c3p1),
    "coci20c6p1": (4188008, gen_coci20c6p1),
    "imo22p6checker": (4955728, gen_imo22p6checker),
    "dmopc21c6p4": (4342347, gen_dmopc21c6p4),
    "cco19p2": (3613676, gen_cco19p2),
    "noi10p3": (5672180, gen_noi10p3),
}


def sh(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


INTERPRETED = {"coci25c3p1"}


def build(name, sub, tmp):
    ref = os.path.join(tmp, name + ".ref")
    r = sh(["g++", "-std=c++17", "-O2", "-w", "-o", ref, os.path.join(ROOT, "corpus", "submissions", f"{sub}.cpp")])
    assert r.returncode == 0, r.stderr
    if name in INTERPRETED:
        return [ref], [sys.executable, CP, "lib-run", os.path.join(HERE, "today", name + ".cp")]
    src = os.path.join(tmp, name + ".cp")
    shutil.copy(os.path.join(HERE, "today", name + ".cp"), src)
    r = sh([sys.executable, CP, "build", src], cwd=os.path.join(ROOT, "proto"))
    assert os.path.exists(src + "p"), r.stdout + r.stderr
    port = os.path.join(tmp, name + ".port")
    r = sh(["g++", "-std=c++17", "-O2", "-w", "-o", port, src + "p"])
    assert r.returncode == 0, r.stderr
    return [ref], [port]


def main():
    names = [sys.argv[1]] if len(sys.argv) > 1 else list(PORTS)
    cases = int(sys.argv[2]) if len(sys.argv) > 2 else 150
    bad = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            sub, gen = PORTS[name]
            ref, port = build(name, sub, tmp)
            rng = random.Random(31)
            failed = None
            for idx in range(cases):
                inp = gen(rng, idx)
                want = sh(ref, input=inp)
                got = sh(port, input=inp)
                if want.returncode != 0 or got.returncode != 0 or want.stdout.split() != got.stdout.split():
                    failed = (idx, inp, want, got)
                    break
            if failed:
                bad += 1
                idx, inp, want, got = failed
                print(f"FAIL {name} case {idx} (exit {want.returncode} vs {got.returncode})")
                print(inp[:600], "--- want", want.stdout[:300], "--- got", got.stdout[:300], got.stderr[:300], sep="\n")
            else:
                print(f"ok   {name}: {cases} cases match corpus/submissions/{sub}.cpp")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
