"""btoi19p5 (Necklace) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

Two lowercase strings of length 1..=3000. Small alphabets make long rotated and reversed matches
likely; some cases plant a rotated or reversed block of s inside t. --max: both 3000 over "ab".
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 3000 if MAX else max(1, min(3000, int(sys.argv[2])))
letters = "ab" if MAX else "abcdefghijklmnopqrstuvwxyz"[: rng.choice((1, 2, 3, 26))]
n, m = (3000, 3000) if MAX else (rng.randint(1, size), rng.randint(1, size))
s = "".join(rng.choice(letters) for _ in range(n))
t = [rng.choice(letters) for _ in range(m)]
if not MAX and rng.random() < 0.5:
    a = rng.randrange(n)
    blk = s[a : rng.randint(a + 1, n)][: m]
    c = rng.randrange(len(blk) + 1)
    blk = blk[c:] + blk[:c]
    if rng.random() < 0.5:
        blk = blk[::-1]
    p = rng.randint(0, m - len(blk))
    t[p : p + len(blk)] = blk
print(s)
print("".join(t))
