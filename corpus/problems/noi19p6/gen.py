import sys, random
# Prints L_m L_q L_c, N M and the edges. N's last digit names the kind, as in the statement: 8 a
# perfect matching, 7 a tree where each x > 0 has one neighbour y < x, 6 a graph with a Hamiltonian
# path, 5 connected, 4 N = 1004 with a tight query limit, otherwise any simple graph.
# seed size: N <= 500 any graph (M <= 10 N), or a larger N of a random kind scaled by size, the
# limits those of the statement's row for that kind; --max: N = 2e5, M = 3e5 (test 25)
mx = sys.argv[1] == "--max"
r = random.Random(0 if mx else int(sys.argv[1])); S = 0 if mx else int(sys.argv[2])


def simple(n, m, base):
    es = set(base)
    while len(es) < m:
        x, y = r.randrange(n), r.randrange(n)
        if x != y:
            es.add((min(x, y), max(x, y)))
    return list(es)


if mx:
    kind, n = "0", 200000
elif S < 8 or r.random() < 0.3:
    kind, n = "s", r.randint(2, min(500, 3 + 20 * S))
else:
    kind = r.choice("876540")
    n = 1004 if kind == "4" else 10 * r.randint(51, min(19999, 50 + 100 * S)) + int(kind)
perm = list(range(n)); r.shuffle(perm)
if kind == "s":
    m = r.randint(1, min(10 * n, n * (n - 1) // 2))
    es = simple(n, m, [])
    Lm, Lq, Lc = max(100, n - 1), max(100, n * n if n < 500 else 150000), max(100, 2 * m)
elif kind in "87":
    if kind == "8":
        es = [(min(perm[2 * i], perm[2 * i + 1]), max(perm[2 * i], perm[2 * i + 1])) for i in range(n // 2)]
    else:
        es = [(r.randint(max(0, x - r.choice([1, 3, x])), x - 1), x) for x in range(1, n)]
    c = 17 if n <= 59998 else 18 if n <= 99998 else 19
    Lm, Lq, Lc = c * n, c * n, 0
else:
    if kind == "6":
        es = [(min(perm[i], perm[i + 1]), max(perm[i], perm[i + 1])) for i in range(n - 1)]
    elif kind == "5":
        es = [(min(perm[x], perm[r.randrange(x)]), max(perm[x], perm[r.randrange(x)])) for x in range(1, n)]
        es = [(min(perm[x], perm[y]), max(perm[x], perm[y])) for x, y in ((x, r.randrange(x)) for x in range(1, n))]
    elif kind == "4":
        es = simple(n, r.randint(2000, 3000), [])
    else:
        es = simple(n, 300000 if mx else r.randint(1, 2 * n), [])
    Lm, Lq, Lc = 10**7, 50000 if kind == "4" else 10**7, 2 * len(es)
r.shuffle(es)
out = [f"{Lm} {Lq} {Lc}", f"{n} {len(es)}"]
out += [f"{x} {y}" if r.random() < 0.5 else f"{y} {x}" for x, y in es]
print("\n".join(out))
