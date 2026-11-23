# SHOI 2016 Reconnaissance Guards: `gen.py SEED S`. Positive N <= S and D <= 20, costs 1..=1000,
# 1 <= M <= N marked vertices given ascending, any tree shape (statement constraints).
import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S)
d = r.choice([1, 1, 2, 3, r.randint(1, 20)])
shape = r.choice(["random", "path", "star", "caterpillar", "deep"])
par = [0, 0] + [{"random": r.randint(1, i - 1), "path": i - 1, "star": 1,
                 "caterpillar": i - 1 if i % 2 == 0 else max(1, i - 2),
                 "deep": r.randint(max(1, i - 3), i - 1)}[shape] for i in range(2, n + 1)]
label = list(range(1, n + 1)); r.shuffle(label)
hi = r.choice([3, 1000])
m = r.choice([1, n, r.randint(1, n)])
print(n, d)
print(*[r.randint(1, hi) for _ in range(n)])
print(m)
print(*sorted(r.sample(range(1, n + 1), m)))
edges = [(label[par[i] - 1], label[i - 1]) for i in range(2, n + 1)]
r.shuffle(edges)
for u, v in edges: print(*((u, v) if r.random() < 0.5 else (v, u)))
