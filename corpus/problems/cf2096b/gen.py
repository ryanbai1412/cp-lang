import sys, random
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
