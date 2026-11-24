import sys, random
if sys.argv[1] == "--max":  # sum n^3 <= 500^3: one case n = 500, every a_i unknown (-1)
    print(1); print(500); print(*[-1] * 500); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    n = r.randint(1, S + 2); print(n)
    a = [-1 if r.random() < 0.65 else r.randint(0, 2) for _ in range(n)]
    print(*a)
