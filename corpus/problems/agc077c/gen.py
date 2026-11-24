#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test with N = M = 2e5 (both sums) and an odd K, the case that needs the parity
    # components; distinct pairs, never both ways
    r = random.Random(1)
    n = m = 200_000
    seen, out = set(), ["1", f"{n} {m} 99999"]
    while len(seen) < m:
        u, v = r.randint(1, n), r.randint(1, n)
        if u == v or (min(u, v), max(u, v)) in seen: continue
        seen.add((min(u, v), max(u, v))); out.append(f"{u} {v}")
    print("\n".join(out))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(4, 10 if size == 0 else 30)
    k = r.randint(2, n - 2)
    all_edges = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    r.shuffle(all_edges)
    m = r.randint(0, len(all_edges) if r.random() < 0.5 else min(len(all_edges), 2 * n))
    print(n, m, k)
    for u, v in all_edges[:m]:
        if r.random() < 0.5:
            u, v = v, u
        print(u, v)
