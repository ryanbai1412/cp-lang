import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(2, S + 3); print(n)
    p = list(range(1, n + 1)); r.shuffle(p)
    q = list(range(1, n + 1)); r.shuffle(q)
    if q < p: p, q = q, p
    print(*p)
    print(*q)
