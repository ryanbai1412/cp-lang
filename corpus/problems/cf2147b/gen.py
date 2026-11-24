#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # sum n = 2e5 in one test of the largest n.
    print(1)
    print(200_000)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    print(r.randint(1, 8 if size == 0 else 40))
