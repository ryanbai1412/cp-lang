#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One string of the full 2e5: mostly isolated u with some runs of u to break.
    rng = random.Random(2178)
    out = []
    while len(out) < 200_000:
        out.append("s")
        out.append("u" * (1 if rng.random() < 0.8 else rng.randint(2, 4)))
    print(1)
    print("".join(out)[:200_000])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 8)
print(T)
for _ in range(T):
    n = rng.randint(3, 18)
    print("".join(rng.choice("su") for _ in range(n)))
