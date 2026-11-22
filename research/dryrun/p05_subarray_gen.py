import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); M = random.choice([2, 5, 10**9])
print(n, random.randint(1, min(M * 3, 10**9))); print(*[random.randint(1, M) for _ in range(n)])
