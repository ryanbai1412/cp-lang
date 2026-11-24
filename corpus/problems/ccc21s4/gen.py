"""gen.py SEED SIZE | --max: CCC 2021 S4. n stations, w one-way walkways, d swaps of two route
positions other than the first; the route starts at station 1. --max is n = w = d = 2e5."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 200000 if MAX else r.randint(3, 3 + 3 * S)
w = 200000 if MAX else r.randint(0, 3 * n)
d = 200000 if MAX else r.randint(1, 2 + 2 * S)
print(n, w, d)
for _ in range(w):
    a = r.randint(1, n); b = r.randint(1, n)
    while b == a: b = r.randint(1, n)
    print(a, b)
rest = list(range(2, n + 1)); r.shuffle(rest)
print(1, *rest)
for _ in range(d):
    x, y = r.sample(range(2, n + 1), 2)
    print(x, y)
