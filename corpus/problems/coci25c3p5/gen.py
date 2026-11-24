import sys, random
# statement: 1 <= n <= 1e5 columns, 1 <= a_i, b_i <= 1e9.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 100000
n = 100000 if big else r.randint(1, max(1, min(S, 100000)))
top = 10 ** 9 if big else r.choice([10, 1000, 10 ** 9])
a, b = [], []
for _ in range(n):
    if r.random() < 0.5:
        q = r.randint(1, min(top, 1000))
        p = r.randint(1, max(1, top // q))
        a.append(p * q); b.append(q)
    else:
        a.append(r.randint(1, top)); b.append(r.randint(1, top))
print(n)
print(*a)
print(*b)
