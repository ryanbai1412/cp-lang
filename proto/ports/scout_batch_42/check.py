#!/usr/bin/env python3
"""Differential test harness for Scout Batch 42 (Hard Mixed Corpus Classics).

Compiles all 10 original C++ submissions from corpus/submissions/ with
`g++ -O2 -std=c++17` into a temporary directory and checks every runnable
CP Lang port in `today/` via `proto/cp.py lib-run` on small and medium
randomized + edge-case inputs.

Usage:
    python3 proto/ports/scout_batch_42/check.py                # all 10 ports, 30 cases each
    python3 proto/ports/scout_batch_42/check.py mccc6s3 50     # single port, 50 cases
"""
import os, random, resource, subprocess, sys, tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CP = os.path.join(ROOT, "proto", "cp.py")

ORIG = {
    "mccc6s3": "6103125",
    "occ19g4": "3578469",
    "mccc5j5": "5333196",
    "mccc5j3": "5333116",
    "mccc4j4": "4264934",
    "mccc3j4": "4223440",
    "mccc3j5": "4223449",
    "mccc5j4": "5333145",
    "mccc3j3": "4223424",
    "mccc4j3": "4264907",
}


def rows(*ls):
    return "\n".join(" ".join(map(str, r)) for r in ls) + "\n"


def brute_mccc6s3(arr):
    n = len(arr)
    best = None
    for i1 in range(n):
        sl = sum(arr[:i1])
        sr = sum(arr[i1 + 1:])
        cand = (abs(sl - sr), i1 + 1, -1)
        if best is None or cand < best:
            best = cand
        for i2 in range(n):
            if i2 == i1:
                continue
            if i2 > i1:
                diff = abs((sl + arr[i2]) - (sr - arr[i2]))
            else:
                diff = abs((sl - arr[i2]) - (sr + arr[i2]))
            cand = (diff, i1 + 1, i2 + 1)
            if cand < best:
                best = cand
    return best


def eval_mccc6s3(arr, i1, i2):
    sl = sum(arr[: i1 - 1])
    sr = sum(arr[i1:])
    if i2 == -1:
        return abs(sl - sr)
    if i2 > i1:
        return abs((sl + arr[i2 - 1]) - (sr - arr[i2 - 1]))
    return abs((sl - arr[i2 - 1]) - (sr + arr[i2 - 1]))


def brute_occ19g4(pts):
    m = len(pts)
    u = [x - y for x, y in pts]
    v = [x + y for x, y in pts]
    d = [0] * m
    for k in range(m // 2):
        dist = max(abs(u[2 * k] - u[2 * k + 1]), abs(v[2 * k] - v[2 * k + 1])) - 1
        d[2 * k] = d[2 * k + 1] = dist
    ans = []
    for i in range(m):
        c = 0
        for j in range(m):
            if i == j:
                continue
            if max(abs(u[i] - u[j]), abs(v[i] - v[j])) <= min(d[i], d[j]):
                c += 1
        ans.append(c)
    return ans


def gen_mccc6s3(rng, idx):
    if idx == 0:
        return rows((5,), [1, 4, 2, 7, 6])
    if idx == 1:
        return rows((1,), [5])
    if idx == 2:
        return rows((6,), [6, 6, 6, 6, 6, 6])
    n = rng.randint(2, 16) if idx < 18 else rng.randint(20, 60)
    hi = rng.choice([n + 5, n * 3, 200])
    arr = [rng.randint(1, hi) for _ in range(n)]
    # Keep max(arr) >= n so corpus 6103125.cpp avoids its `next_ind[m]` stack overflow.
    if max(arr) < n:
        arr[rng.randrange(n)] = n
    return rows((n,), arr)


def gen_occ19g4(rng, idx):
    if idx == 0:
        return rows((3,), (1, 1), (4, 2), (2, 2), (5, 5), (3, 1), (3, 4))
    if idx == 1:
        return rows((1,), (10, 20), (15, 25))
    n = rng.randint(2, 10) if idx < 18 else rng.randint(12, 30)
    bound = 15 if idx < 12 else 500
    pts = []
    for _ in range(n):
        x1, y1 = rng.randint(0, bound), rng.randint(0, bound)
        x2, y2 = rng.randint(0, bound), rng.randint(0, bound)
        while (x1, y1) == (x2, y2):
            x2 = rng.randint(0, bound)
        pts.append((x1, y1))
        pts.append((x2, y2))
    return rows((n,), *pts)


def gen_mccc5j5(rng, idx):
    if idx == 0:
        return rows((5, 4), [3, 1, 7, 2, 9], (0, 0), (0, 4), (3, 2), (5, 10))
    if idx == 1:
        return rows((1, 3), [42], (0, 0), (1, 0), (1, 1))
    n = rng.randint(1, 20) if idx < 18 else rng.randint(30, 120)
    q = rng.randint(3, 15) if idx < 18 else rng.randint(20, 40)
    lim = max(n * 2, 16)
    a = [rng.randint(1, 1000) for _ in range(n)]
    qs = [(rng.randint(0, lim), rng.randint(0, lim)) for _ in range(q)]
    return rows((n, q), a, *qs)


def gen_mccc5j3(rng, idx):
    if idx == 0:
        return rows((5, 2, 6), [1, 2, 3, 2, 4], [5, 4, 3, 4, 2])
    if idx == 1:
        return rows((4, 0, 5), [2, 3, 2, 3], [3, 2, 3, 2])
    if idx == 2:
        return rows((5, 5, 10), [5, 5, 5, 5, 5], [5, 5, 5, 5, 5])
    n = rng.randint(1, 20) if idx < 18 else rng.randint(30, 120)
    k = rng.choice([0, 1, rng.randint(0, n)])
    vals = [rng.randint(-8, 8) for _ in range(n)]
    vals2 = [rng.randint(-8, 8) for _ in range(n)]
    m = rng.randint(-10, 10)
    return rows((n, k, m), vals, vals2)


def gen_mccc4j4(rng, idx):
    if idx == 0:
        return rows((6, 3), (1, 2, 3), (4, 5, 6), (1, 2, "W"), (2, 3, "T"), (1, 3, "L"),
                    (4, 5, "T"), (5, 6, "T"), (4, 6, "T"), (2,))
    g = rng.randint(1, 4) if idx < 18 else rng.randint(3, 8)
    m = rng.randint(2, 5) if idx < 18 else rng.randint(4, 8)
    n = g * m
    teams = list(range(1, n + 1))
    rng.shuffle(teams)
    groups = [teams[i * m : (i + 1) * m] for i in range(g)]
    matches = []
    for grp in groups:
        for i in range(m):
            for j in range(i + 1, m):
                u, v = grp[i], grp[j]
                if rng.random() < 0.5:
                    u, v = v, u
                matches.append((u, v, rng.choice(["W", "T", "L"])))
    rng.shuffle(matches)
    k = rng.randint(1, m)
    return rows((n, m), *groups, *matches, (k,))


def gen_mccc3j4(rng, idx):
    if idx == 0:
        return rows((3, 4), [2, 3, 1])
    if idx == 1:
        return rows((4, 1), [2, 1, 4, 3])
    n = rng.randint(2, 12) if idx < 18 else rng.randint(15, 45)
    t = rng.randint(1, 15) if idx < 18 else rng.randint(20, 60)
    init = [rng.randint(1, n) for _ in range(n)]
    for i in range(n):
        if init[i] == i + 1:
            init[i] = (i + 1) % n + 1
    return rows((n, t), init)


def gen_mccc3j5(rng, idx):
    if idx == 0:
        return rows((4, 3), (1, 2), (2, 3), (3, 4))
    if idx == 1:
        return rows((5, 0))
    n = rng.randint(1, 8) if idx < 18 else rng.randint(8, 12)
    m = rng.randint(0, min(25, n * (n - 1) // 2))
    all_edges = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    edges = rng.sample(all_edges, m) if m else []
    return rows((n, m), *edges)


def gen_mccc5j4(rng, idx):
    if idx == 0:
        return rows((10,), ("a*b*",))
    if idx == 1:
        return rows((100,), ("a*b",))
    if idx == 2:
        return rows((5,), ("***",))
    if idx == 3:
        return rows((6,), ("abc",))
    length = rng.randint(1, 15) if idx < 18 else rng.randint(20, 80)
    chars = []
    fixed_sum = 0
    wild = 0
    for _ in range(length):
        if rng.random() < 0.45:
            chars.append("*")
            wild += 1
        else:
            c = chr(ord("a") + rng.randint(0, 25))
            chars.append(c)
            fixed_sum += ord(c) - ord("a") + 1
    mode = rng.random()
    if mode < 0.75 and wild > 0:
        k = fixed_sum + rng.randint(wild, 26 * wild)
    elif mode < 0.88:
        k = fixed_sum + max(0, wild - 1)
    else:
        k = fixed_sum + 26 * wild + rng.randint(1, 5)
    return rows((k,), ("".join(chars),))


def gen_mccc3j3(rng, idx):
    if idx == 0:
        return rows((4, 3), ("ABAC",), ("BBC",))
    if idx == 1:
        return rows((3, 3), ("AAA",), ("AAA",))
    n = rng.randint(1, 20) if idx < 18 else rng.randint(30, 150)
    m = rng.randint(1, 20) if idx < 18 else rng.randint(30, 150)
    alpha = "ABCDEF" if idx < 10 else "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    a = "".join(rng.choice(alpha) for _ in range(n))
    b = "".join(rng.choice(alpha) for _ in range(m))
    return rows((n, m), (a,), (b,))


def gen_mccc4j3(rng, idx):
    if idx == 0:
        return rows((4,), [1, 2, 3, 0])
    if idx == 1:
        return rows((3,), [1, 1, 1])
    if idx == 2:
        return rows((5,), [0, 0, 0, 0, 0])
    n = rng.randint(1, 20) if idx < 18 else rng.randint(30, 150)
    a = [rng.randint(0, 20) for _ in range(n)]
    if rng.random() < 0.55 and n >= 2:
        split = rng.randint(1, n - 1)
        s1 = sum(a[:split])
        s2 = sum(a[split:])
        if s1 > s2:
            a[split] += s1 - s2
        elif s2 > s1:
            a[0] += s2 - s1
    return rows((n,), a)


GENS = {
    "mccc6s3": gen_mccc6s3,
    "occ19g4": gen_occ19g4,
    "mccc5j5": gen_mccc5j5,
    "mccc5j3": gen_mccc5j3,
    "mccc4j4": gen_mccc4j4,
    "mccc3j4": gen_mccc3j4,
    "mccc3j5": gen_mccc3j5,
    "mccc5j4": gen_mccc5j4,
    "mccc3j3": gen_mccc3j3,
    "mccc4j3": gen_mccc4j3,
}


def run_bin(cmd, inp):
    res = subprocess.run(cmd, input=inp, text=True, capture_output=True, timeout=30)
    if res.returncode != 0:
        raise RuntimeError(f"command {cmd} exited {res.returncode}:\n{res.stderr}")
    return res.stdout.strip().split()


def check_one(prob, cases, tmpdir):
    orig_cpp = os.path.join(ROOT, "corpus", "submissions", f"{ORIG[prob]}.cpp")
    orig_bin = os.path.join(tmpdir, f"{prob}_orig")
    subprocess.run(
        ["g++", "-O2", "-std=c++17", orig_cpp, "-o", orig_bin],
        check=True,
        capture_output=True,
        text=True,
    )
    port_cp = os.path.join(HERE, "today", f"{prob}.cp")
    rng = random.Random(4200 + sum(ord(c) for c in prob))
    gen = GENS[prob]
    odd_diff_notes = 0

    for idx in range(cases):
        inp = gen(rng, idx)
        got = run_bin(["python3", CP, "lib-run", port_cp], inp)
        exp = run_bin([orig_bin], inp)
        if prob == "mccc6s3":
            toks = list(map(int, inp.split()))
            arr = toks[1:]
            b_diff, b_i1, b_i2 = brute_mccc6s3(arr)
            g_i1, g_i2 = int(got[0]), int(got[1])
            e_i1, e_i2 = int(exp[0]), int(exp[1])
            if (g_i1, g_i2) != (b_i1, b_i2):
                raise AssertionError(
                    f"{prob} case {idx} port != brute: got={(g_i1, g_i2)} brute={(b_i1, b_i2)} orig={(e_i1, e_i2)}\n{inp}"
                )
            if eval_mccc6s3(arr, g_i1, g_i2) != eval_mccc6s3(arr, e_i1, e_i2):
                raise AssertionError(
                    f"{prob} case {idx} imbalance mismatch: port={(g_i1, g_i2)} orig={(e_i1, e_i2)}\n{inp}"
                )
            if (g_i1, g_i2) != (e_i1, e_i2):
                odd_diff_notes += 1
        elif prob == "occ19g4":
            toks = list(map(int, inp.split()))
            n = toks[0]
            pts = [(toks[1 + 2 * i], toks[2 + 2 * i]) for i in range(2 * n)]
            brute = list(map(str, brute_occ19g4(pts)))
            if got != exp or got != brute:
                raise AssertionError(
                    f"{prob} case {idx} mismatch:\ngot={got}\norig={exp}\nbrute={brute}\n{inp}"
                )
        else:
            if got != exp:
                raise AssertionError(
                    f"{prob} case {idx} mismatch:\ngot={got}\nexp={exp}\ninput:\n{inp}"
                )

    extra_msg = ""
    if prob == "mccc6s3":
        # Also test max(a) < n (where 6103125.cpp overflows next_ind[m]) against brute force.
        for j in range(10):
            n = rng.randint(5, 16)
            arr = [rng.randint(1, 3) for _ in range(n)]
            inp = rows((n,), arr)
            got = run_bin(["python3", CP, "lib-run", port_cp], inp)
            b_diff, b_i1, b_i2 = brute_mccc6s3(arr)
            assert (int(got[0]), int(got[1])) == (b_i1, b_i2), f"mccc6s3 small-val case {j} failed"
        extra_msg = f" (+10 brute-only small-val cases; orig odd-diff tie misses: {odd_diff_notes})"
    elif prob == "occ19g4":
        # Also test negative/huge coords + coincident pairs (where 3578469.cpp infinite-loops) vs brute.
        for j in range(10):
            n = rng.randint(2, 12)
            pts = []
            for _ in range(n):
                if j == 0:
                    x1, y1 = rng.randint(-10**8, 10**8), rng.randint(-10**8, 10**8)
                    pts.append((x1, y1))
                    pts.append((x1, y1))
                else:
                    pts.append((rng.randint(-10**8, 10**8), rng.randint(-10**8, 10**8)))
                    pts.append((rng.randint(-10**8, 10**8), rng.randint(-10**8, 10**8)))
            inp = rows((n,), *pts)
            got = run_bin(["python3", CP, "lib-run", port_cp], inp)
            brute = list(map(str, brute_occ19g4(pts)))
            assert got == brute, f"occ19g4 extreme case {j} failed: got={got} brute={brute}"
        extra_msg = " (+10 brute-only extreme/negative-coord cases)"

    print(f"PASS {prob:10s} ({cases}/{cases} vs {ORIG[prob]}.cpp{extra_msg})")


def main():
    resource.setrlimit(resource.RLIMIT_STACK, (resource.RLIM_INFINITY, resource.RLIM_INFINITY))
    args = sys.argv[1:]
    cases = 30
    if args and args[-1].isdigit():
        cases = int(args.pop())
    probs = args if args else list(ORIG.keys())
    with tempfile.TemporaryDirectory() as tmpdir:
        for p in probs:
            check_one(p, cases, tmpdir)
    print(f"\nAll {len(probs)} port(s) passed ({len(probs) * cases} differential cases + brute edge suites).")


if __name__ == "__main__":
    main()
