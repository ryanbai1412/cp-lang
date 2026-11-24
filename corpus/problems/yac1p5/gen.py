"""yac1p5 inputs: `gen.py SEED SIZE`, or `gen.py --max` (N = Q = 1e5).

N Q, N kangaroos "s v a", then Q events "1 i x y z" or "2 l r", with 2 <= N, 1 <= Q,
|s|, |v|, |a| <= 1e8, 1 <= l < r <= N, starts distinct after every event. Some seeds keep
a = 0 (subtasks 1-2). Most kangaroos are planted to sit at one spot at one of two times,
so long ranges do meet; queries are a mix of long and short ranges.
--max: N = Q = 1e5, every kangaroo planted on the same meeting, half the events moving
one kangaroo (keeping the meeting 999 times in 1000) and half asking about ranges of
length at least 0.8 N.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
LIM = 10**8
if MAX:
    n = q = 10**5
    m, flat, times = 10**4, False, [37]
else:
    size = max(2, int(sys.argv[2]))
    n, q = rng.randint(2, min(10**5, size)), rng.randint(1, min(10**5, size))
    m = rng.choice([2, 5, 10, 10**4]) if size < 1000 else rng.choice([10, 10**4])
    flat = rng.random() < 0.4
    times = [rng.randint(1, 6), rng.randint(1, 30)]
spot = rng.randint(-m, m)
used = set()


def kangaroo(i_old=None):
    if i_old is not None:
        used.discard(i_old)
    for _ in range(100):
        v, a = rng.randint(-m, m), 0 if flat else rng.randint(-m, m)
        if (MAX and i_old is None) or rng.random() < (0.999 if MAX else 0.8):
            t = times[0] if rng.random() < 0.85 else rng.choice(times)
            s = spot - v * t - a * t * (t - 1) // 2
        else:
            s = rng.randint(-min(LIM, 4 * m), min(LIM, 4 * m))
        if abs(s) <= LIM and s not in used:
            used.add(s)
            return s, v, a
    s = rng.randint(-LIM, LIM)
    while s in used:
        s = rng.randint(-LIM, LIM)
    used.add(s)
    return s, 0, 0


ks = [kangaroo() for _ in range(n)]
out = [f"{n} {q}"] + [f"{s} {v} {a}" for s, v, a in ks]
for _ in range(q):
    if rng.random() < 0.5:
        i = rng.randint(1, n)
        ks[i - 1] = kangaroo(ks[i - 1][0])
        out.append("1 %d %d %d %d" % ((i,) + ks[i - 1]))
    else:
        if MAX or rng.random() < 0.5:
            l, r = rng.randint(1, max(1, n // 10)), rng.randint(n - n // 10, n)
        else:
            l = rng.randint(1, n - 1)
            r = rng.randint(l + 1, min(n, l + rng.choice([1, 3, n])))
        out.append(f"2 {l} {r}")
print("\n".join(out))
