import random, sys
random.seed(int(sys.argv[1])); n = random.randint(1, min(10, int(sys.argv[2])))
x = random.randint(1, 30); print(n, x); print(*[random.randint(1, x) for _ in range(n)])
