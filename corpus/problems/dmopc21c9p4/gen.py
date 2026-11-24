import sys, random
# statement: N, M odd, 1 <= N, M <= 1000 (so N * M <= 1e6); each coin H or T.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 10 ** 6
def odd(hi): return r.randrange(1, hi + 1, 2)
n = 999 if big else odd(max(1, min(999, S)))
m = 999 if big else odd(max(1, min(999, S)))
p = r.choice([0.5, 0.1, 0.9, 1.0])
print(n, m)
for _ in range(n): print("".join("H" if r.random() < p else "T" for _ in range(m)))
