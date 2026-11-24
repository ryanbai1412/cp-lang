"""gen.py SEED SIZE | --max: DMOPC '20 Contest 5 P6 (TypeRacer substrings, online).
1 <= |S| <= 1e5, 1 <= Q <= 5e5, 1 <= l <= r <= |S|, lowercase S. Rounds are XOR-encrypted
with the previous answer, so the generator computes every answer itself: the sum over the
distinct substrings of S[l..r] of |T| + 1 is the closed form for all (start, length) pairs
minus the pairs (j, L) whose next occurrence, ending at e, still fits (l <= j, e <= r),
counted offline with a Fenwick tree. Small sizes include unary, periodic and Fibonacci
strings; --max is |S| = 1e5 over four letters with Q = 5e5 rounds, half of them long."""
import random, sys

MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
size = 10**9 if MAX else max(1, int(sys.argv[2]))
n = 10**5 if MAX else r.randint(1, min(10**5, size))
q = 5 * 10**5 if MAX else r.randint(1, min(5 * 10**5, size))
kind = "random4" if MAX else r.choice(["random1", "random2", "random3", "random26", "periodic", "fib"])
if kind in ("random1", "periodic", "fib"):
    n = min(n, 2000)
if kind.startswith("random"):
    k = int(kind[6:])
    s = "".join(r.choice("abcdefghijklmnopqrstuvwxyz"[:k]) for _ in range(n))
elif kind == "periodic":
    w = "".join(r.choice("abc") for _ in range(r.randint(1, 6)))
    s = (w * n)[:n]
else:
    a, b = "a", "ab"
    while len(b) < n:
        a, b = b, b + a
    s = b[:n]

# points (j, e, weight): s[j..j+L-1] occurs again starting at its next occurrence, ending at e
pts = []
ids = [ord(c) for c in s]
alive = list(range(n))
L = 1
while alive:
    nxt_alive = []
    last = {}
    for j in reversed(alive):
        key = ids[j]
        if key in last:
            pts.append((last[key] + L - 1, j, L + 1))
        last[key] = j
    groups = {}
    for j in alive:
        groups[ids[j]] = groups.get(ids[j], 0) + 1
    for j in alive:
        if groups[ids[j]] >= 2 and j + L < n:
            nxt_alive.append(j)
    new = {}
    for j in nxt_alive:
        ids[j] = new.setdefault((ids[j], s[j + L]), len(new))
    alive = nxt_alive
    L += 1

rounds = []
for _ in range(q):
    if MAX and r.random() < 0.5:
        lo = r.randint(1, n // 10)
        rounds.append((lo, r.randint(n - n // 10, n)))
    else:
        x, y = r.randint(1, n), r.randint(1, n)
        rounds.append((min(x, y), max(x, y)))

order = sorted(range(q), key=lambda i: rounds[i][1])
pts.sort()
tree = [0] * (n + 1)
ans = [0] * q
p = 0
for i in order:
    lo, hi = rounds[i]
    while p < len(pts) and pts[p][0] + 1 <= hi:
        j = pts[p][1] + 1
        while j <= n:
            tree[j] += pts[p][2]
            j += j & -j
        p += 1
    sub, j = 0, n
    while j > 0:
        sub += tree[j]
        j -= j & -j
    j = lo - 1
    while j > 0:
        sub -= tree[j]
        j -= j & -j
    m = hi - lo + 1
    ans[i] = (m * (m + 1) * (2 * m + 1) // 6 + 3 * m * (m + 1) // 2) // 2 - sub

out = [s, str(q)]
last = 0
for i in range(q):
    lo, hi = rounds[i]
    out.append(f"{lo ^ last} {hi ^ last}")
    last = ans[i]
print("\n".join(out))
