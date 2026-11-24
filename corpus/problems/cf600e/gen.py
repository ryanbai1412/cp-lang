"""gen.py SEED SIZE | --max: Codeforces 600E (Lomsat gelral). A tree on n <= 1e5 vertices rooted
at 1, colours 1..n. Small cases use few colours so dominating ties are common. --max is n = 1e5:
a broom (a long path, then a star) with colours drawn from 1..n, so merges are deep and wide."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 100_000 if MAX else r.randint(1, 1 + 2 * S)
k = n if MAX else r.randint(1, max(1, min(n, S)))
col = [r.randint(1, k) for _ in range(n)]
if MAX:
    par = [i - 1 if i < n // 2 else r.randint(1, n // 2) for i in range(2, n + 1)]
else:
    par = [r.randint(max(1, i - 1 - r.randint(0, i)), i - 1) for i in range(2, n + 1)]
perm = [1] + r.sample(range(2, n + 1), n - 1)
edges = [(perm[p - 1], perm[i - 1]) for i, p in zip(range(2, n + 1), par)]
r.shuffle(edges)
edges = [(a, b) if r.random() < 0.5 else (b, a) for a, b in edges]
c = [0] * (n + 1)
for v in range(1, n + 1):
    c[perm[v - 1]] = col[v - 1]
sys.stdout.write(f"{n}\n{' '.join(map(str, c[1:]))}\n" + "".join(f"{a} {b}\n" for a, b in edges))
