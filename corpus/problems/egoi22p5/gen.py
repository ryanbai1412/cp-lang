import sys, random
# statement: 1 <= n <= 1e5 centres, 0..=1e9 free each, 1 <= s <= 5000 services m c, always enough.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "100000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 100000
n = 100000 if big else r.randint(1, max(1, min(S, 8)))
s = 5000 if big else r.randint(1, 8)
a = [r.randint(10 ** 8, 10 ** 9) if big else r.randint(0, r.choice([5, 30])) for _ in range(n)]
cur = sorted(a, reverse=True)
ops = []
for _ in range(s):
    c = r.randint(1, n)
    if cur[c - 1] == 0:
        c = 1
    if cur[c - 1] == 0:
        cur[0] += 1  # keep a valid service: nothing to take is not allowed
        a[a.index(max(a))] += 1
    m = r.randint(1, max(1, min(cur[c - 1], 3 if big else cur[c - 1])))
    ops.append((m, c))
    top = [x - m for x in cur[:c]]
    cur = sorted(top + cur[c:], reverse=True)
print(n, s)
print(*a)
print("\n".join(f"{m} {c}" for m, c in ops))
