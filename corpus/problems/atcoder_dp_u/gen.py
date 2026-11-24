import sys, random
if sys.argv[1] == "--max":
    r = random.Random(0)
    n = 16
    a = [[0] * n for _ in range(n)]
    for i in range(n):
        for j in range(i): a[i][j] = a[j][i] = r.randint(-10**9, 10**9)
    print(n)
    for row in a: print(*row)
    sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); m = r.choice([5, 10**9])
a = [[0] * n for _ in range(n)]
for i in range(n):
    for j in range(i): a[i][j] = a[j][i] = r.randint(-m, m)
print(n)
for row in a: print(*row)
