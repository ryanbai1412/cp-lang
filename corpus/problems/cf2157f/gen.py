#!/usr/bin/env python3
import random, sys
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = rng.randint(2, max(4, size * 12))
print(n)
