#!/usr/bin/env python3
import random
import sys

if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "2999"]
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
n = r.randrange(1, min(size, 2999) + 1, 2) if size < 2999 else 2999
print(n)
print(*(f"0.{r.randint(1, 99):02}" for _ in range(n)))
