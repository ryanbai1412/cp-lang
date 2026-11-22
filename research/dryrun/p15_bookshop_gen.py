import random, sys
random.seed(int(sys.argv[1])); n = random.randint(1, min(12, int(sys.argv[2])))
print(n, random.randint(1, 60)); print(*[random.randint(1, 20) for _ in range(n)]); print(*[random.randint(1, 1000) for _ in range(n)])
