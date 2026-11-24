#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    r = random.Random(1)
    n, q = 200_000, 100_000
    print(1)
    print(n, q)
    print(*[r.randint(1, 50) for _ in range(n)])
    print("\n".join(f"{r.randint(1, n)} {r.randint(1, 50) if i % 2 else r.randint(1, n)}" for i in range(q)))
    sys.exit()

r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 8 if size == 0 else 40)
    q = r.randint(1, 8 if size == 0 else 40)
    print(n, q)
    print(*[r.randint(1, n) for _ in range(n)])
    for _ in range(q):
        print(r.randint(1, n), r.randint(1, n))
