"""btoi18p2 (DNA) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N, K, R with 1 <= R <= K <= N, then N symbols in 0..K (each symbol at least once), then R lines
`B Q` with distinct B and Q in 1..=N; Q above B's count makes it impossible. --max: N = 2e5,
K = R = 1e5, every symbol required once or twice, so the window spans most of the string.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 2 * 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
k = 10**5 if MAX else rng.randint(1, n) if rng.random() < 0.7 else rng.randint(1, min(n, 3))
r = k if MAX or rng.random() < 0.3 else rng.randint(1, k)
d = list(range(k)) + [rng.randrange(k) for _ in range(n - k)]
rng.shuffle(d)
count = [0] * k
for x in d:
    count[x] += 1
need = rng.sample(range(k), r)
print(n, k, r)
print(*d)
for b in need:
    hi = count[b] if MAX or rng.random() < 0.9 else min(n, count[b] + 1)
    print(b, rng.randint(1, hi) if not MAX else min(count[b], rng.randint(1, 2)))
