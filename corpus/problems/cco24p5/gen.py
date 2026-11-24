"""CCO 2024 P5 (Heavy Light Decomposition): N (1 <= N <= 5e5), then a_1..a_N in 1..=N.

`gen.py SEED SIZE`: N up to SIZE, values from a pool of random width (all distinct up to all
equal), sometimes alternating. `gen.py --max`: N = 5e5, values alternating between a light
stream of fresh values and a heavy pool of 1000 repeated ones, with random noise, so most
prefixes stay good and every step touches long ranges of both segment trees.
"""
import random
import sys

if sys.argv[1:] == ["--max"]:
    g = random.Random(1)
    n = 500_000
    fresh = list(range(1001, n + 1))
    g.shuffle(fresh)
    a = []
    for i in range(n):
        if g.random() < 0.02:
            a.append(g.randint(1, n))
        elif i % 2 == 0:
            a.append(fresh.pop())
        else:
            a.append(g.randint(1, 1000))
    print(n)
    print(*a)
    sys.exit()

g = random.Random(int(sys.argv[1]))
n = g.randint(1, max(1, min(500_000, int(sys.argv[2]))))
w = g.choice([1, 2, 3, max(1, n // 3), n])
if g.random() < 0.3:
    a = [g.randint(1, n) if i % 2 == 0 else g.randint(1, w) for i in range(n)]
else:
    a = [g.randint(1, w) for _ in range(n)]
print(n)
print(*a)
