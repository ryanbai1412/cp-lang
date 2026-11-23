import sys
def path(tree, a, b):
    adj = {}
    for x, y in tree: adj.setdefault(x, []).append(y); adj.setdefault(y, []).append(x)
    par, st = {a: None}, [a]
    while st:
        x = st.pop()
        for y in adj.get(x, []):
            if y not in par: par[y] = x; st.append(y)
    es = []
    while b != a: es.append(tuple(sorted((b, par[b])))); b = par[b]
    return es
def solve(n, edges, ops):
    states = {frozenset(tuple(sorted(e)) for e in edges)}
    for v in ops:
        nxt = set()
        for tr in states:
            nxt.add(tr)
            for e in path(tr, 1, v): nxt.add(tr - {e} | {(1, v)})
        states = nxt
    return max(len(path(tr, 1, n)) for tr in states)
it = iter(sys.stdin.read().split())
for _ in range(int(next(it))):
    n, m = int(next(it)), int(next(it))
    edges = [(int(next(it)), int(next(it))) for _ in range(n - 1)]
    print(solve(n, edges, [int(next(it)) for _ in range(m)]))
