import sys, random
# usage: gen.py seed size; a hidden case for the interactor: "N K Q T", then per tree its
# parents (-1 at the root) and its edge weights (0 at the root), then T pairs of indices
# 1..=K into the chosen labels. size >= 1e6 gives N = 1e6, K = T = 1e5.
# --max: N = 1e6, K = T = 1e5, tree 1 a star and tree 2 a path of weight-2000 edges (distances near 2e9).
if sys.argv[1] == "--max":
    sys.argv[1:] = ["0", "1000000"]
r = random.Random(int(sys.argv[1])); S = int(sys.argv[2])
if S >= 1000000: n, k = 1000000, 100000
else:
    n = r.randint(2, max(2, S)); k = r.randint(2, n)
q, t = k - 1, (100000 if S >= 1000000 else r.randint(1, min(k * k, 100000, 3 * S)))
hi = 2000 if S >= 1000000 or r.random() < 0.5 else r.randint(1, 2000)
# at full size a seed divisible by 3 puts a long chosen path in tree 2: distances near 2e9.
wide = S >= 1000000 and int(sys.argv[1]) % 3 == 0
def tree(i):
    shape = ["star", "path"][i] if wide else r.choice(["random", "path", "star", "caterpillar", "shallow"])
    order = list(range(1, n + 1)); r.shuffle(order)
    par = [-1] * (n + 1)
    for i in range(1, n):
        if shape == "path": j = i - 1
        elif shape == "star": j = 0
        elif shape == "caterpillar": j = i - 1 if i % 2 else max(0, i - 2)
        elif shape == "shallow": j = r.randint(0, min(i - 1, 30))
        else: j = r.randint(max(0, i - r.choice([1, 3, i])), i - 1)
        par[order[i]] = order[j]
    w = [0 if par[v] == -1 else (hi if r.random() < 0.5 else r.randint(1, hi)) for v in range(1, n + 1)]
    return par[1:], w
print(n, k, q, t)
for i in range(2):
    par, w = tree(i); print(*par); print(*w)
for _ in range(t): print(r.randint(1, k), r.randint(1, k))
