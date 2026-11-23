import random
random.seed(1)
n = m = 200000
print(1); print(n, m); print("".join(random.choice("RW") for _ in range(n))); print("".join(random.choice("RW") for _ in range(m)))
