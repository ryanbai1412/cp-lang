import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, min(20, S)); m = r.randint(1, min(30, S))
dense = r.random()
print(n, m)
for i in range(n):
    for j in range(m):
        score = r.choice([r.randint(-10**4, 10**4), r.randint(-20, 20)])
        k = r.randint(0, 3) if r.random() < dense else 0
        cells = [(r.randrange(n), r.randrange(m)) for _ in range(k)]
        print(score, k, *[x for c in cells for x in c])
