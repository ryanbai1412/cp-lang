import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    n = r.randint(1, S + 2); print(n)
    a = [-1 if r.random() < 0.65 else r.randint(0, 2) for _ in range(n)]
    print(*a)
