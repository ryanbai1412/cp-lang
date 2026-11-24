#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 1e4 triples near 1e6: primes, highly composite numbers and numbers whose prime factors
    # have smooth p - 1, so every factorisation is as long as it gets.
    r = random.Random(2147)
    special = [999983, 999979, 720720, 997920, 982800, 1000000, 524288, 531441, 65537, 999999,
               998244, 2, 3, 4]
    print(10_000)
    out = []
    for _ in range(10_000):
        out.append(" ".join(str(r.choice(special) if r.random() < 0.5 else r.randint(900_000, 10**6))
                            for _ in range(3)))
    print("\n".join(out))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 4)
print(T)
for _ in range(T):
    mx = 50 if size == 0 else 100000
    print(r.randint(1, mx), r.randint(1, mx), r.randint(1, mx))
