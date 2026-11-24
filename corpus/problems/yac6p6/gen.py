"""Yet Another Contest 6 P6 (No More Snakes and Ladders): `N M Q` (N, M <= 200, Q <= 1e5), the
grid G (values 1..=1e9), then Q queries `A B C D K` with A <= C, B <= D, (A, B) != (C, D) and
1 <= K <= 5.

`gen.py SEED SIZE`: N x M about SIZE cells, Q up to SIZE. `gen.py --max`: N = M = Q = max, K = 5,
starts near the top-left and ends spread over the whole grid (the most distinct pivots, each
with a large region for its backward DP), plus some queries of sides <= 4 for the direct DP.
"""
import random
import sys

big = sys.argv[1:] == ["--max"]
g = random.Random(1 if big else int(sys.argv[1]))
if big:
    n, m, q, hi = 200, 200, 100_000, 10**9
else:
    size = max(2, int(sys.argv[2]))
    n = g.randint(1, min(200, max(1, int(size**0.5) + 1)))
    m = g.randint(1 if n > 1 else 2, min(200, max(2, size // n)))
    q, hi = g.randint(1, size), g.choice([9, 10**9])
out = [f"{n} {m} {q}"] + [" ".join(str(g.randint(1, hi)) for _ in range(m)) for _ in range(n)]
for _ in range(q):
    while True:
        if big and g.random() < 0.9:
            a, b = g.randint(1, 5), g.randint(1, 5)
            c, d = g.randint(a, n), g.randint(b, m)
        elif g.random() < 0.5:
            a, b = g.randint(1, n), g.randint(1, m)
            c, d = g.randint(a, min(n, a + 3)), g.randint(b, min(m, b + 3))
        else:
            a, c = sorted((g.randint(1, n), g.randint(1, n)))
            b, d = sorted((g.randint(1, m), g.randint(1, m)))
        if (a, b) != (c, d):
            break
    out.append(f"{a} {b} {c} {d} {5 if big else g.randint(1, 5)}")
print("\n".join(out))
