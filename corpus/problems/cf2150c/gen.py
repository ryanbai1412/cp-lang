#!/usr/bin/env python3
import random, sys

if sys.argv[1:] == ["--max"]:
    # One test with n = 2e5, values at +-1e9 alternating in runs (the increment map keeps
    # growing and getting cancelled), Bob's order a locally shuffled reverse of Alice's.
    r = random.Random(2150)
    n = 200_000
    vals = [(10**9 - r.randint(0, 9)) * (1 if (i // r.randint(1, 4)) % 2 else -1) for i in range(n)]
    a = list(range(1, n + 1))
    r.shuffle(a)
    b = a[::-1]
    for i in range(0, n - 1, 2):
        if r.random() < 0.5:
            b[i], b[i + 1] = b[i + 1], b[i]
    print(1)
    print(n)
    print(*vals)
    print(*a)
    print(*b)
    sys.exit()
r = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = r.randint(1, 3)
print(T)
for _ in range(T):
    n = r.randint(1, 8 if size == 0 else 40)
    vals = [r.randint(-50, 50) for _ in range(n)]
    a = list(range(1, n + 1))
    b = list(range(1, n + 1))
    r.shuffle(a)
    r.shuffle(b)
    print(n)
    print(*vals)
    print(*a)
    print(*b)
