import sys, random
rng = random.Random(int(sys.argv[1]))
size = int(sys.argv[2])
T = rng.randint(1, 6)
print(T)
for _ in range(T):
    print(rng.randint(2, min(100, size * 5 + 2)))
