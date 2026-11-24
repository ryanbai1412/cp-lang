"""gen.py SEED SIZE | --max: CCO 2003 P4. n <= 9 and k constraints "x before y" (x == y allowed,
cycles possible). --max is n = 9 with k = 72."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 9 if MAX else r.randint(1, min(9, 2 + S))
k = 72 if MAX else r.randint(0, 2 * n)
if MAX:
    order = list(range(1, n + 1)); r.shuffle(order)
    pairs = [(order[i], order[j]) for i in range(n) for j in range(n) if i < j and r.random() < .2]
    pairs = (pairs * k)[:k] if pairs else [(1, 1)] * k
else:
    pairs = [(r.randint(1, n), r.randint(1, n)) for _ in range(k)]
out = [f"{n} {k}"] + [f"{x} {y}" for x, y in pairs]
sys.stdout.write("\n".join(out) + "\n")
