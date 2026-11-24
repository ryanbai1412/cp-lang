#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test at n = 100 (sum n^2 <= 1e4), every score free: no fixed score prunes the DP
    print(1); print(100); print(*[-1] * 100)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 6 if size == 0 else 14)
    print(n)
    print(*[-1 if r.random() < 0.6 else r.randint(0, 3) for _ in range(n)])
