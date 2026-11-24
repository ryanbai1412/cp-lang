#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    r = random.Random(1)
    t, top = 10**4, 2**30 - 1
    print(t)
    edge = [(top, top), (0, 0), (top, 0), (0, top), (2**29, 2**29), (2**29 - 1, 2**29)]
    for i in range(t):
        print(*(edge[i] if i < len(edge) else (r.randint(0, top), r.randint(0, top))))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    print(rng.randint(0, 100000), rng.randint(0, 100000))
