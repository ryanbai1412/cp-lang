import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5, k = n, counts up to 1e9 (answers near 4e14)
    rng = random.Random(1)
    n = 200_000
    print(1)
    print(n, n)
    print(*[rng.randint(10**8, 10**9) for _ in range(n)])
    print(*[rng.randint(10**8, 10**9) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, size * 4 + 2)
    k = rng.randint(1, n)
    print(n, k)
    print(*[rng.randint(1, 50) for _ in range(n)])
    print(*[rng.randint(1, 50) for _ in range(n)])
