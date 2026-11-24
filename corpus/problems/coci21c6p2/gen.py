import sys, random
# statement: 1 <= r, s <= 500, 1 <= a, b <= 1e9, prices 1 <= c <= 1e9.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "500"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 500
n = 500 if big else r.randint(1, max(1, min(S, 500)))
m = 500 if big else r.randint(1, max(1, min(S, 500)))
top = 2000 if big else r.choice([3, 10, 10 ** 9])
lim = 10 ** 9 if top == 10 ** 9 else max(1, top * n * m // 2)
a, b = 10 ** 8 if big else r.randint(1, lim), 10 ** 9 if big else r.randint(1, lim)
print(n, m, a, b)
for _ in range(n):
    print(*(r.randint(1, top) for _ in range(m)))
