#!/usr/bin/env python3
# usage: gen.py seed [size] | --max [--m M]; `n m l`, then m trails `a b t` forming a forest.
# --max: n = 1e5 and m = n - 1, one tree; --max --m M: a forest of n - M trees.
import random, sys
MAX = sys.argv[1] == "--max"  # n = 1e5 billabongs, a forest of a few large trees
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else r.randint(1, 9)
m = (int(sys.argv[3]) if sys.argv[2:3] == ["--m"] else n - 1) if MAX else r.randint(0, n - 1)
l = r.randint(1, 10**4)
order = list(range(n)); r.shuffle(order)
print(n, m, l)
# the last m billabongs (in shuffled order) each hang off an earlier one
for i in range(n - m, n):
    print(order[r.randrange(i)], order[i], r.randint(1, 10**4 if MAX else 9))
