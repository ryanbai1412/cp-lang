"""apio07p1 (Mobiles) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

n, then n lines "l r": rod i's left and right ends, a rod numbered above i or -1 for a toy;
rod 1 hangs at the top. --max: n = 1e5 as one long chain of rods with toys hanging off it on
random sides (depth 1e5 for a recursive walk), then a near-complete mobile hung off its end.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
kids = [[-1, -1] for _ in range(n + 1)]
if MAX:
    chain = n - 2**15 + 1
    for v in range(1, chain):
        kids[v][rng.randrange(2)] = v + 1
    for v in range(chain + 1, n + 1):
        p = chain + (v - chain - 1) // 2
        kids[p][(v - chain - 1) % 2] = v
else:
    free = [(1, 0), (1, 1)]
    full = rng.random() < 0.5
    for v in range(2, n + 1):
        i = 0 if full else rng.randrange(len(free))
        u, side = free.pop(i)
        kids[u][side] = v
        free += [(v, 0), (v, 1)]
    if rng.random() < 0.5:
        for v in range(1, n + 1):
            if rng.random() < 0.3:
                kids[v].reverse()
print(n)
print("\n".join(f"{l} {r}" for l, r in kids[1:]))
