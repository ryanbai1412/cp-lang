#!/usr/bin/env python3
import sys, random
if sys.argv[1] == "--max":
    # one test at the bound, then the most tests the sums of n and m allow
    t = 10**4
    big = 10**6 - 3 * (t - 1)
    print(t)
    print(big, big)
    print("\n".join(["3 3"] * (t - 1)))
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = r.randint(1, 6); print(t)
for _ in range(t):
    hi = r.choice([3 + S, 50, 1000, 10**6])
    print(r.randint(3, hi), r.randint(3, hi))
