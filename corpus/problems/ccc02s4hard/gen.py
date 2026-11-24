"""ccc02s4hard (Bridge Crossing, Hard) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

M, Q, then Q people as name and time on their own lines. --max: M = Q = 1e5, times up to 1e9
with long falling stretches (deep monotone stacks) between big spikes.
"""
import random
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
size = 10**5 if MAX else max(1, int(sys.argv[2]))
q = size if MAX else rng.randint(1, size)
m = 10**5 if MAX else rng.randint(1, max(1, rng.choice([q, 3, size])))
hi = 10**9 if MAX else rng.choice([1, 5, 100, 10**9])
times, cur = [], hi
for i in range(q):
    if MAX:
        cur = hi if rng.random() < 0.001 else max(1, cur - rng.randint(0, 20000))
        times.append(cur)
    else:
        times.append(rng.randint(1, hi))
letters = "abcdefghijklmnopqrstuvwxyz"
print(m)
print(q)
for t in times:
    print("".join(rng.choice(letters) for _ in range(rng.randint(1, 10))))
    print(t)
