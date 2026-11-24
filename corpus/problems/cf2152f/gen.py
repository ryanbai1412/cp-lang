#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = q = 2.5e5: steps of 1..3 with z = 2, so the jump tree is a
    # chain about n / 2 deep, and wide queries.
    rng = random.Random(2152)
    n = q = 250_000
    xs, cur = [], 1
    for _ in range(n):
        xs.append(cur)
        cur += rng.randint(0, 3) if rng.random() < 0.1 else 1
    out = ["1", f"{n} 2", " ".join(map(str, xs)), str(q)]
    for _ in range(q):
        l = rng.randint(1, n // 10)
        out.append(f"{l} {rng.randint(n - n // 10, n)}" if rng.random() < 0.7 else f"{l} {rng.randint(l, n)}")
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(1, max(4, size * 5))
    z = rng.randint(1, 20)
    xs = []
    cur = rng.randint(0, 5)
    for _ in range(n):
        cur += rng.randint(1, 10)
        xs.append(cur)
    q = rng.randint(1, max(3, size * 4))
    print(n, z)
    print(*xs)
    print(q)
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        print(l, r)
