import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 5); print(t)
for _ in range(t):
    w = random.choice([1, 1, 2, 3] + list(range(1, 13)))
    shape = random.random()
    if shape < 0.15: l = -100
    elif shape < 0.3: l = 101 - w
    elif shape < 0.75: l = random.randint(-w + 1, 0)
    else: l = random.randint(-100, 101 - w)
    print(l, l + w - 1)
