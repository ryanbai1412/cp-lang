import sys, random
MAX = sys.argv[1] == "--max"  # sums n = 1e5, m = 2e5 in one chain case: answer ~ n^2 / 2
r = random.Random(0 if MAX else int(sys.argv[1])); S = 10**6 if MAX else int(sys.argv[2])
# statement: T <= 5e4, 2 <= N <= 1e5, 1 <= M <= min(N(N-1), 2e5), simple digraph (2-cycles allowed),
# sums of N <= 1e5 and of M <= 2e5
MAX_N, MAX_M, MAX_T = 100_000, 200_000, 50_000

def case(n, m, shape):
    m = min(m, n * (n - 1) if shape == "mixed" else n * (n - 1) // 2 + (shape == "chain")); es = set()
    if shape == "chain":  # 2-cycle at 1, 2 then a path n -> n-1 -> ... -> 3 -> 1: answer ~ n^2 / 2
        es |= {(1, 2), (2, 1)} | {(v, v - 1) for v in range(4, n + 1)}
        if n >= 3: es.add((3, 1))
    while len(es) < m:
        u = r.randint(1, n); v = u - r.randint(1, 3) if shape == "chain" and r.random() < 0.8 else r.randint(1, n)
        if u == v or v < 1: continue
        if shape in ("dag", "chain") and u < v: u, v = v, u
        if shape == "mixed" and r.random() < 0.6 and u > v: u, v = v, u
        es.add((u, v))
    es = list(es)[:max(m, 1)] if shape != "chain" else list(es)
    r.shuffle(es)
    return n, es

shapes = ["mixed", "dag", "chain"]
if S < 1000:
    cases = []
    for _ in range(5):
        n = r.randint(2, 2 * S + 1); cases.append(case(n, r.randint(1, 3 * S), r.choice(shapes)))
else:
    k = 0 if MAX else r.randrange(5)
    if k == 0: cases = [case(MAX_N, MAX_M, "chain")]
    elif k == 1: cases = [case(MAX_N, MAX_M, "dag")]
    elif k == 2: cases = [case(MAX_N, MAX_M, "mixed")]
    elif k == 3: cases = [case(2, r.randint(1, 2), "mixed") for _ in range(MAX_T)]
    else: cases = [case(MAX_N, MAX_N, "chain")]  # only the path: answer ~ n^2 / 2 > 2^32
print(len(cases))
for n, es in cases:
    print(n, len(es))
    for u, v in es: print(u, v)
