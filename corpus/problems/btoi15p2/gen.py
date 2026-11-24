"""btoi15p2 (Editor) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

n, then n operations a_i in -n..=n, a_i != 0: an edit a_i > 0, or an undo of level -a_i that
always finds an active operation of a lower level. --max: n = 3e5, deep stacks of edits undone
and redone by undos of mixed levels, so the chains under each operation are long.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 3 * 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
cap = n if MAX or rng.random() < 0.3 else rng.randint(1, max(1, min(n, 5)))
# The active chain: op i sits on nxt[i]; low[i] is the first op below i of a lower level.
level, nxt, low, minlv = [0], [-1], [-1], [10**9]  # node 0: the empty chain's bottom
top, ops = 0, []


def below(t, lv):
    """The topmost op in the chain from t whose level is under lv."""
    while t > 0 and level[t] >= lv:
        t = low[t]
    return t


for i in range(n):
    undo = minlv[top] < cap and rng.random() < (0.45 if MAX else 0.5)
    if undo:
        hi = min(cap, n)
        lv = rng.randint(minlv[top] + 1, hi) if rng.random() < 0.3 else rng.randint(minlv[top] + 1, min(hi, minlv[top] + 3))
        u = below(top, lv)
        under = nxt[u]
        ops.append(-lv)
    else:
        lv, under = 0, top
        ops.append(rng.randint(1, n))
    level.append(lv)
    nxt.append(under)
    low.append(below(under, lv))
    minlv.append(min(lv, minlv[under]))
    top = len(level) - 1
print(n)
print("\n".join(map(str, ops)))
