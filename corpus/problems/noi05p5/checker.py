"""noi05p5 checker: `checker.py input output answer`. The output's n - 1 distinct roads form
a spanning tree in which person i has at most k_i contacts, the first line is their total
comfort, and that total is at least the answer's (the task is scored against the
organizers' network, so a more comfortable one also passes)."""
import sys

d = open(sys.argv[1]).read().split()
n, m = int(d[1]), int(d[2])
k = [int(x) for x in d[3:3 + n]]
E = [tuple(int(x) for x in d[3 + n + 3 * i:6 + n + 3 * i]) for i in range(m)]


def comfort(path):
    s = open(path).read().split()
    if len(s) != n or not all(x.lstrip("-").isdigit() for x in s):
        return None
    ids = [int(x) for x in s[1:]]
    if len(set(ids)) != n - 1 or not all(1 <= e <= m for e in ids):
        return None
    p, deg = list(range(n + 1)), [0] * (n + 1)

    def find(x):
        while p[x] != x:
            p[x] = p[p[x]]
            x = p[x]
        return x

    for e in ids:
        u, v, _ = E[e - 1]
        deg[u] += 1
        deg[v] += 1
        if find(u) == find(v):
            return None
        p[find(u)] = find(v)
    if any(deg[i + 1] > k[i] for i in range(n)):
        return None
    total = sum(E[e - 1][2] for e in ids)
    return total if total == int(s[0]) else None


mine, ref = comfort(sys.argv[2]), comfort(sys.argv[3])
sys.exit(0 if mine is not None and ref is not None and mine >= ref else 1)
