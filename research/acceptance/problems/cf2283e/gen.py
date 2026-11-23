import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    n = random.choice([1, 1, 2, random.randint(1, 8), random.randint(5, 8)]); print(n)
    shape = random.random()
    if shape < 0.2: s = random.choice("01") * n
    elif shape < 0.4: s = "".join("01"[(i + (shape < 0.3)) % 2] for i in range(n))
    else:
        p = random.random(); s = "".join("1" if random.random() < p else "0" for _ in range(n))
    print(s)
