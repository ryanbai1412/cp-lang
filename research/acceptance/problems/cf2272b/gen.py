import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    n = random.choice([1, 2, 3, 3, 4]); cols = [random.choice(["fo", "of", "ff"]) for _ in range(n)]
    print(n); print("".join(c[0] for c in cols)); print("".join(c[1] for c in cols))
