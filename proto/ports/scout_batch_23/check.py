#!/usr/bin/env python3
"""Differential test harness for scout_batch_23 (Hard Combinatorics, Game Theory,
Expected Value & Polynomials/NTT).

Compiles the 10 original AC C++ submissions from corpus/submissions/ into a
temporary directory and checks every runnable statement-first CP Lang port in
today/ via `proto/cp.py lib-run` on randomized and edge-case inputs.

Also documents edge-case bugs/UBs found in the accepted corpus C++ submissions:
  - ioi08p3 (5032789.cpp): `1 << (__lg(n - 1) + 2)` is UB when F == 1 or K == 1.
  - noi19p5 (5652798.cpp): `mint(n - j).inv()` asserts `y == 1` when n <= 2.
  - noimps22p4 (4547602.cpp) & coci21c1p2 (4080842.cpp): multi-MB stack VLAs
    segfault under default 8 MB Linux stacks without raising RLIMIT_STACK.

Usage:
    python3 proto/ports/scout_batch_23/check.py              # all 10 ports, 20 cases each
    python3 proto/ports/scout_batch_23/check.py noi18p2 30   # single port, 30 cases
"""
import os
import random
import resource
import subprocess
import sys
import tempfile


def raise_stack():
    soft, hard = resource.getrlimit(resource.RLIMIT_STACK)
    target = 256 * 1024 * 1024
    if hard == resource.RLIM_INFINITY or hard >= target:
        resource.setrlimit(resource.RLIMIT_STACK, (target, hard))
    elif soft < hard:
        resource.setrlimit(resource.RLIMIT_STACK, (hard, hard))


HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")


def lines(*rows):
    return "\n".join(" ".join(map(str, r)) for r in rows) + "\n"


def gen_noi18p2(rng, idx):
    if idx == 0:
        return lines((3,), (2,), [1, 2], (3,), [1, 2, 3], (4,), [2, 1, 4, 3])
    t = rng.randint(1, 4)
    rows = [(t,)]
    for _ in range(t):
        n = rng.randint(1, 10)
        p = list(range(1, n + 1))
        rng.shuffle(p)
        rows.append((n,))
        rows.append(p)
    return lines(*rows)


def gen_ioi08p3(rng, idx):
    # Note: 5032789.cpp does `1 << (__lg(n - 1) + 2)` which is UB for F == 1 or K == 1,
    # so keep K >= 2 and F >= K when comparing against the C++ binary.
    if idx == 0:
        return lines((5, 3, 7), (2, 2), (5, 1), (8, 3), (4, 1), (2, 3))
    k = rng.randint(2, 6)
    f = rng.randint(k, 16)
    m = rng.randint(2, 100)
    gems = list(range(1, k + 1)) + [rng.randint(1, k) for _ in range(f - k)]
    rng.shuffle(gems)
    rows = [(f, k, m)]
    for g in gems:
        rows.append((rng.randint(1, 30), g))
    return lines(*rows)


def gen_noi09p5(rng, idx):
    if idx == 0:
        return lines((2, 2), ["AB"], ["BA"])
    if idx == 1:
        return lines((3, 2), ["AAB"], ["AB"])
    n = rng.randint(1, 12)
    m = rng.randint(1, 12)
    s1 = "".join(rng.choice("AB") for _ in range(n))
    s2 = "".join(rng.choice("AB") for _ in range(m))
    return lines((n, m), [s1], [s2])


def gen_noimps22p4(rng, idx):
    primes = []
    for x in range(2, 2001):
        if all(x % d != 0 for d in range(2, int(x**0.5) + 1)):
            primes.append(x)
    small_p = primes[:13]
    big_p = primes[13:30]
    if idx == 0:
        return lines((4,), [2, 3, 6, 47], (3,), (1, 2), (2, 2, 3), (2, 2, 47))
    n = rng.randint(1, 15)
    pool = [1, 2, 3, 4, 5, 6, 10, 12, 15, 30, 42, 47, 53, 94, 106, 141, 1999]
    arr = [rng.choice(pool) if rng.random() < 0.7 else rng.randint(1, 2000) for _ in range(n)]
    q = rng.randint(1, 5)
    rows = [(n,), arr, (q,)]
    for _ in range(q):
        ms = rng.randint(0, 4)
        mb = rng.randint(0, 3)
        if ms + mb == 0:
            ms = 1
        ps = rng.sample(small_p[:6], ms) + rng.sample(big_p[:8], mb)
        rng.shuffle(ps)
        rows.append([len(ps)] + ps)
    return lines(*rows)


def gen_noi19p5(rng, idx):
    # Note: 5652798.cpp asserts on n <= 2 (inverts n - 2 == 0), so compare C++ on n >= 3.
    if idx == 0:
        return lines((5, 2, 1), [2, 3], (5,), (1,), (2,), (3,), (4,), (5,))
    n = rng.randint(3, 25)
    m = rng.randint(0, 8)
    kind = rng.choice([1, 2])
    cuts = [rng.randint(0, n) for _ in range(m)]
    q = rng.randint(1, min(n, 8))
    asks = [(rng.randint(1, n),) for _ in range(q)]
    return lines((n, m, kind), cuts, (q,), *asks)


def gen_dpi(rng, idx):
    if idx == 0:
        return lines((3,), ["0.30", "0.60", "0.80"])
    if idx == 1:
        return lines((1,), ["0.50"])
    n = rng.randrange(1, 21, 2)
    ps = [f"{rng.uniform(0.0, 1.0):.4f}" for _ in range(n)]
    return lines((n,), ps)


def gen_noi07p1(rng, idx):
    if idx == 0:
        return lines((4, 4), (1, 2, 1), (2, 3, 1), (3, 4, 1), (4, 1, 1))
    n = rng.randint(2, 10)
    edges = {}
    for v in range(2, n + 1):
        u = rng.randint(1, v - 1)
        edges[(u, v)] = rng.randint(1, 5)
    extra = rng.randint(0, n)
    for _ in range(extra):
        u, v = sorted(rng.sample(range(1, n + 1), 2))
        if (u, v) not in edges:
            edges[(u, v)] = rng.randint(1, 5)
    rows = [(n, len(edges))]
    for (u, v), w in edges.items():
        rows.append((u, v, w))
    return lines(*rows)


def gen_egoi22p2(rng, idx):
    if idx == 0:
        return lines((3, 2),)
    if idx == 1:
        return lines((4, 3),)
    w = rng.randint(1, 25)
    h = rng.randint(1, 25)
    return lines((w, h),)


def gen_ioi17p3(rng, idx):
    if idx == 0:
        return lines((2, 4), [0, 1], [1, 0], (0, 0), (0, 1), (1, 0), (1, 1))
    n = rng.randint(1, 10)
    owner = [rng.randint(0, 1) for _ in range(n)]
    charge = [rng.randint(0, 1) for _ in range(n)]
    edge_set = set()
    for u in range(n):
        edge_set.add((u, rng.randint(0, n - 1)))
    extra = rng.randint(0, 2 * n)
    for _ in range(extra):
        edge_set.add((rng.randint(0, n - 1), rng.randint(0, n - 1)))
    edges = sorted(edge_set)
    return lines((n, len(edges)), owner, charge, *edges)


def gen_coci21c1p2(rng, idx):
    if idx == 0:
        return lines((4, 1), ["CCCP"])
    if idx == 1:
        return lines((8, 2), ["PCPCCHCP".replace("H", "C")])
    k = rng.randint(1, 6)
    n = rng.randint(2 * k - 1, 2 * k + 6)
    red_cnt = rng.randint(2 * k - 1, n)
    chars = ["C"] * red_cnt + ["P"] * (n - red_cnt)
    rng.shuffle(chars)
    return lines((n, k), ["".join(chars)])


PROBLEMS = [
    ("noi18p2", "6512891.cpp", gen_noi18p2, "exact"),
    ("ioi08p3", "5032789.cpp", gen_ioi08p3, "exact"),
    ("noi09p5", "5108108.cpp", gen_noi09p5, "exact"),
    ("noimps22p4", "4547602.cpp", gen_noimps22p4, "exact"),
    ("noi19p5", "5652798.cpp", gen_noi19p5, "exact"),
    ("dpi", "4131648.cpp", gen_dpi, "float_1e-6"),
    ("noi07p1", "5032737.cpp", gen_noi07p1, "float_1e-3"),
    ("egoi22p2", "5199888.cpp", gen_egoi22p2, "exact"),
    ("ioi17p3", "3921067.cpp", gen_ioi17p3, "tokens"),
    ("coci21c1p2", "4080842.cpp", gen_coci21c1p2, "exact"),
]


def compare(out_cp, out_ref, mode):
    if mode == "exact":
        return out_cp.strip() == out_ref.strip()
    if mode == "tokens":
        t_cp = out_cp.replace("[", " ").replace("]", " ").replace(",", " ").split()
        t_ref = out_ref.split()
        return t_cp == t_ref
    if mode.startswith("float_"):
        tol = float(mode.split("_")[1])
        a = [float(x) for x in out_cp.split()]
        b = [float(x) for x in out_ref.split()]
        if len(a) != len(b):
            return False
        return all(abs(x - y) <= tol * max(1.0, abs(y)) + 1e-12 for x, y in zip(a, b))
    return out_cp.strip() == out_ref.strip()


def main():
    only = sys.argv[1] if len(sys.argv) > 1 else None
    ncases = int(sys.argv[2]) if len(sys.argv) > 2 else 20

    failed = False
    with tempfile.TemporaryDirectory() as tmp:
        for name, cpp_file, gen, mode in PROBLEMS:
            if only and name != only:
                continue
            cp_path = os.path.join(HERE, "today", f"{name}.cp")
            cpp_path = os.path.join(ROOT, "corpus", "submissions", cpp_file)
            bin_path = os.path.join(tmp, name)
            r = subprocess.run(
                ["g++", "-O2", "-std=c++17", cpp_path, "-o", bin_path],
                capture_output=True,
                text=True,
            )
            if r.returncode != 0:
                print(f"[COMPILE FAIL] {name} ({cpp_file}): {r.stderr[:200]}")
                failed = True
                continue

            rng = random.Random(2300 + hash(name) % 1000)
            ok_cases = 0
            for idx in range(ncases):
                inp = gen(rng, idx)
                r_cp = subprocess.run(
                    ["python3", CP, "lib-run", cp_path],
                    input=inp,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                if r_cp.returncode != 0:
                    print(f"[FAIL] {name} case {idx}: CP Lang runtime error:\n{r_cp.stderr}\nInput:\n{inp}")
                    failed = True
                    break
                r_cpp = subprocess.run(
                    [bin_path],
                    input=inp,
                    capture_output=True,
                    text=True,
                    timeout=10,
                    preexec_fn=raise_stack,
                )
                if r_cpp.returncode != 0:
                    print(f"[CPP FAIL] {name} case {idx}: rc={r_cpp.returncode}\nInput:\n{inp}")
                    failed = True
                    break
                if not compare(r_cp.stdout, r_cpp.stdout, mode):
                    print(f"[MISMATCH] {name} case {idx}:\n  CP:  {r_cp.stdout.strip()!r}\n  C++: {r_cpp.stdout.strip()!r}\nInput:\n{inp}")
                    failed = True
                    break
                ok_cases += 1
            else:
                print(f"[PASS] {name:12s} {ok_cases}/{ncases} cases")

    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
