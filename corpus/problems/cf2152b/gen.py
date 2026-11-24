#!/usr/bin/env python3
import random, sys
# gen.py seed size: t tests on grids with n <= 10^min(size, 9), 0 <= coordinates <= n,
# Krug and Doran on different cells. No arguments, or size >= 9, gives t = 1e4 tests with
# n up to 1e9 (half of them n = 1e9 with both on the border or corners).
# --max is the no-argument full mode.
max_mode = sys.argv[1:] == ["--max"]
seed = int(sys.argv[1]) if len(sys.argv) > 1 and not max_mode else 0
size = int(sys.argv[2]) if len(sys.argv) > 2 else 9
rng = random.Random(seed)
full = size >= 9
t = 10 ** 4 if full else rng.randint(1, 5)
print(t)
for _ in range(t):
    n = 10 ** 9 if full and rng.random() < 0.5 else rng.randint(1, 10 ** rng.randint(1, min(size, 9)))
    pick = lambda: rng.choice([0, n, rng.randint(0, n)])
    while True:
        rk, ck, rd, cd = pick(), pick(), pick(), pick()
        if (rk, ck) != (rd, cd): break
    print(n, rk, ck, rd, cd)
