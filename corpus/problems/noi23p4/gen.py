"""NOI 2023 P4 (Trade): `n m`, a_2..a_{2^n - 1} (city i's road to i / 2), then m roads `u v w`,
v a proper descendant of u; 2 <= n <= 18, m <= 2^n, lengths in 1..=1e9.

`gen.py SEED SIZE`: 2^n - 1 about SIZE cities, m up to 2^n. `gen.py --max`: n = 18, m = 2^18,
every road from a shallow city down to a leaf, so each subtree Dijkstra sees the most roads.
"""
import random
import sys

if sys.argv[1:] == ["--max"]:
    r = random.Random(1)
    n, deep, wmax = 18, True, 10**9
else:
    r = random.Random(int(sys.argv[1]))
    size = int(sys.argv[2])
    n = r.randint(2, max(2, min(18, size.bit_length())))
    deep, wmax = r.random() < 0.3, r.choice([10, 10**9])
cities = (1 << n) - 1
m = 1 << n if sys.argv[1:] == ["--max"] else r.randint(0, 1 << n)
out = [f"{n} {m}", " ".join(str(r.randint(1, wmax)) for _ in range(2, cities + 1))]
for _ in range(m):
    du = r.randint(0, min(3, n - 2)) if deep else r.randint(0, n - 2)
    u = r.randint(1 << du, (1 << (du + 1)) - 1)
    dv = n - 1 if deep else r.randint(du + 1, n - 1)
    v = u
    for _ in range(dv - du):
        v = 2 * v + r.randint(0, 1)
    out.append(f"{u} {v} {r.randint(1, wmax)}")
print("\n".join(out))
