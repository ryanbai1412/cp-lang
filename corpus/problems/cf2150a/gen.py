#!/usr/bin/env python3
import random, sys
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 4)
print(T)
for _ in range(T):
    n = r.randint(1, 10 if size == 0 else 40)
    m = r.randint(1, 8 if size == 0 else 30)
    s = "".join(r.choice("AB") for _ in range(n))
    a = sorted(r.sample(range(1, m * 3 + 5), m))
    print(n, m)
    print(s)
    print(*a)
