import random, sys
random.seed(int(sys.argv[1])); s = int(sys.argv[2])
n = random.randint(1, s); M = random.choice([10, 100, 10**9])
print(n)
for _ in range(n):
    a = random.randint(1, M - 1); b = random.randint(a + 1, M); print(a, b)
