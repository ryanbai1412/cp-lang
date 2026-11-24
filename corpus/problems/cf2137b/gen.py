#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5, a random permutation.
    r = random.Random(2137)
    n = 200_000
    p = list(range(1, n + 1))
    r.shuffle(p)
    print(1)
    print(n)
    print(*p)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(2, 10 if size == 0 else 40)
    p = list(range(1, n + 1))
    r.shuffle(p)
    print(n)
    print(*p)
