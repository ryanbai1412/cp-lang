import sys, random
# IOI 2025 Migrations, sample-grader stdin: N = 10000, then P[1..N-1] with 0 <= P[i] < i.
# `gen.py SEED SIZE`: SIZE sets how local the attachments are and how often the diameter's ends
# move; `--sample` is the statement's example and `--max` a path, whose ends move every step.
N = 10000


def emit(p):
    print(N)
    print(*p[1:])


def ends(p, upto):
    """A diameter's ends in the tree on sites 0..upto, by two breadth-first passes."""
    adj = [[] for _ in range(upto + 1)]
    for i in range(1, upto + 1):
        adj[i].append(p[i])
        adj[p[i]].append(i)

    def farthest(s):
        dist = [-1] * (upto + 1)
        dist[s], order = 0, [s]
        for x in order:
            for y in adj[x]:
                if dist[y] < 0:
                    dist[y] = dist[x] + 1
                    order.append(y)
        return max(range(upto + 1), key=dist.__getitem__)

    u = farthest(0)
    return u, farthest(u)


def local(r, S):
    w = r.choice([1, 2, S, 10 * S, N])
    return [0] + [r.randrange(max(0, i - w), i) for i in range(1, N)]


def race(r, S, start=None):
    """Two arms that take turns growing; a site that grows neither hangs off a random site."""
    p = list(start or [0, 0])
    a, b = (len(p) - 1, len(p) - 2) if start else (1, 0)
    grow = r.choice([0.5, 1 / S, min(0.9, 2 / S)])
    while len(p) < N:
        i = len(p)
        if r.random() < grow:
            if r.random() < 0.5:
                a, b = b, a
            p.append(a)
            a = i
        else:
            p.append(r.randrange(i) if r.random() < 0.5 else r.choice([0, 1, p[i - 1]]))
    return p


def from_root(r, S):
    """Subtask 1: site 0 ends a diameter. A spine grows from 0; a branch hangs off spine site k
    and is never longer than k, so no two branches are farther apart than the spine's tip."""
    p, anchor, off, spine = [0], [0], [0], [0]
    for i in range(1, N):
        if r.random() < 1 / (S + 1) or len(spine) == 1:
            y, k, a = spine[-1], len(spine), 0
            spine.append(i)
        else:
            y = r.choice([r.randrange(1, i), i - 1, r.choice(spine[1:])])
            while off[y] + 1 > anchor[y]:
                y = p[y]
            k, a = anchor[y], off[y] + 1
        p.append(y)
        anchor.append(k)
        off.append(a)
    return p


def adaptive(r, S):
    """The official adaptive tests' start: arms under sites 10 and 5 from 512..1023 on."""
    p = [0, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4]
    u, v = r.sample(range(512, 1024), 2)
    p += [10 if i == u else 5 if i == v else 1 for i in range(11, max(u, v) + 1)]
    return race(r, S, p[:max(u, v) + 1])


def tail(r, p, k):
    """The last k sites hang off a current end, next to one, or anywhere: the endgame's cases."""
    for i in range(N - k, N):
        u, v = ends(p, i - 1)
        p[i] = r.choice([u, v, u, v, p[u], p[v], r.randrange(i), i - 1])


if sys.argv[1] == "--sample":
    emit([0, 0, 1, 2, 2] + [1] * (N - 5))
    sys.exit()
if sys.argv[1] == "--max":
    emit(list(range(-1, N - 1)))
    sys.exit()
r = random.Random(int(sys.argv[1]))
S = int(sys.argv[2])
shape = r.choice([local, race, from_root, adaptive])
p = shape(r, S)
if shape is not from_root and r.random() < 0.7:
    tail(r, p, r.randint(1, 8))
emit(p)
