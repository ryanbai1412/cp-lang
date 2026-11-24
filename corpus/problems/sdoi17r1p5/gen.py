"""sdoi17r1p5 (SDOI 2017 Coin Game) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

"n m" then n distinct strings of length m over H/T, 1 <= n, m <= 300 (so n <= 2^m).
Some seeds take n = 2 (a subtask); strings are random or built from a short repeated
pattern with a few flips so the guesses overlap heavily.
--max: n = m = 300, every guess a few flips away from (HT)^150 or H^300.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
if MAX:
    n = m = 300
    pats = ["HT", "H"]
else:
    size = max(1, int(sys.argv[2]))
    m = rng.randint(1, min(300, size))
    n = 2 if rng.random() < 0.2 else rng.randint(1, min(300, size))
    n = min(n, 2**m)
    pats = rng.choice([["HT"], ["H"], ["HTT", "HH"], None])
seen = set()
while len(seen) < n:
    if pats is None or (2**m < 4 * n):
        w = "".join(rng.choice("HT") for _ in range(m))
    else:
        p = rng.choice(pats)
        w = list((p * m)[rng.randrange(len(p)):][:m])
        for _ in range(rng.randint(0, 3)):
            i = rng.randrange(m)
            w[i] = "HT"[w[i] == "H"]
        w = "".join(w)
    seen.add(w)
print(n, m)
print("\n".join(sorted(seen, key=lambda _: rng.random())))
