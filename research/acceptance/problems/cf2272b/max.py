import random
random.seed(1)
n = 200000
cols = [random.choice(["fo", "of", "ff"]) for _ in range(n)]
print(1); print(n); print("".join(c[0] for c in cols)); print("".join(c[1] for c in cols))
