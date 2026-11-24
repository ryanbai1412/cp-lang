"""ioi21p5 (Dungeons Game) inputs: `gen.py SEED SIZE`, or `gen.py --max`, in the sample grader's format.

n q, then s, p, w, l (1 <= s, p <= 1e7, i < w[i] <= n, 0 <= l[i] <= n), then q lines "x z"
(0 <= x < n, 1 <= z <= 1e7). Strengths come from a small or a wide range so a hero climbs many
levels. --max: n = 4e5, q = 5e4, losses that send the hero back near the start and p = 1, so
every simulate takes long climbs.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 4 * 10**5 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
q = 5 * 10**4 if MAX else rng.randint(1, size)
V = 10**7 if MAX else rng.choice([2, 10, 1000, 10**7])
s = [rng.randint(1, V) for _ in range(n)]
p = [1 if MAX else rng.randint(1, V) for _ in range(n)]
w = [rng.randint(i + 1, min(n, i + 3)) if MAX else rng.randint(i + 1, n) for i in range(n)]
l = [rng.randint(0, min(n, 50)) if MAX else rng.randint(0, n) for _ in range(n)]
print(n, q)
for v in (s, p, w, l):
    print(*v)
for _ in range(q):
    print(rng.randrange(n), rng.randint(1, V))
