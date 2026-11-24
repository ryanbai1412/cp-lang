import sys, random
# seed size: n <= size + 2 nodes, K in 1..3, parents drawn so the tree is random, binary
# (property A), a broom, a caterpillar, a star or a heap; --max: n = 1000, K = 3, the heap
# f_i = i / 2 (500 leaves, full leaf windows at every merge, answer nonzero)
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])
n = 1000 if mx else r.randint(2, min(1000, S + 2))
K = 3 if mx else r.randint(1, 3)
kind = "heap" if mx else r.choice(["rand", "binary", "broom", "comb", "star", "heap"])
f, kids = [], [0] * (n + 1)
for i in range(1, n):
    if kind == "rand": p = r.randint(1, i)
    elif kind == "binary":
        p = r.choice([j for j in range(max(1, i - 6), i + 1) if kids[j] < 2] or [i])
    elif kind == "broom": p = i if i <= n // 3 else r.randint(max(1, n // 3), i)
    elif kind == "comb": p = r.randint(max(1, i - 3), i)
    elif kind == "star": p = 1
    else: p = (i + 1) // 2
    kids[p] += 1; f.append(p)
print(n, K); print(*f)
