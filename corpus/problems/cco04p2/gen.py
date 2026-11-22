#!/usr/bin/env python3
# One random CCO 2004 P2 input: gen.py <seed> [size]. The typed generator
# cannot make `a-b` words (backlog item S8), so this stands in for it.
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 6
tests = rng.randint(1, 3)
print(tests)
for _ in range(tests):
    n = rng.randint(1, size)
    print(n)
    print(" ".join(f"{rng.randint(1, size)}-{rng.randint(1, size)}" for _ in range(n)))
