import sys, random
if sys.argv[1] == "--max":
    print(1); print(100, 5000, 25)  # one test: n = 100, l = sum l = 5000, k = 25
    sys.exit()
seed, size = int(sys.argv[1]), int(sys.argv[2])
rng = random.Random(seed)
if size <= 20:
    T = rng.randint(1, 3)
    print(T)
    for _ in range(T):
        print(rng.randint(1, size + 2), rng.randint(1, size + 2), rng.randint(0, min(25, size)))
else:
    # sum l = 5000 split over T cases, n = 100 and k = 25 for the most work.
    T = [1, 2, 5, 50][seed % 4]
    print(T)
    for _ in range(T):
        print(100, 5000 // T, 25 if seed % 5 else rng.randint(0, 25))
