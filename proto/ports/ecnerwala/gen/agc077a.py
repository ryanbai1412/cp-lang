#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(2, 16)
    print(n)
    cb = rng.randint(0, n)
    x = ["B"] * cb + ["A"] * (n - cb)
    y = x[:]
    rng.shuffle(x)
    rng.shuffle(y)
    if rng.random() < 0.2:
        y[0] = "A" if y[0] == "B" else "B"
    print("".join(x))
    print("".join(y))
