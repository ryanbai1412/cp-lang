#!/usr/bin/env python3
# apio09p2: n requests [s, e], 1 <= n <= 2e5, 1 <= s <= e <= 1e9.
# --max: n = 2e5 over all of 1..1e9: 1e5 disjoint short requests in shuffled order, each overlapped by
# a longer request that blocks it, so the answer is large and every pick splits a free window.
import random, sys
if sys.argv[1] == "--max":
    r, n = random.Random(0), 2 * 10**5
    step = 10**9 // (n // 2)
    reqs = []
    for i in range(n // 2):
        s = i * step + 1
        reqs.append((s + 1, s + step // 2))
        reqs.append((s + step // 4, min(10**9, s + step + step // 4)))
    r.shuffle(reqs)
    print(n)
    print("\n".join(f"{s} {e}" for s, e in reqs))
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2]) if len(sys.argv) > 2 else 12
n = r.randint(1, S)
hi = r.choice([3, 10, 3 * n, 10**9])
lim = r.choice([1, 3, max(1, hi // 4), hi])
print(n)
for _ in range(n):
    s = r.randint(1, hi)
    e = min(hi, s + r.randint(0, lim - 1))
    print(s, e)
