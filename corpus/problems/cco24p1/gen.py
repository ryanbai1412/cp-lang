"""gen.py seed size: CCO '24 P1. 1 <= N <= 1e6, 0 <= M <= 1e6, 0 <= v_i, c_i <= 1e9,
distinct endpoints, at most one route per pair. size >= 1e6 gives max shapes by seed:
random graph, one long path, all costs 0, all v = 1e9 with c = 1e9, a star."""
import random, sys
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
big = size >= 10 ** 6
n = 10 ** 6 if big else rng.randint(1, size + 1)
shape = ["random", "path", "zero", "max", "star"][seed % 5] if big else rng.choice(["random", "path", "zero"])
cap = 10 ** 9 if big or rng.random() < 0.5 else 10
edges = set()
if shape == "path":
    perm = list(range(1, n + 1)); rng.shuffle(perm)
    edges = {(perm[i], perm[i + 1]) for i in range(n - 1)}
elif shape == "star":
    edges = {(1, v) for v in range(2, n + 1)}
elif n > 1:
    m = 10 ** 6 if big else rng.randint(0, min(n * (n - 1) // 2, 3 * size))
    while len(edges) < m:
        u, v = rng.randint(1, n), rng.randint(1, n)
        if u != v and (u, v) not in edges and (v, u) not in edges:
            edges.add((u, v))
def val():
    if shape == "zero" and rng.random() < 0.5: return 0
    if shape == "max": return 10 ** 9
    return rng.randint(0, cap)
vs = [val() for _ in range(n)]
out = [f"{n} {len(edges)}", " ".join(map(str, vs))]
for u, v in edges:
    c = 0 if shape == "zero" else (10 ** 9 if shape == "max" else rng.randint(0, cap))
    out.append(f"{u} {v} {c}")
sys.stdout.write("\n".join(out) + "\n")
