#!/usr/bin/env python3
import random, sys
if sys.argv[1] == "--max":
    # t = 100 tests of n = 10 divisors of 100, about half holding a 100
    g = random.Random(1)
    divs = [d for d in range(1, 101) if 100 % d == 0]
    print(100)
    for i in range(100):
        a = [g.choice(divs[:-1]) for _ in range(10)]
        if i % 2:
            a[g.randrange(10)] = 100
        print(10)
        print(*a)
    sys.exit()

rng = random.Random(int(sys.argv[1]))
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    n = rng.randint(1, 10)
    print(n)
    vals = [rng.choice([1, 50, 99, 100, 101]) for _ in range(n)]
    print(*vals)
