"""ucrpc21g generator: n <= 100, p, b <= 1000 drops `r c t` (t in 1..=100); no popcorn and
boulder share a cell and time, nor do two boulders."""
import random
import sys


def emit(rng, n, p, b, tmax):
    cells = [(r, c, t) for r in range(n) for c in range(n) for t in range(1, tmax + 1)] if n * n * tmax <= 10**5 else None

    def spot():
        return (rng.randrange(n), rng.randrange(n), rng.randint(1, tmax))
    boulders = set()
    while len(boulders) < b:
        boulders.add(rng.choice(cells) if cells else spot())
    pops = []
    while len(pops) < p:
        s = spot()
        if s not in boulders:
            pops.append(s)
    print(n, p, b)
    print(rng.randrange(n), rng.randrange(n))
    for s in pops + sorted(boulders, key=lambda _: rng.random()):
        print(*s)


if sys.argv[1] == "--max":
    rng = random.Random(0)
    # Popcorn crowded round the start keeps many (cell, held) states alive at once.
    n, p, b = 100, 1000, 1000
    boulders = set()
    while len(boulders) < b:
        boulders.add((rng.randrange(40, 60), rng.randrange(40, 60), rng.randint(1, 100)))
    pops = []
    while len(pops) < p:
        s = (rng.randrange(35, 65), rng.randrange(35, 65), rng.randint(1, 100))
        if s not in boulders:
            pops.append(s)
    print(n, p, b)
    print(50, 50)
    for s in pops + sorted(boulders):
        print(*s)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(1, max(1, min(100, size)))
tmax = rng.randint(1, 100)
b = rng.randint(0, min(1000, size, n * n * tmax // 2))
emit(rng, n, rng.randint(0, min(1000, 3 * size)), b, tmax)
