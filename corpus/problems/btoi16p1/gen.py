"""btoi16p1 (Bosses) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

n in 2..=5000, then per employee k_i and the k_i distinct bosses they accept (not themselves),
sum k_i <= 1e4, such that some root reaches everyone. --max: n = 5000 on a ring where each
accepts both neighbours (sum k = 1e4), so every root reaches everyone and every BFS runs full.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 5000 if MAX else rng.randint(2, max(2, min(5000, int(sys.argv[2]))))
if MAX:
    acc = [[(i - 1) % n + 1, (i + 1) % n + 1] for i in range(n)]
else:
    order = rng.sample(range(1, n + 1), n)
    acc = [set() for _ in range(n + 1)]
    for j in range(1, n):
        acc[order[j]].add(order[rng.randrange(max(0, j - 3) if rng.random() < 0.5 else 0, j)])
    budget = rng.randint(n - 1, min(10**4, n * (n - 1)))
    for _ in range(budget - (n - 1)):
        i, b = rng.randint(1, n), rng.randint(1, n)
        if i != b:
            acc[i].add(b)
    acc = [rng.sample(sorted(s), len(s)) for s in acc[1:]]
print(n)
for a in acc:
    print(len(a), *a)
