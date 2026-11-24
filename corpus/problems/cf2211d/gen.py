#!/usr/bin/env python3
import math, random, sys
if sys.argv[1] == "--max":
    # one test at sum n = 1e5 from a valid a: every one of the 29 bits set in a random count
    # of elements (one in all n), so every b_k is a full sum of binomials
    MOD, g = 10**9 + 7, random.Random(1)
    n = 100_000
    fact = [1] * (n + 1)
    for i in range(1, n + 1):
        fact[i] = fact[i - 1] * i % MOD
    inv = [1] * (n + 1)
    inv[n] = pow(fact[n], MOD - 2, MOD)
    for i in range(n, 0, -1):
        inv[i - 1] = inv[i] * i % MOD
    cnts = [n] + [g.randint(n // 2, n) for _ in range(28)]
    b = [0] * (n + 1)
    for bit, c in enumerate(cnts):
        w = (1 << bit) * fact[c] % MOD
        for i in range(1, c + 1):
            b[i] = (b[i] + w * inv[i] % MOD * inv[c - i]) % MOD
    print(1)
    print(n)
    print(*b[1:])
    sys.exit()

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
