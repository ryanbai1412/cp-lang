"""gen.py SEED SIZE | --max: Riolku's Mock CCC S3. A permutation of 1..=N (N <= 1000) and
M <= N(N-1)/2 distinct unordered swap pairs. Cases mix sparse graphs (many components), paths
(b = a + 1), max-degree-2 cycles and dense ones; --max is N = 1000 with one long path of
adjacent swaps under a reversed permutation, the most swaps a tree walk needs."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 1000 if MAX else r.randint(1, min(1000, 1 + 3 * S))
p = list(range(n, 0, -1)) if MAX else r.sample(range(1, n + 1), n)
kind = 0 if MAX else r.randint(0, 3)
if kind == 0:
    pairs = [(i, i + 1) for i in range(1, n) if MAX or r.random() < 0.8]
elif kind == 1:
    order = r.sample(range(1, n + 1), n)
    pairs = [(order[i], order[i + 1]) for i in range(n - 1) if r.random() < 0.9]
    if n > 2 and r.random() < 0.5:
        pairs.append((order[-1], order[0]))
else:
    allp = [(a, b) for a in range(1, n + 1) for b in range(a + 1, n + 1)]
    pairs = r.sample(allp, r.randint(0, len(allp) if kind == 3 else min(len(allp), n)))
pairs = [(b, a) if r.random() < 0.5 else (a, b) for a, b in pairs]
print(n, len(pairs))
print(*p)
for a, b in pairs:
    print(a, b)
