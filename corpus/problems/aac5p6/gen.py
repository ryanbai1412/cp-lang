"""gen.py SEED SIZE | --max: An Animal Contest 5 P6. A tree on N nodes (2 <= N <= 2e5), teleport
range 1 <= D < N, values 1 <= a_i <= 1e9 (sometimes all 1, subtask 1). Trees are random, path-like
or star-like. --max is N = 2e5 on a caterpillar with D = 5 and values up to 1e9."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 200_000 if MAX else r.randint(2, min(200_000, 2 + S))
d = 5 if MAX else r.randint(1, max(1, min(n - 1, r.choice([1, 2, 3, n - 1]))))
hi = 10**9 if MAX or r.random() < 0.7 else 1
a = [r.randint(1, hi) for _ in range(n)]
shape = "cat" if MAX else r.choice(["random", "path", "star", "cat"])
edges = []
for v in range(2, n + 1):
    if shape == "path":
        p = v - 1
    elif shape == "star":
        p = r.randint(1, min(v - 1, 3))
    elif shape == "cat":
        p = v - 1 if v % 3 else r.randint(max(1, v - 6), v - 1)
    else:
        p = r.randint(1, v - 1)
    edges.append((p, v))
perm = list(range(1, n + 1))
r.shuffle(perm)
perm[perm.index(1)], perm[0] = perm[0], 1  # node 1 stays the start, somewhere in the tree
lab = {i + 1: perm[i] for i in range(n)}
r.shuffle(edges)
print(n, d)
print(*a)
print("\n".join(f"{lab[u]} {lab[v]}" if r.random() < 0.5 else f"{lab[v]} {lab[u]}" for u, v in edges))
