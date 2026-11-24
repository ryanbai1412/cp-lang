import sys, random
if sys.argv[1] == "--max":
    # one case N = 2.5e5 (sum N): a path on shuffled labels with weights falling along it up to
    # 1e12, so the Kruskal reconstruction tree is a chain of depth N.
    r = random.Random(0); n = 250000
    lab = list(range(1, n + 1)); r.shuffle(lab)
    w = sorted(r.sample(range(1, 10**12 + 1), n - 1), reverse=True)
    out = ["1", str(n)] + [f"{lab[i]} {lab[i + 1]} {w[i]}" for i in range(n - 1)]
    print("\n".join(out)); sys.exit()
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
t = 4; print(t)
for _ in range(t):
    n = r.randint(2, 2 * S + 2); print(n)
    wmax = r.choice([10, 1000, 10**9])
    for v in range(2, n + 1):
        print(r.randint(1, v - 1), v, r.randint(1, wmax))
