#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(2, 8 if size == 0 else 25)
    # strictly increasing positions in 1..=3*n (with enough room on left/right so 0 <= pos < 4*n)
    a = sorted(r.sample(range(3, min(3 * n + 4, 4 * n + 1)), n))
    t = [r.randint(0, 2) for _ in range(n)]
    print(n)
    print(*a)
    print(*t)
