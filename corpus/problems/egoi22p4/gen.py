#!/usr/bin/env python3
"""gen.py seed size | --max: EGOI 2022 Tourists. n, m, q <= 2e5, a tree, ops "t l r c" (move
tourists l..r to c), "e c d" (d <= 1e9), "q v". --max: n = m = q = 2e5 on a long caterpillar
(deep paths), wide overlapping moves that keep splitting intervals, events and queries mixed."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 200_000 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(1, max(1, S))
m = 200_000 if MAX else r.randint(1, max(1, S))
q = 200_000 if MAX else r.randint(1, max(1, S))
out = [f"{n} {m} {q}", " ".join(str(r.randint(1, n)) for _ in range(m))]
for v in range(2, n + 1):
    out.append(f"{max(1, v - r.randint(1, 2)) if MAX or r.random() < 0.5 else r.randint(1, v - 1)} {v}")
D = 10 ** 9 if MAX or r.random() < 0.5 else 10
for _ in range(q):
    k = r.random()
    if k < 0.4:
        a = r.randint(1, m); b = r.randint(a, min(m, a + r.choice([1, 10, m])))
        out.append(f"t {a} {b} {r.randint(1, n)}")
    elif k < 0.7:
        out.append(f"e {r.randint(1, n)} {r.randint(1, D)}")
    else:
        out.append(f"q {r.randint(1, m)}")
print("\n".join(out))
