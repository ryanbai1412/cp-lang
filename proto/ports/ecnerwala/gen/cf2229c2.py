import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S + 2); print(n)
    vals = [r.choice([-1, 1]) * r.randint(1, 10**9) for _ in range(n)]
    print(*vals)
