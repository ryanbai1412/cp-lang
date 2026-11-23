import sys, random
# seed size: a tree on 1..size cities: random, path, star, caterpillar or complete binary
rng = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = S if S >= 1000 or rng.random() < 0.4 else rng.randint(1, S)  # size 1000: the limit
kind = rng.randrange(5)
par = [0, 0]
for v in range(2, n + 1):
    par.append([rng.randint(1, v - 1), v - 1, 1, max(1, v - 1 - (v % 2)) if v % 2 else max(1, v - 2) , v // 2][kind])
perm = list(range(1, n + 1)); rng.shuffle(perm)
edges = [(perm[v - 1], perm[par[v] - 1]) for v in range(2, n + 1)]
rng.shuffle(edges)
print(n)
for u, v in edges: print(*((u, v) if rng.random() < 0.5 else (v, u)))
