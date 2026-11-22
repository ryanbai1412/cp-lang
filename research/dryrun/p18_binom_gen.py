import random, sys
random.seed(int(sys.argv[1])); n = random.randint(1, int(sys.argv[2])); print(n)
for _ in range(n):
    a = random.choice([random.randint(1, 20), random.randint(1, 10**6)]); print(a, random.randint(0, a))
