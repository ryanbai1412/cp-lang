"""cco22p6 (CCO 2022 Good Game) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"N" then a string of N letters A/B, 1 <= N <= 10^6, N <= max(1, SIZE). Strings are built
from runs: mostly single letters with some longer runs, one longer run planted at or near
the middle (winnable or one step short), or built backwards from a cleared board by
inserting 2 or 3 equal letters (always winnable), or uniform random.
--max: N = 10^6, single letters except runs of 3 and 2 at distances a, c from the middle
with a + c = H, the tight winnable case with about 5 * 10^5 moves.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))


def from_runs(rs):
    return "".join(("A" if i % 2 == 0 else "B") * x for i, x in enumerate(rs))


def tight(k, slack):
    """k (odd) runs, middle a single, big runs at middle - a and middle + c, a + c = H + slack."""
    h = (k - 1) // 2
    if h == 0:
        return [rng.randint(1, 3)]
    a = rng.randint(1, h + slack - 1) if h + slack > 1 else 1
    c = max(1, min(h, h + slack - a))
    rs = [1] * k
    rs[h - min(a, h)] = 2
    rs[h + c] = 2
    return rs


def runs_case(n):
    rs, tot = [], 0
    p = rng.choice([0.05, 0.2, 0.5])
    while tot < n:
        x = 1 if rng.random() > p else rng.randint(2, 4)
        x = min(x, n - tot)
        rs.append(x)
        tot += x
    if rng.random() < 0.5 and len(rs) >= 3:
        mid = (len(rs) - 1) // 2 if len(rs) % 2 else rng.randrange(len(rs))
        rs[min(len(rs) - 1, max(0, mid + rng.randint(-2, 2)))] = 2
    return from_runs(rs)


def backwards(n):
    s = []
    while len(s) + 2 <= n:
        j = 2 if len(s) + 3 > n else rng.choice([2, 3])
        i = rng.randint(0, len(s))
        s[i:i] = [rng.choice("AB")] * j
    return "".join(s)


if MAX:
    k = 10**6 - 3  # odd run count; runs of 3 and 2 at distances a, c from the middle
    h = (k - 1) // 2
    rs = [1] * k
    rs[h - h // 2] = 3
    rs[h + (h - h // 2)] = 2
    s = from_runs(rs)
else:
    size = int(sys.argv[2])
    n = rng.randint(1, max(1, min(size, 10**6)))
    kind = rng.choice(["runs", "runs", "tight", "back", "random"])
    if kind == "tight" and n >= 3:
        k = n - 2 if n % 2 else n - 3
        k = max(k, 1)
        s = from_runs(tight(k, rng.choice([0, 0, 1])))
    elif kind == "back" and 2 <= n <= 20000:
        s = backwards(n) or "AA"
    elif kind == "random":
        s = "".join(rng.choice("AB") for _ in range(n))
    else:
        s = runs_case(n)
print(len(s))
print(s)
