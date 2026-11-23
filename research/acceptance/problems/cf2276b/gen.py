import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 5); print(t)
for _ in range(t):
    n = random.choice([3, 3, 4, random.randint(3, 12)])
    shape = random.random()
    if shape < 0.2:
        print(random.choice("01") * n)
    elif shape < 0.35:
        c = random.randint(0, 1); print("".join(str((c + i) % 2) for i in range(n)))
    else:
        p = random.random(); print("".join("1" if random.random() < p else "0" for _ in range(n)))
