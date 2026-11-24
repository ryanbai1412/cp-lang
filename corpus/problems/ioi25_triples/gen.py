import sys, random
# IOI 2025 Triple Peaks, sample-grader stdin. Part I: "1", N (3..=2e5), then H (1..=N-1).
# Part II: "2", then M K; gen.py makes Part I only (Part II's six (M, K) are in tests.toml).


def peaks(r, n):
    """Peaks at the midpoints of pairs from a random even point set (many mythical triples)."""
    h = [0] * n
    pts = sorted({r.randrange(-n // 2, 3 * n // 2) // 2 * 2 for _ in range(max(3, int((6 * n) ** 0.5)))})
    for a in range(len(pts)):
        for b in range(a + 1, len(pts)):
            m, v = (pts[a] + pts[b]) // 2, (pts[b] - pts[a]) // 2
            if 0 <= m < n and 1 <= v < n and (h[m] == 0 or v < h[m]):
                h[m] = v
    return [v or r.randint(1, 2) for v in h]


def diagonal(r, n):
    """h[x] - x constant on a few long runs: the crossed case's large groups."""
    gs = [r.randrange(-n, n) for _ in range(r.randint(1, 3))]
    return [min(n - 1, max(1, x + r.choice(gs))) if r.random() < 0.8 else r.randint(1, n - 1) for x in range(n)]


def heights(r, n):
    shape = r.randrange(6)
    if shape == 0:
        return [r.randint(1, n - 1) for _ in range(n)]
    if shape == 1:
        return [r.randint(1, min(10, n - 1)) for _ in range(n)]
    if shape == 2:
        return sorted(r.randint(1, n - 1) for _ in range(n))
    if shape == 3:
        return [r.randint(1, min(n - 1, r.choice([3, 6, 20]))) for _ in range(n)]
    if shape == 4:
        return diagonal(r, n)
    return peaks(r, n)


if sys.argv[1] == "--max":
    n = 200000
    h = peaks(random.Random(2025), n)
else:
    r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
    n = r.randint(max(3, S // 2), max(3, S))
    h = heights(r, n)
print(1)
print(n)
print(*h)
