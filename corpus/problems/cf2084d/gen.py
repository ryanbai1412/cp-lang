import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5 with m k just below n
    print(1)
    print(200_000, 447, 447)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 5 + 3)
    m = rng.randint(1, n - 1)
    k = rng.randint(1, (n - 1) // m)
    print(n, m, k)
