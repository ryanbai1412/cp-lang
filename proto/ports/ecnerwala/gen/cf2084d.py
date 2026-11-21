import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, size * 5 + 3)
    m = rng.randint(1, n - 1)
    k = rng.randint(1, (n - 1) // m)
    print(n, m, k)
