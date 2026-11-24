#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # t = 400 tests at the largest k and x.
    print(400)
    for i in range(400):
        print(20 - i % 20, 20 - (i // 20) % 20)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 10)
print(T)
for _ in range(T):
    print(r.randint(1, 10 if size == 0 else 20), r.randint(1, 20))
