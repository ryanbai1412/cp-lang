import sys, random
# statement: 2 <= n, m <= 100; '.' and '#', no '#' in the last row or column.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 6
n = 100 if big else r.randint(2, max(2, min(S, 100)))
m = 100 if big else r.randint(2, max(2, min(S, 100)))
p = 0.5 if big else r.random()
print(n, m)
for i in range(n):
    print("".join("#" if i < n - 1 and j < m - 1 and r.random() < p else "." for j in range(m)))
