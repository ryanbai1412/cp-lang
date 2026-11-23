import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 5)
print(T)
for _ in range(T):
    n = rng.randint(2, min(100, size * 4 + 2))
    print(n)
    print("".join(rng.choice("<>") for _ in range(n - 1)))
