import sys, random
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
