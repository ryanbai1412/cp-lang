#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # sum n^2 <= 9e6: one test of n = 3000 distinct pairs in 1..=6000, many overlapping and
    # sharing endpoints (lots of cycles).
    r = random.Random(2129)
    n = 3000
    pairs = set()
    while len(pairs) < n:
        a = r.randint(1, 2 * n - 1) if r.random() < 0.5 else r.randint(1, 200)
        b = r.randint(a + 1, min(2 * n, a + r.choice([3, 50, 2 * n])))
        pairs.add((a, b))
    pairs = list(pairs)
    r.shuffle(pairs)
    print(1)
    print(n)
    print("\n".join(f"{a} {b}" for a, b in pairs))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(1, 8 if size == 0 else 30)
    print(n)
    for _ in range(n):
        a = r.randint(1, 2 * n - 1)
        b = r.randint(a + 1, 2 * n)
        print(a, b)
