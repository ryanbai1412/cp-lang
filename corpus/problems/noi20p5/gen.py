#!/usr/bin/env python3
"""gen.py seed size | --max: NOI 2020 Surreal Trees. T <= 100 tests of m binary trees (root 1,
children "l r", 0 for none), sum n, sum m and height <= 2e6. --max: T = 100 with sum n = 2e6:
long spine trees (height near 1e6 in the first test) whose shapes share prefixes, many small
trees, and some tests made complete by trees of all four shapes."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else max(1, int(sys.argv[2]))

def spine(size, prefix):
    # a chain where each node has one child or two (one of them a leaf); about size nodes
    kids = [[0, 0]]
    cur, step = 0, 0
    while len(kids) < size - 1:
        shape = prefix[step] if step < len(prefix) else r.randrange(4)
        step += 1
        nxt = len(kids); kids.append([0, 0])
        if shape == 0: kids[cur][0] = nxt
        elif shape == 1: kids[cur][1] = nxt
        else:
            leaf = len(kids); kids.append([0, 0])
            kids[cur] = [nxt, leaf] if shape == 2 else [leaf, nxt]
        cur = nxt
    return kids

def bushy(size):
    kids = [[0, 0]]
    free = [(0, 0), (0, 1)]
    while len(kids) < size:
        i = r.randrange(len(free)); free[i], free[-1] = free[-1], free[i]
        v, side = free.pop()
        u = len(kids); kids.append([0, 0]); kids[v][side] = u
        free += [(u, 0), (u, 1)]
    return kids

def show(kids):
    return "\n".join([str(len(kids))] + [f"{l + 1 if l else 0} {c + 1 if c else 0}" for l, c in kids])

tests = []
budget = 2_000_000 if MAX else S
for t in range(100 if MAX else r.randint(1, 3)):
    trees = []
    if MAX:
        size = 1_000_000 if t == 0 else 10_101 if t < 99 else 2_000_000 - 1_000_000 - 98 * 10_101
        pre = [r.randrange(4) for _ in range(size)]
        left = size
        if t == 0:
            trees += [spine(600_000, pre), spine(399_990, pre[:200_000])]
            left = size - sum(map(len, trees))
        if t % 3 == 1:
            trees += [spine(3, [s]) for s in range(4)]
            left -= sum(map(len, trees))
        while left > 0:
            sz = min(left, r.choice([1 + r.randrange(40), 2 + r.randrange(400), 2 + r.randrange(3000)]))
            if sz == 1 and t % 3 != 2:
                sz = min(left, 2)
            trees.append(spine(sz, pre) if r.random() < 0.6 or sz < 3 else bushy(sz))
            left -= len(trees[-1])
    else:
        for _ in range(r.randint(1, 5)):
            sz = r.randint(1, max(1, S // 4))
            trees.append(spine(sz, []) if r.random() < 0.5 else bushy(sz))
    tests.append("\n".join([str(len(trees))] + [show(k) for k in trees]))
print(len(tests))
print("\n".join(tests))
