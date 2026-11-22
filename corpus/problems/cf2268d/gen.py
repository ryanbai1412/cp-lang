import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 4 * S); p = list(range(1, n + 1)); r.shuffle(p); print(n); print(*p)
