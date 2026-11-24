#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # m = n - 2 extra edges: a random laminar family (binary splits of 1..n), shuffled
    rng = random.Random(1)
    n = 200_000
    st, edges = [(1, n)], []
    while st:
        lo, hi = st.pop()
        if hi - lo < 2:
            continue
        edges.append((lo, hi))
        mid = rng.randint(lo + 1, hi - 1)
        st += [(lo, mid), (mid, hi)]
    rng.shuffle(edges)
    print(1)
    print(n, len(edges))
    print("\n".join(f"{u} {v}" for u, v in edges))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, 14)
    edges = set()
    m = rng.randint(0, n)
    for _ in range(m):
        u = rng.randint(1, n - 1)
        v = rng.randint(u + 1, n)
        edges.add((u, v))
    print(n, len(edges))
    for u, v in sorted(edges):
        print(u, v)
