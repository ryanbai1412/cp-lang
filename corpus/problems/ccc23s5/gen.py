#!/usr/bin/env python3
# gen.py SEED SIZE | --max: one n in 2..=min(SIZE, 1e9), favouring powers of 3 and their
# neighbours; --max is n = 3^18 (2^19 kept numerators).
import random, sys
if sys.argv[1] == "--max":
    print(3 ** 18); sys.exit()
r = random.Random(int(sys.argv[1]))
hi = max(2, min(int(sys.argv[2]), 10**9))
pows = [3**k * m for k in range(19) for m in (1, 2, 4) if 2 <= 3**k * m <= hi]
choice = r.random()
if choice < 0.4:
    n = r.randint(2, hi)
elif choice < 0.9:
    n = min(hi, max(2, r.choice(pows) + r.choice([-1, 0, 0, 1])))
else:
    n = hi
print(n)
