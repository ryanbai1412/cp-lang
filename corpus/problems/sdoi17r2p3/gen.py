"""sdoi17r2p3 (SDOI 2017 Cut Tree Game) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"n m", n weights in [0, m), n - 1 edges, q, then q lines "Change x y" or "Query k" with
0 <= y, k < m, n, q <= 30000, at most 10000 changes, m one of the statement's 4, 8, 64, 128.
Follows the table: n <= 2000, m = 64, q <= 64 and no changes; a path a_i = i, b_i = i + 1
with m = 8 or 128; any tree with m = 4 or 128. Trees are random, paths, stars or
caterpillars, labels shuffled except on the path subtask.
--max: n = q = 30000, m = 128, a caterpillar with a 15000-node spine, 10000 changes on
deep nodes interleaved with 20000 queries.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))


def edges(n, shape):
    if shape == "line":
        return [(i, i + 1) for i in range(1, n)]
    if shape == "star":
        par = [0] + [1] * (n - 1)
    elif shape == "caterpillar":
        h = max(1, n // 2)
        par = list(range(h)) + [rng.randint(1, h) for _ in range(n - h)]
    else:
        par = [0] + [rng.randint(max(1, i - rng.choice([1, 5, i])), i - 1) for i in range(2, n + 1)]
    lab = list(range(1, n + 1))
    rng.shuffle(lab)
    out = [(lab[i], lab[par[i] - 1]) for i in range(1, n)]
    return [(a, b) if rng.random() < 0.5 else (b, a) for a, b in out]


if MAX:
    n, m, q, changes, shape = 30000, 128, 30000, 10000, "caterpillar"
else:
    size = max(1, int(sys.argv[2]))
    sub = rng.choice(["static", "line", "any"])
    if sub == "static":
        n, m, q, changes = rng.randint(1, min(2000, size)), 64, rng.randint(1, min(64, size)), 0
        shape = rng.choice(["random", "star", "caterpillar", "line"])
    else:
        n, q = rng.randint(1, min(30000, size)), rng.randint(1, min(30000, size))
        changes = rng.randint(0, min(10000, q))
        m = rng.choice([8, 128] if sub == "line" else [4, 128])
        shape = "line" if sub == "line" else rng.choice(["random", "star", "caterpillar"])
E = edges(n, shape)
ops = ["C"] * changes + ["Q"] * (q - changes)
rng.shuffle(ops)
if MAX:
    ops = (["C", "Q", "Q"] * 10000)[:q]
deep = list(range(n // 2, n + 1)) if n > 1 else [1]
lines = [f"{n} {m}", " ".join(str(rng.randrange(m)) for _ in range(n))]
lines += [f"{a} {b}" for a, b in E]
lines.append(str(q))
for o in ops:
    if o == "C":
        x = rng.choice(deep) if MAX else rng.randint(1, n)
        lines.append(f"Change {x} {rng.randrange(m)}")
    else:
        lines.append(f"Query {rng.randrange(m)}")
print("\n".join(lines))
