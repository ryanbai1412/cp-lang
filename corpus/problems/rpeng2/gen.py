"""rpeng2 generator: a polygon of N <= 50 vertices with edges t (+) or x (*), its labels kept
small enough that every merge order stays in [-32768, 32767]."""
import random
import sys

LIMIT = 32767


def fits(ops, vals):
    """The largest magnitude any merge order reaches, over every chain round the ring."""
    n = len(vals)
    big = [[0] * n for _ in range(n)]
    for i in range(n):
        big[i][0] = abs(vals[i])
    for ln in range(1, n):
        for i in range(n):
            best = 0
            for s in range(ln):
                a, b = big[i][s], big[(i + s + 1) % n][ln - s - 1]
                best = max(best, a * b if ops[(i + s + 1) % n] else a + b)
            if best > LIMIT:
                return False
            big[i][ln] = best
    return True


def emit(rng, n, xs, hi):
    while True:
        ops = [rng.random() < xs for _ in range(n)]
        vals = [rng.randint(-hi, hi) for _ in range(n)]
        if fits(ops, vals):
            break
        hi = max(1, hi * 3 // 4)
    print(n)
    print(" ".join(f"{'x' if o else 't'} {v}" for o, v in zip(ops, vals)))


if sys.argv[1] == "--max":
    emit(random.Random(0), 50, 0.15, 6)
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
n = rng.randint(3, max(3, min(50, size)))
emit(rng, n, rng.choice([0.0, 0.3, 0.6]), rng.choice([2, 10, 32767 // n]))
