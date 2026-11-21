import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 3)
print(T)
for _ in range(T):
    n = rng.randint(1, size + 2)
    l = rng.randint(1, size + 2)
    k = rng.randint(0, min(5, size))
    print(n, l, k)
