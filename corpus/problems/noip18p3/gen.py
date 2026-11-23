"""noip18p3 (Track Construction) generator: `seed size`; a tree on 1..=size junctions
(clamped to 2..=5e4), then m in 1..=n - 1.

Shapes: random parent, path, star, caterpillar, broom, deep binary, short-hop random. Road
lengths are all 1, small, or up to 1e4; m is 1, n - 1, or random. Labels and edge order are
shuffled.
"""
import random
import sys

rng = random.Random(int(sys.argv[1]))
S = max(2, min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 50_000))
n = S if rng.random() < 0.3 else rng.randint(2, S)
kind = rng.randrange(7)
par = [0] * n
for i in range(1, n):
    if kind == 0: par[i] = rng.randrange(i)
    elif kind == 1: par[i] = i - 1
    elif kind == 2: par[i] = 0
    elif kind == 3: par[i] = i - 1 if i % 2 or i < 2 else i - 2
    elif kind == 4: par[i] = i - 1 if i < n // 2 else max(0, n // 2 - 1)
    elif kind == 5: par[i] = (i - 1) // 2
    else: par[i] = rng.randrange(max(0, i - 3), i)
cap = rng.choice([1, 10, 1000, 10_000])
m = rng.choice([1, n - 1, rng.randint(1, n - 1), rng.randint(1, max(1, (n - 1) // 10))])
lab = list(range(1, n + 1))
rng.shuffle(lab)
es = []
for i in range(1, n):
    u, v = lab[par[i]], lab[i]
    if rng.random() < 0.5:
        u, v = v, u
    es.append((u, v, rng.randint(1, cap)))
rng.shuffle(es)
print(n, m)
print("\n".join(f"{u} {v} {w}" for u, v, w in es))
