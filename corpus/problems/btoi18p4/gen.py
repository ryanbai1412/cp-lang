"""Random BOI 2018 "Alternating Current" input: python3 gen.py SEED [SIZE].
Mode 0: uniform arcs. Mode 1: k overlapping top arcs round the ring (k often
odd, the hard branch) plus arcs nested inside them. 3 <= n, 2 <= m, both <= 1e5."""
import sys, random

if sys.argv[1] == "--max":
    # n = m = 1e5: an odd 49999 maximal arcs, each overlapping both neighbours (the hard branch,
    # every pair a candidate), and 50001 strict sub-arcs nested in them
    rng = random.Random(1)
    n, k = 100_000, 49_999
    cuts = [2 * i for i in range(k)]
    tops = [(cuts[i], (cuts[i + 1] if i + 1 < k else cuts[0] + n) - cuts[i] + 1) for i in range(k)]
    out = []
    for a, length in tops:
        out.append((a % n + 1, (a + length - 1) % n + 1))
    while len(out) < 100_000:
        a, length = tops[rng.randrange(k)]
        off = rng.randrange(length)
        sub = rng.randint(1, length - off - (1 if off == 0 else 0))
        out.append(((a + off) % n + 1, (a + off + sub - 1) % n + 1))
    rng.shuffle(out)
    print(n, len(out))
    print("\n".join(f"{a} {b}" for a, b in out))
    sys.exit()

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
