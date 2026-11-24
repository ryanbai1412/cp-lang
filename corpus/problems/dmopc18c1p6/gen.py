import sys, random
# statement: 1 <= n <= 1000 numbers of exactly 1 <= m <= 50 bits.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 1000
n = 1000 if big else r.randint(1, max(1, min(1000, S)))
m = 50 if big else r.randint(1, 8)
q = r.choice([0.1, 0.5, 0.9])
rows = sorted(["".join("1" if r.random() < q else "0" for _ in range(m)) for _ in range(n)])
rows = ["".join(ch if r.random() > 0.15 else "01"[ch == "0"] for ch in row) for row in rows]
print(n, m)
print("\n".join(rows))
