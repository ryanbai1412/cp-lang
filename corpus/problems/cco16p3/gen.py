"""gen.py SEED SIZE | --max: CCO 2016 Legends. Line 1 is the subtask S and the graph count;
each graph is connected and simple: a random tree plus extra edges, sum of n and m <= 1000
(S = 3: 1e5). --max is one S = 3 graph with n = m = 1e5 (a long cycle with chords)."""
import random, sys


def graph(rng, n, extra):
    edges = {tuple(sorted((v, rng.randint(1, v - 1)))) for v in range(2, n + 1)}
    pairs = n * (n - 1) // 2
    while len(edges) < min(pairs, n - 1 + extra):
        u, v = rng.sample(range(1, n + 1), 2)
        edges.add((min(u, v), max(u, v)))
    edges = list(edges)
    rng.shuffle(edges)
    return [f"{n} {len(edges)}"] + [f"{u} {v}" if rng.random() < 0.5 else f"{v} {u}" for u, v in edges]


if sys.argv[1] == "--max":
    rng, n = random.Random(0), 100000
    order = list(range(1, n + 1))
    rng.shuffle(order)
    edges = [(order[i], order[i + 1]) for i in range(n - 1)] + [(order[0], order[-1])]
    print(3, 1)
    print(n, n)
    print("\n".join(f"{u} {v}" for u, v in edges))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
s = rng.randint(1, 5)
budget = min(1000, max(10, size))
out = []
while budget > 0 and len(out) < 1000:
    n = rng.randint(1, max(1, min(9, budget // 2)))
    extra = rng.choice([0, 0, 1, 1, 2, 3, n])
    g = graph(rng, n, extra)
    budget -= n + len(g) - 1
    if budget < 0 and out:
        break
    out.append(g)
print(s, len(out))
for g in out:
    print("\n".join(g))
