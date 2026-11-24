#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":
    # one test with n = q = 3e5 (both sums), b from a real run on values up to 1e9 so an answer
    # exists and every version node is fixed
    rng = random.Random(1)
    n = q = 300_000
    b = [rng.randint(1, 10**9) for _ in range(n)]
    ops = []
    for _ in range(q):
        x, y, z = rng.randint(1, n), rng.randint(1, n), rng.randint(1, n)
        ops.append(f"{x} {y} {z}")
        b[z - 1] = min(b[x - 1], b[y - 1])
    print(1); print(n, q); print(*b); print("\n".join(ops))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, 10)
    q = rng.randint(1, 12)
    a = [rng.randint(1, 15) for _ in range(n)]
    b = list(a)
    ops = []
    for _ in range(q):
        x = rng.randint(1, n)
        y = rng.randint(1, n)
        z = rng.randint(1, n)
        ops.append((x, y, z))
        b[z - 1] = min(b[x - 1], b[y - 1])
    if rng.random() < 0.35:
        b[rng.randrange(n)] = rng.randint(1, 15)
    print(n, q)
    print(*b)
    for x, y, z in ops:
        print(x, y, z)
