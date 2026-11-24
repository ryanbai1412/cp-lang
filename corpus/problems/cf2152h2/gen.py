#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at n = q = 250000 (both sums): a path with labels shuffled, edge weights up to 1e9, so
    # one component grows across the whole tree and its breakpoint heap holds every vertex; queries
    # spread over 1..1e9 with the extremes
    rng = random.Random(0)
    n = q = 250_000
    lab = list(range(1, n + 1)); rng.shuffle(lab)
    out = ["1", str(n)]
    out += [f"{lab[i]} {lab[i + 1]} {rng.randint(1, 10**9)}" for i in range(n - 1)]
    out.append(str(q))
    out += ["1", str(10**9)] + [str(rng.randint(1, 10**9)) for _ in range(q - 2)]
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(2, max(3, size * 4))
    print(n)
    for v in range(2, n + 1):
        u = rng.randint(1, v - 1)
        w = rng.randint(1, 20)
        print(u, v, w)
    q = rng.randint(1, max(2, size * 3))
    print(q)
    for _ in range(q):
        print(rng.randint(1, 40))
