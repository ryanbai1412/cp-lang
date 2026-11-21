import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 4)
print(T)
for _ in range(T):
    n = rng.randint(1, size * 2 + 2)
    l = rng.randint(1, size * 3 + 2)
    print(n, l, 0)
