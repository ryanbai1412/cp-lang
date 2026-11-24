import sys, random
# seed size: C = 1, T <= 4 tests of one k in 3..=5 (a test point has a single k) and n <= size (within the statement's
# n <= max(T / i, 2 log2 T) for test i), m <= 3n constraints that a hidden sequence keeps, q <= 4;
# --max: T = 600, k = 5, every n at its bound, m = 3n, sum q = 3e5, v up to 1e12.
import math
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
T = 600 if mx else r.randint(1, 4)
lim = lambda i: max(T // i, int(2 * math.log2(T))) if T > 1 else 1
ns = [lim(i) if mx else r.randint(1, max(1, min(S, lim(i)))) for i in range(1, T + 1)]
qs = [500] * T if mx else [r.randint(1, 4) for _ in range(T)]
out = [f"1 {T}"]
k = 5 if mx else r.randint(3, 5)
for i, n in enumerate(ns):
    x = [r.randint(1, k) for _ in range(n)]
    lr = []
    for v in x:
        lo, hi = r.randint(1, v), r.randint(v, k)
        if not mx and r.random() < 0.3: lo = hi = v
        lr.append((lo, hi))
    m = 3 * n if mx else r.randint(0, 3 * n)
    cons = []
    for _ in range(m):
        p, q = r.randint(1, n), r.randint(1, n)
        d = abs(x[p - 1] - x[q - 1])
        cons.append((p, q, r.randint(d, k - 1) if r.random() < 0.3 else d))
    cap = 10**12 if mx or r.random() < 0.5 else r.choice([0, 10**6, 10**8])
    out.append(f"{k} {n} {m} {qs[i]}")
    out += [f"{a} {b}" for a, b in lr]
    out += [f"{p} {q} {b}" for p, q, b in cons]
    out += [" ".join(str(r.randint(0, cap)) for _ in range(k - 2)) for _ in range(qs[i])]
print("\n".join(out))
