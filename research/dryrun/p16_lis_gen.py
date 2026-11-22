import random, sys
random.seed(int(sys.argv[1])); n = random.randint(1, int(sys.argv[2])); hi = random.choice([3, 10, 10**9])
print(n); print(*[random.randint(1, hi) for _ in range(n)])
