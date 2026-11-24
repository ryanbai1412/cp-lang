#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":  # one case n = 1e5 (sum n), A dense (97%) so the mex runs are long
    rng = random.Random(0); n = 10**5
    pre = [0]
    for _ in range(n): pre.append(pre[-1] + (rng.random() < 0.97))
    a = []
    for k in range(1, n + 1):
        j = 0
        while pre[min(j * k + k, n)] - pre[min(j * k, n)] > 0: j += 1
        a.append(j)
    print(1); print(n); print(*a); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); print(n)
    p = r.random()
    A = [y for y in range(n) if r.random() < p]
    a = []
    for k in range(1, n + 1):
        seen = {y // k for y in A}
        m = 0
        while m in seen: m += 1
        a.append(m)
    print(*a)
