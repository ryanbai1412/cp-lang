#!/usr/bin/env python3
# usage: gen.py seed [size] | --max; `n m l`, then m trails `a b t` forming a forest.
import random, sys
MAX = sys.argv[1] == "--max"  # n = 1e5 billabongs, a forest of a few large trees
r = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else r.randint(1, 9)
m = n - 3 if MAX and n > 3 else (r.randint(0, n - 1))
l = r.randint(1, 10**4)
order = list(range(n)); r.shuffle(order)
print(n, m, l)
# the last m billabongs (in shuffled order) each hang off an earlier one
for i in range(n - m, n):
    print(order[r.randrange(i)], order[i], r.randint(1, 10**4 if MAX else 9))
