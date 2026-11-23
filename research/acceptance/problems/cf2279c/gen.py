import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    n = random.randint(2, 5)
    v = {2: 12, 3: 8, 4: 6, 5: 4}[n]
    shape = random.randint(0, 3)
    if shape == 0: a = [random.randint(1, v)] * n
    elif shape == 1: a = [random.choice((1, v)) for _ in range(n)]
    else: a = [random.randint(1, v) for _ in range(n)]
    l = random.choice((1, 1, 2, v * v, v * v + 1, random.randint(1, v * v + 2)))
    print(n, l); print(*a)
