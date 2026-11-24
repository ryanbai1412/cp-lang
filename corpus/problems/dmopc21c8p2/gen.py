import sys, random
# statement: a 2 x N grid, 2 <= N <= 1e6, M <= 1e6, cells 0 (unknown) or 1..=M, a filling may exist or not.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 6
n = 10 ** 6 if big else r.randint(2, max(2, min(S, 8)))
m = 10 ** 6 if big else r.randint(1, max(1, min(S, 6)))
s = [r.randint(2, 2 * m), r.randint(2, 2 * m)]
top, bot = [], []
for i in range(n):
    x = s[i % 2]
    t = r.randint(max(1, x - m), min(m, x - 1))
    b = x - t
    if r.random() < 0.1 and not big:
        t = r.randint(1, m)
    keep = r.random()
    top.append(t if keep < 0.4 or (keep > 0.8 and r.random() < 0.5) else 0)
    bot.append(b if 0.2 < keep < 0.6 else 0)
print(n, m)
print(*top)
print(*bot)
