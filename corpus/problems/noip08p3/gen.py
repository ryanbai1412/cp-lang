import sys, random
# seed size: an n x m class (2 <= n, m <= 50: a one-wide class has no two disjoint paths, and the
# judge data never has one), kindness 0..=100 with both corners 0
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 50 if mx else int(sys.argv[2])
while True:
    n = 50 if mx else r.randint(2, max(2, min(50, S + 1))); m = 50 if mx else r.randint(2, max(2, min(50, S + 1)))
    if n * m > 1: break
g = [[r.randint(0, 100) for _ in range(m)] for _ in range(n)]
g[0][0] = g[-1][-1] = 0
print(n, m)
for row in g: print(*row)
