"""ioi23p1 (Closing Time): `gen.py SEED SIZE`, or `gen.py --max`.

C, then C scenarios "N X Y K" with N - 1 roads "U V W" (2 <= N, sum of N <= 2e5,
0 <= X < Y < N, 0 <= K <= 1e18, 0 <= U < V < N, 1 <= W <= 1e6); the output is each
max_score. --max: one scenario with N = 2e5 on a long X..Y spine with hanging branches,
K a third of the total road length.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))


def scenario(n, spine, wmax):
    lab = list(range(n))
    rng.shuffle(lab)
    edges = []
    for v in range(1, n):
        u = v - 1 if v < spine else rng.randrange(v)
        a, b = sorted((lab[u], lab[v]))
        edges.append((a, b, rng.randint(1, wmax)))
    rng.shuffle(edges)
    x, y = sorted((lab[0], lab[spine - 1])) if spine > 1 else sorted(rng.sample(range(n), 2))
    total = sum(w for _, _, w in edges)
    k = rng.choice([0, rng.randint(0, total), rng.randint(0, 3 * total), 10**18]) if not MAX else total // 3
    return [f"{n} {x} {y} {k}", *(f"{a} {b} {w}" for a, b, w in edges)]


if MAX:
    cases = [scenario(200_000, 100_000, 1_000_000)]
else:
    budget = min(200_000, max(2, int(sys.argv[2])))
    cases = []
    while budget >= 2 and len(cases) < rng.randint(1, 5):
        n = rng.randint(2, budget)
        budget -= n
        cases.append(scenario(n, rng.randint(1, n), rng.choice([3, 1_000_000])))
print(len(cases))
print("\n".join(line for c in cases for line in c))
