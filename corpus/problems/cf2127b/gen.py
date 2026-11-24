#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5, Hamid in the middle of an almost empty row.
    r = random.Random(2127)
    n = 200_000
    s = ["#" if r.random() < 0.01 else "." for _ in range(n)]
    s[n // 2 - 1] = "."
    print(1)
    print(n, n // 2)
    print("".join(s))
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(2, 12 if size == 0 else 50)
    x = r.randint(1, n)
    s = [r.choice(".#") for _ in range(n)]
    s[x - 1] = "."
    print(n, x)
    print("".join(s))
