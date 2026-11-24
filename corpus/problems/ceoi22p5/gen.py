import sys, random
if sys.argv[1:] == ["--max"]:
    # N = M = 2e5, D = 1e9, positions spread over 1..=1e9 in random order, so every insertion
    # lands mid-segment and the answers reach about 2e14
    g = random.Random(1)
    n = m = 200_000
    xs = [g.randint(1, 10**9) for _ in range(n + m)]
    print(n, m, 10**9)
    print(*xs[:n])
    print(*xs[n:])
    sys.exit()

rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
seed = int(sys.argv[1])
if size <= 20:
    n = rng.randint(0, size)
    m = rng.randint(1, size + 1)
    d = rng.randint(1, 6)
    hi = rng.choice([3, 10, 30])
    xs = [rng.randint(1, hi) for _ in range(n + m)]
else:
    mode = seed % 4
    n, m = [(0, size), (size, 10), (size, size), (0, size)][mode]
    d = 10**9 if mode != 1 else rng.randint(1, 10**9)
    if mode == 0:
        xs = [10**9] * (n + m)  # all on one spot: largest answers
    elif mode == 3:
        xs = sorted(rng.randint(1, 10**9) for _ in range(n + m))
    else:
        xs = [rng.randint(1, 10**9) for _ in range(n + m)]
print(n, m, d)
print(*xs[:n])
print(*xs[n:])
