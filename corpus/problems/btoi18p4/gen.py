"""Random BOI 2018 "Alternating Current" input: python3 gen.py SEED [SIZE].
Mode 0: uniform arcs. Mode 1: k overlapping top arcs round the ring (k often
odd, the hard branch) plus arcs nested inside them. 3 <= n, 2 <= m, both <= 1e5."""
import sys, random

rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2]) if len(sys.argv) > 2 else 30
n = rng.randint(3, max(3, size))
arcs = []


def arc(a, length):
    length = max(1, min(n, length))
    return (a % n + 1, (a + length - 1) % n + 1)


if rng.random() < 0.3:
    for _ in range(rng.randint(2, max(2, size))):
        arcs.append(arc(rng.randrange(n), rng.randint(1, n)))
else:
    k = rng.randint(1, min(n, 9))
    cuts = sorted(rng.sample(range(n), k))
    for i, c in enumerate(cuts):
        nxt = cuts[(i + 1) % k] + (n if i + 1 == k else 0)
        arcs.append(arc(c - rng.randint(0, 2), nxt - c + rng.randint(0, 3)))
    for _ in range(rng.randint(0, max(1, size))):
        a, b = arcs[rng.randrange(k)]
        length = (b - a) % n + 1
        off = rng.randrange(length)
        arcs.append(arc(a - 1 + off, rng.randint(1, length - off)))
    if len(arcs) < 2:
        arcs.append(arc(rng.randrange(n), rng.randint(1, n)))
rng.shuffle(arcs)
print(n, len(arcs))
for a, b in arcs:
    print(a, b)
