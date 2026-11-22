#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, 14)
    print(n)
    xs = rng.sample(range(-30, 31), n)
    pin = rng.random() < 0.5
    for x in xs:
        c = rng.choice("AB?") if pin else "?"
        print(rng.randint(1, 20), x, c)
