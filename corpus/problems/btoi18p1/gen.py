"""btoi18p1 (Love Polygon) inputs: `gen.py SEED SIZE`, or `gen.py --max`.

N, then N lines "s t": person s loves t, names of 1..10 lowercase letters, each person's line
once. --max: N = 1e5 of 10-letter names, long in-trees hanging off a few long cycles.
"""
import random
import string
import sys

MAX = sys.argv[1] == "--max"
rng = random.Random(1 if MAX else int(sys.argv[1]))
n = 10**5 if MAX else rng.randint(1, max(1, int(sys.argv[2])))
names = set()
while len(names) < n:
    names.add("".join(rng.choice(string.ascii_lowercase) for _ in range(10 if MAX else rng.randint(1, 10))))
names = list(names)
rng.shuffle(names)
loves = [0] * n
ring = n // 10 if MAX else rng.randint(1, n)
for i in range(ring):
    loves[i] = (i + 1) % ring if rng.random() < 0.9 or MAX else rng.randrange(ring)
for i in range(ring, n):
    loves[i] = i - 1 if MAX and rng.random() < 0.9 else rng.randrange(i)
order = list(range(n))
rng.shuffle(order)
print(n)
print("\n".join(f"{names[i]} {names[loves[i]]}" for i in order))
