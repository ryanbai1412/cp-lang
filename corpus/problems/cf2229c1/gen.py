import sys, random
if sys.argv[1] == "--max":
    # one n = 2e5 test (sum n): signs alternate so every suffix flip lands on a positive
    # value and the answer prints n operations; values near the 1e9 bound
    g = random.Random(1)
    n = 200_000
    print(1)
    print(n)
    print(*[g.randint(10**9 - 1000, 10**9) * (1 if i % 2 else -1) for i in range(n)])
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S + 2); print(n)
    vals = [r.choice([-1, 1]) * r.randint(1, 10**9) for _ in range(n)]
    print(*vals)
