"""ioi17p6 (Ancient Books): `gen.py SEED SIZE`, or `gen.py --max`.

"n s" then the permutation p[0..n-1] (1 <= n <= 1e6, 0 <= s < n); the output is the
minimum walk. --max: n = 1e6, s in the middle of a wide run of fixed points, with
swaps and long cycles on both sides so every gap must be bridged.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 1_000_000 if MAX else rng.randint(1, min(1_000_000, max(1, int(sys.argv[2]))))
p = list(range(n))
if MAX:
    s = n // 2
    blocks = [(i, i + 8) for i in range(0, n // 2 - 20_000, 40)]
    blocks += [(i, i + 8) for i in range(n // 2 + 20_000, n - 8, 40)]
    for a, b in blocks:
        seg = p[a:b]
        rng.shuffle(seg)
        p[a:b] = seg
    a, b = n // 2 - 20_000 - 100_000, n // 2 + 20_000 + 100_000
    p[a], p[b] = p[b], p[a]
else:
    s = rng.randrange(n)
    mode = rng.randrange(3)
    if mode == 0:
        rng.shuffle(p)
    else:
        for _ in range(rng.randint(0, n)):
            i = rng.randrange(n)
            j = min(n - 1, max(0, i + rng.randint(-3, 3))) if mode == 1 else rng.randrange(n)
            p[i], p[j] = p[j], p[i]
print(n, s)
print(*p)
