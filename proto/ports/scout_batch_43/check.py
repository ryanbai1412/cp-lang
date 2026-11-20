#!/usr/bin/env python3
"""Differential test harness for Scout Batch 43 (hard ETSR '23 classics).

Compiles the 10 original AC C++ submissions from corpus/submissions/ into a
temporary directory and checks every runnable CP Lang port in today/ via
`proto/cp.py lib-run` on randomized and edge-case inputs.

    python3 proto/ports/scout_batch_43/check.py               # all 10 ports, 30 cases each
    python3 proto/ports/scout_batch_43/check.py etsr23r1p4 50 # single port, 50 cases
"""
import os, random, resource, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_etsr23r1p3(rng, idx):
    if idx == 0:
        return lines((3,), (2, "aba"), (2, "ab"), (22, "abcdefghij"))
    if idx == 1:
        return lines((2,), (3, "abcabcabc"), (1, "a"))
    tcs = rng.randint(1, 3)
    rows = [(tcs,)]
    for _ in range(tcs):
        if rng.random() < 0.15:
            m = rng.randint(22, 26)
            s = "".join(rng.choice("abcde") for _ in range(rng.randint(1, 12)))
            rows.append((m, s))
            continue
        m = rng.randint(1, 5)
        alpha = [chr(ord("a") + i) for i in range(m)]
        if rng.random() < 0.5:
            # Build a string likely to cover all permutations.
            parts = []
            for _ in range(rng.randint(m, m + 3)):
                p = alpha[:]
                rng.shuffle(p)
                parts.extend(p)
            s = "".join(parts)
        else:
            length = rng.randint(1, m * m + 4)
            s = "".join(rng.choice(alpha) for _ in range(length))
        rows.append((m, s))
    return lines(*rows)


def gen_etsr23r1p4(rng, idx):
    if idx == 0:
        return lines(
            (5, 6),
            [-7, -1, 0, 5, 13],
            (2, 1, 5, 2),
            (3, 1, 5),
            (4, 1, 5),
            (1, 2, 4, -3),
            (2, 1, 4, 3),
            (4, 1, 5),
        )
    if idx == 1:
        return lines((1, 4), [-10], (2, 1, 1, 3), (3, 1, 1), (1, 1, 1, 5), (4, 1, 1))
    n = rng.randint(2, 18)
    q = rng.randint(5, 25)
    arr = [rng.randint(-80, 80) for _ in range(n)]
    rows = [(n, q), arr]
    for j in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        ty = rng.choice([1, 2, 2, 3, 4]) if j < q - 1 else rng.choice([3, 4])
        if ty == 1:
            rows.append((1, l, r, rng.randint(-40, 40)))
        elif ty == 2:
            rows.append((2, l, r, rng.randint(1, 9)))
        else:
            rows.append((ty, l, r))
    return lines(*rows)


def gen_etsr23r2p2(rng, idx):
    if idx == 0:
        return lines((3,), ("123", 6), ("123", 7), ("000", 0))
    if idx == 1:
        return lines((2,), ("999", 250), ("1010", 2))
    tcs = rng.randint(1, 2)
    rows = [(tcs,)]
    for _ in range(tcs):
        n = rng.randint(1, 7)
        s = "".join(rng.choice("0123456789") for _ in range(n))
        if rng.random() < 0.65:
            # Construct a reachable target by inserting random ops.
            expr = s[0]
            for ch in s[1:]:
                op = rng.choice(["", "+", "*"])
                expr += op + ch
            # Evaluate avoiding leading-zero octal issues in Python.
            total = 0
            for term in expr.split("+"):
                prod = 1
                for fac in term.split("*"):
                    prod *= int(fac)
                total += prod
            kv = total if total <= 250 else rng.randint(0, 250)
        else:
            kv = rng.randint(0, 250)
        rows.append((s, kv))
    return lines(*rows)


def gen_etsr23r2p3(rng, idx):
    if idx == 0:
        return lines((4,), (2, 6, 10), (1, 3, 5), (4, 8, 12), (5, 5, 5))
    if idx == 1:
        return lines((3,), (16, 48, 80), (3, 4, 5), (8, 24, 40))
    tcs = rng.randint(2, 6)
    rows = [(tcs,)]
    for _ in range(tcs):
        if rng.random() < 0.5:
            a = rng.randint(1, 500)
            step = 1 << ((a & -a).bit_length())
            b = a + step * rng.randint(0, 20)
            c = b + step * rng.randint(0, 20)
            trip = [a, b, c]
            rng.shuffle(trip)
            rows.append(tuple(trip))
        else:
            rows.append((rng.randint(1, 5000), rng.randint(1, 5000), rng.randint(1, 5000)))
    return lines(*rows)


def gen_etsr23r2p4(rng, idx):
    if idx == 0:
        return lines((2,), (3,), (5, 2), (3, 1), (7, 0), (1,), (10, 0))
    if idx == 1:
        return lines((1,), (5,), (1, 3), (4, 2), (2, 2), (6, 1), (3, 0))
    tcs = rng.randint(1, 2)
    rows = [(tcs,)]
    for _ in range(tcs):
        n = rng.randint(1, 8)
        rows.append((n,))
        for i in range(n):
            val = rng.randint(0, 63)
            tol = rng.randint(0, min(5, n - 1 - i))
            rows.append((val, tol))
    return lines(*rows)


def gen_etsr23r3p1(rng, idx):
    if idx == 0:
        return lines(
            (4, 2, 10),
            (1, 3),
            (3, 2),
            (2, 5),
            (4, 1),
            (5,),
            [0, 1, 2, 5, 10],
        )
    if idx == 1:
        return lines((2, -5, 5), (3, 0), (4, 2), (4,), [-10, -1, 0, 10])
    n = rng.randint(1, 14)
    ql = rng.randint(-50, 20)
    qr = rng.randint(ql, ql + 60)
    rows = [(n, ql, qr)]
    for _ in range(n):
        t = rng.randint(1, 4)
        if t in (1, 2):
            x = rng.randint(0, 40)
        elif t == 3:
            x = rng.randint(0, 3)
        else:
            x = rng.randint(0, 3)
        rows.append((t, x))
    q = rng.randint(1, 15)
    rows.append((q,))
    arr = [rng.randint(-60, 60) for _ in range(q)]
    rows.append(arr)
    return lines(*rows)


def gen_etsr23r3p3(rng, idx):
    if idx == 0:
        return lines(
            (1,),
            (4, 4),
            (1, 2, 3, 5),
            (2, 3, 4, 2),
            (3, 4, 2, 6),
            (1, 4, 10, 4),
            (4, 1, 6),
            (3, 3),
            (2, 5),
            (4, 1),
            (1, 6),
        )
    if idx == 1:
        return lines((1,), (2, 1), (1, 2, 7, 3), (3, 0, 5), (2, 0), (2, 3), (2, 2))
    tcs = rng.randint(1, 2)
    rows = [(tcs,)]
    for _ in range(tcs):
        n = rng.randint(2, 12)
        extra = rng.randint(0, min(12, n * (n - 1) // 2 - (n - 1)))
        m = (n - 1) + extra
        rows.append((n, m))
        # Connected spanning tree first.
        for u in range(2, n + 1):
            p = rng.randint(1, u - 1)
            rows.append((p, u, rng.randint(1, 30), rng.randint(1, 15)))
        for _ in range(extra):
            u = rng.randint(1, n)
            v = rng.randint(1, n)
            while v == u:
                v = rng.randint(1, n)
            rows.append((u, v, rng.randint(1, 30), rng.randint(1, 15)))
        q = rng.randint(3, 12)
        online = rng.choice([0, 1])
        tmax = rng.randint(5, 20)
        rows.append((q, online, tmax))
        for _ in range(q):
            rows.append((rng.randint(1, n), rng.randint(0, tmax)))
    return lines(*rows)


def gen_etsr23r4p1(rng, idx):
    if idx == 0:
        return lines((4,), [2, 3, 4, 1])
    if idx == 1:
        return lines((6,), [2, 1, 4, 5, 6, 3])
    if idx == 2:
        # Odd cycle -> empty output in both.
        return lines((3,), [2, 3, 1])
    half = rng.randint(1, 7)
    n = 2 * half
    # Build a random Dyck word of length n.
    opens, closes = [], []
    bal = 0
    rem_open, rem_close = half, half
    for i in range(n):
        if rem_open == 0:
            pick_open = False
        elif bal == 0:
            pick_open = True
        else:
            pick_open = rng.random() < 0.5
        if pick_open:
            opens.append(i)
            rem_open -= 1
            bal += 1
        else:
            closes.append(i)
            rem_close -= 1
            bal -= 1
    # Build a permutation mapping opens -> closes and closes -> opens (all cycles even).
    p = [0] * n
    c_perm = closes[:]
    rng.shuffle(c_perm)
    o_perm = opens[:]
    rng.shuffle(o_perm)
    for o, c in zip(opens, c_perm):
        p[o] = c + 1
    for c, o in zip(closes, o_perm):
        p[c] = o + 1
    return lines((n,), p)


def gen_etsr23r4p2(rng, idx):
    if idx == 0:
        return lines((4, 5, 100, 3), [1, 2, 0, 5], (1, 1, 4), (0, 1, 3), (1, 1, 4), (0, 2, 4), (1, 1, 4))
    if idx == 1:
        return lines((3, 4, 1, 2), [0, 5, 9], (1, 1, 3), (0, 1, 3), (1, 2, 3), (0, 1, 2))
    # Moduli whose totient chain length <= 10 (matching 5529698.cpp maxb = 11).
    moduli = [1, 2, 3, 5, 6, 7, 10, 12, 16, 30, 37, 64, 100, 127, 256, 512, 997, 1000, 1009]
    n = rng.randint(1, 10)
    q = rng.randint(4, 16)
    p = rng.choice(moduli)
    base = rng.randint(2, 30)
    arr = [rng.randint(0, 40) for _ in range(n)]
    rows = [(n, q, p, base), arr]
    for j in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        ty = rng.choice([0, 1]) if j < q - 1 else 1
        rows.append((ty, l, r))
    return lines(*rows)


def gen_etsr23r4p3(rng, idx):
    if idx == 0:
        return lines((4, 4), (1, 2), (2, 3), (3, 4), (4, 1), (1, 2), (2, 3), (3, 4))
    if idx == 1:
        # Complete graph K_5 + star tree.
        edges = [(i, j) for i in range(1, 6) for j in range(i + 1, 6)]
        tree = [(1, 2), (1, 3), (1, 4), (1, 5)]
        return lines((5, len(edges)), *edges, *tree)
    n = rng.randint(2, 7)
    all_pairs = [(i, j) for i in range(1, n + 1) for j in range(i + 1, n + 1)]
    rng.shuffle(all_pairs)
    m = rng.randint(0, len(all_pairs))
    gedges = all_pairs[:m]
    tedges = []
    mode = idx % 3
    for u in range(2, n + 1):
        if mode == 0:
            p = rng.randint(1, u - 1)
        elif mode == 1:
            p = 1
        else:
            p = u - 1
        tedges.append((p, u))
    return lines((n, m), *gedges, *tedges)


PROBLEMS = [
    ("etsr23r1p3", "5494303.cpp", gen_etsr23r1p3),
    ("etsr23r1p4", "5495163.cpp", gen_etsr23r1p4),
    ("etsr23r2p2", "5494433.cpp", gen_etsr23r2p2),
    ("etsr23r2p3", "5494453.cpp", gen_etsr23r2p3),
    ("etsr23r2p4", "5494510.cpp", gen_etsr23r2p4),
    ("etsr23r3p1", "5527625.cpp", gen_etsr23r3p1),
    ("etsr23r3p3", "5521298.cpp", gen_etsr23r3p3),
    ("etsr23r4p1", "5521420.cpp", gen_etsr23r4p1),
    ("etsr23r4p2", "5529698.cpp", gen_etsr23r4p2),
    ("etsr23r4p3", "5635626.cpp", gen_etsr23r4p3),
]


def norm(s):
    return [line.strip() for line in s.strip().splitlines() if line.strip()]


def run_one(name, orig_file, gen, cases, tmpdir):
    orig_src = os.path.join(ROOT, "corpus", "submissions", orig_file)
    orig_bin = os.path.join(tmpdir, name + "_orig")
    cp_src = os.path.join(HERE, "today", name + ".cp")
    comp = subprocess.run(
        ["g++", "-O2", "-std=c++17", orig_src, "-o", orig_bin],
        capture_output=True,
        text=True,
    )
    if comp.returncode != 0:
        print(f"FAIL {name}: C++ compile error:\n{comp.stderr}")
        return False

    rng = random.Random(4300 + sum(ord(c) for c in name))
    for i in range(cases):
        inp = gen(rng, i)
        r_cpp = subprocess.run([orig_bin], input=inp, capture_output=True, text=True, timeout=15)
        if r_cpp.returncode != 0:
            print(f"FAIL {name} case {i}: C++ exited {r_cpp.returncode}: {r_cpp.stderr}")
            return False
        r_cp = subprocess.run(
            ["python3", CP, "lib-run", cp_src],
            input=inp,
            capture_output=True,
            text=True,
            timeout=30,
        )
        if r_cp.returncode != 0:
            print(f"FAIL {name} case {i}: CP lib-run exited {r_cp.returncode}:\n{r_cp.stderr}\ninput:\n{inp}")
            return False
        if norm(r_cpp.stdout) != norm(r_cp.stdout):
            print(f"FAIL {name} case {i}: mismatch")
            print("  input:\n" + inp)
            print("  cpp:", norm(r_cpp.stdout))
            print("  cp :", norm(r_cp.stdout))
            return False
    print(f"PASS {name}: {cases}/{cases} cases matched {orig_file}")
    return True


def main():
    try:
        resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))
    except Exception:
        pass
    only = sys.argv[1] if len(sys.argv) > 1 else None
    cases = int(sys.argv[2]) if len(sys.argv) > 2 else 30
    selected = [p for p in PROBLEMS if only is None or p[0] == only]
    if not selected:
        print(f"Unknown problem: {only}")
        sys.exit(2)
    ok = True
    with tempfile.TemporaryDirectory() as tmpdir:
        for name, orig_file, gen in selected:
            if not run_one(name, orig_file, gen, cases, tmpdir):
                ok = False
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
