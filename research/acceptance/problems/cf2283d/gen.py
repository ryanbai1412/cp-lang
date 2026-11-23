import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    n = random.randint(3, 9); print(n)
    shape = random.choice(["random", "path", "star", "caterpillar"])
    lab = list(range(1, n + 1))
    if random.random() < 0.7: random.shuffle(lab)
    for v in range(1, n):
        p = {"random": random.randint(0, v - 1), "path": v - 1, "star": 0, "caterpillar": max(0, v - random.randint(1, 2))}[shape]
        print(lab[p], lab[v]) if random.random() < 0.5 else print(lab[v], lab[p])
