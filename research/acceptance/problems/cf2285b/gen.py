import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 5); print(t)
for _ in range(t):
    n = random.randint(1, 7); print(n, random.randint(0, n)); print("".join(random.choice("01") for _ in range(n)))
