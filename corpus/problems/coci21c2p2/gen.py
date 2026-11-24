import sys, random
# statement: 1 <= n, m <= 1000 permutations of 1..n, 1 <= q <= 5e5 queries a b (1 <= a, b <= n).
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "500000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
big = S >= 500000
n = 1000 if big else r.randint(1, max(1, min(S, 1000)))
m = 1000 if big else r.randint(1, max(1, min(S, 1000)))
q = 500000 if big else r.randint(1, max(1, min(S, 500000)))
print(n, m, q)
for _ in range(m):
    p = list(range(1, n + 1))
    # a few long cycles keep many classes apart
    k = r.randint(1, 4) if not big else 2
    idx = list(range(n))
    r.shuffle(idx)
    if r.random() < 0.8:
        for _ in range(k):
            i, j = r.sample(range(n), 2) if n > 1 else (0, 0)
            p[i], p[j] = p[j], p[i]
    else:
        r.shuffle(p)
    print(*p)
print("\n".join(f"{r.randint(1, n)} {r.randint(1, n)}" for _ in range(q)))
