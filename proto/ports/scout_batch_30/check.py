#!/usr/bin/env python3
"""Build and differential-test all 10 Scout Batch 30 ports against their
original accepted C++ submissions on small and medium random inputs."""
import os
import random
import resource
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
BATCH_DIR = os.path.dirname(os.path.abspath(__file__))
CP_PY = os.path.join(ROOT, "proto", "cp.py")
CORPUS = os.path.join(ROOT, "corpus", "submissions")

PROBLEMS = [
    ("ioi99p2", "4896839.cpp"),
    ("cco16p6", "4849378.cpp"),
    ("cco17p2", "6416978.cpp"),
    ("cco21p5", "4554642.cpp"),
    ("cco22p3", "4801968.cpp"),
    ("coci21c1p5", "4086660.cpp"),
    ("noi06p1", "5032743.cpp"),
    ("noi06p3", "3868871.cpp"),
    ("noi07p6", "3756979.cpp"),
    ("noi19p4", "3800719.cpp"),
]


def _unlimit_stack():
    soft, hard = resource.getrlimit(resource.RLIMIT_STACK)
    resource.setrlimit(resource.RLIMIT_STACK, (hard, hard))


def run_bin(path, inp, timeout=15):
    r = subprocess.run(
        [path], input=inp, text=True, capture_output=True, timeout=timeout,
        preexec_fn=_unlimit_stack,
    )
    if r.returncode != 0:
        raise RuntimeError(f"{path} exited {r.returncode}: {r.stderr[:400]}")
    return r.stdout.strip()


def gen_ioi99p2(rng, medium=False):
    m = rng.randint(10, 30) if medium else rng.randint(1, 8)
    alpha = "abcdef" if medium else "abcd"
    words = ["".join(rng.choice(alpha) for _ in range(rng.randint(1, 8 if medium else 4))) for _ in range(m)]
    s_len = rng.randint(80, 200) if medium else rng.randint(3, 25)
    s = "".join(rng.choice(alpha) for _ in range(s_len))
    return "\n".join([str(m)] + words + [s]) + "\n"


def check_ioi99p2(inp, out_cp, out_ref):
    lines = inp.strip().splitlines()
    m = int(lines[0])
    words = lines[1 : 1 + m]
    s = lines[1 + m]
    n = len(s)

    def parse(out):
        ls = [ln.strip() for ln in out.splitlines() if ln.strip()]
        tot = int(ls[0])
        trips = [tuple(map(int, ln.split())) for ln in ls[1:]]
        return tot, trips

    tc, trips_c = parse(out_cp)
    tr, _ = parse(out_ref)
    assert tc == tr, f"total length mismatch: cp={tc} ref={tr}"
    intervals = []
    sum_len = 0
    for wid, l, r in trips_c:
        assert 1 <= wid <= m and 1 <= l <= r <= n and (r - l) <= 1000
        w = words[wid - 1]
        # Verify w is a subsequence of s[l-1..r]
        wi = 0
        for ch in s[l - 1 : r]:
            if wi < len(w) and ch == w[wi]:
                wi += 1
        assert wi == len(w), f"word {w!r} not subseq of {s[l-1:r]!r}"
        intervals.append((l, r))
        sum_len += len(w)
    assert sum_len == tc, f"sum of matched word lengths {sum_len} != {tc}"
    intervals.sort()
    for i in range(1, len(intervals)):
        assert intervals[i - 1][1] < intervals[i][0], f"overlapping intervals {intervals[i-1]} and {intervals[i]}"


def gen_cco17p2(rng, medium=False):
    # Reverse-simulate 2:1 cuts from (n, 0) so every (n, m) is strictly tileable.
    n = rng.randint(1, 30) if medium else rng.randint(1, 10)
    m = 0
    steps = rng.randint(4, 12) if medium else rng.randint(1, 5)
    for _ in range(steps):
        k = rng.randint(1, 6 if medium else 3)
        jump = (n // 2) if (n % 2 == 0 and rng.random() < 0.5) else (2 * n)
        if m + k * jump > 100000000:
            break
        m += k * jump
        if rng.random() < 0.7:
            n, m = m, n
    if m == 0:
        m = 2 * n
    return f"{n} {m}\n"


def gen_noi07p6(rng, medium=False):
    n = rng.randint(40, 200) if medium else rng.randint(3, 20)
    edges = []
    for i in range(2, n + 1):
        p = rng.randint(1, i - 1)
        edges.append((p, i))
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    rng.shuffle(edges)
    lines = [str(n)] + [f"{perm[u - 1]} {perm[v - 1]}" for u, v in edges]
    return "\n".join(lines) + "\n"


def gen_cco16p6(rng, medium=False):
    n = rng.randint(50, 200) if medium else rng.randint(1, 18)
    kv = rng.randint(0, 500 if medium else 30)
    arr = [rng.randint(1, i + 1) for i in range(n)]
    return f"{n} {kv}\n" + "\n".join(map(str, arr)) + "\n"


def gen_cco21p5(rng, medium=False):
    n = rng.randint(40, 150) if medium else rng.randint(2, 16)
    max_e = min(n * (n - 1) // 2, 300 if medium else 25)
    m = rng.randint(1, max(1, max_e))
    edges = set()
    while len(edges) < m:
        u = rng.randint(1, n)
        v = rng.randint(1, n)
        if u != v:
            edges.add((min(u, v), max(u, v)))
    lines = [f"{n} {m}"] + [f"{u} {v}" for u, v in edges]
    return "\n".join(lines) + "\n"


def gen_cco22p3(rng, medium=False):
    n = rng.randint(20, 80) if medium else rng.randint(1, 10)
    m = rng.randint(20, 80) if medium else rng.randint(1, 10)
    k = rng.randint(1, 30) if medium else rng.randint(1, 10)

    def make_sched(cnt):
        segs = []
        t = 0
        for idx in range(cnt):
            if idx > 0:
                t += rng.randint(1, 8)
            r = t + rng.randint(1, 12)
            segs.append((t, r))
            t = r
        return segs

    # Note: original 4801968.cpp does --lower_bound(arr[!loc], {d + k + 1, -1}),
    # which triggers ASan heap-buffer-overflow unless arr[1][0].l <= arr[0][0].l + k.
    s0 = make_sched(n)
    s1 = make_sched(m)
    lines = [f"{n} {m} {k}"] + [f"{l} {r}" for l, r in s0] + [f"{l} {r}" for l, r in s1]
    return "\n".join(lines) + "\n"


def gen_coci21c1p5(rng, medium=False):
    n = rng.randint(50, 200) if medium else rng.randint(1, 18)
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    return f"{n}\n" + " ".join(map(str, perm)) + "\n"


def check_coci21c1p5(inp, out_cp, out_ref):
    toks = list(map(int, inp.split()))
    n = toks[0]
    arr = toks[1:]

    def parse(out):
        lines = [ln.strip() for ln in out.splitlines() if ln.strip()]
        k, length = map(int, lines[0].split())
        seqs = [list(map(int, ln.split())) for ln in lines[1:]]
        return k, length, seqs

    kc, lc, seqs_c = parse(out_cp)
    kr, lr, _ = parse(out_ref)
    assert (kc, lc) == (kr, lr), f"header mismatch: cp={(kc, lc)} ref={(kr, lr)}"
    assert len(seqs_c) == kc
    used = set()
    for seq in seqs_c:
        assert len(seq) == lc
        for i in range(lc):
            idx = seq[i]
            assert 1 <= idx <= n and idx not in used
            used.add(idx)
            if i > 0:
                assert seq[i - 1] < seq[i] and arr[seq[i - 1] - 1] < arr[seq[i] - 1]


def gen_noi06p1(rng, medium=False):
    m = rng.randint(4, 5) if medium else rng.randint(1, 3)
    n = 1 << m
    arr = [rng.randint(0, 1) for _ in range(n)]
    cost = [rng.randint(0, 500) for _ in range(n)]
    lines = [str(m), " ".join(map(str, arr)), " ".join(map(str, cost))]
    for i in range(n):
        row = [str(rng.randint(0, 200)) for _ in range(i + 1, n)]
        if row:
            lines.append(" ".join(row))
    return "\n".join(lines) + "\n"


def gen_noi06p3(rng, medium=False):
    n = rng.randint(50, 200) if medium else rng.randint(1, 18)
    hi = 30 if medium else 10
    lines = [str(n)]
    for _ in range(n):
        a = rng.randint(0, hi)
        b = a + rng.randint(1, hi)
        lines.append(f"{a} {b}")
    return "\n".join(lines) + "\n"


def gen_noi19p4(rng, medium=False):
    n = rng.randint(40, 120) if medium else rng.randint(2, 15)
    w = rng.randint(20, 200) if medium else rng.randint(2, 15)
    h = rng.randint(20, 200) if medium else rng.randint(2, 15)
    m = rng.randint(60, 200) if medium else rng.randint(n, n * 2 + 5)
    pts = [(rng.randint(1, w), rng.randint(1, h)) for _ in range(n)]
    portals = []
    for i in range(1, n):
        u = rng.randint(1, i)
        t = rng.randint(1, 100)
        x, y = pts[i]
        portals.append((u, t, x, x, y, y))
    while len(portals) < m:
        u = rng.randint(1, n)
        t = rng.randint(1, 100)
        x1, x2 = sorted((rng.randint(1, w), rng.randint(1, w)))
        y1, y2 = sorted((rng.randint(1, h), rng.randint(1, h)))
        portals.append((u, t, x1, x2, y1, y2))
    rng.shuffle(portals)
    lines = [f"{n} {len(portals)} {w} {h}"]
    lines += [f"{x} {y}" for x, y in pts]
    lines += [f"{u} {t} {lx} {rx} {ly} {ry}" for u, t, lx, rx, ly, ry in portals]
    return "\n".join(lines) + "\n"


GENS = {
    "ioi99p2": (gen_ioi99p2, check_ioi99p2),
    "cco16p6": (gen_cco16p6, None),
    "cco17p2": (gen_cco17p2, None),
    "cco21p5": (gen_cco21p5, None),
    "cco22p3": (gen_cco22p3, None),
    "coci21c1p5": (gen_coci21c1p5, check_coci21c1p5),
    "noi06p1": (gen_noi06p1, None),
    "noi06p3": (gen_noi06p3, None),
    "noi07p6": (gen_noi07p6, None),
    "noi19p4": (gen_noi19p4, None),
}


def main():
    tmp = tempfile.mkdtemp(prefix="scout30_")
    try:
        rng = random.Random(202630)
        only = set(sys.argv[1:])
        for name, ref_file in PROBLEMS:
            if only and name not in only:
                continue
            t0 = time.time()
            cp_src = os.path.join(BATCH_DIR, f"{name}.cp")
            cp_copy = os.path.join(tmp, f"{name}.cp")
            shutil.copy(cp_src, cp_copy)

            r = subprocess.run(["python3", CP_PY, "build", cp_copy], capture_output=True, text=True)
            if r.returncode != 0:
                print(f"[FAIL] {name}: cp build failed:\n{r.stderr}{r.stdout}")
                sys.exit(1)

            cp_bin = os.path.join(tmp, f"{name}_cp")
            r = subprocess.run(
                ["g++", "-O2", "-std=c++17", "-o", cp_bin, os.path.join(tmp, f"{name}.cpp")],
                capture_output=True, text=True,
            )
            if r.returncode != 0:
                print(f"[FAIL] {name}: g++ cp failed:\n{r.stderr}")
                sys.exit(1)

            ref_bin = os.path.join(tmp, f"{name}_ref")
            r = subprocess.run(
                ["g++", "-O2", "-std=c++17", "-o", ref_bin, os.path.join(CORPUS, ref_file)],
                capture_output=True, text=True,
            )
            if r.returncode != 0:
                print(f"[FAIL] {name}: g++ ref failed:\n{r.stderr}")
                sys.exit(1)

            gen_fn, checker = GENS[name]
            small_n = 80
            med_n = 20
            for idx in range(small_n + med_n):
                medium = idx >= small_n
                inp = gen_fn(rng, medium=medium)
                out_cp = run_bin(cp_bin, inp)
                out_ref = run_bin(ref_bin, inp)
                if checker is not None:
                    try:
                        checker(inp, out_cp, out_ref)
                    except AssertionError as e:
                        print(f"[FAIL] {name} case {idx} ({'med' if medium else 'small'}): {e}")
                        print("INPUT:\n" + inp[:600])
                        print("CP:\n" + out_cp[:300])
                        print("REF:\n" + out_ref[:300])
                        sys.exit(1)
                else:
                    if out_cp.split() != out_ref.split():
                        print(f"[FAIL] {name} case {idx} ({'med' if medium else 'small'})")
                        print("INPUT:\n" + inp[:600])
                        print("CP:\n" + out_cp[:300])
                        print("REF:\n" + out_ref[:300])
                        sys.exit(1)
            dt = time.time() - t0
            print(f"[PASS] {name:12s} ({ref_file}): {small_n} small + {med_n} medium cases matched in {dt:.2f}s")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    main()
