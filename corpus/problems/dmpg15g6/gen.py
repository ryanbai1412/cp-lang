import sys, random
# statement: 1 <= N, M, Q <= 2e5, flavours in 1..=M, queries 1 <= l <= r <= N.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "200000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 200000
n = 200000 if big else r.randint(1, max(1, min(S, 12)))
m = 200000 if big else r.randint(1, max(1, min(S, 5)))
q = 200000 if big else r.randint(1, 10)
print(n, m)
# at max two flavours near a third each, a third one less, spread over 1..=m, plus noise keep every summary
# full and the answers close to the threshold
def piece():
    if not big: return r.randint(1, m)
    x = r.random()
    return 1 if x < 0.35 else m // 2 if x < 0.684 else m if x < 0.9 else r.randint(1, m)
print(*(piece() for _ in range(n)))
print(q)
out = []
for _ in range(q):
    if big and r.random() < 0.5:
        l = r.randint(1, 1000); hi = r.randint(n - 1000, n)
    else:
        l = r.randint(1, n); hi = r.randint(l, n)
    out.append(f"{l} {hi}")
print("\n".join(out))
