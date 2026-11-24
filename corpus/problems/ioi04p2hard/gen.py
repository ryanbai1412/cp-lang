import sys, random
# statement: 1 <= n <= 1e6 junctions, -1e9 <= x, y <= 1e9.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 1000000
n = 1000000 if big else r.randint(1, max(1, min(S, 1000000)))
c = 10 ** 9 if big or r.random() < 0.5 else 10
print(n)
print("\n".join(f"{r.randint(-c, c)} {r.randint(-c, c)}" for _ in range(n)))
