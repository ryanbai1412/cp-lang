"""sac22cc4p2 (Obligatory String Problem) inputs: `gen.py SEED SIZE`, or `gen.py --max` (K = 52).

K (0..=52), then a 4-letter lowercase string C; small sizes keep K small.
--max: K = 52, so all 26^4 strings print.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
k = 52 if MAX else rng.randint(0, min(52, max(1, int(sys.argv[2]))))
print(k)
print("".join(rng.choice("abcdefghijklmnopqrstuvwxyz") for _ in range(4)))
