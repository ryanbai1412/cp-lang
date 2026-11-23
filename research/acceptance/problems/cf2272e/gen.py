import random, sys
random.seed(int(sys.argv[1]))
t = random.randint(1, 3); print(t)
for _ in range(t):
    for _ in range(1):
        n, m = random.randint(1, 5), random.randint(1, 5)
        s = [random.choice("RW") for _ in range(n)]; u = [random.choice("RW") for _ in range(m)]
        s[random.randrange(n)] = "R"; u[random.randrange(m)] = "R"
        print(n, m); print("".join(s)); print("".join(u))
