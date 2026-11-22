import random, sys

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
