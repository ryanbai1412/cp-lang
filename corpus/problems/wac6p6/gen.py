#!/usr/bin/env python3
"""gen.py SEED [SIZE] | --max: WAC 6 P6 Santa's Candy Factory (easy). The interactor's case: T
factories, each its counts A B C (A + B + C = 100) and the 100 canes in arrival order.
--max: T = 100 factories, mixing balanced and skewed counts."""
import random, sys
MAX = sys.argv[1:] == ["--max"]
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 100 if MAX else min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 100)
t = 100 if MAX else r.randint(1, size)
out = [str(t)]
for _ in range(t):
    x, y = sorted(r.randint(0, 100) for _ in range(2))
    counts = [x, y - x, 100 - y]
    canes = [ch for ch, k in zip("RGB", counts) for _ in range(k)]
    r.shuffle(canes)
    out += [" ".join(map(str, counts)), "".join(canes)]
print("\n".join(out))
