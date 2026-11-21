import random, sys

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(1, max(1, 2 * size + 2))
    pairs = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    rng.shuffle(pairs)
    m = rng.randint(0, len(pairs))
    mult = rng.choice([1, 1, 1, 2, 3, 6])
    print(n, m)
    for u, v in pairs[:m]:
        w = rng.randint(1, 20) * mult
        print(u, v, w)
