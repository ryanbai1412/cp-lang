"""ioi07p4 (Miners) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N (1 <= N <= 1e5), then a string of N letters from M, F, B.
--max: N = 1e5 random shipments.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 100_000 if MAX else rng.randint(1, min(100_000, max(1, int(sys.argv[2]))))
foods = "MFB" if MAX or rng.random() < 0.7 else rng.choice(["M", "MF", "FB"])
print(n)
print("".join(rng.choice(foods) for _ in range(n)))
