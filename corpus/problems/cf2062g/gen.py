#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test with n = 100 (sum n^3 = 1e6), random p and q
    r = random.Random(1)
    p, q = list(range(1, 101)), list(range(1, 101))
    r.shuffle(p); r.shuffle(q)
    print(1); print(100); print(*p); print(*q)
    sys.exit()

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
T = 5 if S == 0 else 4
print(T)
for _ in range(T):
    n = r.randint(2, 7 if S == 0 else min(35, S + 8))
    p = list(range(1, n + 1))
    q = list(range(1, n + 1))
    r.shuffle(p)
    r.shuffle(q)
    print(n)
    print(*p)
    print(*q)
