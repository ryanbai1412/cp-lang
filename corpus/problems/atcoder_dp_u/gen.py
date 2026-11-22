import sys, random
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
n = r.randint(1, S + 2); m = r.choice([5, 10**9])
a = [[0] * n for _ in range(n)]
for i in range(n):
    for j in range(i): a[i][j] = a[j][i] = r.randint(-m, m)
print(n)
for row in a: print(*row)
