# usage: gen.py SEED [SUB]: the sample grader's input (sub, T, then per case N and the
# N x N town distances) for T random trees: towns are leaves, every other settlement has
# degree >= 3 (exactly 3 when sub = 4), 6 <= N <= 110, distances in 1..=1e6.
import random, sys

r = random.Random(int(sys.argv[1]))
sub = int(sys.argv[2]) if len(sys.argv) > 2 else r.randint(1, 6)


class Tree:
    def __init__(self):
        self.adj = []

    def node(self):
        self.adj.append({})
        return len(self.adj) - 1

    def link(self, u, v, w):
        self.adj[u][v] = w
        self.adj[v][u] = w

    def cut(self, u, v):
        w = self.adj[u].pop(v)
        self.adj[v].pop(u)
        return w


def grow(t, root, edges, leaves, want, weight, local):
    """Grow the subtree below `root` (its `edges`) until it has `want` leaves."""
    while len(leaves) < want:
        inner = [u for u in {x for e in edges for x in e} if u not in leaves]
        if sub != 4 and r.random() < 0.4:
            u = r.choice(inner[-3:] if local else inner)
        else:
            e = r.choice(edges[-3:] if local else edges)
            edges.remove(e)
            u = t.node()
            w = t.cut(*e)
            a = r.randint(1, w - 1) if w > 1 else 1
            t.link(e[0], u, a)
            t.link(u, e[1], max(1, w - a))
            edges += [(e[0], u), (u, e[1])]
        leaf = t.node()
        t.link(u, leaf, weight())
        edges.append((u, leaf))
        leaves.add(leaf)


def subtree(t, top, size, weight, local, up=None):
    """Hang a subtree with `size` towns below `top` by an edge of weight `up`."""
    x = t.node()
    t.link(top, x, up or weight())
    if size == 1:
        return {x}
    edges, leaves = [], set()
    for _ in range(2 if sub == 4 or size == 2 or r.random() < 0.5 else 3):
        leaf = t.node()
        t.link(x, leaf, weight())
        edges.append((x, leaf))
        leaves.add(leaf)
    grow(t, x, edges, leaves, max(size, len(leaves)), weight, local)
    return leaves


def case(n):
    t = Tree()
    hi = r.choice([1, 2, 3, 10, 1000, 4500])
    weight = lambda: r.randint(1, hi)
    if sub != 4 and r.random() < 0.5:
        # One settlement h with two long legs and branches whose sizes straddle n / 2,
        # so the hub's balance and the majority branch are close calls.
        h = t.node()
        legs = [r.choice([1, 1, 2, 3]) for _ in range(2)]
        if sum(legs) >= n:
            legs = [1, 1]
        rest = n - sum(legs)
        big = min(rest, max(1, n // 2 + r.choice([-1, 0, 0, 1, 1, 2, rest])))
        sizes = [big]
        while sum(sizes) < rest:
            sizes.append(r.randint(1, rest - sum(sizes)))
        short = lambda: r.randint(1, max(1, hi // 4))
        for s in sizes:
            subtree(t, h, s, short, r.random() < 0.5)
        reach = 2 * hi * n
        for s in legs:
            subtree(t, h, s, short, False, up=r.randint(reach, reach + hi))
    else:
        c = t.node()
        edges, leaves = [], set()
        for _ in range(3):
            leaf = t.node()
            t.link(c, leaf, weight())
            edges.append((c, leaf))
            leaves.add(leaf)
        grow(t, c, edges, leaves, n, weight, r.random() < 0.4)
    towns = [u for u in range(len(t.adj)) if len(t.adj[u]) == 1]
    assert len(towns) == n
    assert all(len(a) >= 3 if sub != 4 else len(a) == 3 for a in t.adj if len(a) > 1)
    r.shuffle(towns)
    dist = []
    for s in towns:
        d = {s: 0}
        stack = [s]
        while stack:
            u = stack.pop()
            for v, w in t.adj[u].items():
                if v not in d:
                    d[v] = d[u] + w
                    stack.append(v)
        dist.append([d[x] for x in towns])
    return dist


cases = []
while len(cases) < r.randint(1, 4):
    n = r.choice([6, 7, 8, 11, r.randint(6, 40), r.randint(6, 110), 110])
    dist = case(n)
    if max(map(max, dist)) <= 10**6:
        cases.append(dist)
print(sub, len(cases))
for dist in cases:
    print(len(dist))
    for row in dist:
        print(*row)
