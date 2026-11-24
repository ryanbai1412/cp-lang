import sys, random
# seed size: n <= size + 1 cities with distinct heights in 1..1e5 (city 1 sometimes not the lowest),
# k in 1..n + 1 or 1e9, p the precision of the smallest subtask holding n; --max: n = 8000, p = 3000,
# k = 1e9, city 1 lowest so every other city is worth merging
P = [(10, 5), (100, 40), (250, 100), (500, 200), (700, 300), (2500, 1000), (4000, 1500), (8000, 3000)]
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 8000 if mx else r.randint(1, min(8000, S + 1))
k = 10**9 if mx else r.choice([r.randint(1, n + 1), 10**9])
p = next(q for m, q in P if n <= m)
h = r.sample(range(1, 100001), n)
if mx or r.random() < 0.7:
    i = h.index(min(h)); h[0], h[i] = h[i], h[0]
print(n, k, p); print(*h)
