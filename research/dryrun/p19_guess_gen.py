import random, sys
random.seed(int(sys.argv[1])); n = random.choice([1, 2, 3, random.randint(1, 100), random.randint(1, 10**9), 10**9])
print(n, random.choice([1, n, random.randint(1, n)]))
