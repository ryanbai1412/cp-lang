#!/usr/bin/env python3
"""Differential test harness for scout_batch_18 statement-first ports.

Compiles the 10 original AC C++ submissions from corpus/submissions/ into a
temporary directory and checks every runnable statement-first CP Lang port in
today/ via `proto/cp.py lib-run` on randomized and edge-case inputs.

    python3 proto/ports/scout_batch_18/check.py            # all 10 ports, 20 cases each
    python3 proto/ports/scout_batch_18/check.py cco21p4 50 # single port, 50 cases
"""
import os, random, re, resource, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_dpw(rng, idx):
    if idx == 0:
        return lines((1, 1), (1, 1, -5))
    if idx == 1:
        return lines((5, 3), (1, 3, 10), (2, 4, -5), (3, 5, 10))
    n = rng.randint(1, 12)
    m = rng.randint(1, 20)
    rows = [(n, m)]
    for _ in range(m):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        a = rng.randint(-50, 50)
        rows.append((l, r, a))
    return lines(*rows)


def gen_cco21p1(rng, idx):
    if idx == 0:
        return lines((4, 3, 3), [3, 1, 2, 1], (1,), (2,), (1,))
    n = rng.randint(1, 15)
    k = rng.randint(2, 8)
    q = rng.randint(1, 15)
    arr = [rng.randint(1, k) for _ in range(n)]
    swaps = [(rng.randint(1, k - 1),) for _ in range(q)]
    return lines((n, k, q), arr, *swaps)


def gen_btoi17p2(rng, idx):
    if idx == 0:
        return lines((2, 6, 4, 4), (0, 2, 5), (0, 3, 2), (3, 5, 4), (2, 5, 1), (0, 5), (0, 0), (1, 0), (5, 0))
    k = rng.randint(1, 4)
    layers = rng.randint(1, 5)
    n = (layers - 1) * k + rng.randint(1, k)
    edge_map = {}
    for g in range(layers - 1):
        for sa in range(k):
            u = g * k + sa
            for sb in range(k):
                v = (g + 1) * k + sb
                if v < n and rng.random() < 0.55:
                    edge_map[(u, v)] = rng.randint(0, 20)
    edges = [(u, v, w) for (u, v), w in edge_map.items()]
    o = rng.randint(1, 12)
    qs = [(rng.randint(0, n - 1), rng.randint(0, n - 1)) for _ in range(o)]
    return lines((k, n, len(edges), o), *edges, *qs)


def gen_ioi05p2(rng, idx):
    if idx == 0:
        return "4\nQ 1\nI 1 4 2\nQ 3\nQ 1\nI 2 2 -1\nQ 3\nE\n"
    n = rng.choice([rng.randint(1, 20), rng.randint(50, 10000)])
    q = rng.randint(3, 18)
    out = [str(n)]
    for _ in range(q):
        if rng.random() < 0.5:
            a = rng.randint(1, n)
            b = rng.randint(a, n)
            d = rng.randint(-10, 10)
            out.append(f"I {a} {b} {d}")
        else:
            h = rng.randint(0, max(1, min(500, n * 5)))
            out.append(f"Q {h}")
    out.append("E")
    return "\n".join(out) + "\n"


def gen_noi10p2(rng, idx):
    if idx == 0:
        return lines((4, 3, 2, 3), (3,), (2,), (-6,), (8,))
    n = rng.randint(1, 14)
    lo = rng.randint(1, n)
    hi = rng.randint(lo, n)
    total_valid = sum(n - length + 1 for length in range(lo, hi + 1))
    k = rng.randint(1, min(total_valid, 25))
    arr = [(rng.randint(-20, 20),) for _ in range(n)]
    return lines((n, k, lo, hi), *arr)


def gen_btoi20p3(rng, idx):
    if idx == 0:
        # Already bipartite graph
        return lines((4, 3, 3), (1, 2), (2, 3), (3, 4), (1, 1), (1, 3), (2, 2))
    if idx == 1:
        # Single triangle
        return lines((3, 3, 4), (1, 2), (2, 3), (3, 1), (1, 1), (2, 2), (1, 3), (2, 3))
    n = rng.randint(2, 10)
    m = rng.randint(1, 16)
    q = rng.randint(1, 15)
    edges = []
    for _ in range(m):
        u = rng.randint(1, n)
        v = rng.randint(1, n)
        while v == u:
            v = rng.randint(1, n)
        edges.append((u, v))
    qs = []
    for _ in range(q):
        l = rng.randint(1, m)
        r = rng.randint(l, m)
        qs.append((l, r))
    return lines((n, m, q), *edges, *qs)


def gen_cco21p4(rng, idx):
    if idx == 0:
        return lines((4, 4), (1, 2, 10, 5), (2, 3, 15, 0), (3, 2, 5, 0), (4, 1, 1, 0))
    n = rng.randint(1, 10)
    m = rng.randint(0, 18)
    edges = []
    for _ in range(m):
        a = rng.randint(1, n)
        b = rng.randint(1, n)
        r = rng.randint(0, 30)
        p = rng.randint(0, 20)
        edges.append((a, b, r, p))
    return lines((n, m), *edges)


def gen_noi09p4(rng, idx):
    if idx == 0:
        return lines((2, 2), (10, 0), (-5, 1, 0, 0), (10, 1, 1, 1), (10, 0))
    n = rng.randint(1, 4)
    m = rng.randint(1, 4)
    rows = [(n, m)]
    for r in range(n):
        for c in range(m):
            s = rng.randint(-20, 20)
            w = rng.randint(0, min(3, n * m - 1))
            targets = set()
            while len(targets) < w:
                pr = rng.randint(0, n - 1)
                pc = rng.randint(0, m - 1)
                if (pr, pc) != (r, c):
                    targets.add((pr, pc))
            row = [s, len(targets)]
            for pr, pc in targets:
                row.extend([pr, pc])
            rows.append(row)
    return lines(*rows)


def gen_ccc22s5(rng, idx):
    if idx == 0:
        return lines((3,), (1, 2), (2, 3), ("NYN",), [4, 3, 4])
    if idx == 1:
        return lines((1,), ("Y",), [7])
    n = rng.randint(2, 14)
    edges = []
    # Mix random trees, paths, and stars
    mode = idx % 3
    for i in range(2, n + 1):
        if mode == 0:
            p = rng.randint(1, i - 1)
        elif mode == 1:
            p = i - 1
        else:
            p = 1
        edges.append((p, i))
    s = [rng.choice("YN") for _ in range(n)]
    if "Y" not in s:
        s[rng.randint(0, n - 1)] = "Y"
    costs = [rng.randint(1, 20) for _ in range(n)]
    return lines((n,), *edges, ("".join(s),), costs)


def gen_noi18p1(rng, idx):
    t = rng.randint(1, 3)
    rows = [(t,)]
    for _ in range(t):
        n = rng.randint(2, 10)
        # Connected graph: random spanning tree + extra edges
        edges = []
        for i in range(2, n + 1):
            p = rng.randint(1, i - 1)
            edges.append((p, i, rng.randint(1, 20), rng.randint(1, 30)))
        extra = rng.randint(0, n)
        for _ in range(extra):
            u = rng.randint(1, n)
            v = rng.randint(1, n)
            if u != v:
                edges.append((u, v, rng.randint(1, 20), rng.randint(1, 30)))
        q = rng.randint(1, 10)
        k_flag = rng.choice([0, 1])
        max_s = rng.randint(1, 35)
        rows.append((n, len(edges)))
        rows.extend(edges)
        rows.append((q, k_flag, max_s))
        for _ in range(q):
            rows.append((rng.randint(1, n), rng.randint(0, max_s)))
    return lines(*rows)


GENS = {
    "dpw": gen_dpw,
    "cco21p1": gen_cco21p1,
    "btoi17p2": gen_btoi17p2,
    "ioi05p2": gen_ioi05p2,
    "noi10p2": gen_noi10p2,
    "btoi20p3": gen_btoi20p3,
    "cco21p4": gen_cco21p4,
    "noi09p4": gen_noi09p4,
    "ccc22s5": gen_ccc22s5,
    "noi18p1": gen_noi18p1,
}


def check_one(name, iters, tmp):
    port = os.path.join(HERE, "today", name + ".cp")
    src = open(port).read()
    m = re.search(r"^// original: (\S+)", src, re.M)
    orig = os.path.join(ROOT, m.group(1))
    exe = os.path.join(tmp, name + "_ref")
    comp = subprocess.run(
        ["g++", "-std=c++17", "-O2", "-o", exe, orig],
        capture_output=True, text=True, cwd=ROOT,
    )
    if comp.returncode != 0:
        return False, f"g++ failed: {comp.stderr.splitlines()[:3]}"

    def _unlimited_stack():
        resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))

    rng = random.Random(20261120 + hash(name) % 10000)
    gen = GENS[name]
    for i in range(iters):
        inp = gen(rng, i)
        ref = subprocess.run([exe], input=inp, capture_output=True, text=True, timeout=10, preexec_fn=_unlimited_stack)
        if ref.returncode != 0:
            return False, f"ref crashed on case {i}: {ref.stderr}"
        ours = subprocess.run(
            ["python3", CP, "lib-run", port],
            input=inp, capture_output=True, text=True, cwd=ROOT, timeout=20,
        )
        if ours.returncode != 0:
            return False, f"lib-run failed on case {i}:\n{ours.stderr.strip()}\ninput:\n{inp}"
        if ours.stdout.split() != ref.stdout.split():
            return False, (
                f"MISMATCH on case {i}:\ninput:\n{inp}"
                f"ours: {ours.stdout.strip()}\nref:  {ref.stdout.strip()}"
            )
    return True, f"OK ({iters}/{iters} cases match {os.path.basename(orig)})"


def main():
    names = [sys.argv[1]] if len(sys.argv) > 1 else list(GENS.keys())
    iters = int(sys.argv[2]) if len(sys.argv) > 2 else 20
    failed = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name in names:
            ok, msg = check_one(name, iters, tmp)
            status = "PASS" if ok else "FAIL"
            print(f"[{status}] {name:10s} {msg}")
            if not ok:
                failed += 1
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
