#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = q = 1e5: values below 1e9 with the high bits mostly set and a
    # few holes, budgets across 0..1e9.
    r = random.Random(2147)
    n = q = 100_000
    print(1)
    print(n, q)
    print(*[r.randint(0, 10**9) & ~(1 << r.randint(0, 29)) for _ in range(n)])
    print("\n".join(str(r.choice([0, 1, 10**9, r.randint(0, 10**9), r.randint(0, 2**r.randint(0, 29))]))
                    for _ in range(q)))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 8 if size == 0 else 30)
    q = r.randint(1, 8 if size == 0 else 30)
    print(n, q)
    print(*[r.randint(0, 1000 if size == 0 else 10**9) for _ in range(n)])
    for _ in range(q):
        print(r.randint(0, 1000 if size == 0 else 10**9))
