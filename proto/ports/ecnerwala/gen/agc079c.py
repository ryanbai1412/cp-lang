import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 8; print(t)
for _ in range(t):
    n = r.randint(1, S + 3); p = list(range(1, n + 1))
    if r.random() < 0.5: r.shuffle(p)
    else:
        for _ in range(r.randint(0, n)):   # a few reversals of a sorted run: more solvable cases
            i = r.randrange(n); j = r.randrange(n); i, j = min(i, j), max(i, j); p[i:j + 1] = p[i:j + 1][::-1]
    print(n); print(*p)
