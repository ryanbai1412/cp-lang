#!/usr/bin/env python3
import math, random, sys
MOD = 10**9 + 7
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, 12)
    bits = 10
    cnts = [rng.randint(0, n) for _ in range(bits)]
    B = []
    for i in range(1, n + 1):
        val = 0
        for b in range(bits):
            if cnts[b] >= i:
                val = (val + (1 << b) * math.comb(cnts[b], i)) % MOD
        B.append(val)
    print(n)
    print(*B)
