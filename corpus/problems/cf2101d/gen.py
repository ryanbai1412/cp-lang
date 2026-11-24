import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5: a random permutation
    rng = random.Random(1)
    n = 200_000
    p = list(range(1, n + 1))
    rng.shuffle(p)
    print(1)
    print(n)
    print(*p)
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(1, size * 4 + 2)
    p = list(range(1, n + 1))
    rng.shuffle(p)
    print(n)
    print(*p)
