"""gen.py SEED SIZE | --max: IATI 2021 senior day 2 task 1 (Hint), in the local grader's format:
N M, A, B, then K and one longest common subsequence C. Values are in [0, min(N, M)). C comes
from Hunt-Szymanski (a longest increasing chain over the matching pairs), so the alphabet
stays large enough to keep the pairs near N + M; B is A edited (changed, dropped, inserted
and swapped elements) or independent. --max is N = M = 200 000."""
import random, sys
from bisect import bisect_left


def lcs(a, b):
    where = {}
    for j, x in enumerate(b):
        where.setdefault(x, []).append(j)
    tails, tail_node, nodes = [], [], []  # nodes: (value, previous node)
    for x in a:
        for j in reversed(where.get(x, ())):
            k = bisect_left(tails, j)
            nodes.append((x, tail_node[k - 1] if k else -1))
            if k == len(tails):
                tails.append(j)
                tail_node.append(len(nodes) - 1)
            else:
                tails[k], tail_node[k] = j, len(nodes) - 1
    out, v = [], tail_node[-1] if tail_node else -1
    while v >= 0:
        out.append(nodes[v][0])
        v = nodes[v][1]
    return out[::-1]


if sys.argv[1] == "--max":
    r = random.Random(2021)
    n = m = 200_000
    mode = 0
else:
    r = random.Random(int(sys.argv[1]))
    top = max(1, min(200_000, 30 * int(sys.argv[2])))
    n, m, mode = r.randint(1, top), r.randint(1, top), r.randrange(3)
v = min(n, m)
if mode == 2 and n * m <= 400_000:
    v = r.randint(1, v)
a = [r.randrange(v) for _ in range(n)]
if mode == 1:
    b = [r.randrange(v) for _ in range(m)]
else:
    b = []
    for x in a:
        roll = r.random()
        if roll < 0.15:
            continue
        b.append(r.randrange(v) if roll < 0.3 else x)
        if r.random() < 0.15:
            b.append(r.randrange(v))
    for _ in range(len(b) // 20):
        i = r.randrange(len(b)) if b else 0
        if i + 1 < len(b):
            b[i], b[i + 1] = b[i + 1], b[i]
    b = (b + [r.randrange(v) for _ in range(m)])[:m]
c = lcs(a, b)
print(n, m)
print(*a)
print(*b)
print(len(c))
print(*c)
