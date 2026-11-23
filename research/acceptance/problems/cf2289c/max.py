import random
random.seed(1)
n = 200000
p = list(range(1, n + 1)); random.shuffle(p)
print(1); print(n); print(*p)
