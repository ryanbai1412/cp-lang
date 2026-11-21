import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 4)
print(t)
for _ in range(t):
    n = r.randint(1, 4 * S)
    hi = r.choice([1, 3, 7, 10**9])
    pool = [r.randint(1, hi) for _ in range(r.randint(1, 4))]
    a = [r.choice(pool) if r.random() < 0.7 else r.randint(1, hi) for _ in range(n)]
    print(n)
    print(*a)
