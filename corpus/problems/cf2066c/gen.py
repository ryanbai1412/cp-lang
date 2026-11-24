import sys, random

if sys.argv[1] == "--max":
    # one test at sum n = 2e5: values from a small pool (prefix xors repeat, so ways[] entries
    # grow) mixed with values up to 1e9 (many distinct keys)
    r = random.Random(1)
    n = 200_000
    pool = [r.randint(1, 10**9) for _ in range(4)]
    print(1)
    print(n)
    print(*[r.choice(pool) if r.random() < 0.7 else r.randint(1, 10**9) for _ in range(n)])
    sys.exit()

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
