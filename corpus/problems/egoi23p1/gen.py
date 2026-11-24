import sys, random
# statement: 1 <= N <= 3e5 prices 1..=1e6, 1 <= Q <= 1e5 days "INFLATION x" or "SET x y", 1 <= x, y <= 1e6.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "300000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 300000
n = 300000 if big else r.randint(1, max(1, min(S, 300000)))
q = 100000 if big else r.randint(1, max(1, min(S, 100000)))
top = 10 ** 6 if big else r.choice([5, 20, 10 ** 6])
p = [r.randint(1, top) for _ in range(n)]
print(n)
print(*p)
print(q)
out = []
for _ in range(q):
    if r.random() < 0.3:
        out.append(f"INFLATION {r.randint(1, 3 if top < 10 ** 6 else 10 ** 6)}")
    else:
        out.append(f"SET {r.randint(1, top)} {r.randint(1, top)}")
print("\n".join(out))
