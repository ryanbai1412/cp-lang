import sys, random

if sys.argv[1] == "--max":
    r = random.Random(1)
    print(1)
    print(10**6)
    print(*[r.randint(0, 10**9) for _ in range(10**6)])
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
t = r.randint(1, 5)
print(t)
for _ in range(t):
    n = r.randint(1, max(1, S))
    hi = r.choice([1, 3, 20, 10**9])
    print(n)
    print(*[r.randint(0, hi) if r.random() < 0.7 else 0 for _ in range(n)])
