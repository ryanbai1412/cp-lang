#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 500 strings of length 100, with 0, 1 or many Ys.
    rng = random.Random(2178)
    print(500)
    for i in range(500):
        s = ["N"] * 100
        for j in rng.sample(range(100), [0, 1, rng.randint(2, 100)][i % 3]):
            s[j] = "Y"
        print("".join(s))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(2, 15)
    print("".join(rng.choice("YN") for _ in range(n)))
