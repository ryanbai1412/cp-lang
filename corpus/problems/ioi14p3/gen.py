"""ioi14p3 (Game) hidden question orders for interactor.py: `gen.py SEED SIZE`, or `gen.py --max`.

n (4 <= n <= 1500), then all n(n-1)/2 pairs "u v" in the order Mei-Yu asks them: random,
lexicographic, or every pair at one city first.
--max: n = 1500, the pairs in random order.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n, order = 1500, "random"
else:
    size = max(4, int(sys.argv[2]))
    n, order = rng.randint(4, min(1500, size)), rng.choice(["random", "lex", "star"])
pairs = [(u, v) for u in range(n) for v in range(u + 1, n)]
if order == "random":
    rng.shuffle(pairs)
elif order == "star":
    hub = rng.randrange(n)
    pairs.sort(key=lambda e: hub not in e)
out = [str(n)] + [f"{u} {v}" if rng.random() < 0.5 else f"{v} {u}" for u, v in pairs]
print("\n".join(out))
