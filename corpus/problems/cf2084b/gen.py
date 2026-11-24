import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 1e5: a small minimum g and multiples of it up to 1e18
    rng = random.Random(1)
    n = 100_000
    g = 6
    arr = [g] + [g * rng.randint(1, 10**18 // g) for _ in range(n - 1)]
    rng.shuffle(arr)
    print(1)
    print(n)
    print(*arr)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 4 + 2)
    print(n)
    if rng.random() < 0.5:
        g = rng.randint(1, 10)
        arr = [g] + [g * rng.randint(1, 12) for _ in range(n - 1)]
        rng.shuffle(arr)
    else:
        arr = [rng.randint(1, 30) for _ in range(n)]
    print(*arr)
