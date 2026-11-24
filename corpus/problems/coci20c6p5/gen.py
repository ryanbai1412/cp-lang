import sys, random
# statement: 1 <= n, q <= 2e5, 1 <= p_i <= 2e5, 1 <= l <= r <= n.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 200000
n = 200000 if big else r.randint(1, max(1, min(S, 200000)))
q = 200000 if big else r.randint(1, max(1, min(S, 200000)))
top = 200000 if big or r.random() < 0.5 else max(1, n)
print(n, q)
print(*(r.randint(1, top) for _ in range(n)))
out = []
for _ in range(q):
    l, rr = sorted((r.randint(1, n), r.randint(1, n)))
    out.append(f"{l} {rr}")
print("\n".join(out))
