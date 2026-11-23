import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 4); print(t)
for _ in range(t):
    n = random.choice([1, 2, 3, 4, random.randint(4, 14), random.randint(10, 14)])
    alpha = random.choice(["0256", "02567", "0123456789", "25", "2"])
    s = "".join(random.choice(alpha) for _ in range(n))
    if n >= 4 and random.random() < 0.4:
        p = random.randint(0, n - 4); s = s[:p] + random.choice(["2025", "2026"]) + s[p + 4:]
    print(n); print(s)
