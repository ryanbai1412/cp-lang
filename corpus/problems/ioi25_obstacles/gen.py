import sys, random
# IOI 2025 Obstacles for a Llama, sample-grader stdin: N M, T[0..N), H[0..M), Q, then Q lines
# L R S D with L <= S, D <= R and (0, S), (0, D) free (T[0] > H[S], H[D]); values 0..=1e9.
if sys.argv[1] == "--max":
    # Every row a new maximum and humidities spread so each row frees a few more columns:
    # deep layer chains and long lifting paths.
    r = random.Random(0)
    n = m = q = 200000
    t = sorted(r.sample(range(10**9 // 2, 10**9 + 1), n))
    h = [r.randrange(10**9) if r.random() < 0.9 else r.randrange(10**9 // 2) for _ in range(m)]
    sub = None
else:
    r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
    sub = r.choice(["n1", "sorted", "t213", "full", "full", "full", "full"])
    n = 1 if sub == "n1" else 3 if sub == "t213" else r.randint(1, S)
    m, q = r.randint(1, S), r.randint(1, S)
    hi = r.choice([3, 10, S, 10**9])
    if sub == "t213":
        t, hi = [2, 1, 3], 3
    else:
        t = [r.randint(0, hi) for _ in range(n)]
        if sub == "sorted": t.sort()
    h = [r.randint(0, hi) for _ in range(m)]
if min(h) >= t[0]:
    h[r.randrange(m)] = r.randint(0, t[0] - 1) if t[0] > 0 else 0
    if t[0] == 0: t[0] = 1
free = [j for j in range(m) if h[j] < t[0]]
print(n, m)
print(*t)
print(*h)
print(q)
whole = sub is not None and sub != "full"
for _ in range(q):
    s, d = r.choice(free), r.choice(free)
    a, b = min(s, d), max(s, d)
    if whole or r.random() < 0.2: lo, hi_ = 0, m - 1
    else: lo, hi_ = r.randint(max(0, a - r.choice([0, 1, 3, m])), a), r.randint(b, min(m - 1, b + r.choice([0, 1, 3, m])))
    print(lo, hi_, s, d)
