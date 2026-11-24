"""sac22cc1p3 generator: an N x M grid (N, M, Q <= 100) of X, W and '.', its blood already
stable, then Q actions 1 or 2."""
import random
import sys


def settle(g, n, m):
    """Blood alternates sliding to the leftmost reachable cell and dropping one row until still."""
    moved = True
    while moved:
        moved = False
        for r in range(n):
            for c in range(m):
                if g[r][c] == "W":
                    k = c
                    while k > 0 and g[r][k - 1] == ".":
                        k -= 1
                    if k != c:
                        g[r][k], g[r][c], moved = "W", ".", True
        drops = [(r, c) for r in range(n - 1) for c in range(m) if g[r][c] == "W" and g[r + 1][c] == "."]
        for r, c in drops:
            g[r][c], g[r + 1][c], moved = ".", "W", True


def emit(rng, n, m, q, mix):
    g = [[rng.choices("XW.", mix)[0] for _ in range(m)] for _ in range(n)]
    settle(g, n, m)
    print(n, m)
    print("\n".join("".join(r) for r in g))
    print(q)
    print("\n".join(str(rng.choice((1, 2))) for _ in range(q)))


if sys.argv[1] == "--max":
    rng = random.Random(0)
    print(100, 100)
    # Heads on top over a pool of blood: every drop sends blood sliding across whole rows.
    g = [["X" if r < 50 and rng.random() < 0.5 else "W" if r >= 50 and rng.random() < 0.6 else "." for _ in range(100)]
         for r in range(100)]
    settle(g, 100, 100)
    print("\n".join("".join(r) for r in g))
    print(100)
    print("\n".join(["1"] * 99 + ["2"]))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n, m = rng.randint(1, max(1, min(100, size))), rng.randint(1, max(1, min(100, size)))
emit(rng, n, m, rng.randint(1, max(1, min(100, size))), rng.choice([(1, 1, 1), (2, 1, 3), (1, 3, 2)]))
