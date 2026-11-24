"""gen.py SEED SIZE | --max: CCO 2020 Mountains and Valleys. 4 <= N <= 500000, N-1 <= M <= 2000000;
the N-1 weight-1 trails form a spanning tree, every other trail has ceil(N/3) <= w <= N, and no pair
of sites has two trails. --max is N = 500000, M = 2000000."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 500000 if MAX else r.randint(4, max(4, min(500000, S + 3)))
cap = n * (n - 1) // 2
m = 2000000 if MAX else r.randint(n - 1, min(cap, 2000000, n - 1 + r.choice([0, 1, 2, n, 2 * n, cap])))
perm = list(range(n)); r.shuffle(perm)
shape = 0 if MAX else r.randrange(4)
edges = set(); out = []
for i in range(1, n):
    j = i - 1 if shape == 0 else (r.randrange(i) if shape == 1 else (0 if shape == 2 else max(0, i - r.randint(1, 3))))
    a, b = perm[i], perm[j]
    edges.add((min(a, b), max(a, b))); out.append(f"{a} {b} 1")
lo = -(-n // 3)
while len(out) < m:
    a, b = r.randrange(n), r.randrange(n)
    if a == b or (min(a, b), max(a, b)) in edges: continue
    edges.add((min(a, b), max(a, b)))
    out.append(f"{a} {b} {lo if MAX and r.random() < 0.5 else r.randint(lo, n)}")
r.shuffle(out)
sys.stdout.write(f"{n} {m}\n" + "\n".join(out) + "\n")
