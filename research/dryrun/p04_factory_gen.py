import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); M = random.choice([3, 100, 10**9])
t = random.randint(1, random.choice([10, 10**9]))
print(n, t); print(*[random.randint(1, M) for _ in range(n)])
