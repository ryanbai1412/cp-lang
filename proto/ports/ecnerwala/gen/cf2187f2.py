import sys, random

r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])


def dfs_order(par, n):
    kids = [[] for _ in range(n + 1)]
    for v in range(1, n + 1):
        kids[par[v]].append(v)
    out, stack = [], [0]
    while stack:
        v = stack.pop()
        if v:
            out.append(v)
        ks = kids[v][:]
        r.shuffle(ks)
        stack.extend(reversed(ks))
    return out


def case(n):
    labels = list(range(1, n + 1))
    r.shuffle(labels)
    par = [0] * (n + 1)
    for i in range(1, n + 1):
        par[labels[i - 1]] = 0 if i == 1 or r.random() < 0.2 else labels[r.randrange(max(0, i - 1 - r.randint(1, 3)), i - 1)]
    a, b = dfs_order(par, n), dfs_order(par, n)
    for _ in range(r.randint(0, 2)):
        i, j = r.randrange(n), r.randrange(n)
        b[i], b[j] = b[j], b[i]
    return a, b


if S >= 1000:
    tests = [S]
else:
    tests = [r.randint(2, 3 * S + 2) for _ in range(r.randint(1, 5))]
print(len(tests))
for n in tests:
    a, b = case(n)
    print(n)
    print(*a)
    print(*b)
