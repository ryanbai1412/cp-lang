import sys, random
big = sys.argv[1:] == ["--max"]
if big: sys.argv[1:] = ["1", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = S if big else r.randint(2, max(2, min(S, 100000)))
q = 200000 if big else r.randint(n - 1, min(200000, 3 * n))
par = [0, 0] + [r.randint(max(1, v - 3) if r.random() < 0.5 else 1, v - 1) for v in range(2, n + 1)]
edges = [(par[v], v) if r.random() < 0.5 else (v, par[v]) for v in range(2, n + 1)]
r.shuffle(edges)
cuts = [2] * (n - 1) + [1] * (q - (n - 1))
r.shuffle(cuts)
print(n, q)
for u, v in edges: print(u, v)
order = edges[:]; r.shuffle(order); k = 0
for c in cuts:
    if c == 2:
        u, v = order[k]; k += 1
        print(2, *((u, v) if r.random() < 0.5 else (v, u)))
    else: print(1, r.randint(1, n), r.randint(1, n))
