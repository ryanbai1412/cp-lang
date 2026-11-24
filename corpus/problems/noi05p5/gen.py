"""gen.py SEED SIZE | --max: NOI 2005 Party. "0" (the test number, as in the sample), N M,
the bounds k_i in [1, N-1], M roads u v c, then the scoring factor d. The statement bounds
only k; the official files reach N = 1000, M = 20000 and c below 20000, so a seed plants a
random spanning tree (the network always exists), gives each person a bound at or a little
above their tree degree (or no bound at all, k = N-1), and adds further distinct roads.
--max is N = 1000, M = 20000 with bounds within one of a low-degree planted tree."""
import random, sys

if sys.argv[1] == "--max":
    r = random.Random(2005)
    n, m, mode = 1000, 20000, 1
else:
    r = random.Random(int(sys.argv[1]))
    n = r.randint(2, min(1000, 2 + 10 * int(sys.argv[2])))
    m = r.randint(n - 1, min(20000, n * (n - 1) // 2, 20 * n))
    mode = r.randrange(3)
perm = list(range(1, n + 1))
r.shuffle(perm)
tree = [(perm[r.randrange(max(0, i - 3), i)], perm[i]) for i in range(1, n)]
deg = [0] * (n + 1)
for a, b in tree:
    deg[a] += 1
    deg[b] += 1
if mode == 0:
    k = [n - 1] * n
else:
    k = [min(n - 1, deg[i] + (r.randint(0, 1) if mode == 1 else r.randint(0, 3))) for i in range(1, n + 1)]
pairs = {(min(a, b), max(a, b)) for a, b in tree}
while len(pairs) < m:
    a, b = r.sample(range(1, n + 1), 2)
    pairs.add((min(a, b), max(a, b)))
edges = list(pairs)
r.shuffle(edges)
print(0)
print(n, m)
print(*k)
for a, b in edges:
    print(*((a, b) if r.random() < 0.5 else (b, a)), r.randint(1, 20000))
print("0.00001")
