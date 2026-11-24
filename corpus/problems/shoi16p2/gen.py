"""shoi16p2 generator: N, M <= 1e6 and K <= 2000 distinct deleted lattice points (x <= N, y <= M)."""
import random
import sys


def emit(rng, n, m, k, near):
    if near:
        k = min(k, (min(n, near) + 1) * (min(m, near) + 1))
    pts = set()
    while len(pts) < k:
        if near:  # clustered points share many squares, the pair part's worst case
            pts.add((rng.randint(0, min(n, near)), rng.randint(0, min(m, near))))
        else:
            pts.add((rng.randint(0, n), rng.randint(0, m)))
    print(n, m, k)
    print("\n".join(f"{x} {y}" for x, y in sorted(pts, key=lambda _: rng.random())))


if sys.argv[1] == "--max":
    print(10**6, 10**6, 2000)
    print("\n".join(f"{500000 + i // 45} {500000 + i % 45}" for i in range(2000)))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n, m = rng.randint(1, max(1, min(10**6, size))), rng.randint(1, max(1, min(10**6, size)))
emit(rng, n, m, rng.randint(0, min(2000, size, (n + 1) * (m + 1))), rng.choice([0, 10 + int(size ** 0.5)]))
