import random, sys

if sys.argv[1] == "--max":
    # One test at the limit (the sums of n and of m over tests are at most 500). Values
    # in [-10**6, 0] make every element its own cool segment, the heaviest shape tried.
    r = random.Random(0)
    print(1); print(500, 500)
    print(*[r.randint(-10**6, 0) for _ in range(500)])
    print(*[r.randint(-10**6, 0) for _ in range(500)])
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
t = rng.randint(1, 5)
print(t)
for _ in range(t):
    n = rng.randint(1, max(1, size + 3))
    m = rng.randint(1, max(1, size + 3))
    lo, hi = rng.choice([(-5, 5), (-10, 10), (-100, 100), (-5, 0)])
    a = [rng.randint(lo, hi) for _ in range(n)]
    b = [rng.randint(lo, hi) for _ in range(m)]
    print(n, m)
    print(*a)
    print(*b)
