import sys, random
if sys.argv[1] == "--max":
    # one test with n = sum n = 3000, a path in shuffled labels: the longest layers per centre
    # and the deepest DFS
    rng = random.Random(1)
    n = 3000
    perm = list(range(1, n + 1)); rng.shuffle(perm)
    print(1); print(n)
    print("\n".join(f"{perm[i]} {perm[i + 1]}" for i in range(n - 1)))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 2 + 2)
    print(n)
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    for i in range(1, n):
        p = rng.randint(0, i - 1)
        print(perm[p], perm[i])
