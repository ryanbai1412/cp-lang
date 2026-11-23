import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    shape = random.randint(0, 4)
    if shape == 0: l = r = random.randint(0, 5)
    elif shape == 1: l = random.randint(0, 40); r = random.randint(l, 60)
    elif shape == 2:  # around 2^a - 1 and 2^a - 2, the cheap values
        c = (1 << random.randint(1, 11)) - random.randint(0, 3); l = max(0, c - random.randint(0, 3)); r = c + random.randint(0, 3)
    elif shape == 3: l = random.randint(0, 2000); r = min(3000, l + random.randint(0, 50))
    else: l = random.randint(0, 3000); r = l
    print(l, r)
