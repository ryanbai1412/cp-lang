import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 5); print(t)
for _ in range(t): print(random.choice([1, 2, 3, 4, 7, 8, random.randint(1, 12)]))
