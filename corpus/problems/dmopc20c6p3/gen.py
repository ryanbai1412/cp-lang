"""gen.py seed size: DMOPC '20 C6 P3. 1 <= N, M <= 5e5, self-loops allowed.
size >= 5e5 gives N = M = 5e5: by seed, a random graph, or long chains with back edges."""
import random, sys
if sys.argv[1:] == ["--max"]: sys.argv[1:] = ["1", "500000"]  # the hardest big case
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
big = size >= 5 * 10 ** 5
n = 5 * 10 ** 5 if big else rng.randint(1, max(1, min(size, 8)))
m = 5 * 10 ** 5 if big else rng.randint(1, max(1, min(size, 12)))
p = [rng.randint(0, 1) for _ in range(n)]
if big and seed % 2:
    e = [(i, i + 1) for i in range(1, n)] + [(rng.randint(2, n), rng.randint(1, n))]
    e = e[:m]
else:
    e = [(rng.randint(1, n), rng.randint(1, n)) for _ in range(m)]
print(n, len(e)); print(*p)
for u, v in e: print(u, v)
