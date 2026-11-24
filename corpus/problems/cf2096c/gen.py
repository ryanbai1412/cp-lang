import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 1000, heights near 1e9 and costs near 1e9 (answers near 1e12): row and
    # column offsets step by -1, 0, 1 or 5, so many neighbour pairs force a raise
    rng = random.Random(1)
    n = 1000
    def offsets():
        o = [0]
        for _ in range(n - 1):
            o.append(o[-1] + rng.choice((-1, 0, 0, 1, 5)))
        return o
    r, c = offsets(), offsets()
    base = 10**9 - max(r) - max(c)
    print(1)
    print(n)
    for i in range(n):
        print(*[base + r[i] + c[j] for j in range(n)])
    print(*[rng.randint(5 * 10**8, 10**9) for _ in range(n)])
    print(*[rng.randint(5 * 10**8, 10**9) for _ in range(n)])
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 2 + 2)
    print(n)
    for _ in range(n):
        print(*[rng.randint(1, 8) for _ in range(n)])
    print(*[rng.randint(1, 100) for _ in range(n)])
    print(*[rng.randint(1, 100) for _ in range(n)])
