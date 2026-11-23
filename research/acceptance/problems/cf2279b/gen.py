import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.choice([1, 2, 3, random.randint(1, 12), random.randint(8, 12)])
    shape = random.random()
    if shape < 0.3:
        lo = random.randint(1, 2 * n + 1); a = list(range(lo, lo + n))
    elif shape < 0.6:
        a = [1]
        while len(a) < n: a.append(a[-1] + random.choice([1, 2, 2, 3]))
        shift = random.randint(0, 3 * n - a[-1]); a = [x + shift for x in a]
    else:
        a = sorted(random.sample(range(1, 3 * n + 1), n))
    print(n); print(*a)
