#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # one test of n = 2e5 series near 1e18, differences odd, even, powers of two and huge
    r = random.Random(1)
    n, top = 200_000, 10**18
    print(1)
    print(n)
    out = []
    for i in range(n):
        d = [1, 2, 3, 2 ** r.randint(1, 57), r.randint(1, 10**6), r.randint(1, 3 * 10**17)][i % 6]
        a = r.randint(1, top // 4)
        c = a + 2 * d
        l = r.randint(c, top)
        out.append(f"{a} {a + d} {c} {l} {r.randint(l, top)}")
    print("\n".join(out))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 6)
    print(n)
    for _ in range(n):
        a = rng.randint(1, 100)
        d = rng.randint(1, 50)
        b = a + d
        c = b + d
        l = c + rng.randint(0, 200)
        r = l + rng.randint(0, 200)
        print(a, b, c, l, r)
