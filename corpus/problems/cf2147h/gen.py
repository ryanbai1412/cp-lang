import random, sys

if sys.argv[1:] == ["--max"]:
    # sum n^4 <= 50^4: one complete graph on 50 vertices, capacities near 1e6 with gcd 1 and
    # about half of them odd, so the two-colour split does real work.
    rng = random.Random(2147)
    n = 50
    pairs = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    rng.shuffle(pairs)
    print(1)
    print(n, len(pairs))
    for u, v in pairs:
        print(u, v, rng.randint(900_000, 1_000_000))
    sys.exit()

seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
t = rng.randint(1, 3)
print(t)
for _ in range(t):
    n = rng.randint(1, min(50, 2 * size + 2))
    pairs = [(u, v) for u in range(1, n + 1) for v in range(u + 1, n + 1)]
    rng.shuffle(pairs)
    m = rng.randint(0, len(pairs))
    mult = rng.choice([1, 1, 1, 2, 3, 6])
    print(n, m)
    for u, v in pairs[:m]:
        w = rng.randint(1, 20) * mult
        print(u, v, w)
