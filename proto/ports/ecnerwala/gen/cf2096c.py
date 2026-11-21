import sys, random
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
