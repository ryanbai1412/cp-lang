import sys, random
if sys.argv[1:] == ["--max"]:  # ten tests, two at n = m = 5e4 and k = 2e4
    sys.argv[1:] = ["1", "50000"]
# seed size: n, m <= size (capped at 5e4), k <= 2e4; T <= 10 tests, at most 2 with n or m > 5000
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
T = r.randint(1, 10) if S <= 5000 else 10
print(T)
for t in range(T):
    big = S > 5000 and t < 2
    cap = min(S, 50000) if big else min(S, 5000)
    n = cap if big else r.randint(2, max(2, cap))
    chain = big and t == 1  # a 1 -> 2 -> ... -> n path at max cost and max LCP: sums near 2^31
    m = cap if big else r.randint(n - 1, max(n - 1, cap))
    m = n - 1 if chain else max(m, n - 1)
    k = 20000 if big else r.randint(1, max(1, min(S, 20000)))
    shape = 1 if chain else r.randint(0, 2)  # trie: random, path (deep LCPs), star
    tr = []
    for v in range(2, k + 1):
        u = v - 1 if shape == 1 else (1 if shape == 2 else r.randint(max(1, v - 5), v - 1) if r.random() < .5 else r.randint(1, v - 1))
        tr.append((u, v))
    # chars distinct among each node's children: use child id mod 2e4 + 1
    cmax = 20000 if r.random() < .5 else 1
    edges = []
    for i in range(2, n + 1):
        edges.append((i - 1 if chain else r.randint(1, i - 1), i))
    while len(edges) < m:
        edges.append((r.randint(1, n), r.randint(1, n)))
    perm = list(range(1, n + 1)) if chain else [1] + r.sample(range(2, n + 1), n - 1)
    print(n, m, k)
    lo_c = r.choice([0, 0, 1])
    for a, b in edges:
        c = 20000 if chain else r.choice([0, cmax, r.randint(lo_c, cmax)])
        print(perm[a - 1], perm[b - 1], c, k if chain else r.randint(1, k))
    tperm = list(range(1, k + 1)) if chain else [1] + r.sample(range(2, k + 1), k - 1)
    for u, v in tr:
        print(tperm[u - 1], tperm[v - 1], (v % 20000) + 1)
