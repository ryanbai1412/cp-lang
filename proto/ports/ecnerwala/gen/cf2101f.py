import sys, random
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
