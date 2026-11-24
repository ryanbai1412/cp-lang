"""sac22cc3jp1 (Normal Problem) inputs: `gen.py SEED SIZE`, or `gen.py --max` (|S| = 1e5).

One lowercase string S, 1 <= |S| <= 1e5; half the cases plant `demello`, the rest hold near misses.
--max: 1e5 letters of repeated `demell` near misses with `demello` at the very end.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n = 10**5
    s = ("demell" * n)[: n - 7] + "demello"
else:
    n = rng.randint(1, max(1, min(10**5, int(sys.argv[2]))))
    parts = []
    while sum(map(len, parts)) < n:
        parts.append(rng.choice(["demell", "emello", "d", "e", "m", "l", "o", rng.choice("abcdelmoz")]))
    s = "".join(parts)[:n]
    if rng.random() < 0.5 and n >= 7:
        i = rng.randint(0, n - 7)
        s = s[:i] + "demello" + s[i + 7:]
print(s)
