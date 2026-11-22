import random, sys
random.seed(int(sys.argv[1])); t = random.randint(1, int(sys.argv[2])); print(t)
for _ in range(t):
    h = random.choice([3, 10**9]); p = [random.randint(-h, h) for _ in range(4)]
    k = random.randint(-3, 3); q = [p[0] + k * (p[2] - p[0]), p[1] + k * (p[3] - p[1])]
    if random.random() >= .3 or max(map(abs, q)) > h: q = [random.randint(-h, h) for _ in range(2)]
    if p[:2] == p[2:]: p[2] += 1
    print(*p, *q)
