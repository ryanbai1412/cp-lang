import sys, random
# usage: gen.py seed size; every tree has fewer than 36 roads between any two cities
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, max(1, min(3000, S))) if S < 3000 else 3000
shape = r.choice(["caterpillar", "shallow", "star", "broom"]) if S >= 3000 else r.choice(["caterpillar", "shallow", "random"])
par = [-1] * n
if shape == "caterpillar":
    spine = min(n, 36)  # a path of up to 35 roads, leaves only on inner spine cities
    for v in range(1, spine): par[v] = v - 1
    for v in range(spine, n): par[v] = r.randint(1, max(1, spine - 2)) if spine > 2 else 0
elif shape == "shallow":
    depth = [0] * n; h = r.randint(1, 17)
    ok = [0]
    for v in range(1, n):
        p = r.choice(ok); par[v] = p; depth[v] = depth[p] + 1
        if depth[v] < h: ok.append(v)
elif shape == "star":
    for v in range(1, n): par[v] = 0
elif shape == "broom":
    for v in range(1, min(n, 18)): par[v] = v - 1
    for v in range(18, n): par[v] = 17
else:
    for v in range(1, n): par[v] = r.randint(max(0, v - 3), v - 1) if v < 30 else r.randint(0, 10)
perm = list(range(1, n + 1)); r.shuffle(perm)
print(n)
es = [(perm[v], perm[par[v]]) if r.random() < .5 else (perm[par[v]], perm[v]) for v in range(1, n)]
r.shuffle(es)
for u, v in es: print(u, v)
