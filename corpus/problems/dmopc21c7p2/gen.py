import sys, random
# statement: colours 1..N, N <= 1e6, Q <= 5e5 steps L R K: L..R is a segment of the current
# scarf and K (0 = the front) is outside it.
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S >= 10 ** 6:
    n, q = 10 ** 6, 5 * 10 ** 5
    print(n, q)
    out = []
    for _ in range(q):
        l = r.randint(1, n)
        k = r.choice([x for x in (r.randint(0, n), r.randint(0, n)) if x != l] or [0])
        out.append(f"{l} {l} {k}")
    print("\n".join(out))
    sys.exit()
n = r.randint(1, max(1, min(S, 10)))
q = r.randint(1, max(1, min(S, 10)))
scarf = list(range(1, n + 1))
print(n, q)
for _ in range(q):
    i = r.randrange(n); j = r.randrange(i, n)
    seg = scarf[i:j + 1]; rest = scarf[:i] + scarf[j + 1:]
    p = r.randint(0, len(rest))
    k = 0 if p == 0 else rest[p - 1]
    print(seg[0], seg[-1], k)
    scarf = rest[:p] + seg + rest[p:]
