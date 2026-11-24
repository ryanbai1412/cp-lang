import sys, random

if sys.argv[1] == "--max":
    # C = 5 queues of n = 1000, tolerance 7 everywhere (the most window states), tastes up to 1000
    rng = random.Random(1)
    print(5)
    for _ in range(5):
        print(1000)
        print("\n".join(f"{rng.randint(0, 1000)} 7" for _ in range(1000)))
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size <= 20:
    c = rng.randint(1, 5)
    print(c)
    for _ in range(c):
        n = rng.randint(1, size + 1)
        hi = rng.choice([1, 7, 1000])
        print(n)
        for _ in range(n):
            print(rng.randint(0, hi), rng.randint(0, 7))
else:
    # Five full queues; patience 7 everywhere gives the most states.
    print(5)
    for t in range(5):
        print(1000)
        for _ in range(1000):
            print(rng.randint(0, 1000), 7 if seed % 2 == 0 or t == 0 else rng.randint(0, 7))
