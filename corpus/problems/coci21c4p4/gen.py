"""coci21c4p4 (Parkovi) generator: `seed size`; a tree on 1..=size towns (capped at 2e5), then k.

Shapes: random parent, path (sometimes the subtask-3 chain a_i = i, b_i = i + 1), star,
caterpillar, broom, deep binary, short-hop random. Road lengths are all 1, small, or up to 1e9;
k is 1, n, or random in 1..=n. Labels and edge order are shuffled except on the chain.
"""
import random
import sys

if sys.argv[1] == "--max":
    # n = 2e5 on a path with shuffled labels, roads near 1e9 (radius search over the full
    # 2e14 range, depth 2e5), k = 1000
    rng = random.Random(1)
    n, k = 200_000, 1000
    lab = list(range(1, n + 1)); rng.shuffle(lab)
    es = [(lab[i - 1], lab[i], rng.randint(10**9 - 1000, 10**9)) for i in range(1, n)]
    rng.shuffle(es)
    print(n, k)
    print("\n".join(f"{u} {v} {w}" for u, v, w in es))
    sys.exit()

rng = random.Random(int(sys.argv[1]))
S = max(1, min(int(sys.argv[2]) if len(sys.argv) > 2 else 8, 200_000))
n = S if rng.random() < 0.3 else rng.randint(1, S)
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
cap = rng.choice([1, 10, 1000, 10**9])
k = rng.choice([1, n, rng.randint(1, n), rng.randint(1, max(1, n // 10))])
chain = kind == 1 and rng.random() < 0.5
lab = list(range(1, n + 1))
if not chain:
    rng.shuffle(lab)
es = []
for i in range(1, n):
    u, v = lab[par[i]], lab[i]
    if not chain and rng.random() < 0.5:
        u, v = v, u
    es.append((u, v, rng.randint(1, cap)))
if not chain:
    rng.shuffle(es)
print(n, k)
print("\n".join(f"{u} {v} {w}" for u, v, w in es))
