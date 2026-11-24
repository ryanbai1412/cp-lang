#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 1e4 pairs with a b near 1e18: balanced, lopsided, powers of two and odd products.
    r = random.Random(2137)
    print(10_000)
    for i in range(10_000):
        kind = i % 4
        if kind == 0:
            a = r.randint(1, 10**9); b = 10**18 // a
        elif kind == 1:
            a = r.choice([1, 2, 3, 4]); b = 10**18 // a
        elif kind == 2:
            e = r.randint(0, 59); a = 1 << e; b = 1 << r.randint(0, 59 - e)
        else:
            a = r.randrange(1, 10**9, 2); b = (10**18 // a) | 1
            if a * b > 10**18: b -= 2
        print(a, max(b, 1))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 10)
print(T)
for _ in range(T):
    mx = 50 if size == 0 else 10**8
    print(r.randint(1, mx), r.randint(1, mx))
