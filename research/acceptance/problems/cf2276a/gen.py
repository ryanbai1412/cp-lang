import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 5); print(t)
for _ in range(t):
    shape = random.random()
    if shape < 0.2:
        a = random.randint(1, 20); b = random.choice([a, a - 1, a + 1, 1]) or 1
    elif shape < 0.35:
        a, b = random.randint(1, 100), random.randint(1, 3)
        if random.random() < 0.5: a, b = b, a
    else:
        a, b = random.randint(1, 20), random.randint(1, 20)
    print(a, b)
