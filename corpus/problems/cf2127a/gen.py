#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 500 tests of n = 100: half one value with gaps (YES), half random (mostly NO).
    r = random.Random(2127)
    print(500)
    for i in range(500):
        print(100)
        if i % 2 == 0:
            v = r.randint(1, 100)
            print(*[-1 if r.random() < 0.5 else v for _ in range(100)])
        else:
            print(*[r.randint(-1, 100) for _ in range(100)])
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 5)
print(T)
for _ in range(T):
    n = r.randint(3, 10 if size == 0 else 40)
    print(n)
    if r.random() < 0.5:
        v = r.randint(1, 10)
        print(*[-1 if r.random() < 0.5 else v for _ in range(n)])
    else:
        print(*[r.randint(-1, 5) for _ in range(n)])
