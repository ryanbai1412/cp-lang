#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # sum n^2 = 5000^2 - 1 with n odd: 4999^2 + 99^2 + 13^2 + 5^2 + 4 * 1^2; values cluster
    # around the median so many odd segments qualify and the O(n^2) dp takes every branch
    g = random.Random(1)
    print(8)
    for n in (4999, 99, 13, 5, 1, 1, 1, 1):
        mid = 5 * 10**8
        a = [mid if g.random() < 0.4 else g.choice((mid - g.randint(1, 3), mid + g.randint(1, 3),
             g.randint(1, 10**9))) for _ in range(n)]
        print(n)
        print(*a)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.choice([1, 3, 5, 7, 9, 11])
    print(n)
    print(*[rng.randint(1, 5) for _ in range(n)])
