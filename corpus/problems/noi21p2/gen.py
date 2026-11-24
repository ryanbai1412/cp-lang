#!/usr/bin/env python3
"""gen.py seed size | --max: NOI 2021 Path Intersections. T <= 5, k <= 100 layers, n_1 = n_k
<= 100, n_1 <= n_i <= 2 n_1, m_j <= n_j n_{j+1} distinct edges between consecutive layers.
--max: T = 5 tests of k = 100 with n_1 = 100 and inner layers of 200: the first complete
(every m_j at its bound), the rest 90% dense so the determinant is not trivially 0."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else max(2, int(sys.argv[2]))
out = []
T = 5 if MAX else r.randint(1, 3)
out.append(str(T))
for t in range(T):
    k = 100 if MAX else r.randint(2, max(2, min(100, S // 5)))
    n1 = 100 if MAX else r.randint(1, max(1, min(100, S // 10)))
    size = [n1] + [2 * n1 if MAX else r.randint(n1, 2 * n1) for _ in range(k - 2)] + [n1]
    dense = 1.0 if MAX and t == 0 else 0.9 if MAX else r.random()
    gaps = []
    for j in range(k - 1):
        a, b = size[j], size[j + 1]
        if dense == 1.0:
            gaps.append([f"{u} {v}" for u in range(1, a + 1) for v in range(1, b + 1)])
        else:
            gaps.append([f"{u} {v}" for u in range(1, a + 1) for v in range(1, b + 1) if r.random() < dense])
    out += [str(k), " ".join(map(str, size)), " ".join(str(len(g)) for g in gaps)]
    for g in gaps: out += g
sys.stdout.write("\n".join(out) + "\n")
