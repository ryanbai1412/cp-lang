"""ioi18p6 (Meetings) inputs: `gen.py SEED SIZE`, or `gen.py --max`, in the sample grader's format.

N Q, then H (1 <= H <= 1e9), then Q distinct meetings "L R" (0 <= L <= R < N). Heights come from
{1, 2}, up to 20 or up to 1e9, so ties and deep Cartesian trees both occur. --max: N = Q = 750000,
heights that rise to a peak and fall (a deep tree) with random noise, and random distinct ranges.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 750_000 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
if MAX:
    h = [min(i, n - i) * 2000 + rng.randint(1, 1000) for i in range(n)]
else:
    v = rng.choice([2, 20, 10**9])
    h = [rng.randint(1, v) for _ in range(n)]
want = size if MAX else rng.randint(1, min(size, n * (n + 1) // 2))
pairs = set()
while len(pairs) < want:
    a, b = rng.randrange(n), rng.randrange(n)
    pairs.add((min(a, b), max(a, b)))
pairs = list(pairs)
rng.shuffle(pairs)
print(n, len(pairs))
print(*h)
for l, r in pairs:
    print(l, r)
