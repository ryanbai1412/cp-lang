"""gen.py SEED SIZE | --max: CCO 2024 Infiltration. Every test has N = 100 rooms (the statement
fixes N); SIZE picks the tree's shape mix, from paths and stars to random and caterpillar trees.
--max is a path on 100 rooms, the tree with the largest diameter."""
import random, sys

N = 100
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
shape = "path" if MAX else r.choice(["path", "star", "random", "caterpillar", "binary", "spider", "broom"])
par = [0] * N
for v in range(1, N):
    if shape == "path":
        par[v] = v - 1
    elif shape == "star":
        par[v] = 0
    elif shape == "binary":
        par[v] = (v - 1) // 2
    elif shape == "caterpillar":
        par[v] = v - 1 if v < N // 2 else r.randrange(N // 2)
    elif shape == "spider":
        legs = r.randint(2, 10) if v == 1 else legs
        par[v] = 0 if v <= legs else v - legs
    elif shape == "broom":
        par[v] = v - 1 if v < N // 2 else N // 2 - 1
    else:
        par[v] = r.randrange(max(0, v - int(sys.argv[2]) - 1), v)
lab = list(range(N))
if not MAX:
    r.shuffle(lab)
edges = [(lab[v], lab[par[v]]) for v in range(1, N)]
if not MAX:
    r.shuffle(edges)
print(N)
for a, b in edges:
    print(a, b)
