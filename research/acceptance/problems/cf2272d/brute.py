import sys
from itertools import combinations
def mis(nv, edges):
    return max(bin(m).count("1") for m in range(1 << nv) if all(not (m >> u & 1 and m >> v & 1) for u, v in edges))
def is_tree(nv, edges):
    par = list(range(nv))
    def f(x):
        while par[x] != x: x = par[x]
        return x
    for u, v in edges:
        if f(u) == f(v): return False
        par[f(u)] = f(v)
    return len(edges) == nv - 1
def cases(text):
    it = iter(text.split()); t = int(next(it))
    for _ in range(t):
        k = int(next(it)); ids, edges, nv = {}, [], 0
        for ti in range(1, k + 1):
            n = int(next(it))
            for j in range(1, n + 1): ids[(ti, j)] = nv; nv += 1
            for _ in range(n - 1): edges.append((ids[(ti, int(next(it)))], ids[(ti, int(next(it)))]))
        yield k, ids, edges, nv
def best(k, ids, edges, nv):
    tree = {v: t for (t, _), v in ids.items()}
    cross = [(u, v) for u in range(nv) for v in range(u + 1, nv) if tree[u] != tree[v]]
    top = (-1, None)
    for add in combinations(cross, k - 1):
        if is_tree(nv, edges + list(add)):
            m = mis(nv, edges + list(add))
            if m > top[0]: top = (m, add)
    return top
if __name__ == "__main__":
    for k, ids, edges, nv in cases(sys.stdin.read()):
        name = {v: key for key, v in ids.items()}
        for u, v in best(k, ids, edges, nv)[1]: print(*name[u], *name[v])
