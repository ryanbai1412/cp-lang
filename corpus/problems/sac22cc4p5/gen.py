"""sac22cc4p5 (Obligatory Interactive Problem) hidden cases for interactor.py:
`gen.py SEED SIZE`, or `gen.py --max`.

"N Q1 Q2 T" for a subtask T (1: N <= 500, Q1 = 300000, Q2 = 300; 2: N <= 500,
Q1 = 150000, Q2 = 300; 3: N <= 1e4, Q1 = 300000, Q2 = 6000; 4: N <= 1e4, Q1 = 100000,
Q2 = 6000), N >= 3, then the heights H, a permutation of 1..N (random, sorted or reversed).
--max: subtask 4 (the tightest budget) at N = 1e4, H random.
"""
import random
import sys

SUBTASKS = {1: (500, 300000, 300), 2: (500, 150000, 300), 3: (10**4, 300000, 6000), 4: (10**4, 100000, 6000)}
MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
t = 4 if MAX else rng.randint(1, 4)
cap, q1, q2 = SUBTASKS[t]
n = cap if MAX else rng.randint(3, max(3, min(cap, int(sys.argv[2]))))
h = list(range(1, n + 1))
order = "random" if MAX else rng.choice(["random", "random", "sorted", "reversed"])
if order == "random":
    rng.shuffle(h)
elif order == "reversed":
    h.reverse()
print(n, q1, q2, t)
print(*h)
