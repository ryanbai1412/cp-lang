"""gen.py SEED SIZE | --max: NOI 2006 Tour Guide. "0" (the test number, as in the sample),
n m, then m roads. The statement gives no bounds; the official files reach n = 10000 and
m = 9999 there, so a seed draws one of their shapes (random sparse graph, grid with roads
missing, tree, hypercube) at up to 10 * SIZE attractions. --max keeps 9999 random roads of
a 100 x 100 grid: cycles and several components, so the reference's search runs in full."""
import random, sys


def grid(r, w, h, keep):
    e = [(i * w + j, i * w + j + 1) for i in range(h) for j in range(w - 1)]
    e += [(i * w + j, (i + 1) * w + j) for i in range(h - 1) for j in range(w)]
    return r.sample(e, min(len(e), keep(len(e))))


if sys.argv[1] == "--max":
    r = random.Random(2006)
    n, edges = 10000, grid(r, 100, 100, lambda k: 9999)
else:
    r = random.Random(int(sys.argv[1]))
    cap = min(10000, 10 * int(sys.argv[2]))
    shape = r.randrange(4)
    if shape == 0:
        n = r.randint(1, cap)
        pairs = set()
        for _ in range(r.randint(0, min(n * (n - 1) // 2, 4 * n))):
            a, b = r.sample(range(n), 2) if n > 1 else (0, 0)
            if a != b:
                pairs.add((min(a, b), max(a, b)))
        edges = sorted(pairs)
    elif shape == 1:
        w = r.randint(1, max(1, int(cap ** 0.5)))
        h = max(1, cap // w)
        n = w * h
        edges = grid(r, w, h, lambda k: r.randint(k * 2 // 3, k))
    elif shape == 2:
        n = r.randint(1, cap)
        edges = [(r.randrange(v), v) for v in range(1, n)]
    else:
        d = r.randint(0, max(0, cap.bit_length() - 1))
        n = 1 << d
        edges = [(v, v ^ 1 << b) for v in range(n) for b in range(d) if v < v ^ 1 << b]
        edges = r.sample(edges, r.randint(len(edges) * 3 // 4, len(edges)))
perm = list(range(1, n + 1))
r.shuffle(perm)
r.shuffle(edges)
print(0)
print(n, len(edges))
for a, b in edges:
    a, b = perm[a], perm[b]
    print(*((a, b) if r.random() < 0.5 else (b, a)))
