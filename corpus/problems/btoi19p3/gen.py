"""btoi19p3 (Alpine Valley) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"N S Q E", then N - 1 roads "A B W" (a tree, W <= 1e9), S distinct shop villages, then Q
queries "I R" (close road I, start at R). --max: N = Q = 1e5, a deep caterpillar hung far
from E with heavy roads, shops spread out, queries cutting deep roads from deeper villages.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else rng.randint(2, max(2, int(sys.argv[2])))
q = 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
s = n // 10 if MAX else rng.randint(1, n)
lab = list(range(1, n + 1))
rng.shuffle(lab)
par = [0] * n
for i in range(1, n):
    par[i] = i - 1 if MAX and rng.random() < 0.9 else rng.randrange(max(0, i - 5) if MAX else 0, i)
top = 10**9 if MAX or rng.random() < 0.5 else 10
roads = [(lab[par[i]], lab[i], rng.randint(1, top) if not MAX else 10**9 - rng.randint(0, 1000)) for i in range(1, n)]
order = list(range(n - 1))
rng.shuffle(order)
roads = [roads[i] for i in order]
exit_v = lab[0] if MAX else rng.randint(1, n)
out = [f"{n} {s} {q} {exit_v}"]
out += [f"{a} {b} {w}" for a, b, w in roads]
out += map(str, rng.sample(range(1, n + 1), s))
for _ in range(q):
    out.append(f"{rng.randint(1, n - 1)} {rng.randint(1, n)}")
print("\n".join(out))
