"""gen.py SEED SIZE | --max: Riolku's Mock CCC S5. N, Q in 1..=2e5, values and x in 0..=2e5,
0 <= l <= r <= 5e5. Small sizes keep values and ranges small so queries overlap; --max is
N = Q = 2e5 over the full value range, two thirds range-xor updates."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(1, min(200_000, 1 + 3 * S))
q = 200_000 if MAX else r.randint(1, min(200_000, 1 + 3 * S))
vh = 200_000 if MAX else r.choice([min(200_000, 3 + 2 * S), 200_000])
rh = 500_000 if MAX else min(500_000, 2 * vh + 3)
def rng():
    a, b = sorted((r.randint(0, rh), r.randint(0, rh)))
    return (0, 500_000) if not MAX and r.random() < 0.1 else (a, b)
print(n, q)
print(*(r.randint(0, vh) for _ in range(n)))
for _ in range(q):
    t = r.choices([1, 2, 3], [1, 4, 1] if MAX else [1, 1, 1])[0]
    if t == 1:
        print(1, r.randint(0, vh))
    elif t == 2:
        print(2, *rng(), r.randint(0, vh))
    else:
        print(3, *rng())
