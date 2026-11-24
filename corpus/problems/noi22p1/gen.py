"""noi22p1 (Major) inputs: `gen.py SEED SIZE`, or `gen.py --max` (n = q = 5e5).

"n q", n sequences "len x..", then q ops: 1 x y, 2 x (x non-empty), 3 m x..x (distinct live
ids), 4 x1 x2 x3 (x3 a fresh id). n, q, the total length and the sum of m are at most 5e5;
values are at most n + q. --max: few values so majorities exist, joins of long sequences, and
queries over many sequences.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 5 * 10**5 if MAX else max(1, int(sys.argv[2]))
n = size if MAX else rng.randint(1, size)
q = size if MAX else rng.randint(1, size)
vals = rng.choice([2, 3, n + q]) if not MAX else 3
cap = 5 * 10**5 if MAX else size  # total length and sum of m
lens = [0] * n
init = (cap if MAX else rng.randint(0, cap)) - (q // 4 if MAX else 0)
for _ in range(max(0, init)):
    lens[rng.randrange(n)] += 1
out = [f"{n} {q}"]
size_of = {}
for i in range(n):
    out.append(" ".join([str(lens[i])] + [str(rng.randint(1, vals)) for _ in range(lens[i])]))
    size_of[i + 1] = lens[i]
alive = list(range(1, n + 1))
fresh, m_left, length = n + 1, cap, max(0, init)
for _ in range(q):
    kind = rng.choice([1, 1, 2, 3, 4]) if MAX else rng.randint(1, 4)
    if kind == 4 and len(alive) < 2:
        kind = 1
    if kind == 2:
        i = rng.randrange(len(alive))
        if size_of[alive[i]] == 0:
            kind = 1
    if kind == 3 and m_left == 0:
        kind = 1
    if kind == 1 and length >= cap:
        kind = 3 if m_left else 4 if len(alive) >= 2 else 2
    if kind == 1:
        x = rng.choice(alive)
        out.append(f"1 {x} {rng.randint(1, vals)}")
        size_of[x] += 1
        length += 1
    elif kind == 2:
        x = alive[rng.randrange(len(alive))]
        if size_of[x] == 0:
            x = max(alive, key=lambda a: size_of[a]) if not MAX else x
        if size_of[x] == 0:
            out.append(f"1 {x} 1")
            size_of[x] += 1
            continue
        out.append(f"2 {x}")
        size_of[x] -= 1
    elif kind == 3:
        m = min(m_left, len(alive), rng.randint(1, 50 if MAX else 5))
        xs = rng.sample(alive, m)
        m_left -= m
        out.append(" ".join(map(str, [3, m] + xs)))
    else:
        i, j = rng.sample(range(len(alive)), 2)
        a, b = alive[i], alive[j]
        for t in sorted((i, j), reverse=True):
            alive[t] = alive[-1]
            alive.pop()
        size_of[fresh] = size_of.pop(a) + size_of.pop(b)
        alive.append(fresh)
        out.append(f"4 {a} {b} {fresh}")
        fresh += 1
print("\n".join(out))
