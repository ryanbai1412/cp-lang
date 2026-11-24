#!/usr/bin/env python3
# gen.py SEED SIZE: SIZE 0/1 small; SIZE >= 2 n = 100 with a_i up to 1e9 and t up to 1e11.
import random, sys
if sys.argv[1:] == ["--max"]:
    # n = 100, a ascending to 1e9 so the f = 50 richest boxes are the last ones: t is their sum
    # and the answer needs the most swaps (best[] runs over the full swap range)
    r = random.Random(0)
    a = sorted(r.randint(10 ** 9 // 2, 10 ** 9) for _ in range(100))
    print(100, 50, sum(a[-50:]))
    print(*a)
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
if S <= 1:
    n = r.randint(1, 4 + 4 * S); A = r.choice([0, 3, 10])
else:
    n = 100; A = 10 ** 9
f = r.randint(1, n) if S <= 1 else r.choice([n, n // 2, r.randint(1, n)])
a = [A if S >= 2 and r.random() < 0.3 else r.randint(0, A) for _ in range(n)]
top = sum(sorted(a)[-f:])
t = r.choice([0, top, max(0, top - r.randint(0, max(1, A))), top + 1, r.randint(0, top)])
if S >= 2 and r.random() < 0.2: t = 10 ** 11
print(n, f, t)
print(*a)
