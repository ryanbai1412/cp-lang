#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test at both sums, n = m = 1e5: mostly B commands and a long consecutive run of black
    # cells right ahead of the start, then scattered cells up to 1e9.
    r = random.Random(2150)
    n = m = 100_000
    s = "".join("B" if r.random() < 0.7 else "A" for _ in range(n))
    run = list(range(2, 2 + m // 2))
    rest = sorted(r.sample(range(2 + m // 2 + 1, 10**9 + 1), m - len(run)))
    print(1)
    print(n, m)
    print(s)
    print(*(run + rest))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 4)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 40)
    m = r.randint(1, 8 if size == 0 else 30)
    s = "".join(r.choice("AB") for _ in range(n))
    a = sorted(r.sample(range(1, m * 3 + 5), m))
    print(n, m)
    print(s)
    print(*a)
