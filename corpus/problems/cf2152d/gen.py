#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = q = 2.5e5: values across 2..1e9 (mostly large), wide ranges.
    rng = random.Random(2152)
    n = q = 250_000
    out = ["1", f"{n} {q}", " ".join(str(rng.randint(2, 10**9) if rng.random() < 0.8 else rng.randint(2, 1000)) for _ in range(n))]
    for _ in range(q):
        l = rng.randint(1, n)
        out.append(f"{l} {rng.randint(l, n)}" if rng.random() < 0.5 else f"{rng.randint(1, 100)} {rng.randint(n - 100, n)}")
    print("\n".join(out))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
t = rng.randint(1, 4)
print(t)
for _ in range(t):
    n = rng.randint(1, max(4, size * 5))
    q = rng.randint(1, max(3, size * 4))
    print(n, q)
    print(*[rng.randint(2, 10 ** rng.randint(1, 7)) for _ in range(n)])
    for _ in range(q):
        l = rng.randint(1, n)
        r = rng.randint(l, n)
        print(l, r)
