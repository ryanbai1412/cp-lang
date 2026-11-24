"""ioi08p2 (Islands) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N (2 <= N <= 1e6), then for island i its bridge "to L" (to != i, 1 <= L <= 1e8). Each island's
group is a cycle with trees hanging off it.
--max: N = 1e6: a 500000-island cycle, a 400000-island path hanging off it and 100000 small
groups, with lengths up to 1e8.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, top = 1_000_000, 100_000_000
else:
    size = max(2, int(sys.argv[2]))
    n, top = rng.randint(2, min(1_000_000, size)), rng.choice([10, 100_000_000])
to = [0] * (n + 1)
order = list(range(1, n + 1))
rng.shuffle(order)
if MAX:
    cycle, path, rest = order[:500_000], order[500_000:900_000], order[900_000:]
    groups = [(cycle, path)] + [(rest[i:i + 2], rest[i + 2:i + 10]) for i in range(0, len(rest), 10)]
else:
    groups, at = [], 0
    while at < n:
        left = n - at
        k = rng.randint(2, left) if left >= 4 and rng.random() < 0.5 else left
        k = left if left - k < 2 else k
        c = rng.randint(2, k)
        groups.append((order[at:at + c], order[at + c:at + k]))
        at += k
for cycle, hang in groups:
    for a, b in zip(cycle, cycle[1:] + cycle[:1]):
        to[a] = b
    seen = list(cycle)
    for v in hang:
        to[v] = seen[-1] if MAX else rng.choice(seen)
        seen.append(v)
out = [str(n)] + [f"{to[i]} {rng.randint(1, top)}" for i in range(1, n + 1)]
print("\n".join(out))
