"""ioi09p7 (Regions) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"N R Q", then employee 1's region, then for i = 2..N "S_i H_i" (supervisor S_i < i), then Q
queries "r1 r2" with r1 != r2. --max: N = Q = 2e5, R = 25000 on a long path with branches; a few regions
hold most employees and the queries are distinct pairs of big and small regions, so neither
per-pair caching nor small-region walks are enough.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, r, q = 2 * 10**5, 25000, 2 * 10**5
else:
    n = rng.randint(2, max(2, int(sys.argv[2])))
    r = rng.randint(2, max(2, n // 2))
    q = rng.randint(1, max(1, int(sys.argv[2])))
big = list(range(1, min(r, 450) + 1))


def region():
    return rng.choice(big) if MAX and rng.random() < 0.6 else rng.randint(1, r)


out = [f"{n} {r} {q}", str(region())]
for i in range(2, n + 1):
    s = i - 1 if MAX and rng.random() < 0.8 else rng.randint(1, i - 1)
    out.append(f"{s} {region()}")
for k in range(q):
    if MAX:
        a, b = big[k % len(big)], 1 + (k * 7919) % r
        if a == b:
            b = b % r + 1
        out.append(f"{a} {b}" if k % 2 else f"{b} {a}")
    else:
        a, b = rng.sample(range(1, r + 1), 2)
        out.append(f"{a} {b}")
print("\n".join(out))
