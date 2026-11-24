import sys, random
if sys.argv[1] == "--max":
    # one test with n = sum n = 7e4, every s_i = 1, a path in shuffled labels: distances up to
    # n - 1, so the most doubling rounds, and the full centroid depth
    rng = random.Random(1)
    n = 70_000
    perm = list(range(1, n + 1)); rng.shuffle(perm)
    print(1); print(n); print("1" * n)
    print("\n".join(f"{perm[i]} {perm[i + 1]}" for i in range(n - 1)))
    sys.exit()
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 3 + 2)
    print(n)
    print("".join(rng.choice("01") for _ in range(n)))
    perm = list(range(1, n + 1))
    rng.shuffle(perm)
    for i in range(1, n):
        p = rng.randint(0, i - 1)
        print(perm[p], perm[i])
