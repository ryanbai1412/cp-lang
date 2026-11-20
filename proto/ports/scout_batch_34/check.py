#!/usr/bin/env python3
"""Differential test harness for Scout Batch 34 (hard DMOPC classics).

Compiles the 10 original AC C++ submissions from corpus/submissions/ into a
temporary directory and checks every runnable CP Lang port in today/ via
`proto/cp.py lib-run` on randomized and edge-case inputs.

    python3 proto/ports/scout_batch_34/check.py               # all 10 ports, 30 cases each
    python3 proto/ports/scout_batch_34/check.py dmopc21c2p6 60 # single port, 60 cases
"""
import os, random, resource, subprocess, sys, tempfile
from collections import deque

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_dmopc21c2p6(rng, idx):
    if idx == 0:
        return lines((5, 6), [3, 1, 5, 2, 4], (2, 1), (3, 5), (1,), (2, 2), (3, 1), (1,))
    if idx == 1:
        return lines((1, 3), [1], (2, 1), (1,), (3, 1))
    n = rng.randint(2, 20)
    q = rng.randint(4, 28)
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    rows = [(n, q), perm]
    has_query = False
    for j in range(q):
        t = rng.choice([1, 2, 2, 3, 3])
        if j == q - 1 and not has_query:
            t = 2
        if t == 1:
            rows.append((1,))
        elif t == 2:
            has_query = True
            rows.append((2, rng.randint(1, n)))
        else:
            has_query = True
            rows.append((3, rng.randint(1, n)))
    return lines(*rows)


def gen_dmopc20c6p6(rng, idx):
    if idx == 0:
        return lines((4, 5), [2, 5, 3, 4])
    if idx == 1:
        return lines((2, 1), [1, 1])
    if idx == 2:
        return lines((6, 6), [6, 6, 6, 6, 6, 6])
    n = rng.randint(2, 20)
    m = rng.randint(1, 30)
    caps = [rng.randint(1, m) for _ in range(n)]
    return lines((n, m), caps)


def gen_dmopc21c2p5(rng, idx):
    if idx == 0:
        return lines((4, 2), (1, 2), (1, 3), (3, 4))
    if idx == 1:
        return lines((2, 1), (1, 2))
    n = rng.randint(2, 12)
    k = rng.randint(0, n)
    rows = [(n, k)]
    mode = idx % 3
    for u in range(2, n + 1):
        if mode == 0:
            p = rng.randint(1, u - 1)
        elif mode == 1:
            p = 1  # star
        else:
            p = u - 1  # path
        rows.append((p, u))
    return lines(*rows)


def gen_dmopc21c8p5(rng, idx):
    if idx == 0:
        return lines((2, 2), [5, 7])
    if idx == 1:
        return lines((25, 4), [3, 8, 12, 17])
    k = rng.randint(2, 7)
    # Cover both small n <= 2*k+2 and larger n that trigger recursive split DP.
    n = rng.choice([rng.randint(2, 2 * k + 5), rng.randint(2 * k + 6, 180)])
    a = [rng.randint(1, 40) for _ in range(k)]
    return lines((n, k), a)


def gen_dmopc21c7p4(rng, idx):
    if idx == 0:
        return lines((3,), (5, 2), (4, 1), (2, 3), (4,), (0, 6), (1, 1, 1), (0, 6), (0, 100))
    if idx == 1:
        return lines((1,), (3, 5), (3,), (0, 3), (0, 4), (1, 7, 2))
    n = rng.randint(1, 16)
    rows = [(n,)]
    for _ in range(n):
        rows.append((rng.randint(0, 25), rng.randint(0, 25)))
    q = rng.randint(3, 24)
    rows.append((q,))
    for _ in range(q):
        if rng.random() < 0.6:
            rows.append((0, rng.randint(1, 90)))
        else:
            rows.append((rng.randint(1, n), rng.randint(0, 25), rng.randint(0, 25)))
    return lines(*rows)


def gen_dmopc21c2p2(rng, idx):
    if idx == 0:
        return lines((4, 4), ("B", 1, 2), ("E", 2, 3), ("Q", 4, 3, 2, 1), ("Q", 1, 2, 3, 4))
    if idx == 1:
        return lines((1, 2), ("B", 1, 1), ("Q", 1))
    n = rng.randint(2, 18)
    q = rng.randint(4, 25)
    rows = [(n, q)]
    has_q = False
    for j in range(q):
        kind = rng.choice(["B", "E", "Q"])
        if j == q - 1 and not has_q:
            kind = "Q"
        if kind in ("B", "E"):
            x = rng.randint(1, n)
            y = rng.randint(1, n)
            rows.append((kind, x, y))
        else:
            has_q = True
            vals = list(range(1, n + 1))
            rng.shuffle(vals)
            rows.append(("Q", *vals))
    return lines(*rows)


def gen_dmopc20c6p3(rng, idx):
    if idx == 0:
        return lines((4, 4), [1, 0, 1, 0], (1, 2), (2, 3), (3, 1), (3, 4))
    if idx == 1:
        return lines((3, 0), [0, 1, 0])
    n = rng.randint(1, 16)
    m = rng.randint(0, min(30, n * n))
    want = [rng.randint(0, 1) for _ in range(n)]
    edges = [(rng.randint(1, n), rng.randint(1, n)) for _ in range(m)]
    return lines((n, m), want, *edges)


def gen_dmopc21c2p4(rng, idx):
    if idx == 0:
        return lines((7, 1), [2, 2, 2, 1, 1, 3, 3])
    if idx == 1:
        return lines((5, 0), [1, 1, 2, 2, 1])
    n = rng.randint(1, 28)
    k = rng.randint(0, 4)
    alpha = rng.randint(2, 5)
    arr = [rng.randint(1, alpha) for _ in range(n)]
    return lines((n, k), arr)


def gen_dmopc20c7p2(rng, idx):
    if idx == 0:
        return lines((1,), (0, 0))
    if idx == 1:
        return lines((2,), (0, 0), (2, 0))
    if idx == 2:
        return lines((3,), (-2, 0), (0, 0), (0, -2))

    # Build a simply-connected cluster of tiles on the even-coordinate lattice
    # (2i, 2j), where (x+y)%4==0 are squares and (x+y)%4==2 are octagons.
    def is_simply_connected(tiles):
        xs = [p[0] for p in tiles]
        ys = [p[1] for p in tiles]
        minx, maxx = min(xs) - 2, max(xs) + 2
        miny, maxy = min(ys) - 2, max(ys) + 2
        # Check outer complement connectivity in 4-grid on even coords
        # (and also ensure no two non-adjacent tiles pinch a vertex!).
        # Actually verify vertex degrees in the exact boundary graph directly:
        sq = [(0, 0), (0, 1), (1, 1), (1, 0)]
        octa = [(-1, 0), (-1, 1), (0, 2), (1, 2), (2, 1), (2, 0), (1, -1), (0, -1)]
        nxt = {}
        for ti, (x, y) in enumerate(tiles):
            sh = sq if (x + y) % 4 == 0 else octa
            m = len(sh)
            for j in range(m):
                u = (x + sh[j][0], y + sh[j][1])
                v = (x + sh[(j + 1) % m][0], y + sh[(j + 1) % m][1])
                nxt.setdefault(u, []).append((v, ti))
        # Every boundary vertex in a single simple polygon without pinches
        # has 1 or 2 outgoing edges (never > 2), and walking from min deg-1
        # vertex visits every deg-1 vertex!
        deg1 = [k for k, v in nxt.items() if len(v) == 1]
        if not deg1:
            return False
        start = min(deg1)
        cur = start
        cur_t = nxt[start][0][1]
        seen = set()
        steps = 0
        while True:
            if cur in seen or steps > 2000:
                return False
            seen.add(cur)
            es = nxt[cur]
            if len(es) not in (1, 2):
                return False
            if len(es) == 1:
                cur, cur_t = es[0]
            else:
                cur, cur_t = es[0] if es[0][1] != cur_t else es[1]
            steps += 1
            if cur == start:
                break
        return all(v in seen for v in deg1)

    target = rng.randint(1, 12)
    x0 = rng.randint(-3, 3) * 2
    y0 = rng.randint(-3, 3) * 2
    tiles = [(x0, y0)]
    tset = {(x0, y0)}
    for _ in range(target * 6):
        if len(tiles) >= target:
            break
        bx, by = rng.choice(tiles)
        dx, dy = rng.choice([(2, 0), (-2, 0), (0, 2), (0, -2)])
        cand = (bx + dx, by + dy)
        if cand in tset:
            continue
        trial = tiles + [cand]
        if is_simply_connected(trial):
            tiles = trial
            tset.add(cand)
    rng.shuffle(tiles)
    return lines((len(tiles),), *tiles)


def gen_dmopc21c8p3(rng, idx):
    if idx == 0:
        return lines((5, 4))
    if idx == 1:
        return lines((5, 5))
    if idx == 2:
        return lines((6, 100))
    n = rng.randint(1, 30)
    ways = 0
    for i in range(1, n + 1):
        l = i // 2
        r = i - l - 1
        ways += l * r
    if idx % 5 == 0:
        k = ways + rng.randint(1, 5)
    else:
        k = rng.randint(0, ways)
    return lines((n, k))


def norm_standard(out):
    return [line.strip() for line in out.strip().splitlines() if line.strip()]


def norm_dmopc20c6p6(out):
    # Original C++ with -DLOCAL emits E/F/D lines before final `A ...` line.
    a_lines = [line.strip() for line in out.strip().splitlines() if line.strip().startswith("A ")]
    return a_lines[-1:] if a_lines else [out.strip()]


PORTS = [
    ("dmopc21c2p6", "3968585.cpp", gen_dmopc21c2p6, [], norm_standard),
    ("dmopc20c6p6", "5200785.cpp", gen_dmopc20c6p6, ["-DLOCAL"], norm_dmopc20c6p6),
    ("dmopc21c2p5", "3968384.cpp", gen_dmopc21c2p5, [], norm_standard),
    ("dmopc21c8p5", "4533533.cpp", gen_dmopc21c8p5, [], norm_standard),
    ("dmopc21c7p4", "4825739.cpp", gen_dmopc21c7p4, [], norm_standard),
    ("dmopc21c2p2", "3946515.cpp", gen_dmopc21c2p2, [], norm_standard),
    ("dmopc20c6p3", "3917634.cpp", gen_dmopc20c6p3, [], norm_standard),
    ("dmopc21c2p4", "3949962.cpp", gen_dmopc21c2p4, [], norm_standard),
    ("dmopc20c7p2", "3686612.cpp", gen_dmopc20c7p2, [], norm_standard),
    ("dmopc21c8p3", "4537755.cpp", gen_dmopc21c8p3, [], norm_standard),
]


def main():
    try:
        resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))
    except Exception:
        pass

    only = sys.argv[1] if len(sys.argv) > 1 else None
    iters = int(sys.argv[2]) if len(sys.argv) > 2 else 30

    total_pass = 0
    total_fail = 0

    with tempfile.TemporaryDirectory() as tmp:
        for name, orig_file, gen_fn, extra_flags, norm_fn in PORTS:
            if only and name != only:
                continue
            orig_path = os.path.join(ROOT, "corpus", "submissions", orig_file)
            cp_path = os.path.join(HERE, "today", f"{name}.cp")
            exe_path = os.path.join(tmp, name)

            comp = subprocess.run(
                ["g++", "-O2", "-std=c++17", *extra_flags, orig_path, "-o", exe_path],
                capture_output=True, text=True,
            )
            if comp.returncode != 0:
                print(f"COMPILE FAIL {name} ({orig_file}):\n{comp.stderr}")
                total_fail += iters
                continue

            rng = random.Random(34_000 + hash(name) % 1000)
            ok = 0
            for idx in range(iters):
                inp = gen_fn(rng, idx)
                r_cpp = subprocess.run([exe_path], input=inp, capture_output=True, text=True, timeout=10)
                if r_cpp.returncode != 0:
                    print(f"CPP CRASH {name} case {idx}: rc={r_cpp.returncode}\nstderr={r_cpp.stderr}\ninp={inp}")
                    total_fail += 1
                    break
                r_cp = subprocess.run(
                    [sys.executable, CP, "lib-run", cp_path],
                    input=inp, capture_output=True, text=True, timeout=30, cwd=ROOT,
                )
                if r_cp.returncode != 0:
                    print(f"CP FAIL {name} case {idx}: rc={r_cp.returncode}\nstderr={r_cp.stderr}\nstdout={r_cp.stdout}\ninp={inp}")
                    total_fail += 1
                    break
                out_cpp = norm_fn(r_cpp.stdout)
                out_cp = norm_fn(r_cp.stdout)
                if out_cpp != out_cp:
                    print(f"DIFF {name} case {idx}:\n  cpp={out_cpp}\n  cp ={out_cp}\n  inp=\n{inp}")
                    total_fail += 1
                    break
                ok += 1
                total_pass += 1
            if ok == iters:
                print(f"OK   {name:14s} ({orig_file}): {ok}/{iters} passed")

    print(f"\nTOTAL: {total_pass} passed, {total_fail} failed")
    sys.exit(0 if total_fail == 0 else 1)


if __name__ == "__main__":
    main()
