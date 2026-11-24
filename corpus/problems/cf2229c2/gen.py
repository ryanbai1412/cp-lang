import sys, random
if sys.argv[1] == "--max":
    # one test at sum n: alternating signs at |a_i| near 1e9 and a final +1 pivot, so the sweep
    # operates on about every other index
    g = random.Random(1)
    n = 200_000
    a = [g.randint(9 * 10**8, 10**9) * (1 if i % 2 else -1) for i in range(n - 1)] + [1]
    print(1)
    print(n)
    print(*a)
    sys.exit()

r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 5; print(t)
for _ in range(t):
    n = r.randint(1, 3 * S + 2); print(n)
    vals = [r.choice([-1, 1]) * r.randint(1, 10**9) for _ in range(n)]
    print(*vals)
