"""dmopc21c2p6 (Strange Function) inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = Q = 3e5).

"N Q", a permutation, then queries "1", "2 i" or "3 i". --max: a reversed permutation with
random swaps (every pass moves many minima), a third of the queries passes.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 3 * 10**5 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
q = size if MAX else rng.randint(1, size)
p = list(range(n, 0, -1)) if MAX or rng.random() < 0.3 else rng.sample(range(1, n + 1), n)
for _ in range(n // 20 if MAX else rng.randint(0, n)):
    i, j = rng.randrange(n), rng.randrange(n)
    p[i], p[j] = p[j], p[i]
out = [f"{n} {q}", " ".join(map(str, p))]
for _ in range(q):
    kind = rng.randint(1, 3)
    out.append("1" if kind == 1 else f"{kind} {rng.randint(1, n)}")
print("\n".join(out))
