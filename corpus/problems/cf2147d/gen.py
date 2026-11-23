#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # One test at the limit (the sum of n over tests is at most 2e5): n = 2e5, values in
    # 1..1e5 so values repeat and both the even base and the odd extras matter.
    r = random.Random(0)
    print(1); print(200000); print(*[r.randint(1, 100000) for _ in range(200000)])
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 4)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 50)
    print(n)
    print(*[r.randint(1, 15 if size == 0 else 1000) for _ in range(n)])
