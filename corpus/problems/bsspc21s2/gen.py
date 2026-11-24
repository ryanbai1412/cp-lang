"""gen.py SEED SIZE | --max: BSSPC '21 S2. An N x N grid (N <= 1e9), K <= 5e3 clockwise rotations
of squares, Q <= 5e3 cell queries. --max is N = 1e9, K = Q = 5e3, every square holding the
grid's middle and every query in it, so each query is moved by every rotation."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 10**9 if MAX else r.randint(1, min(10**9, 1 + S if r.random() < 0.7 else 10**9))
k = 5_000 if MAX else r.randint(1, min(5_000, 1 + S))
q = 5_000 if MAX else r.randint(1, min(5_000, 1 + S))
out = [f"{n} {k} {q}"]
mid = (n + 1) // 2
for _ in range(k):
    if MAX:
        t, l = r.randint(1, mid - 1000), r.randint(1, mid - 1000)
        s = r.randint(max(mid + 1000 - t, mid + 1000 - l), n - max(t, l))
    else:
        s = r.randint(0, n - 1)
        t, l = r.randint(1, n - s), r.randint(1, n - s)
    out.append(f"{t} {l} {t + s} {l + s}")
for _ in range(q):
    if MAX:
        out.append(f"{r.randint(mid - 999, mid + 999)} {r.randint(mid - 999, mid + 999)}")
    else:
        out.append(f"{r.randint(1, n)} {r.randint(1, n)}")
print("\n".join(out))
