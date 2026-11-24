"""ioi10p1 (Cluedo) hidden cases for interactor.py: `gen.py SEED SIZE`, or `gen.py --max`.

T (1 <= T <= 100) cases, then one line per case: the murderer (1..6), location (1..10) and
weapon (1..6). The interactor refutes a wrong theory by a random wrong component.
--max: T = 100, the last of each kind (6 10 6) and random ones.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
t = 100 if MAX else rng.randint(1, min(100, max(1, int(sys.argv[2]))))
cases = [(rng.randint(1, 6), rng.randint(1, 10), rng.randint(1, 6)) for _ in range(t)]
if MAX:
    cases[: t // 2] = [(6, 10, 6)] * (t // 2)
print(t)
print("\n".join(f"{m} {l} {w}" for m, l, w in cases))
