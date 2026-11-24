"""gen.py SEED SIZE | --max: NOI 2014 Magic Forest. 2 <= n <= 5e4, 0 <= m <= 1e5 edges X Y a b
with 1 <= a, b <= 5e4; self-loops and parallel edges are not excluded. Small sizes keep a, b
small so ties are common. --max is n = 5e4, m = 1e5: a long 1-n chain whose a rises while b
falls, so the best spanning forest keeps changing, plus random shortcuts."""
import random, sys
MAX = sys.argv[1] == "--max"
r = random.Random(0 if MAX else int(sys.argv[1]))
S = 0 if MAX else int(sys.argv[2])
n = 50_000 if MAX else r.randint(2, min(50_000, 2 + 2 * S))
m = 100_000 if MAX else r.randint(0, min(100_000, 1 + 4 * S))
w = 50_000 if MAX else r.choice([min(10, 1 + S), min(50_000, 1 + S * S)])
es = []
if MAX:
    order = [1] + r.sample(range(2, n), n - 2) + [n]
    for i, (x, y) in enumerate(zip(order, order[1:])):
        es.append((x, y, 1 + i * (w - 1) // (n - 2), w - i * (w - 1) // (n - 2)))
while len(es) < m:
    x = r.randint(1, n)
    y = x if r.random() < 0.02 else r.randint(1, n)
    es.append((x, y, r.randint(1, w), r.randint(1, w)))
r.shuffle(es)
sys.stdout.write("\n".join([f"{n} {m}"] + [" ".join(map(str, t)) for t in es]) + "\n")
