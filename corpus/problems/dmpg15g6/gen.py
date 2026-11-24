import sys, random
# statement: 1 <= N, M, Q <= 2e5, flavours in 1..=M, queries 1 <= l <= r <= N.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 200000
n = 200000 if big else r.randint(1, max(1, min(S, 12)))
m = 3 if big else r.randint(1, max(1, min(S, 5)))
q = 200000 if big else r.randint(1, 10)
print(n, m)
print(*(r.randint(1, m) for _ in range(n)))
print(q)
out = []
for _ in range(q):
    if big:
        l = r.randint(1, 1000); hi = r.randint(n - 1000, n)
    else:
        l = r.randint(1, n); hi = r.randint(l, n)
    out.append(f"{l} {hi}")
print("\n".join(out))
